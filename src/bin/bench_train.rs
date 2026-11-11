// Smoke test: overfit a fixed random batch, check loss falls, export round-trips.
#[cfg(target_os = "macos")]
use std::time::Instant;
#[cfg(target_os = "macos")]
use tintype::trainer::{OptConfig, Trainer};
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
    let (path, res, bs) = (&a[1], a[2].parse::<usize>()?, a[3].parse::<usize>()?);
    let w = Weights::load(path, "vit.")?;
    let mut t = Trainer::new(&w, VitConfig::small(res), bs, OptConfig::default())?;
    let mut rng = tintype::rng::Rng::new(1);
    let x: Vec<f32> = (0..bs * 3 * res * res).map(|_| rng.range(-1.0, 1.0)).collect();
    let y: Vec<f32> = (0..bs).map(|i| (i % 2) as f32).collect();
    let l0 = t.logits(&x);
    println!("initial logits[0..4] = {:?}", &l0[..4]);
    let exp0 = t.export();
    let (d0, _) = exp0.get("blocks.3.attn.qkv.weight")?;
    let (d1, _) = w.get("blocks.3.attn.qkv.weight")?;
    println!("export round-trip max abs diff = {}", d0.iter().zip(d1).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max));
    let start = Instant::now();
    for i in 0..30 {
        let loss = t.train_step(&x, &y, 1e-4);
        if i % 5 == 0 || i == 29 { println!("step {i:>3} loss {loss:.4}"); }
        if i == 4 { println!("  (first 5 steps incl. compile: {:.1}s)", start.elapsed().as_secs_f64()); }
    }
    let s = Instant::now();
    for _ in 0..20 { t.train_step(&x, &y, 1e-4); }
    println!("train throughput: {:.1} img/s at bs={bs}", (20 * bs) as f64 / s.elapsed().as_secs_f64());
    Ok(())
}
