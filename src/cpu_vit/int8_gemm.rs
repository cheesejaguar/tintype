//! An eight-bit matrix product and the linear layer around it, for
//! measuring only: compiled into the test binary, never into `tintype`, and
//! not used to score anything.
//!
//! It answers two questions the [`Int8`] simulation leaves open. Does an
//! integer kernel give the numbers the simulation says it would
//! (`an_integer_linear_layer_gives_what_the_simulation_gives`)? And how much
//! faster than the single-precision products is it, at this network's
//! shapes, on the machine at hand (`int8_against_f32_at_the_networks_shapes`,
//! ignored by default; see docs/runbooks/cpu-profile-int8.md)?
//!
//! The product is `c[m, n] = a[m, k] * w[n, k]^T` with `i8` operands and
//! `i32` sums. On ARM64 with the dot-product extension (every ARMv8.4 core,
//! and all Apple silicon) it is four rows against four rows, sixteen `SDOT`
//! accumulators; anywhere else, a plain loop. There is no x86-64 kernel
//! here: that one has to be written and timed on such a machine.
use super::*;

/// The definition, and the fallback where there is no kernel.
fn gemm_i8_scalar(a: &[i8], w: &[i8], c: &mut [i32], m: usize, n: usize, k: usize) {
    for i in 0..m {
        for j in 0..n {
            c[i * n + j] = a[i * k..][..k].iter().zip(&w[j * k..][..k]).map(|(&x, &y)| x as i32 * y as i32).sum();
        }
    }
}

/// Four rows of `a` against four rows of `w` at a time, sixteen bytes a
/// step. `w` is walked once and `a` once per four rows of `w`, so `a`
/// (the tokens of one view) is what stays in the cache.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon,dotprod")]
unsafe fn gemm_i8_dotprod(a: &[i8], w: &[i8], c: &mut [i32], m: usize, n: usize, k: usize) {
    use std::arch::aarch64::*;
    let wide = k / 16 * 16;
    let tail = |i: usize, j: usize| -> i32 { a[i * k + wide..(i + 1) * k].iter().zip(&w[j * k + wide..(j + 1) * k]).map(|(&x, &y)| x as i32 * y as i32).sum() };
    let (ap, wp) = (a.as_ptr(), w.as_ptr());
    let mut j = 0;
    while j + 4 <= n {
        let (w0, w1, w2, w3) = (wp.add(j * k), wp.add((j + 1) * k), wp.add((j + 2) * k), wp.add((j + 3) * k));
        let mut i = 0;
        while i + 4 <= m {
            let (a0, a1, a2, a3) = (ap.add(i * k), ap.add((i + 1) * k), ap.add((i + 2) * k), ap.add((i + 3) * k));
            let z = vdupq_n_s32(0);
            let (mut s00, mut s01, mut s02, mut s03) = (z, z, z, z);
            let (mut s10, mut s11, mut s12, mut s13) = (z, z, z, z);
            let (mut s20, mut s21, mut s22, mut s23) = (z, z, z, z);
            let (mut s30, mut s31, mut s32, mut s33) = (z, z, z, z);
            let mut p = 0;
            while p < wide {
                let (x0, x1, x2, x3) = (vld1q_s8(a0.add(p)), vld1q_s8(a1.add(p)), vld1q_s8(a2.add(p)), vld1q_s8(a3.add(p)));
                let (y0, y1, y2, y3) = (vld1q_s8(w0.add(p)), vld1q_s8(w1.add(p)), vld1q_s8(w2.add(p)), vld1q_s8(w3.add(p)));
                s00 = vdotq_s32(s00, x0, y0);
                s01 = vdotq_s32(s01, x0, y1);
                s02 = vdotq_s32(s02, x0, y2);
                s03 = vdotq_s32(s03, x0, y3);
                s10 = vdotq_s32(s10, x1, y0);
                s11 = vdotq_s32(s11, x1, y1);
                s12 = vdotq_s32(s12, x1, y2);
                s13 = vdotq_s32(s13, x1, y3);
                s20 = vdotq_s32(s20, x2, y0);
                s21 = vdotq_s32(s21, x2, y1);
                s22 = vdotq_s32(s22, x2, y2);
                s23 = vdotq_s32(s23, x2, y3);
                s30 = vdotq_s32(s30, x3, y0);
                s31 = vdotq_s32(s31, x3, y1);
                s32 = vdotq_s32(s32, x3, y2);
                s33 = vdotq_s32(s33, x3, y3);
                p += 16;
            }
            let sums = [[s00, s01, s02, s03], [s10, s11, s12, s13], [s20, s21, s22, s23], [s30, s31, s32, s33]];
            for (r, row) in sums.iter().enumerate() {
                for (s, &acc) in row.iter().enumerate() {
                    c[(i + r) * n + j + s] = vaddvq_s32(acc) + if wide < k { tail(i + r, j + s) } else { 0 };
                }
            }
            i += 4;
        }
        // The last one to three rows of `a`, one at a time against the four rows of `w`.
        while i < m {
            let a0 = ap.add(i * k);
            let z = vdupq_n_s32(0);
            let (mut s0, mut s1, mut s2, mut s3) = (z, z, z, z);
            let mut p = 0;
            while p < wide {
                let x = vld1q_s8(a0.add(p));
                s0 = vdotq_s32(s0, x, vld1q_s8(w0.add(p)));
                s1 = vdotq_s32(s1, x, vld1q_s8(w1.add(p)));
                s2 = vdotq_s32(s2, x, vld1q_s8(w2.add(p)));
                s3 = vdotq_s32(s3, x, vld1q_s8(w3.add(p)));
                p += 16;
            }
            for (s, &acc) in [s0, s1, s2, s3].iter().enumerate() {
                c[i * n + j + s] = vaddvq_s32(acc) + if wide < k { tail(i, j + s) } else { 0 };
            }
            i += 1;
        }
        j += 4;
    }
    // The last one to three rows of `w`.
    for j in j..n {
        for i in 0..m {
            c[i * n + j] = a[i * k..][..k].iter().zip(&w[j * k..][..k]).map(|(&x, &y)| x as i32 * y as i32).sum();
        }
    }
}

/// Which kernel [`gemm_i8`] runs here.
fn kernel_name() -> &'static str {
    #[cfg(target_arch = "aarch64")]
    if std::arch::is_aarch64_feature_detected!("dotprod") {
        return "NEON dot product (SDOT), four rows by four";
    }
    "scalar loop (no integer kernel for this processor)"
}

/// `c[m, n] = a[m, k] * w[n, k]^T` on the calling thread.
fn gemm_i8(a: &[i8], w: &[i8], c: &mut [i32], m: usize, n: usize, k: usize) {
    assert!(a.len() >= m * k && w.len() >= n * k && c.len() >= m * n);
    #[cfg(target_arch = "aarch64")]
    if std::arch::is_aarch64_feature_detected!("dotprod") {
        // The feature was just detected, and the lengths were checked above.
        return unsafe { gemm_i8_dotprod(a, w, c, m, n, k) };
    }
    gemm_i8_scalar(a, w, c, m, n, k)
}

/// A linear layer as an integer kernel would hold it: weights as signed
/// bytes with a step per output channel, as [`fake_quantise_weights`] rounds
/// them.
struct LinI8 {
    v: Vec<i8>,
    /// Per output channel: the step, and the step times the sum of the channel's bytes.
    step: Vec<f32>,
    step_sum: Vec<f32>,
    b: Vec<f32>,
    out: usize,
    inp: usize,
}

/// Buffers one thread needs to apply integer layers.
#[derive(Default)]
struct Scratch {
    q: Vec<i8>,
    sums: Vec<i32>,
    /// Per token: the step of its input, and `128 * step + lowest value`.
    scale: Vec<[f32; 2]>,
}

impl LinI8 {
    fn new(lin: &Lin) -> Self {
        let mut v = vec![0i8; lin.w.len()];
        let (mut step, mut step_sum) = (vec![0f32; lin.out], vec![0f32; lin.out]);
        for (j, (row, bytes)) in lin.w.chunks(lin.inp).zip(v.chunks_mut(lin.inp)).enumerate() {
            let max = row.iter().fold(0f32, |a, &x| a.max(x.abs()));
            if max == 0.0 {
                continue;
            }
            step[j] = max / 127.0;
            for (b, &x) in bytes.iter_mut().zip(row) {
                *b = (x / step[j]).round().clamp(-127.0, 127.0) as i8;
            }
            step_sum[j] = step[j] * bytes.iter().map(|&b| b as i32).sum::<i32>() as f32;
        }
        Self { v, step, step_sum, b: lin.b.clone(), out: lin.out, inp: lin.inp }
    }

    /// Each token of `x` to 256 levels between its smallest and largest
    /// value, stored as the level minus 128 so that it fits a signed byte.
    fn quantise(&self, x: &[f32], rows: usize, s: &mut Scratch) {
        s.q.resize(rows * self.inp, 0);
        s.scale.resize(rows, [0.0; 2]);
        for ((row, q), scale) in x.chunks(self.inp).zip(s.q.chunks_mut(self.inp)).zip(s.scale.iter_mut()) {
            let (lo, hi) = row.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &v| (lo.min(v), hi.max(v)));
            let step = (hi - lo) / 255.0;
            let inv = if step > 0.0 { 1.0 / step } else { 0.0 };
            for (o, &v) in q.iter_mut().zip(row) {
                // Never negative, so adding a half and truncating rounds to nearest.
                *o = (((v - lo) * inv + 0.5) as i32 - 128) as i8;
            }
            *scale = [step, 128.0 * step + lo];
        }
    }

    /// `y[rows, out] = x[rows, in] W^T + b` through bytes: round the input,
    /// multiply in integers, and scale the sums back
    /// (docs/runbooks/cpu-profile-int8.md, "What an integer kernel would compute").
    fn apply(&self, x: &[f32], y: &mut [f32], rows: usize, s: &mut Scratch) {
        self.quantise(x, rows, s);
        s.sums.resize(rows * self.out, 0);
        gemm_i8(&s.q, &self.v, &mut s.sums, rows, self.out, self.inp);
        for ((out, sums), &[step, offset]) in y.chunks_mut(self.out).zip(s.sums.chunks(self.out)).zip(&s.scale) {
            for j in 0..self.out {
                out[j] = self.b[j] + self.step_sum[j] * offset + self.step[j] * step * sums[j] as f32;
            }
        }
    }

    /// Only the integer product of [`LinI8::apply`], on an input rounded before.
    fn product(&self, rows: usize, s: &mut Scratch) {
        s.sums.resize(rows * self.out, 0);
        gemm_i8(&s.q, &self.v, &mut s.sums, rows, self.out, self.inp);
    }
}

fn random_lin(rng: &mut crate::rng::Rng, out: usize, inp: usize) -> Lin {
    Lin { w: (0..out * inp).map(|_| rng.range(-0.2, 0.2) * rng.f32()).collect(), b: (0..out).map(|_| rng.range(-0.5, 0.5)).collect(), out, inp }
}

#[test]
fn the_integer_product_matches_the_definition() {
    let mut rng = crate::rng::Rng::new(3);
    // Sizes that are and are not multiples of the kernel's four rows and sixteen bytes, and the network's own.
    for (m, n, k) in [(1, 1, 1), (4, 4, 16), (7, 9, 37), (5, 8, 48), (13, 6, 100), (197, 12, 384), (3, 384, 1536)] {
        let a: Vec<i8> = (0..m * k).map(|_| rng.below(256) as u8 as i8).collect();
        let w: Vec<i8> = (0..n * k).map(|_| (rng.below(255) as i32 - 127) as i8).collect();
        let (mut got, mut want) = (vec![0i32; m * n], vec![0i32; m * n]);
        gemm_i8(&a, &w, &mut got, m, n, k);
        gemm_i8_scalar(&a, &w, &mut want, m, n, k);
        assert_eq!(got, want, "{m} x {n} x {k}");
    }
    // The extremes stay inside 32 bits at the network's longest row.
    let (a, w) = (vec![-128i8; 1536], vec![-127i8; 1536]);
    let mut c = [0i32];
    gemm_i8(&a, &w, &mut c, 1, 1, 1536);
    assert_eq!(c[0], 128 * 127 * 1536);
}

#[test]
fn an_integer_linear_layer_gives_what_the_simulation_gives() {
    let mut rng = crate::rng::Rng::new(9);
    for (rows, out, inp) in [(9, 24, 48), (197, 40, 384), (5, 12, 1536)] {
        let lin = random_lin(&mut rng, out, inp);
        // Lopsided inputs, like the values after GELU.
        let x: Vec<f32> = (0..rows * inp).map(|_| (rng.range(-2.0, 2.0)).exp() * 0.3 - 0.17).collect();
        let mut got = vec![0f32; rows * out];
        LinI8::new(&lin).apply(&x, &mut got, rows, &mut Scratch::default());
        // The simulation: round weights per channel and inputs per token, multiply in single precision.
        let mut sim = Lin { w: lin.w.clone(), b: lin.b.clone(), out, inp };
        fake_quantise_weights(&mut sim.w, inp);
        let mut xq = x.clone();
        xq.chunks_mut(inp).for_each(|row| fake_quantise_inputs(row, 255.0));
        let mut want = vec![0f32; rows * out];
        sim.apply(&xq, &mut want, rows);
        let size = want.iter().fold(0f32, |a, v| a.max(v.abs()));
        let apart = got.iter().zip(&want).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max);
        assert!(apart <= size * 2e-5, "{rows} x {out} x {inp}: {apart} apart at size {size}");
        // And both are close to, but not the same as, the layer in full precision.
        let mut full = vec![0f32; rows * out];
        lin.apply(&x, &mut full, rows);
        let moved = got.iter().zip(&full).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max);
        assert!(moved > apart && moved < size * 0.05, "{rows} x {out} x {inp}: moved {moved} at size {size}");
    }
}

/// The linear layers of the eleven full blocks of one view (`[out, in]` for
/// qkv, proj, fc1, fc2), each block with weights of its own, as in the
/// network: 78 MB in single precision, 19.5 MB in bytes.
const SHAPES: [(&str, usize, usize); 4] = [("qkv", 1152, 384), ("proj", 384, 384), ("fc1", 1536, 384), ("fc2", 384, 1536)];

/// Times the linear layers of a view three ways (single precision as the
/// engine runs them, the integer product alone, and the integer layer with
/// its rounding and scaling) in the two situations the engine has: one view
/// with every core on it, and many views at once.
///
/// `cargo test --release --locked --lib int8_against_f32 -- --ignored --nocapture`,
/// and the same with `--features portable-cpu` for the pure-Rust kernels.
#[test]
#[ignore = "a benchmark: run it alone, with --nocapture"]
fn int8_against_f32_at_the_networks_shapes() {
    const TOKENS: usize = 197;
    const BLOCKS: usize = 11;
    let views: usize = std::env::var("TINTYPE_BENCH_VIEWS").ok().and_then(|v| v.parse().ok()).unwrap_or(48);
    let passes: usize = std::env::var("TINTYPE_BENCH_PASSES").ok().and_then(|v| v.parse().ok()).unwrap_or(7);
    let mut rng = crate::rng::Rng::new(1);
    let f32_layers: Vec<Vec<Lin>> = (0..BLOCKS).map(|_| SHAPES.iter().map(|&(_, out, inp)| random_lin(&mut rng, out, inp)).collect()).collect();
    let i8_layers: Vec<Vec<LinI8>> = f32_layers.iter().map(|b| b.iter().map(LinI8::new).collect()).collect();
    // Inputs: one view's tokens at each width, lopsided like activations.
    let input = |rng: &mut crate::rng::Rng, width: usize, rows: usize| -> Vec<f32> { (0..rows * width).map(|_| rng.range(-2.0, 2.0).exp() * 0.3 - 0.17).collect() };
    let (x384, x1536) = (input(&mut rng, 384, TOKENS), input(&mut rng, 1536, TOKENS));
    let of = |inp: usize| if inp == 384 { &x384 } else { &x1536 };
    let threads = rayon::current_num_threads();
    let best = |f: &mut dyn FnMut()| -> (f64, f64) {
        f();
        let mut ms: Vec<f64> = (0..passes).map(|_| { let t = Instant::now(); f(); t.elapsed().as_secs_f64() * 1e3 }).collect();
        ms.sort_by(|a, b| a.total_cmp(b));
        (ms[0], ms[ms.len() / 2])
    };
    let mac: f64 = SHAPES.iter().map(|&(_, out, inp)| (TOKENS * out * inp) as f64).sum::<f64>() * BLOCKS as f64;
    println!("kernel: {}; f32 products: {}; {threads} threads; {views} views at once; best and median of {passes} passes", kernel_name(), if ONE_VIEW_PER_THREAD { "pure-Rust `gemm` crate" } else { "Accelerate" });
    println!("one view's block linear layers: {:.2} billion multiply-adds, weights {:.1} MB as f32 and {:.1} MB as bytes", mac / 1e9, f32_layers.iter().flatten().map(|l| l.w.len() * 4).sum::<usize>() as f64 / 1e6, i8_layers.iter().flatten().map(|l| l.v.len()).sum::<usize>() as f64 / 1e6);

    // 1. One view, every core on it (the single-image path).
    // Single precision as the engine does it: the product splits itself over the threads.
    let mut y: Vec<Vec<f32>> = SHAPES.iter().map(|&(_, out, _)| vec![0f32; TOKENS * out]).collect();
    let mut per_layer = [[0f64; 3]; 4];
    for (l, &(_, _, inp)) in SHAPES.iter().enumerate() {
        per_layer[l][0] = best(&mut || f32_layers.iter().for_each(|b| b[l].apply(of(inp), &mut y[l], TOKENS))).0;
    }
    let one_f32 = best(&mut || f32_layers.iter().for_each(|b| b.iter().zip(y.iter_mut()).for_each(|(lin, y)| lin.apply(of(lin.inp), y, TOKENS))));
    // Integers: the tokens are split into runs of eight, each run one task.
    let rows_per_task = 8;
    let split = |lin: &LinI8, x: &[f32], y: &mut [f32], whole: bool| {
        y.par_chunks_mut(rows_per_task * lin.out).zip(x.par_chunks(rows_per_task * lin.inp)).for_each_init(Scratch::default, |s, (y, x)| {
            let rows = x.len() / lin.inp;
            if whole {
                lin.apply(x, y, rows, s);
            } else {
                // The product alone: the input was rounded once before timing, per task it is rounded again only to fill the buffer.
                s.q.resize(rows * lin.inp, 1);
                lin.product(rows, s);
            }
        });
    };
    for (l, &(_, _, inp)) in SHAPES.iter().enumerate() {
        per_layer[l][1] = best(&mut || i8_layers.iter().for_each(|b| split(&b[l], of(inp), &mut y[l], false))).0;
        per_layer[l][2] = best(&mut || i8_layers.iter().for_each(|b| split(&b[l], of(inp), &mut y[l], true))).0;
    }
    let one_product = best(&mut || i8_layers.iter().for_each(|b| b.iter().zip(y.iter_mut()).for_each(|(lin, y)| split(lin, of(lin.inp), y, false))));
    let one_whole = best(&mut || i8_layers.iter().for_each(|b| b.iter().zip(y.iter_mut()).for_each(|(lin, y)| split(lin, of(lin.inp), y, true))));
    println!("\n| One view, all threads | f32 | INT8 product alone | INT8 layer (rounding, product, scaling) | f32 / INT8 layer |");
    println!("|---|---:|---:|---:|---:|");
    for (l, &(name, out, inp)) in SHAPES.iter().enumerate() {
        println!("| {name} ({TOKENS} x {inp} by {inp} x {out}), {BLOCKS} blocks | {:.2} ms | {:.2} ms | {:.2} ms | {:.2}x |", per_layer[l][0], per_layer[l][1], per_layer[l][2], per_layer[l][0] / per_layer[l][2]);
    }
    println!("| all four, {BLOCKS} blocks (best) | {:.2} ms | {:.2} ms | {:.2} ms | {:.2}x |", one_f32.0, one_product.0, one_whole.0, one_f32.0 / one_whole.0);
    println!("| all four, {BLOCKS} blocks (median) | {:.2} ms | {:.2} ms | {:.2} ms | {:.2}x |", one_f32.1, one_product.1, one_whole.1, one_f32.1 / one_whole.1);
    println!("| billion multiply-adds per second (best) | {:.0} | {:.0} | {:.0} | |", mac / one_f32.0 / 1e6, mac / one_product.0 / 1e6, mac / one_whole.0 / 1e6);

    // 2. Many views at once (the throughput path).
    // Single precision as the engine does it: one product over all rows with
    // Accelerate, one view per thread with the pure-Rust kernels.
    let many_f32 = if ONE_VIEW_PER_THREAD {
        best(&mut || {
            (0..views).into_par_iter().for_each_init(
                || SHAPES.iter().map(|&(_, out, _)| vec![0f32; TOKENS * out]).collect::<Vec<_>>(),
                |y, _| on_one_thread(|| f32_layers.iter().for_each(|b| b.iter().zip(y.iter_mut()).for_each(|(lin, y)| lin.apply(of(lin.inp), y, TOKENS)))),
            )
        })
    } else {
        let rows = TOKENS * views;
        let (big384, big1536) = (x384.repeat(views), x1536.repeat(views));
        let mut y: Vec<Vec<f32>> = SHAPES.iter().map(|&(_, out, _)| vec![0f32; rows * out]).collect();
        best(&mut || f32_layers.iter().for_each(|b| b.iter().zip(y.iter_mut()).for_each(|(lin, y)| lin.apply(if lin.inp == 384 { &big384 } else { &big1536 }, y, rows))))
    };
    let many = |whole: bool| {
        best(&mut || {
            (0..views).into_par_iter().for_each_init(
                || (Scratch::default(), SHAPES.iter().map(|&(_, out, _)| vec![0f32; TOKENS * out]).collect::<Vec<_>>()),
                |(s, y), _| {
                    for b in &i8_layers {
                        for (lin, y) in b.iter().zip(y.iter_mut()) {
                            if whole {
                                lin.apply(of(lin.inp), y, TOKENS, s);
                            } else {
                                s.q.resize(TOKENS * lin.inp, 1);
                                lin.product(TOKENS, s);
                            }
                        }
                    }
                },
            )
        })
    };
    let (many_product, many_whole) = (many(false), many(true));
    println!("\n| {views} views at once, all threads | f32 | INT8 product alone | INT8 layer | f32 / INT8 layer |");
    println!("|---|---:|---:|---:|---:|");
    for (name, pick) in [("best", 0usize), ("median", 1)] {
        let p = |t: (f64, f64)| if pick == 0 { t.0 } else { t.1 } / views as f64;
        println!("| all four, {BLOCKS} blocks, per view ({name}) | {:.2} ms | {:.2} ms | {:.2} ms | {:.2}x |", p(many_f32), p(many_product), p(many_whole), p(many_f32) / p(many_whole));
    }
    println!("| billion multiply-adds per second (best) | {:.0} | {:.0} | {:.0} | |", mac * views as f64 / many_f32.0 / 1e6, mac * views as f64 / many_product.0 / 1e6, mac * views as f64 / many_whole.0 / 1e6);

    // 3. One thread, one view (what one view costs a thread that has it to itself).
    let mut s = Scratch::default();
    let single_whole = best(&mut || i8_layers.iter().for_each(|b| b.iter().zip(y.iter_mut()).for_each(|(lin, y)| lin.apply(of(lin.inp), y, TOKENS, &mut s))));
    // Accelerate picks its own threads unless VECLIB_MAXIMUM_THREADS=1 holds it to one.
    let held = ONE_VIEW_PER_THREAD || std::env::var("VECLIB_MAXIMUM_THREADS").is_ok_and(|v| v == "1");
    let single_f32 = held.then(|| best(&mut || on_one_thread(|| f32_layers.iter().for_each(|b| b.iter().zip(y.iter_mut()).for_each(|(lin, y)| lin.apply(of(lin.inp), y, TOKENS))))));
    println!("\n| One view on one thread | f32 | INT8 layer | f32 / INT8 layer |");
    println!("|---|---:|---:|---:|");
    match single_f32 {
        Some(f) => println!("| all four, {BLOCKS} blocks (best) | {:.2} ms | {:.2} ms | {:.2}x |", f.0, single_whole.0, f.0 / single_whole.0),
        None => println!("| all four, {BLOCKS} blocks (best) | (run with VECLIB_MAXIMUM_THREADS=1) | {:.2} ms | |", single_whole.0),
    }
    println!("| billion multiply-adds per second | {} | {:.0} | |", single_f32.map_or("".to_string(), |f| format!("{:.0}", mac / f.0 / 1e6)), mac / single_whole.0 / 1e6);
}
