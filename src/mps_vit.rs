//! ViT forward graph on MPSGraph. The same builder serves inference (frozen
//! constants) and training (assignable variables) through [`ParamSource`].
#![cfg(target_os = "macos")]

use crate::mps::{Graph, T};
use crate::vit::VitConfig;
use crate::weights::{transpose2, Weights};
use anyhow::Result;

/// How a named parameter becomes a graph tensor. Linear weights are handed
/// over already transposed to `[in, out]`; `linear` tells the source so it can
/// undo that when exporting.
pub trait ParamSource {
    fn param(&mut self, g: &Graph, name: &str, data: &[f32], shape: &[isize], linear: bool) -> T;
}

/// Inference: every parameter is a frozen constant.
pub struct Frozen;
impl ParamSource for Frozen {
    fn param(&mut self, g: &Graph, _: &str, data: &[f32], shape: &[isize], _: bool) -> T {
        g.constant(data, shape)
    }
}

struct Ctx<'a, P: ParamSource> {
    g: &'a Graph,
    w: &'a Weights,
    p: &'a mut P,
}

impl<P: ParamSource> Ctx<'_, P> {
    fn raw(&mut self, name: &str) -> Result<T> {
        let (d, s) = self.w.get(name)?;
        let shape: Vec<isize> = s.iter().map(|&x| x as isize).collect();
        Ok(self.p.param(self.g, name, d, &shape, false))
    }

    /// `y = x W^T + b` with `W` stored `[out, in]` on disk.
    fn linear(&mut self, x: &T, name: &str) -> Result<T> {
        let (d, s) = self.w.get(&format!("{name}.weight"))?;
        let (o, i) = (s[0], s[1..].iter().product::<usize>());
        let wt = self.p.param(self.g, &format!("{name}.weight"), &transpose2(d, o, i), &[i as isize, o as isize], true);
        let b = self.raw(&format!("{name}.bias"))?;
        Ok(self.g.add(&self.g.matmul(x, &wt), &b))
    }

    fn norm(&mut self, x: &T, name: &str) -> Result<T> {
        let (gm, bt) = (self.raw(&format!("{name}.weight"))?, self.raw(&format!("{name}.bias"))?);
        Ok(self.g.layer_norm(x, &gm, &bt, 1e-6, 3))
    }
}

pub struct VitOut {
    /// Final normalised tokens `[B, N+1, D]`.
    pub tokens: T,
    /// CLS embedding `[B, D]`.
    pub cls: T,
}

/// Build the ViT trunk. `x` is `[B, 3, H, W]` in working precision, already
/// normalised. `batch` must be the static batch size.
pub fn build_vit<P: ParamSource>(g: &Graph, w: &Weights, p: &mut P, cfg: &VitConfig, x: &T, batch: usize) -> Result<VitOut> {
    let mut c = Ctx { g, w, p };
    let (b, d) = (batch as isize, cfg.dim as isize);
    let (ps, side) = (cfg.patch as isize, (cfg.img / cfg.patch) as isize);
    let n = side * side;
    let (h, hd) = (cfg.heads as isize, (cfg.dim / cfg.heads) as isize);

    // Patch embedding as unfold + matmul (equivalent to the stride-P conv).
    let x = g.reshape(x, &[b, 3, side, ps, side, ps]);
    let x = g.permute(&x, &[0, 2, 4, 1, 3, 5]);
    let x = g.reshape(&x, &[b, n, 3 * ps * ps]);
    let x = c.linear(&x, "patch_embed.proj")?;

    let cls = c.raw("cls_token")?;
    let pos = c.raw("pos_embed")?;
    let zeros = |k: usize| g.constant(&vec![0f32; batch * k * cfg.dim], &[b, k as isize, d]);
    let cls_b = g.add(&cls, &zeros(1));
    let mut x = if cfg.pos_on_patches_only {
        let x = g.add(&x, &pos);
        if cfg.reg_tokens > 0 {
            let reg = g.add(&c.raw("reg_token")?, &zeros(cfg.reg_tokens));
            g.concat(&[cls_b, reg, x], 1)
        } else {
            g.concat(&[cls_b, x], 1)
        }
    } else {
        g.add(&g.concat(&[cls_b, x], 1), &pos)
    };
    let nt = n + cfg.prefix_tokens() as isize;

    for i in 0..cfg.depth {
        let pre = format!("blocks.{i}");
        let y = c.norm(&x, &format!("{pre}.norm1"))?;
        let qkv = c.linear(&y, &format!("{pre}.attn.qkv"))?;
        let qkv = g.permute(&g.reshape(&qkv, &[b, nt, 3, h, hd]), &[2, 0, 3, 1, 4]);
        let part = |k: isize| g.reshape(&g.slice(&qkv, 0, k, 1), &[b, h, nt, hd]);
        let (q, k, v) = (part(0), part(1), part(2));
        let attn = g.scale(&g.matmul(&q, &g.transpose(&k, 2, 3)), (hd as f64).powf(-0.5));
        let attn = g.softmax(&attn, 3);
        let y = g.reshape(&g.transpose(&g.matmul(&attn, &v), 1, 2), &[b, nt, d]);
        let mut y = c.linear(&y, &format!("{pre}.attn.proj"))?;
        if cfg.layer_scale {
            y = g.mul(&y, &c.raw(&format!("{pre}.ls1.gamma"))?);
        }
        x = g.add(&x, &y);
        let y = c.norm(&x, &format!("{pre}.norm2"))?;
        let y = g.gelu(&c.linear(&y, &format!("{pre}.mlp.fc1"))?);
        let mut y = c.linear(&y, &format!("{pre}.mlp.fc2"))?;
        if cfg.layer_scale {
            y = g.mul(&y, &c.raw(&format!("{pre}.ls2.gamma"))?);
        }
        x = g.add(&x, &y);
    }
    let tokens = c.norm(&x, "norm")?;
    let cls = g.reshape(&g.slice(&tokens, 1, 0, 1), &[b, d]);
    Ok(VitOut { tokens, cls })
}

/// `logit = cls W^T + b` using the checkpoint's `head.*`.
pub fn build_head<P: ParamSource>(g: &Graph, w: &Weights, p: &mut P, cls: &T) -> Result<T> {
    Ctx { g, w, p }.linear(cls, "head")
}
