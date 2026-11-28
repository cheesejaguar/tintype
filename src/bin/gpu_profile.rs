//! Where the GPU time of the ViT-S/16 networks goes: the network as shipped,
//! the same network with one part changed or taken out, and matrix
//! multiplication on its own, timed on compiled MPSGraph executables.
//!
//! Usage: gpu_profile WEIGHTS IMG PREFIX BATCH [RUNS]
//!   e.g. gpu_profile models/tintype.safetensors 224 "" 64
//!        gpu_profile models/commfor-384.safetensors 384 vit. 64
//!
//! For measurement only: the variants other than `shipped` and `sdpa` do not
//! compute the network.
#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("this needs the Apple GPU engine");
}

#[cfg(target_os = "macos")]
mod run {
    use std::time::Instant;
    use tintype::mps::{Graph, Precision, T};
    use tintype::mps_vit::{build_head, build_vit_cls_with, Shared};
    use tintype::vit::VitConfig;
    use tintype::weights::{transpose2, Weights};

    #[derive(Clone, Copy, PartialEq)]
    pub enum Attn {
        Composed,
        Sdpa,
        Skip,
    }

    #[derive(Clone, Copy)]
    pub struct Var {
        pub name: &'static str,
        pub attn: Attn,
        pub gelu: bool,
        pub norm: bool,
        pub mlp: bool,
        /// Move data as the shipped network does around attention (false:
        /// reshape only, the same shapes without the copies).
        pub moves: bool,
    }

    fn linear(g: &Graph, w: &Weights, x: &T, name: &str) -> T {
        let (d, s) = w.get(&format!("{name}.weight")).unwrap();
        let (o, i) = (s[0], s[1..].iter().product::<usize>());
        let wt = g.constant(&transpose2(d, o, i), &[i as isize, o as isize]);
        let (bd, _) = w.get(&format!("{name}.bias")).unwrap();
        let b = g.constant(bd, &[o as isize]);
        g.add(&g.matmul(x, &wt), &b)
    }

    fn norm(g: &Graph, w: &Weights, x: &T, name: &str, on: bool) -> T {
        if !on {
            return x.clone();
        }
        let gm = g.constant(&w.get(&format!("{name}.weight")).unwrap().0, &[384]);
        let bt = g.constant(&w.get(&format!("{name}.bias")).unwrap().0, &[384]);
        g.layer_norm(x, &gm, &bt, 1e-6, 3)
    }

    fn attention(g: &Graph, q: &T, k: &T, v: &T, hd: isize, how: Attn) -> T {
        let scale = (hd as f64).powf(-0.5);
        match how {
            Attn::Composed => {
                let a = g.softmax(&g.scale(&g.matmul(q, &g.transpose(k, 2, 3)), scale), 3);
                g.matmul(&a, v)
            }
            Attn::Sdpa => unsafe { g.g.scaledDotProductAttentionWithQueryTensor_keyTensor_valueTensor_scale_name(q, k, v, scale as f32, None) },
            Attn::Skip => v.clone(),
        }
    }

    /// The class-token network of `mps_vit::build` (last block for the class
    /// token only), with parts switched by `var`.
    pub fn build(g: &Graph, w: &Weights, cfg: &VitConfig, x: &T, batch: usize, var: Var) -> T {
        let (b, d) = (batch as isize, cfg.dim as isize);
        let (ps, side) = (cfg.patch as isize, (cfg.img / cfg.patch) as isize);
        let n = side * side;
        let (h, hd) = (cfg.heads as isize, (cfg.dim / cfg.heads) as isize);
        let x = g.reshape(x, &[b, 3, side, ps, side, ps]);
        let x = g.permute(&x, &[0, 2, 4, 1, 3, 5]);
        let x = g.reshape(&x, &[b, n, 3 * ps * ps]);
        let x = linear(g, w, &x, "patch_embed.proj");
        let cls = g.constant(&w.get("cls_token").unwrap().0, &[1, 1, d]);
        let pos = g.constant(&w.get("pos_embed").unwrap().0, &[1, n + 1, d]);
        let zeros = g.constant(&vec![0f32; batch * cfg.dim], &[b, 1, d]);
        let mut x = g.add(&g.concat(&[g.add(&cls, &zeros), x], 1), &pos);
        let nt = n + 1;
        for i in 0..cfg.depth {
            let pre = format!("blocks.{i}");
            let y = norm(g, w, &x, &format!("{pre}.norm1"), var.norm);
            let last = i + 1 == cfg.depth;
            let heads = |t: &T, n: isize| g.transpose(&g.reshape(t, &[b, n, h, hd]), 1, 2);
            let (q, k, v, rows) = if last {
                let qkv = linear(g, w, &y, &format!("{pre}.attn.qkv"));
                let q = heads(&g.slice(&g.slice(&qkv, 1, 0, 1), 2, 0, d), 1);
                (q, heads(&g.slice(&qkv, 2, d, d), nt), heads(&g.slice(&qkv, 2, 2 * d, d), nt), 1)
            } else {
                let qkv = linear(g, w, &y, &format!("{pre}.attn.qkv"));
                let qkv = if var.moves { g.permute(&g.reshape(&qkv, &[b, nt, 3, h, hd]), &[2, 0, 3, 1, 4]) } else { g.reshape(&qkv, &[3, b, h, nt, hd]) };
                let part = |k: isize| g.reshape(&g.slice(&qkv, 0, k, 1), &[b, h, nt, hd]);
                (part(0), part(1), part(2), nt)
            };
            let v = if var.attn == Attn::Skip && last { g.slice(&v, 2, 0, 1) } else { v };
            let a = attention(g, &q, &k, &v, hd, var.attn);
            let y = if var.moves || last { g.reshape(&g.transpose(&a, 1, 2), &[b, rows, d]) } else { g.reshape(&a, &[b, rows, d]) };
            let base = if last { g.slice(&x, 1, 0, 1) } else { x.clone() };
            let x1 = g.add(&base, &linear(g, w, &y, &format!("{pre}.attn.proj")));
            x = if var.mlp {
                let y = norm(g, w, &x1, &format!("{pre}.norm2"), var.norm);
                let y = linear(g, w, &y, &format!("{pre}.mlp.fc1"));
                let y = if var.gelu { g.gelu(&y) } else { y };
                g.add(&x1, &linear(g, w, &y, &format!("{pre}.mlp.fc2")))
            } else {
                x1
            };
        }
        let t = norm(g, w, &x, "norm", var.norm);
        let cls = g.reshape(&g.slice(&t, 1, 0, 1), &[b, d]);
        g.cast_out(&linear(g, w, &cls, "head"))
    }

    fn feed(prec: Precision, n: usize) -> Vec<u8> {
        let mut s = 0x9e37_79b9_7f4a_7c15u64;
        let mut out = Vec::with_capacity(n * 4);
        for _ in 0..n {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let v = ((s >> 40) as f32 / (1u64 << 24) as f32) * 4.0 - 2.0;
            match prec {
                Precision::F16 => out.extend_from_slice(&half::f16::from_f32(v).to_bits().to_ne_bytes()),
                Precision::F32 => out.extend_from_slice(&v.to_ne_bytes()),
            }
        }
        out
    }

    /// Milliseconds per run (median of `rounds` timings of `runs` runs each)
    /// and the first output.
    pub fn time(g: &Graph, inp: &T, shape: &[isize], out: &T, runs: usize) -> (f64, Vec<f32>) {
        let exe = g.compile_native(inp, shape, &[out.clone()]);
        let data = feed(g.prec, shape.iter().product::<isize>() as usize);
        let first = g.run_executable(&exe, &data).remove(0);
        for _ in 0..3 {
            g.run_executable(&exe, &data);
        }
        let mut t: Vec<f64> = (0..5)
            .map(|_| {
                let s = Instant::now();
                for _ in 0..runs {
                    g.run_executable(&exe, &data);
                }
                s.elapsed().as_secs_f64() * 1e3 / runs as f64
            })
            .collect();
        t.sort_by(f64::total_cmp);
        (t[2], first)
    }

    /// FLOPs (a multiply-add counted as two) of one view through the
    /// class-token network: (linear layers, attention products).
    pub fn flops(cfg: &VitConfig) -> (f64, f64) {
        let (d, f) = (cfg.dim as f64, (cfg.dim * cfg.mlp_ratio) as f64);
        let n = ((cfg.img / cfg.patch).pow(2) + 1) as f64;
        let hd = (cfg.dim / cfg.heads) as f64;
        let h = cfg.heads as f64;
        let full = cfg.depth as f64 - 1.0;
        let lin_block = 2.0 * n * d * (3.0 * d + d + 2.0 * f);
        let att_block = 2.0 * 2.0 * h * n * n * hd;
        let last_lin = 2.0 * n * d * 2.0 * d + 2.0 * d * (d + d + 2.0 * f);
        let last_att = 2.0 * 2.0 * h * n * hd;
        let patch = 2.0 * (n - 1.0) * (3 * cfg.patch * cfg.patch) as f64 * d;
        (full * lin_block + last_lin + patch, full * att_block + last_att)
    }

    pub fn main() -> anyhow::Result<()> {
        let a: Vec<String> = std::env::args().collect();
        let (path, img, prefix, batch) = (&a[1], a[2].parse::<usize>()?, &a[3], a[4].parse::<usize>()?);
        let runs: usize = a.get(5).map_or(Ok(20), |r| r.parse())?;
        let w = Weights::load(path, prefix)?;
        let cfg = VitConfig::small(img);
        let (lin, att) = flops(&cfg);
        let per_run = (lin + att) * batch as f64;
        println!("ViT-S/16 at {img}: {:.2} GFLOP per view ({:.2} in linear layers, {:.2} in attention products); {:.0} GFLOP per {batch}-view run", (lin + att) / 1e9, lin / 1e9, att / 1e9, per_run / 1e9);
        let s = img as isize;
        let shape = [batch as isize, 3, s, s];
        // The network as shipped (mps_vit), for checking the variants against.
        let mut shipped = Vec::new();
        for prec in [Precision::F16, Precision::F32] {
            let g = Graph::new(prec)?;
            let inp = g.placeholder_native(&shape);
            let v = build_vit_cls_with(&g, &w, &mut Shared::default(), &cfg, &inp, batch)?;
            let out = g.cast_out(&build_head(&g, &w, &mut Shared::default(), &v.cls)?);
            let (ms, first) = time(&g, &inp, &shape, &out, runs);
            println!("shipped {prec:?}: {ms:.2} ms per run, {:.2} TFLOPS, {:.0} views/s", per_run / ms / 1e9, batch as f64 * 1e3 / ms);
            shipped.push(first);
        }
        let variants = [
            Var { name: "as shipped (rebuilt)", attn: Attn::Composed, gelu: true, norm: true, mlp: true, moves: true },
            Var { name: "fused attention (SDPA)", attn: Attn::Sdpa, gelu: true, norm: true, mlp: true, moves: true },
            Var { name: "no attention core", attn: Attn::Skip, gelu: true, norm: true, mlp: true, moves: true },
            Var { name: "no GELU", attn: Attn::Composed, gelu: false, norm: true, mlp: true, moves: true },
            Var { name: "no layer norm", attn: Attn::Composed, gelu: true, norm: false, mlp: true, moves: true },
            Var { name: "linear layers only", attn: Attn::Skip, gelu: false, norm: false, mlp: true, moves: true },
            Var { name: "no MLP", attn: Attn::Composed, gelu: true, norm: true, mlp: false, moves: true },
            Var { name: "no head transposes", attn: Attn::Composed, gelu: true, norm: true, mlp: true, moves: false },
        ];
        let g = Graph::new(Precision::F16)?;
        for var in variants {
            let inp = g.placeholder_native(&shape);
            let out = build(&g, &w, &cfg, &inp, batch, var);
            let (ms, first) = time(&g, &inp, &shape, &out, runs);
            let dmax = first.iter().zip(&shipped[0]).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max);
            let same = first.iter().zip(&shipped[0]).all(|(a, b)| a.to_bits() == b.to_bits());
            println!("F16 {:<24} {ms:7.2} ms per run   max |logit - shipped F16| {dmax:.3e}{}", var.name, if same { " (bit for bit)" } else { "" });
        }
        for prec in [Precision::F16, Precision::F32] {
            let g = Graph::new(prec)?;
            let k = 4096isize;
            let inp = g.placeholder_native(&[k, k]);
            let wt = g.constant(&vec![0.01f32; (k * k) as usize], &[k, k]);
            let out = g.cast_out(&g.sum(&g.matmul(&inp, &wt), &[0, 1]));
            let (ms, _) = time(&g, &inp, &[k, k], &out, 5);
            println!("matmul 4096^3 {prec:?}: {ms:.2} ms, {:.2} TFLOPS", 2.0 * (k as f64).powi(3) / ms / 1e9);
            let m = (batch * ((img / 16).pow(2) + 1)) as isize;
            let inp = g.placeholder_native(&[m, 384]);
            let mut x = inp.clone();
            for _ in 0..6 {
                let w1 = g.constant(&vec![0.001f32; 384 * 1536], &[384, 1536]);
                let w2 = g.constant(&vec![0.001f32; 1536 * 384], &[1536, 384]);
                x = g.matmul(&g.matmul(&x, &w1), &w2);
            }
            let out = g.cast_out(&g.sum(&x, &[0, 1]));
            let (ms, _) = time(&g, &inp, &[m, 384], &out, 10);
            println!("MLP-shaped matmuls ({m}x384 -> 1536 -> 384, six times) {prec:?}: {ms:.2} ms, {:.2} TFLOPS", 12.0 * 2.0 * m as f64 * 384.0 * 1536.0 / ms / 1e9);
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    run::main()
}
