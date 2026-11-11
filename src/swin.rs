//! Swin Transformer V2 forward graph (Hugging Face `Swinv2ForImageClassification`
//! layout) on MPSGraph. Used to run the SwinV2 reference detector from Rust.
//!
//! Everything that does not depend on the input — the continuous relative
//! position bias, the shifted-window masks, the clamped logit scales — is
//! computed once on the CPU and baked into the graph as constants.
#![cfg(target_os = "macos")]

use crate::mps::{Graph, T};
use crate::weights::{transpose2, Weights};
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct SwinConfig {
    pub img: usize,
    pub patch: usize,
    pub dim: usize,
    pub depths: Vec<usize>,
    pub heads: Vec<usize>,
    pub window: usize,
    pub pretrained_windows: Vec<usize>,
}

impl SwinConfig {
    /// `swinv2-large-patch4-window12to16-192to256` geometry.
    pub fn large_w16_256() -> Self {
        Self { img: 256, patch: 4, dim: 192, depths: vec![2, 2, 18, 2], heads: vec![6, 12, 24, 48], window: 16, pretrained_windows: vec![12, 12, 12, 6] }
    }
}

struct Ctx<'a> {
    g: &'a Graph,
    w: &'a Weights,
}

impl Ctx<'_> {
    fn raw(&self, name: &str) -> Result<T> {
        let (d, s) = self.w.get(name)?;
        Ok(self.g.constant(d, &s.iter().map(|&x| x as isize).collect::<Vec<_>>()))
    }

    fn linear(&self, x: &T, name: &str, bias: bool) -> Result<T> {
        let (d, s) = self.w.get(&format!("{name}.weight"))?;
        let (o, i) = (s[0], s[1..].iter().product::<usize>());
        let wt = self.g.constant(&transpose2(d, o, i), &[i as isize, o as isize]);
        let y = self.g.matmul(x, &wt);
        Ok(if bias { self.g.add(&y, &self.raw(&format!("{name}.bias"))?) } else { y })
    }

    fn norm(&self, x: &T, name: &str, rank: isize) -> Result<T> {
        Ok(self.g.layer_norm(x, &self.raw(&format!("{name}.weight"))?, &self.raw(&format!("{name}.bias"))?, 1e-5, rank))
    }
}

/// `16 * sigmoid(MLP(log-spaced relative coordinates))`, gathered into
/// `[heads, N, N]` for a `ws x ws` window.
fn relative_bias(w: &Weights, pre: &str, ws: usize, pretrained: usize, heads: usize) -> Result<Vec<f32>> {
    let (w0, _) = w.get(&format!("{pre}.continuous_position_bias_mlp.0.weight"))?;
    let (b0, _) = w.get(&format!("{pre}.continuous_position_bias_mlp.0.bias"))?;
    let (w2, _) = w.get(&format!("{pre}.continuous_position_bias_mlp.2.weight"))?;
    let hidden = b0.len();
    let side = 2 * ws - 1;
    let denom = if pretrained > 0 { pretrained - 1 } else { ws - 1 } as f32;
    let coord = |d: isize| {
        let c = d as f32 / denom * 8.0;
        c.signum() * (c.abs() + 1.0).log2() / 3.0
    };
    let mut table = vec![0f32; side * side * heads];
    let mut h = vec![0f32; hidden];
    for a in 0..side {
        for b in 0..side {
            let (c0, c1) = (coord(a as isize - (ws as isize - 1)), coord(b as isize - (ws as isize - 1)));
            for k in 0..hidden {
                h[k] = (w0[k * 2] * c0 + w0[k * 2 + 1] * c1 + b0[k]).max(0.0);
            }
            for head in 0..heads {
                let z: f32 = (0..hidden).map(|k| w2[head * hidden + k] * h[k]).sum();
                table[(a * side + b) * heads + head] = 16.0 / (1.0 + (-z).exp());
            }
        }
    }
    let n = ws * ws;
    let mut out = vec![0f32; heads * n * n];
    for i in 0..n {
        for j in 0..n {
            let (hi, wi, hj, wj) = ((i / ws) as isize, (i % ws) as isize, (j / ws) as isize, (j % ws) as isize);
            let rel = ((hi - hj + ws as isize - 1) as usize) * side + (wi - wj + ws as isize - 1) as usize;
            for head in 0..heads {
                out[(head * n + i) * n + j] = table[rel * heads + head];
            }
        }
    }
    Ok(out)
}

/// Additive mask `[windows, N, N]` that stops attention across the seams a
/// cyclic shift creates.
fn shift_mask(res: usize, ws: usize, shift: usize) -> Vec<f32> {
    let region = |v: usize| {
        if v < res - ws {
            0
        } else if v < res - shift {
            1
        } else {
            2
        }
    };
    let (nw, n) = (res / ws, ws * ws);
    let mut out = vec![0f32; nw * nw * n * n];
    for wy in 0..nw {
        for wx in 0..nw {
            let id = |k: usize| region(wy * ws + k / ws) * 3 + region(wx * ws + k % ws);
            let base = (wy * nw + wx) * n * n;
            for i in 0..n {
                for j in 0..n {
                    if id(i) != id(j) {
                        out[base + i * n + j] = -100.0;
                    }
                }
            }
        }
    }
    out
}

fn roll(g: &Graph, x: &T, axis: usize, len: isize, by: isize) -> T {
    // torch.roll(x, -by): element `by` moves to the front.
    g.concat(&[g.slice(x, axis, by, len - by), g.slice(x, axis, 0, by)], axis as isize)
}

/// Build the network. `x` is `[B, 3, H, W]`, normalised. Returns `[B, 1]`:
/// `logit(class 0) - logit(class 1)`, whose sigmoid is the softmax probability
/// of class 0.
pub fn build_swin(g: &Graph, w: &Weights, cfg: &SwinConfig, x: &T, batch: usize) -> Result<T> {
    let c = Ctx { g, w };
    let b = batch as isize;
    let p = cfg.patch as isize;
    let mut res = cfg.img / cfg.patch;
    let mut dim = cfg.dim;

    // Patch embedding as unfold + matmul, then LayerNorm.
    let side = res as isize;
    let t = g.reshape(x, &[b, 3, side, p, side, p]);
    let t = g.permute(&t, &[0, 2, 4, 1, 3, 5]);
    let t = g.reshape(&t, &[b, side * side, 3 * p * p]);
    let t = c.linear(&t, "swinv2.embeddings.patch_embeddings.projection", true)?;
    let mut h = c.norm(&t, "swinv2.embeddings.norm", 3)?;

    for (stage, &depth) in cfg.depths.iter().enumerate() {
        let heads = cfg.heads[stage];
        let hd = (dim / heads) as isize;
        let (ws, can_shift) = if res <= cfg.window { (res, false) } else { (cfg.window, true) };
        let (r, d, wsi, n) = (res as isize, dim as isize, ws as isize, (ws * ws) as isize);
        let nw = (res / ws) as isize;
        for blk in 0..depth {
            let pre = format!("swinv2.encoder.layers.{stage}.blocks.{blk}");
            let shift = if can_shift && blk % 2 == 1 { ws / 2 } else { 0 };
            let mut v = g.reshape(&h, &[b, r, r, d]);
            if shift > 0 {
                v = roll(g, &roll(g, &v, 1, r, shift as isize), 2, r, shift as isize);
            }
            // Window partition.
            let v = g.reshape(&v, &[b, nw, wsi, nw, wsi, d]);
            let v = g.reshape(&g.permute(&v, &[0, 1, 3, 2, 4, 5]), &[b * nw * nw, n, d]);

            let sa = format!("{pre}.attention.self");
            let split = |t: &T| g.permute(&g.reshape(t, &[b * nw * nw, n, heads as isize, hd]), &[0, 2, 1, 3]);
            let unit = |t: &T| {
                let norm = g.sqrt(&g.sum(&g.square(t), &[3]));
                g.div(t, &g.max(&norm, &g.scalar(1e-12)))
            };
            let q = unit(&split(&c.linear(&v, &format!("{sa}.query"), true)?));
            let k = unit(&split(&c.linear(&v, &format!("{sa}.key"), false)?));
            let val = split(&c.linear(&v, &format!("{sa}.value"), true)?);
            let (ls, _) = w.get(&format!("{sa}.logit_scale"))?;
            let scale: Vec<f32> = ls.iter().map(|&s| s.min(100f32.ln()).exp()).collect();
            let mut att = g.mul(&g.matmul(&q, &g.transpose(&k, 2, 3)), &g.constant(&scale, &[1, heads as isize, 1, 1]));
            let bias = relative_bias(w, &sa, ws, cfg.pretrained_windows[stage], heads)?;
            att = g.add(&att, &g.constant(&bias, &[1, heads as isize, n, n]));
            if shift > 0 {
                let mask = g.constant(&shift_mask(res, ws, shift), &[1, nw * nw, 1, n, n]);
                att = g.reshape(&g.add(&g.reshape(&att, &[b, nw * nw, heads as isize, n, n]), &mask), &[b * nw * nw, heads as isize, n, n]);
            }
            let att = g.softmax(&att, 3);
            let o = g.reshape(&g.transpose(&g.matmul(&att, &val), 1, 2), &[b * nw * nw, n, d]);
            let o = c.linear(&o, &format!("{pre}.attention.output.dense"), true)?;

            // Window reverse, undo the shift.
            let o = g.reshape(&o, &[b, nw, nw, wsi, wsi, d]);
            let mut o = g.reshape(&g.permute(&o, &[0, 1, 3, 2, 4, 5]), &[b, r, r, d]);
            if shift > 0 {
                let back = r - shift as isize;
                o = roll(g, &roll(g, &o, 1, r, back), 2, r, back);
            }
            let o = g.reshape(&o, &[b, r * r, d]);

            // Post-norm residuals (Swin V2).
            h = g.add(&h, &c.norm(&o, &format!("{pre}.layernorm_before"), 3)?);
            let m = g.gelu(&c.linear(&h, &format!("{pre}.intermediate.dense"), true)?);
            let m = c.linear(&m, &format!("{pre}.output.dense"), true)?;
            h = g.add(&h, &c.norm(&m, &format!("{pre}.layernorm_after"), 3)?);
        }
        if stage + 1 < cfg.depths.len() {
            // Patch merging: concatenate each 2x2 neighbourhood, reduce, normalise.
            let pre = format!("swinv2.encoder.layers.{stage}.downsample");
            let v = g.reshape(&h, &[b, r / 2, 2, r / 2, 2, d]);
            // (h parity, w parity) -> channel blocks ordered (h0,w0), (h1,w0), (h0,w1), (h1,w1)
            let v = g.reshape(&g.permute(&v, &[0, 1, 3, 4, 2, 5]), &[b, (r / 2) * (r / 2), 4 * d]);
            h = c.norm(&c.linear(&v, &format!("{pre}.reduction"), false)?, &format!("{pre}.norm"), 3)?;
            res /= 2;
            dim *= 2;
        }
    }
    let h = c.norm(&h, "swinv2.layernorm", 3)?;
    let pooled = g.reshape(&g.mean(&h, &[1]), &[b, dim as isize]);
    let logits = c.linear(&pooled, "classifier", true)?;
    Ok(g.sub(&g.slice(&logits, 1, 0, 1), &g.slice(&logits, 1, 1, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_mask_blocks_only_across_seams() {
        // 4x4 grid, 2x2 windows, shift 1: the last window mixes four regions.
        let m = shift_mask(4, 2, 1);
        let (nw, n) = (2, 4);
        assert_eq!(m.len(), nw * nw * n * n);
        // first window: all tokens in region 0 -> no masking
        assert!(m[..n * n].iter().all(|&v| v == 0.0));
        // last window: every token is its own region -> only the diagonal is open
        let last = &m[3 * n * n..];
        for i in 0..n {
            for j in 0..n {
                assert_eq!(last[i * n + j], if i == j { 0.0 } else { -100.0 });
            }
        }
    }
}
