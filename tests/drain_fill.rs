//! Early exit fills the padding slots of a run that is not full with views
//! the same photos would look at next. That must not change what a photo
//! looks at: the logits early exit returns must be those of scoring every
//! view and stopping by the same rule, bit for bit. One test, on the GPU
//! engine, so that it has the process to itself.
mod common;

/// A picture with gradients and grain whose strength varies with `seed`, so
/// that photos stop at different stages.
#[cfg(target_os = "macos")]
fn picture(w: u32, h: u32, seed: u32) -> image::RgbImage {
    image::RgbImage::from_fn(w, h, |x, y| {
        let mut v = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
        v ^= v >> 15;
        v = v.wrapping_mul(0x2C1B_3C6D);
        v ^= v >> 12;
        let strength = 1 + (seed % 9) * 5;
        let grain = (v % (2 * strength + 1)) as i32 - strength as i32;
        let base = [(x * 255 / w.max(2)) as i32, (y * 255 / h.max(2)) as i32, ((x * y / 7 + 50 * seed) % 256) as i32];
        let c = |i: usize| (base[i] + grain).clamp(0, 255) as u8;
        image::Rgb([c(0), c(1), c(2)])
    })
}

/// Needs the Apple GPU: `cargo test --release --test drain_fill -- --ignored`.
#[cfg(target_os = "macos")]
#[test]
#[ignore]
fn early_exit_with_padding_filled_looks_at_what_scoring_every_view_would() {
    use tintype::detector::{Detector, Device, ModelSpec};
    use tintype::stats::SCORED_VIEWS;
    let dir = common::scratch("drain-fill");
    // The only test in this process, so the only one that reads this.
    std::env::set_var("TINTYPE_GPU_CACHE", dir.join("gpu-cache"));
    let sizes = [(120, 90), (224, 224), (224, 300), (448, 224), (320, 256), (640, 480), (500, 700), (800, 600), (1024, 768)];
    let paths: Vec<std::path::PathBuf> = (0..45u32)
        .map(|k| {
            let (w, h) = sizes[k as usize % sizes.len()];
            let p = dir.join(format!("{k:02}-{w}x{h}.png"));
            picture(w, h, k + 1).save(&p).unwrap();
            p
        })
        .collect();
    let weights = common::photo_weights();
    let spec = ModelSpec::load_for(&weights).unwrap();
    let exit = spec.early_exit.clone().expect("the shipped model has early exit");
    let bits = |l: &[f32]| l.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
    for batch in [16, 64] {
        let det = Detector::load(&weights, Some(spec.clone()), batch, Device::Gpu).unwrap();
        let thr = det.threshold_logit();
        let (all, _) = det.score_files(&paths, 4);
        let want: Vec<Vec<f32>> = all.iter().map(|l| exit.looked_at(l.as_ref().unwrap(), thr)).collect();
        let looked: usize = want.iter().map(Vec::len).sum();
        assert!(want.iter().any(|l| l.len() == 1) && want.iter().any(|l| l.len() > 2), "photos stop at different stages");
        for threads in [1, 4] {
            let before = SCORED_VIEWS.get();
            let (got, _) = det.score_files_adaptive(&paths, threads, &exit);
            let scored = (SCORED_VIEWS.get() - before) as usize;
            for (k, (got, want)) in got.iter().zip(&want).enumerate() {
                assert_eq!(bits(got.as_ref().unwrap()), bits(want), "batch {batch}, {threads} threads: {}", paths[k].display());
            }
            // Views are scored at most once each, and some beyond what the
            // photos looked at only where runs had padding to fill.
            assert!(scored <= paths.len() * spec.views.len(), "batch {batch}: {scored} views scored");
            eprintln!("batch {batch}, {threads} threads: {looked} views looked at, {scored} scored");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
