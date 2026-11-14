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
//!
//! [`Int8`] is a simulation, not a faster engine: the same pass with the
//! weights and inputs of the blocks' linear layers rounded to the values an
//! eight-bit kernel would hold, to find out what such a kernel would do to
//! the scores before anyone writes it.

use crate::stats::{ATTENTION_TIME, FC1_TIME, FC2_TIME, FORWARD_TIME, GATHER_TIME, GELU_TIME, HEAD_TIME, NORM_TIME, PATCH_TIME, PROJ_TIME, QKV_TIME, RESIDUAL_TIME, SOFTMAX_TIME};
use crate::vit::VitConfig;
use crate::weights::Weights;
use anyhow::Result;
use rayon::prelude::*;
use std::time::Instant;

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

/// The four linear layers of a block, in the order [`Int8`] lists them.
#[derive(Clone, Copy)]
enum Layer {
    Qkv = 0,
    Proj = 1,
    Fc1 = 2,
    Fc2 = 3,
}

/// Which values of a quantised input share one range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ranges {
    /// Each token's own smallest and largest value. An integer kernel gets
    /// this for nothing: a token is a row of the product, so its step and
    /// offset come out of the sum.
    Token,
    /// The smallest and largest value over all tokens of the view.
    Tensor,
}

/// What `--device cpu-int8` simulates being held in eight bits. Everything
/// else stays in single precision: the patch embedding, layer
/// normalisation, the query-key and attention-value products, softmax, GELU,
/// the residual stream and the head.
///
/// Weights are rounded once, when the model is loaded, to 255 levels
/// symmetric about zero (127 at seven bits) with one step per output channel
/// (row). Inputs are
/// rounded as they arrive to `2^input_bits` evenly spaced levels between
/// the smallest and largest value of their token or view, so nothing is
/// ever clipped and no calibration data is needed; the ranges of one view
/// never depend on the views it is run with. The rounded values are kept as
/// `f32` and go through the same products as before, so the logits are what
/// an integer kernel would give up to the rounding of the sums, and the
/// pass is slower than the one it imitates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Int8 {
    /// Whose weights are rounded: qkv, proj, fc1, fc2.
    pub weights: [bool; 4],
    /// Whose inputs are rounded, in the same order.
    pub inputs: [bool; 4],
    pub ranges: Ranges,
    /// Bits of a rounded input.
    pub input_bits: u32,
    /// Bits of a rounded weight, sign included. The AVX2 multiply-add
    /// (`vpmaddubsw`) adds two products in sixteen bits, which holds without
    /// saturating when either the inputs or the weights have seven bits.
    pub weight_bits: u32,
}

impl Int8 {
    /// The variants by name: whose weights and whose inputs are rounded.
    #[rustfmt::skip]
    pub const VARIANTS: &'static [(&'static str, [bool; 4], [bool; 4])] = &[
        //            weights: qkv   proj  fc1   fc2      inputs: qkv    proj   fc1    fc2
        ("weights",          [true, true, true, true],          [false, false, false, false]),
        ("mlp",              [true, true, true, true],          [false, false, true,  true]),
        ("all",              [true, true, true, true],          [true,  true,  true,  true]),
        // One input at a time, to find the sensitive one.
        ("qkv",              [true, true, true, true],          [true,  false, false, false]),
        ("proj",             [true, true, true, true],          [false, true,  false, false]),
        ("fc1",              [true, true, true, true],          [false, false, true,  false]),
        ("fc2",              [true, true, true, true],          [false, false, false, true]),
        // Inputs only, weights left alone.
        ("inputs",           [false, false, false, false],      [true,  true,  true,  true]),
    ];

    /// `variant[:ranges[:input bits[:weight bits]]]`, e.g. `all`,
    /// `mlp:tensor`, `all:token:7`, `all:token:8:7`. Ranges default to
    /// `token`, bits to 8.
    pub fn parse(s: &str) -> Result<Self> {
        let mut fields = s.split(':');
        let name = fields.next().unwrap_or("");
        let &(_, weights, inputs) = Self::VARIANTS
            .iter()
            .find(|v| v.0 == name)
            .ok_or_else(|| anyhow::anyhow!("unknown INT8 variant `{name}` (use one of: {})", Self::VARIANTS.iter().map(|v| v.0).collect::<Vec<_>>().join(", ")))?;
        let ranges = match fields.next() {
            None | Some("token") => Ranges::Token,
            Some("tensor") => Ranges::Tensor,
            Some(other) => anyhow::bail!("unknown INT8 ranges `{other}` (use token or tensor)"),
        };
        let mut bits = || match fields.next() {
            None => Ok(8),
            Some(b) => b.parse::<u32>().ok().filter(|b| (2..=16).contains(b)).ok_or_else(|| anyhow::anyhow!("INT8 bits must be a number from 2 to 16, not `{b}`")),
        };
        let (input_bits, weight_bits) = (bits()?, bits()?);
        anyhow::ensure!(fields.next().is_none(), "an INT8 variant is `name[:ranges[:input bits[:weight bits]]]`, not `{s}`");
        Ok(Self { weights, inputs, ranges, input_bits, weight_bits })
    }

    /// The variant `TINTYPE_INT8` names, or `all` (every block linear layer's
    /// weights and inputs, a range per token, eight bits).
    pub fn from_env() -> Result<Self> {
        match std::env::var("TINTYPE_INT8") {
            Ok(v) if !v.is_empty() => Self::parse(&v),
            _ => Self::parse("all"),
        }
    }

    /// Steps between the smallest and the largest value of a rounded input.
    fn input_steps(&self) -> f32 {
        ((1u32 << self.input_bits) - 1) as f32
    }

    /// The largest level of a rounded weight: 127 at eight bits.
    fn weight_levels(&self) -> f32 {
        ((1u32 << (self.weight_bits - 1)) - 1) as f32
    }
}

/// Round each row of `w` (`[out, inp]`) to `2 * levels + 1` levels symmetric
/// about zero (`levels` is 127 for eight bits), the step set by the row's
/// largest magnitude: per-output-channel INT8 weights, kept as the `f32`
/// values they stand for.
fn fake_quantise_weights(w: &mut [f32], inp: usize, levels: f32) {
    for row in w.chunks_mut(inp) {
        let max = row.iter().fold(0f32, |a, &v| a.max(v.abs()));
        if max == 0.0 || !max.is_finite() {
            continue;
        }
        let step = max / levels;
        for v in row.iter_mut() {
            *v = (*v / step).round().clamp(-levels, levels) * step;
        }
    }
}

/// Round `x` to `steps + 1` evenly spaced values from its smallest to its
/// largest: an unsigned integer per value with one step and one offset for
/// the slice, kept as the `f32` values they stand for.
fn fake_quantise_inputs(x: &mut [f32], steps: f32) {
    let (lo, hi) = x.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let step = (hi - lo) / steps;
    // A constant slice needs no rounding; a non-finite one is left for the caller to notice.
    if step.is_nan() || step <= 0.0 || !step.is_finite() {
        return;
    }
    for v in x.iter_mut() {
        *v = lo + ((*v - lo) / step).round().min(steps) * step;
    }
}

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
    /// What is rounded to eight bits on the way; nothing, for the engine proper.
    int8: Option<Int8>,
}

impl CpuVit {
    pub fn new(w: &Weights, cfg: VitConfig) -> Result<Self> {
        Self::with_int8(w, cfg, None)
    }

    /// The same network with the weights `int8` names rounded now and the
    /// inputs it names rounded in every pass (see [`Int8`]).
    pub fn with_int8(w: &Weights, cfg: VitConfig, int8: Option<Int8>) -> Result<Self> {
        let mut vit = Self::load(w, cfg)?;
        if let Some(q) = int8 {
            for blk in &mut vit.blocks {
                for (layer, lin) in [(Layer::Qkv, &mut blk.qkv), (Layer::Proj, &mut blk.proj), (Layer::Fc1, &mut blk.fc1), (Layer::Fc2, &mut blk.fc2)] {
                    if q.weights[layer as usize] {
                        fake_quantise_weights(&mut lin.w, lin.inp, q.weight_levels());
                    }
                }
            }
        }
        vit.int8 = int8;
        Ok(vit)
    }

    fn load(w: &Weights, cfg: VitConfig) -> Result<Self> {
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
            int8: None,
        })
    }

    /// Round the input of `layer` in place where the simulation asks for it:
    /// `x` is rows of `width` values, `view_rows` of them to a view. Does
    /// nothing at all on the engine proper.
    fn round_input(&self, layer: Layer, x: &mut [f32], width: usize, view_rows: usize) {
        let Some(q) = self.int8.filter(|q| q.inputs[layer as usize]) else { return };
        let together = match q.ranges {
            Ranges::Token => width,
            Ranges::Tensor => width * view_rows,
        };
        x.par_chunks_mut(together).for_each(|c| fake_quantise_inputs(c, q.input_steps()));
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
        // Timed on this thread from here on: one pass, however many views it holds.
        let started = Instant::now();
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
        PATCH_TIME.since(started);

        let rows = b * n;
        let (h, hd) = (c.heads, d / c.heads);
        let scale = (hd as f32).powf(-0.5);
        let mut y = vec![0f32; rows * d];
        let mut qkv = vec![0f32; rows * 3 * d];
        let mut att = vec![0f32; rows * d];
        let mut hid = vec![0f32; rows * d * c.mlp_ratio];
        // `s[i, j] = softmax_j(scale * q_i . k_j)` for `nq` queries against `n` keys.
        // `spent` takes the nanoseconds of the product and of the softmax.
        let attend = |q: &[f32], k: &[f32], s: &mut [f32], nq: usize, spent: &mut [u64; 2]| {
            let t = Instant::now();
            gemm(q, k, s, nq, n, hd, true, 0.0);
            let product = t.elapsed();
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
            spent[0] += product.as_nanos() as u64;
            spent[1] += (t.elapsed() - product).as_nanos() as u64;
        };
        // The attention of a block runs one task per view, side by side, so
        // the time its parts take on their threads can add up to more than
        // the block's attention took on this one. Divide the latter between
        // products, softmax and the rest (copying each head's rows together
        // and its output back) in the proportions of the former; with one
        // view, that is the time itself.
        let attention_took = |t: Instant, spent: [u64; 2], tasks: u64| {
            let wall = t.elapsed().as_nanos() as u64;
            let share = |part: u64| if tasks == 0 { 0 } else { (wall as u128 * part as u128 / tasks as u128) as u64 };
            let (products, softmax) = (share(spent[0]), share(spent[1]));
            ATTENTION_TIME.add_nanos(products);
            SOFTMAX_TIME.add_nanos(softmax);
            GATHER_TIME.add_nanos(wall.saturating_sub(products + softmax));
        };
        let add = |a: [u64; 3], b: [u64; 3]| [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
        let last = self.blocks.len() - 1;
        for blk in &self.blocks[..last] {
            NORM_TIME.time(|| blk.norm1.apply(&t, &mut y));
            self.round_input(Layer::Qkv, &mut y, d, n);
            QKV_TIME.time(|| blk.qkv.apply(&y, &mut qkv, rows));
            // One task per view. Each hands back the nanoseconds of its
            // products, of its softmax and of the whole task.
            let at = Instant::now();
            let spent = att.par_chunks_mut(n * d).zip(qkv.par_chunks(n * 3 * d)).map(|(out, src)| {
                let task = Instant::now();
                let mut spent = [0u64; 2];
                let (mut q, mut k, mut v) = (vec![0f32; n * hd], vec![0f32; n * hd], vec![0f32; n * hd]);
                let (mut s, mut o) = (vec![0f32; n * n], vec![0f32; n * hd]);
                for head in 0..h {
                    for i in 0..n {
                        let r = &src[i * 3 * d..];
                        q[i * hd..][..hd].copy_from_slice(&r[head * hd..][..hd]);
                        k[i * hd..][..hd].copy_from_slice(&r[d + head * hd..][..hd]);
                        v[i * hd..][..hd].copy_from_slice(&r[2 * d + head * hd..][..hd]);
                    }
                    attend(&q, &k, &mut s, n, &mut spent);
                    let t = Instant::now();
                    gemm(&s, &v, &mut o, n, hd, n, false, 0.0);
                    spent[0] += t.elapsed().as_nanos() as u64;
                    for i in 0..n {
                        out[i * d + head * hd..][..hd].copy_from_slice(&o[i * hd..][..hd]);
                    }
                }
                [spent[0], spent[1], task.elapsed().as_nanos() as u64]
            });
            let spent = spent.reduce(|| [0; 3], add);
            attention_took(at, [spent[0], spent[1]], spent[2]);
            self.round_input(Layer::Proj, &mut att, d, n);
            PROJ_TIME.time(|| blk.proj.apply(&att, &mut y, rows));
            RESIDUAL_TIME.time(|| t.par_iter_mut().zip(y.par_iter()).for_each(|(a, b)| *a += b));

            NORM_TIME.time(|| blk.norm2.apply(&t, &mut y));
            self.round_input(Layer::Fc1, &mut y, d, n);
            FC1_TIME.time(|| blk.fc1.apply(&y, &mut hid, rows));
            GELU_TIME.time(|| gelu_inplace(&mut hid));
            self.round_input(Layer::Fc2, &mut hid, d * c.mlp_ratio, n);
            FC2_TIME.time(|| blk.fc2.apply(&hid, &mut y, rows));
            RESIDUAL_TIME.time(|| t.par_iter_mut().zip(y.par_iter()).for_each(|(a, b)| *a += b));
        }

        // Last block: only its class-token output is read, so only the class
        // token's query is attended (against every key and value) and only
        // that one token goes through the projection and the MLP. Same
        // result as running the block in full, at about a quarter of the cost.
        let blk = &self.blocks[last];
        NORM_TIME.time(|| blk.norm1.apply(&t, &mut y));
        self.round_input(Layer::Qkv, &mut y, d, n);
        QKV_TIME.time(|| blk.qkv.apply(&y, &mut qkv, rows));
        let mut cls_att = vec![0f32; b * d];
        let at = Instant::now();
        let spent = cls_att.par_chunks_mut(d).zip(qkv.par_chunks(n * 3 * d)).map(|(out, src)| {
            let task = Instant::now();
            let mut spent = [0u64; 2];
            let (mut k, mut v) = (vec![0f32; n * hd], vec![0f32; n * hd]);
            let mut s = vec![0f32; n];
            for head in 0..h {
                for i in 0..n {
                    let r = &src[i * 3 * d..];
                    k[i * hd..][..hd].copy_from_slice(&r[d + head * hd..][..hd]);
                    v[i * hd..][..hd].copy_from_slice(&r[2 * d + head * hd..][..hd]);
                }
                attend(&src[head * hd..][..hd], &k, &mut s, 1, &mut spent);
                let t = Instant::now();
                gemm(&s, &v, &mut out[head * hd..][..hd], 1, hd, n, false, 0.0);
                spent[0] += t.elapsed().as_nanos() as u64;
            }
            [spent[0], spent[1], task.elapsed().as_nanos() as u64]
        });
        let spent = spent.reduce(|| [0; 3], add);
        attention_took(at, [spent[0], spent[1]], spent[2]);
        // Class tokens only from here on: `[b, d]`.
        let mut cls: Vec<f32> = (0..b).flat_map(|i| t[i * n * d..][..d].iter().copied()).collect();
        let mut yc = vec![0f32; b * d];
        // One token to a view from here on, so a view's range is that token's.
        self.round_input(Layer::Proj, &mut cls_att, d, 1);
        PROJ_TIME.time(|| blk.proj.apply(&cls_att, &mut yc, b));
        cls.iter_mut().zip(&yc).for_each(|(a, v)| *a += v);
        NORM_TIME.time(|| blk.norm2.apply(&cls, &mut yc));
        self.round_input(Layer::Fc1, &mut yc, d, 1);
        let mut hc = vec![0f32; b * d * c.mlp_ratio];
        FC1_TIME.time(|| blk.fc1.apply(&yc, &mut hc, b));
        GELU_TIME.time(|| gelu_inplace(&mut hc));
        self.round_input(Layer::Fc2, &mut hc, d * c.mlp_ratio, 1);
        FC2_TIME.time(|| blk.fc2.apply(&hc, &mut yc, b));
        cls.iter_mut().zip(&yc).for_each(|(a, v)| *a += v);

        let mut cn = vec![0f32; b * d];
        NORM_TIME.time(|| self.norm.apply(&cls, &mut cn));
        let mut out = vec![0f32; b * self.head.out];
        HEAD_TIME.time(|| self.head.apply(&cn, &mut out, b));
        FORWARD_TIME.since(started);
        out
    }
}

#[cfg(test)]
mod int8_gemm;

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
    fn weights_round_to_255_levels_per_output_channel() {
        let (out, inp) = (5, 64);
        // Rows of very different sizes, one of them all zero.
        let before: Vec<f32> = (0..out * inp).map(|i| ((i as f32 * 0.731).sin() + 0.2) * [1.0, 0.01, 30.0, 0.0, 1e-4][i / inp]).collect();
        // Eight bits and seven.
        for levels in [127.0f32, 63.0] {
            let mut w = before.clone();
            fake_quantise_weights(&mut w, inp, levels);
            for (row, was) in w.chunks(inp).zip(before.chunks(inp)) {
                let max = was.iter().fold(0f32, |a, &v| a.max(v.abs()));
                if max == 0.0 {
                    assert!(row.iter().all(|&v| v == 0.0));
                    continue;
                }
                let step = max / levels;
                for (&v, &u) in row.iter().zip(was) {
                    let level = v / step;
                    assert!((level - level.round()).abs() < 1e-3 && level.round().abs() <= levels, "{v} is not a level of step {step}");
                    assert!((v - u).abs() <= step * 0.5001, "{u} moved to {v}, step {step}");
                }
                // The largest weight of a row is a level itself, so each row uses its whole range.
                assert!((row.iter().fold(0f32, |a, &v| a.max(v.abs())) - max).abs() <= max * 1e-6);
            }
            // Rounding again changes nothing.
            let once = w.clone();
            fake_quantise_weights(&mut w, inp, levels);
            for (a, b) in w.iter().zip(&once) {
                assert!((a - b).abs() <= a.abs() * 1e-6);
            }
        }
    }

    #[test]
    fn inputs_round_to_evenly_spaced_levels_between_their_extremes() {
        // Lopsided like the values after GELU: a short negative side and a long positive tail.
        let x: Vec<f32> = (0..1536).map(|i| ((i as f32 * 0.37).sin() * 3.0).exp() * 0.4 - 0.17).collect();
        let (lo, hi) = x.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        for (bits, steps) in [(8, 255.0f32), (7, 127.0)] {
            let mut q = x.clone();
            fake_quantise_inputs(&mut q, steps);
            let step = (hi - lo) / steps;
            let mut levels: Vec<i64> = q.iter().map(|&v| ((v - lo) / step).round() as i64).collect();
            for ((&v, &u), &l) in q.iter().zip(&x).zip(&levels) {
                assert!((0..=steps as i64).contains(&l), "{bits} bits: level {l}");
                assert!((v - (lo + l as f32 * step)).abs() <= step * 1e-3 && (v - u).abs() <= step * 0.5001, "{bits} bits: {u} moved to {v}, step {step}");
            }
            levels.sort_unstable();
            levels.dedup();
            assert!(levels.len() <= steps as usize + 1 && levels.len() > steps as usize / 4, "{bits} bits: {} levels", levels.len());
            // The extremes are levels themselves: nothing is clipped.
            assert_eq!((levels[0], *levels.last().unwrap()), (0, steps as i64));
        }
        // A constant slice, and an empty one, are left alone.
        let mut flat = vec![1.25f32; 9];
        fake_quantise_inputs(&mut flat, 255.0);
        assert!(flat.iter().all(|&v| v == 1.25));
        fake_quantise_inputs(&mut [], 255.0);
    }

    #[test]
    fn int8_variants_parse_by_name_with_optional_ranges_and_bits() {
        let all = Int8::parse("all").unwrap();
        assert_eq!(all, Int8 { weights: [true; 4], inputs: [true; 4], ranges: Ranges::Token, input_bits: 8, weight_bits: 8 });
        assert_eq!((all.input_steps(), all.weight_levels()), (255.0, 127.0));
        assert_eq!(Int8::parse("weights").unwrap().inputs, [false; 4]);
        assert_eq!(Int8::parse("mlp:tensor").unwrap(), Int8 { weights: [true; 4], inputs: [false, false, true, true], ranges: Ranges::Tensor, input_bits: 8, weight_bits: 8 });
        let seven = Int8::parse("all:token:7").unwrap();
        assert_eq!((seven.input_bits, seven.input_steps(), seven.weight_bits), (7, 127.0, 8));
        let narrow = Int8::parse("all:token:8:7").unwrap();
        assert_eq!((narrow.input_bits, narrow.weight_bits, narrow.weight_levels()), (8, 7, 63.0));
        assert_eq!(Int8::parse("inputs").unwrap().weights, [false; 4]);
        for (name, ..) in Int8::VARIANTS {
            assert!(Int8::parse(name).is_ok(), "{name}");
        }
        for bad in ["", "everything", "all:row", "all:token:1", "all:token:x", "all:token:8:1", "all:token:8:8:8"] {
            assert!(Int8::parse(bad).is_err(), "`{bad}` should not parse");
        }
    }

    /// A small random network: two blocks of width 24 on 32-pixel views.
    fn small_network() -> (Weights, VitConfig) {
        let cfg = VitConfig { img: 32, patch: 16, dim: 24, depth: 2, heads: 2, mlp_ratio: 4, reg_tokens: 0, layer_scale: false, pos_on_patches_only: false };
        let mut rng = crate::rng::Rng::new(11);
        let mut w = Weights::default();
        let mut tensor = |name: &str, shape: &[usize], scale: f32| {
            let data = (0..shape.iter().product::<usize>()).map(|_| (rng.below(2001) as f32 / 1000.0 - 1.0) * scale).collect();
            w.insert(name, data, shape.to_vec());
        };
        let d = cfg.dim;
        tensor("patch_embed.proj.weight", &[d, 3, 16, 16], 0.05);
        tensor("patch_embed.proj.bias", &[d], 0.1);
        tensor("cls_token", &[1, 1, d], 0.5);
        tensor("pos_embed", &[1, 5, d], 0.5);
        for i in 0..cfg.depth {
            let p = format!("blocks.{i}");
            for (name, out, inp) in [("attn.qkv", 3 * d, d), ("attn.proj", d, d), ("mlp.fc1", 4 * d, d), ("mlp.fc2", d, 4 * d)] {
                tensor(&format!("{p}.{name}.weight"), &[out, inp], 0.3);
                tensor(&format!("{p}.{name}.bias"), &[out], 0.1);
            }
            for n in ["norm1", "norm2"] {
                tensor(&format!("{p}.{n}.weight"), &[d], 1.0);
                tensor(&format!("{p}.{n}.bias"), &[d], 0.1);
            }
        }
        tensor("norm.weight", &[d], 1.0);
        tensor("norm.bias", &[d], 0.1);
        tensor("head.weight", &[1, d], 0.5);
        tensor("head.bias", &[1], 0.1);
        (w, cfg)
    }

    #[test]
    fn the_int8_simulation_moves_logits_a_little_and_leaves_the_engine_proper_alone() {
        let (w, cfg) = small_network();
        let mut rng = crate::rng::Rng::new(5);
        let views: Vec<u8> = (0..3 * 3 * 32 * 32).map(|_| rng.below(256) as u8).collect();
        let plain = CpuVit::new(&w, cfg).unwrap().forward(&views);
        assert!(plain.iter().all(|l| l.is_finite()) && plain.len() == 3);
        // Asking for nothing to be rounded is the engine proper, bit for bit.
        assert_eq!(CpuVit::with_int8(&w, cfg, None).unwrap().forward(&views), plain);
        let nothing = Int8 { weights: [false; 4], inputs: [false; 4], ranges: Ranges::Token, input_bits: 8, weight_bits: 8 };
        assert_eq!(CpuVit::with_int8(&w, cfg, Some(nothing)).unwrap().forward(&views), plain);
        let largest = |q: &str| {
            let got = CpuVit::with_int8(&w, cfg, Some(Int8::parse(q).unwrap())).unwrap().forward(&views);
            got.iter().zip(&plain).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max)
        };
        // Every variant moves the logits, by little at eight bits and by more at four.
        for (name, ..) in Int8::VARIANTS {
            for ranges in ["token", "tensor"] {
                let moved = largest(&format!("{name}:{ranges}"));
                assert!(moved > 0.0 && moved < 0.2, "{name}:{ranges} moved a logit by {moved}");
            }
        }
        assert!(largest("all:token:4") > largest("all:token:8"));
        assert!(largest("weights:token:8:4") > largest("weights"));
        // A view's ranges are its own: its logit does not depend on its neighbours.
        let q = Some(Int8::parse("all:tensor").unwrap());
        let vit = CpuVit::with_int8(&w, cfg, q).unwrap();
        let (together, alone) = (vit.forward(&views), vit.forward(&views[3 * 32 * 32..2 * 3 * 32 * 32]));
        assert!((together[1] - alone[0]).abs() < 1e-4, "{} against {}", together[1], alone[0]);
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
