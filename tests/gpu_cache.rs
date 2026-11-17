//! The GPU engine's compiled networks: the batch sizes it keeps for short
//! runs and the cache of them on disk. Needs the Apple GPU:
//! `cargo test --release --test gpu_cache -- --ignored`.
mod common;

#[cfg(target_os = "macos")]
use tintype::detector::{Detector, Device, ModelSpec};

#[cfg(target_os = "macos")]
fn views(n: usize) -> Vec<u8> {
    let vl = 3 * 224 * 224;
    let mut rng = tintype::rng::Rng::new(5);
    (0..n * vl).map(|i| ((i / 3 % 224) as u32 * (i / vl + 1) as u32 + rng.below(64) as u32) as u8).collect()
}

/// One test, so that it alone sets `TINTYPE_GPU_CACHE` in this process.
#[cfg(target_os = "macos")]
#[test]
#[ignore]
fn compiled_networks_give_the_same_logits_from_the_cache_and_in_short_runs() {
    let dir = common::scratch("gpu-cache");
    let cache = dir.join("cache");
    std::env::set_var("TINTYPE_GPU_CACHE", &cache);
    let weights = common::photo_weights();
    let spec = ModelSpec::load_for(&weights).unwrap();
    let vl = 3 * 224 * 224;
    let x = views(64);
    for device in [Device::Gpu, Device::GpuF32] {
        for batch in [6usize, 64] {
            let _ = std::fs::remove_dir_all(&cache);
            // Built (and stored), then loaded from the cache.
            let built = Detector::load(&weights, Some(spec.clone()), batch, device).unwrap();
            let entries: Vec<_> = std::fs::read_dir(&cache).unwrap().flatten().collect();
            assert_eq!(entries.len(), 1, "{device:?} batch {batch}: one cache entry");
            let ready = std::fs::read_to_string(entries[0].path().join("ready")).unwrap();
            let sizes: Vec<usize> = ready.split_whitespace().map(|s| s.parse().unwrap()).collect();
            assert_eq!(sizes.last(), Some(&batch), "{device:?}: the full batch is compiled");
            assert!(sizes.windows(2).all(|w| w[0] < w[1]), "{device:?}: sizes {sizes:?} go up");
            let cached = Detector::load(&weights, Some(spec.clone()), batch, device).unwrap();
            let full = built.logits(&x[..batch * vl]);
            assert_eq!(full.len(), batch);
            let again = cached.logits(&x[..batch * vl]);
            assert!(full.iter().zip(&again).all(|(a, b)| a.to_bits() == b.to_bits()), "{device:?} batch {batch}: the cache changed logits");
            // Fewer views than the batch, which may go through a smaller
            // compiled batch: the same logits as in the full batch.
            for n in [1, batch / 2 + 1, batch - 1] {
                for det in [&built, &cached] {
                    let short = det.logits(&x[..n * vl]);
                    assert_eq!(short.len(), n);
                    assert!(short.iter().zip(&full).all(|(a, b)| a.to_bits() == b.to_bits()), "{device:?} batch {batch}: a run of {n} views changed logits");
                }
            }
            // More views than the batch: several runs, the same logits.
            let long = built.logits(&[&x[..batch * vl], &x[..vl]].concat());
            assert!(long[..batch].iter().zip(&full).all(|(a, b)| a.to_bits() == b.to_bits()) && long[batch].to_bits() == full[0].to_bits());
        }
    }
    // Turned off, nothing is written.
    let _ = std::fs::remove_dir_all(&cache);
    std::env::set_var("TINTYPE_GPU_CACHE", "0");
    Detector::load(&weights, Some(spec), 6, Device::Gpu).unwrap();
    assert!(!cache.exists(), "TINTYPE_GPU_CACHE=0 writes no cache");
    let _ = std::fs::remove_dir_all(&dir);
}
