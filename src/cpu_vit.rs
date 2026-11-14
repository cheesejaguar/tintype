//! CPU ViT forward pass.
//!
//! Every linear layer runs as one large `sgemm` over all tokens of all views
//! in the batch, attention heads are spread over cores, and the element-wise
//! work (layer norm, GELU, softmax) is vectorised. This is what lets the
//! default six-view profile keep up on machines without a usable GPU engine.
//!
//! The matrix products go to Accelerate on macOS and to the pure-Rust `gemm`
//! crate (NEON, AVX2 or AVX-512 kernels picked at run time) everywhere else;
//! the `portable-cpu` feature selects the latter on macOS too.

use crate::vit::VitConfig;
use crate::weights::Weights;
use anyhow::Result;
use rayon::prelude::*;

#[cfg(all(target_os = "macos", not(feature = "portable-cpu")))]
mod kernels {
    #[link(name = "Accelerate", kind = "framework")]
    extern "C" {
        fn cblas_sgemm(
            order: i32,
            trans_a: i32,
            trans_b: i32,
            m: i32,
            n: i32,
            k: i32,
            alpha: f32,
            a: *const f32,
            lda: i32,
            b: *const f32,
            ldb: i32,
            beta: f32,
            c: *mut f32,
            ldc: i32,
        );
        fn vvexpf(y: *mut f32, x: *const f32, n: *const i32);
    }

    const ROW_MAJOR: i32 = 101;
    const NO_TRANS: i32 = 111;
    const TRANS: i32 = 112;

    /// `c[m, n] = a[m, k] * b + beta * c` where `b` is `[n, k]` (`b_trans`) or `[k, n]`.
    pub fn gemm(a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize, b_trans: bool, beta: f32) {
        assert!(a.len() >= m * k && b.len() >= n * k && c.len() >= m * n);
        unsafe {
            cblas_sgemm(
                ROW_MAJOR,
                NO_TRANS,
                if b_trans { TRANS } else { NO_TRANS },
                m as i32,
                n as i32,
                k as i32,
                1.0,
                a.as_ptr(),
                k as i32,
                b.as_ptr(),
                if b_trans { k } else { n } as i32,
                beta,
                c.as_mut_ptr(),
                n as i32,
            );
        }
    }

    /// Accelerate schedules its own threads; the batch goes to it whole.
    pub const ONE_VIEW_PER_THREAD: bool = false;

    pub fn on_one_thread<R>(f: impl FnOnce() -> R) -> R {
        f()
    }

    pub fn exp_inplace(x: &mut [f32]) {
        let n = x.len() as i32;
        unsafe { vvexpf(x.as_mut_ptr(), x.as_ptr(), &n) }
    }
}

#[cfg(not(all(target_os = "macos", not(feature = "portable-cpu"))))]
mod kernels {
    /// Products below this many multiply-adds stay on the calling thread: the
    /// attention products already run one task per view.
    const PARALLEL_FROM: usize = 1 << 22;

    /// A batch runs one view per thread, each view's products on that thread:
    /// splitting every product across the threads instead spent about 40% of
    /// the processor time on coordination (eight cores, ViT-S/16).
    pub const ONE_VIEW_PER_THREAD: bool = true;

    thread_local! {
        static ONE_THREAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    /// Run `f` with every product it makes kept on the calling thread.
    pub fn on_one_thread<R>(f: impl FnOnce() -> R) -> R {
        let was = ONE_THREAD.replace(true);
        let r = f();
        ONE_THREAD.set(was);
        r
    }

    /// `c[m, n] = a[m, k] * b + beta * c` where `b` is `[n, k]` (`b_trans`) or `[k, n]`.
    pub fn gemm(a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize, b_trans: bool, beta: f32) {
        assert!(a.len() >= m * k && b.len() >= n * k && c.len() >= m * n);
        let (b_cs, b_rs) = if b_trans { (k as isize, 1) } else { (1, n as isize) };
        let par = if m * n * k >= PARALLEL_FROM && !ONE_THREAD.get() { gemm::Parallelism::Rayon(0) } else { gemm::Parallelism::None };
        // dst := alpha * dst + beta * lhs * rhs, in this crate's naming.
        unsafe {
            gemm::gemm(m, n, k, c.as_mut_ptr(), 1, n as isize, beta != 0.0, a.as_ptr(), 1, k as isize, b.as_ptr(), b_cs, b_rs, beta, 1.0f32, false, false, false, par);
        }
    }

    /// `exp` to about one unit in the last place, written so the loop
    /// vectorises: reduce to `2^n * exp(r)` with `|r| <= ln 2 / 2`.
    pub fn exp_inplace(x: &mut [f32]) {
        const ROUND: f32 = 12_582_912.0; // 1.5 * 2^23: adding and subtracting rounds to nearest
        const LN2_HI: f32 = 0.693_359_4;
        const LN2_LO: f32 = -2.121_944_4e-4;
        for v in x.iter_mut() {
            let a = v.clamp(-87.0, 88.0);
            let n = (a * std::f32::consts::LOG2_E + ROUND) - ROUND;
            let r = (a - n * LN2_HI) - n * LN2_LO;
            let p = 1.0 + r * (1.0 + r * (0.5 + r * (1.0 / 6.0 + r * (1.0 / 24.0 + r * (1.0 / 120.0 + r * (1.0 / 720.0))))));
            *v = p * f32::from_bits(((n as i32 + 127) as u32) << 23);
        }
    }
}

use kernels::{exp_inplace, gemm, on_one_thread, ONE_VIEW_PER_THREAD};

/// Whether a view's logit depends on what [`CpuVit::forward`] is given with
/// it. With Accelerate it does: every linear layer is one product over all
/// rows of the batch, and the library rounds differently for different row
/// counts and positions, so the same view scored alone and in a batch of six
/// differs in the last bits (up to 2.4e-6 on the logit, measured on an M6).
/// The same run always gives the same bits. The pure-Rust kernels run a
/// batch one view at a time, so there a view's logit is the same whatever it
/// is run with.
pub const RUN_SIZE_CHANGES_LOGITS: bool = !ONE_VIEW_PER_THREAD;

struct Lin {
    /// `[out, in]`, as stored in the checkpoint.
    w: Vec<f32>,
    b: Vec<f32>,
    out: usize,
    inp: usize,
}

impl Lin {
    fn load(w: &Weights, name: &str) -> Result<Self> {
        let (wd, ws) = w.get(&format!("{name}.weight"))?;
        let (bd, _) = w.get(&format!("{name}.bias"))?;
        Ok(Self { w: wd.clone(), b: bd.clone(), out: ws[0], inp: ws[1..].iter().product() })
    }

    /// `y[rows, out] = x[rows, in] W^T + b`
    fn apply(&self, x: &[f32], y: &mut [f32], rows: usize) {
        y.par_chunks_mut(self.out).for_each(|r| r.copy_from_slice(&self.b));
        gemm(x, &self.w, y, rows, self.out, self.inp, true, 1.0);
    }
}

struct Norm {
    g: Vec<f32>,
    b: Vec<f32>,
}

impl Norm {
    fn load(w: &Weights, name: &str) -> Result<Self> {
        Ok(Self { g: w.get(&format!("{name}.weight"))?.0.clone(), b: w.get(&format!("{name}.bias"))?.0.clone() })
    }

    fn apply(&self, x: &[f32], y: &mut [f32]) {
        let d = self.g.len();
        y.par_chunks_mut(d).zip(x.par_chunks(d)).for_each(|(o, r)| {
            let mean = r.iter().sum::<f32>() / d as f32;
            let var = r.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / d as f32;
            let inv = 1.0 / (var + 1e-6).sqrt();
            for i in 0..d {
                o[i] = (r[i] - mean) * inv * self.g[i] + self.b[i];
            }
        });
    }
}

/// Exact-to-single-precision GELU: `0.5 x (1 + erf(x / sqrt 2))`, with erf from
/// Abramowitz & Stegun 7.1.26 (|error| < 1.5e-7).
fn gelu_inplace(x: &mut [f32]) {
    x.par_chunks_mut(1 << 14).for_each(|c| {
        let mut e: Vec<f32> = c.iter().map(|&v| -0.5 * v * v).collect();
        exp_inplace(&mut e);
        for (v, ex) in c.iter_mut().zip(&e) {
            let z = v.abs() * std::f32::consts::FRAC_1_SQRT_2;
            let t = 1.0 / (1.0 + 0.327_591_1 * z);
            let poly = t * (0.254_829_6 + t * (-0.284_496_74 + t * (1.421_413_7 + t * (-1.453_152_ + t * 1.061_405_4))));
            let erf = (1.0 - poly * ex).copysign(*v);
            *v = 0.5 * *v * (1.0 + erf);
        }
    });
}

struct Block {
    norm1: Norm,
    qkv: Lin,
    proj: Lin,
    norm2: Norm,
    fc1: Lin,
    fc2: Lin,
}

pub struct CpuVit {
    cfg: VitConfig,
    patch: Lin,
    cls: Vec<f32>,
    pos: Vec<f32>,
    blocks: Vec<Block>,
    norm: Norm,
    head: Lin,
}

impl CpuVit {
    pub fn new(w: &Weights, cfg: VitConfig) -> Result<Self> {
        anyhow::ensure!(cfg.reg_tokens == 0 && !cfg.layer_scale && !cfg.pos_on_patches_only, "this architecture runs on the GPU engine only");
        let blocks = (0..cfg.depth)
            .map(|i| {
                let p = format!("blocks.{i}");
                Ok(Block {
                    norm1: Norm::load(w, &format!("{p}.norm1"))?,
                    qkv: Lin::load(w, &format!("{p}.attn.qkv"))?,
                    proj: Lin::load(w, &format!("{p}.attn.proj"))?,
                    norm2: Norm::load(w, &format!("{p}.norm2"))?,
                    fc1: Lin::load(w, &format!("{p}.mlp.fc1"))?,
                    fc2: Lin::load(w, &format!("{p}.mlp.fc2"))?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            cfg,
            patch: Lin::load(w, "patch_embed.proj")?,
            cls: w.get("cls_token")?.0.clone(),
            pos: w.get("pos_embed")?.0.clone(),
            blocks,
            norm: Norm::load(w, "norm")?,
            head: Lin::load(w, "head")?,
        })
    }

    /// Logits for `x`: views of interleaved 8-bit RGB, one after another.
    pub fn forward(&self, x: &[u8]) -> Vec<f32> {
        let c = &self.cfg;
        let (p, side, d) = (c.patch, c.img / c.patch, c.dim);
        let (np, n) = (side * side, side * side + 1);
        let view = 3 * c.img * c.img;
        let b = x.len() / view;
        if ONE_VIEW_PER_THREAD && b > 1 {
            return x.par_chunks(view).flat_map_iter(|v| on_one_thread(|| self.forward(v))).collect();
        }
        let pdim = 3 * p * p;
        let (k, o) = crate::image_io::norm_consts();

        // Unfold into normalised patch rows (channel, row, column order, as
        // the conv weight).
        let mut patches = vec![0f32; b * np * pdim];
        patches.par_chunks_mut(np * pdim).zip(x.par_chunks(view)).for_each(|(out, img)| {
            for py in 0..side {
                for px in 0..side {
                    let row = &mut out[(py * side + px) * pdim..][..pdim];
                    for dy in 0..p {
                        let src = &img[((py * p + dy) * c.img + px * p) * 3..][..p * 3];
                        for (dx, rgb) in src.chunks_exact(3).enumerate() {
                            for ch in 0..3 {
                                row[ch * p * p + dy * p + dx] = rgb[ch] as f32 * k[ch] - o[ch];
                            }
                        }
                    }
                }
            }
        });
        let mut emb = vec![0f32; b * np * d];
        self.patch.apply(&patches, &mut emb, b * np);
        drop(patches);

        let mut t = vec![0f32; b * n * d];
        t.par_chunks_mut(n * d).zip(emb.par_chunks(np * d)).for_each(|(o, e)| {
            o[..d].copy_from_slice(&self.cls);
            o[d..].copy_from_slice(e);
            for (v, pe) in o.iter_mut().zip(&self.pos) {
                *v += pe;
            }
        });
        drop(emb);

        let rows = b * n;
        let (h, hd) = (c.heads, d / c.heads);
        let scale = (hd as f32).powf(-0.5);
        let mut y = vec![0f32; rows * d];
        let mut qkv = vec![0f32; rows * 3 * d];
        let mut att = vec![0f32; rows * d];
        let mut hid = vec![0f32; rows * d * c.mlp_ratio];
        // `s[i, j] = softmax_j(scale * q_i . k_j)` for `nq` queries against `n` keys.
        let attend = |q: &[f32], k: &[f32], s: &mut [f32], nq: usize| {
            gemm(q, k, s, nq, n, hd, true, 0.0);
            for row in s.chunks_mut(n) {
                let m = row.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                for e in row.iter_mut() {
                    *e = (*e - m) * scale;
                }
            }
            exp_inplace(s);
            for row in s.chunks_mut(n) {
                let inv = 1.0 / row.iter().sum::<f32>();
                row.iter_mut().for_each(|e| *e *= inv);
            }
        };
        let last = self.blocks.len() - 1;
        for blk in &self.blocks[..last] {
            blk.norm1.apply(&t, &mut y);
            blk.qkv.apply(&y, &mut qkv, rows);
            // One task per view.
            att.par_chunks_mut(n * d).zip(qkv.par_chunks(n * 3 * d)).for_each(|(out, src)| {
                let (mut q, mut k, mut v) = (vec![0f32; n * hd], vec![0f32; n * hd], vec![0f32; n * hd]);
                let (mut s, mut o) = (vec![0f32; n * n], vec![0f32; n * hd]);
                for head in 0..h {
                    for i in 0..n {
                        let r = &src[i * 3 * d..];
                        q[i * hd..][..hd].copy_from_slice(&r[head * hd..][..hd]);
                        k[i * hd..][..hd].copy_from_slice(&r[d + head * hd..][..hd]);
                        v[i * hd..][..hd].copy_from_slice(&r[2 * d + head * hd..][..hd]);
                    }
                    attend(&q, &k, &mut s, n);
                    gemm(&s, &v, &mut o, n, hd, n, false, 0.0);
                    for i in 0..n {
                        out[i * d + head * hd..][..hd].copy_from_slice(&o[i * hd..][..hd]);
                    }
                }
            });
            blk.proj.apply(&att, &mut y, rows);
            t.par_iter_mut().zip(y.par_iter()).for_each(|(a, b)| *a += b);

            blk.norm2.apply(&t, &mut y);
            blk.fc1.apply(&y, &mut hid, rows);
            gelu_inplace(&mut hid);
            blk.fc2.apply(&hid, &mut y, rows);
            t.par_iter_mut().zip(y.par_iter()).for_each(|(a, b)| *a += b);
        }

        // Last block: only its class-token output is read, so only the class
        // token's query is attended (against every key and value) and only
        // that one token goes through the projection and the MLP. Same
        // result as running the block in full, at about a quarter of the cost.
        let blk = &self.blocks[last];
        blk.norm1.apply(&t, &mut y);
        blk.qkv.apply(&y, &mut qkv, rows);
        let mut cls_att = vec![0f32; b * d];
        cls_att.par_chunks_mut(d).zip(qkv.par_chunks(n * 3 * d)).for_each(|(out, src)| {
            let (mut k, mut v) = (vec![0f32; n * hd], vec![0f32; n * hd]);
            let mut s = vec![0f32; n];
            for head in 0..h {
                for i in 0..n {
                    let r = &src[i * 3 * d..];
                    k[i * hd..][..hd].copy_from_slice(&r[d + head * hd..][..hd]);
                    v[i * hd..][..hd].copy_from_slice(&r[2 * d + head * hd..][..hd]);
                }
                attend(&src[head * hd..][..hd], &k, &mut s, 1);
                gemm(&s, &v, &mut out[head * hd..][..hd], 1, hd, n, false, 0.0);
            }
        });
        // Class tokens only from here on: `[b, d]`.
        let mut cls: Vec<f32> = (0..b).flat_map(|i| t[i * n * d..][..d].iter().copied()).collect();
        let mut yc = vec![0f32; b * d];
        blk.proj.apply(&cls_att, &mut yc, b);
        cls.iter_mut().zip(&yc).for_each(|(a, v)| *a += v);
        blk.norm2.apply(&cls, &mut yc);
        let mut hc = vec![0f32; b * d * c.mlp_ratio];
        blk.fc1.apply(&yc, &mut hc, b);
        gelu_inplace(&mut hc);
        blk.fc2.apply(&hc, &mut yc, b);
        cls.iter_mut().zip(&yc).for_each(|(a, v)| *a += v);

        let mut cn = vec![0f32; b * d];
        self.norm.apply(&cls, &mut cn);
        let mut out = vec![0f32; b * self.head.out];
        self.head.apply(&cn, &mut out, b);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gemm_matches_the_definition() {
        let (m, n, k) = (5, 4, 3);
        let a: Vec<f32> = (0..m * k).map(|i| (i as f32 * 0.37).sin()).collect();
        let bt: Vec<f32> = (0..n * k).map(|i| (i as f32 * 0.91).cos()).collect(); // [n, k]
        let b: Vec<f32> = (0..k * n).map(|i| bt[(i % n) * k + i / n]).collect(); // [k, n]
        let want = |i: usize, j: usize| (0..k).map(|l| a[i * k + l] * bt[j * k + l]).sum::<f32>();
        for (rhs, trans) in [(&bt, true), (&b, false)] {
            let mut c = vec![2.0f32; m * n];
            gemm(&a, rhs, &mut c, m, n, k, trans, 0.0);
            let mut c1 = vec![2.0f32; m * n];
            gemm(&a, rhs, &mut c1, m, n, k, trans, 1.0);
            for i in 0..m {
                for j in 0..n {
                    assert!((c[i * n + j] - want(i, j)).abs() < 1e-5);
                    assert!((c1[i * n + j] - want(i, j) - 2.0).abs() < 1e-5);
                }
            }
        }
    }

    #[test]
    fn exp_is_accurate_over_the_range_the_network_uses() {
        // Softmax and GELU only ever pass arguments at or below zero.
        let xs: Vec<f32> = (0..20_000).map(|i| -95.0 + i as f32 * 0.005).collect();
        let mut ys = xs.clone();
        exp_inplace(&mut ys);
        for (&x, &y) in xs.iter().zip(&ys) {
            if x < -87.0 {
                assert!((0.0..2e-38).contains(&y), "exp({x}) = {y}");
            } else {
                let want = x.exp();
                assert!((y - want).abs() <= 4e-7 * want, "exp({x}) = {y}, want {want}");
            }
        }
    }
}
