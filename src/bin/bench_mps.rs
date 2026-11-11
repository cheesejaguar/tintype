#[cfg(target_os = "macos")]
use std::time::Instant;
#[cfg(target_os = "macos")]
use tintype::mps::{Graph, Precision};
#[cfg(target_os = "macos")]
use tintype::mps_vit::{build_head, build_vit, Frozen};
#[cfg(target_os = "macos")]
use tintype::vit::VitConfig;
#[cfg(target_os = "macos")]
use tintype::weights::Weights;

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("this check needs the Apple GPU engine");
}

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let (path, res) = (&a[1], a[2].parse::<usize>()?);
    let w = Weights::load(path, "vit.")?;
    for prec in [Precision::F32, Precision::F16] {
        for bs in [1usize, 16, 64] {
            let g = Graph::new(prec)?;
            let shape = [bs as isize, 3, res as isize, res as isize];
            let inp = g.placeholder(&shape);
            let out = build_vit(&g, &w, &mut Frozen, &VitConfig::small(res), &g.cast_in(&inp), bs)?;
            let logit = g.cast_out(&build_head(&g, &w, &mut Frozen, &out.cls)?);
            let n = 3 * res * res;
            let probe: Vec<f32> = (0..n * bs).map(|i| (((i % n) % 251) as f32 / 125.0) - 1.0).collect();
            let r = g.run(&[(&inp, &probe, &shape)], &[logit.clone()], &[]);
            for _ in 0..3 { g.run(&[(&inp, &probe, &shape)], &[logit.clone()], &[]); }
            let iters = (640 / bs).max(8);
            let t = Instant::now();
            for _ in 0..iters { g.run(&[(&inp, &probe, &shape)], &[logit.clone()], &[]); }
            println!("mpsgraph vit-s/16@{res} {prec:?} bs={bs}: {:.1} img/s  (probe logit {:.5})", (bs * iters) as f64 / t.elapsed().as_secs_f64(), r[0][0]);
        }
    }
    Ok(())
}
