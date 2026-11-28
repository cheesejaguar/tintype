//! The photo second opinion run beside early exit must give what the two
//! stages give one after the other: the same first-stage logits, the same
//! photos asked and the same probabilities, bit for bit. One test, on the
//! GPU engine, so that it has the process to itself.
mod common;

/// A picture with gradients, a hard-edged disc and grain, from integer
/// arithmetic only, different for every `seed`.
#[cfg(target_os = "macos")]
fn picture(w: u32, h: u32, seed: u32) -> image::RgbImage {
    image::RgbImage::from_fn(w, h, |x, y| {
        let mut v = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
        v ^= v >> 15;
        v = v.wrapping_mul(0x2C1B_3C6D);
        v ^= v >> 12;
        let strength = 2 + (seed % 7) * 4 + x * 16 / w.max(2);
        let grain = (v % (2 * strength + 1)) as i32 - strength as i32;
        let disc = (x as i32 - w as i32 / 3).pow(2) + (y as i32 - h as i32 / 2).pow(2) < (w.min(h) as i32 / 4).pow(2);
        let base = [(x * 255 / w.max(2)) as i32, (y * 255 / h.max(2)) as i32, ((x + y + 40 * seed) % 256) as i32];
        let c = |i: usize| (if disc { 255 - base[i] / 2 } else { base[i] } + grain).clamp(0, 255) as u8;
        image::Rgb([c(0), c(1), c(2)])
    })
}

/// Needs the Apple GPU: `cargo test --release --test second_beside -- --ignored`.
#[cfg(target_os = "macos")]
#[test]
#[ignore]
fn the_second_opinion_beside_early_exit_gives_what_the_stages_give_one_after_the_other() {
    use tintype::detector::{self, Detector, Device, ModelSpec, PhotoSecondOpinion};
    let dir = common::scratch("second-beside");
    // The only test in this process, so the only one that reads this.
    std::env::set_var("TINTYPE_GPU_CACHE", dir.join("gpu-cache"));
    // Sizes below, at and above the crop size, and one unreadable file.
    let sizes = [(120, 90), (224, 224), (224, 300), (448, 224), (320, 256), (230, 230), (640, 480), (500, 700), (800, 600), (300, 1000)];
    let mut paths: Vec<std::path::PathBuf> = (0..60u32)
        .map(|k| {
            let (w, h) = sizes[k as usize % sizes.len()];
            let p = dir.join(format!("{k:02}-{w}x{h}.png"));
            picture(w, h, k + 1).save(&p).unwrap();
            p
        })
        .collect();
    let broken = dir.join("broken.png");
    std::fs::write(&broken, b"not a picture").unwrap();
    paths.insert(17, broken);
    let weights = common::photo_weights();
    let spec = ModelSpec::load_for(&weights).unwrap();
    let exit = spec.early_exit.clone().expect("the shipped model has early exit");
    let shipped = spec.second_opinion.expect("the shipped model has a second-opinion band");
    // Small batches, so that both stages run many batches, full and not.
    let first = Detector::load(&weights, Some(spec), 16, Device::Gpu).unwrap();
    let second = detector::load_photo_second(&common::repo().join("models").join(detector::BUNDLED_PHOTO_SECOND), 16, Device::Gpu).unwrap();
    let bits = |l: &Option<Vec<f32>>| l.as_ref().map(|l| l.iter().map(|v| v.to_bits()).collect::<Vec<_>>());
    // The shipped band, and one that asks about every photo.
    for band in [shipped, PhotoSecondOpinion { lo: -1e6, hi: 1e6, ..shipped }] {
        let (want_first, _) = first.score_files_adaptive(&paths, 4, &exit);
        let calibrated: Vec<Option<f32>> = want_first.iter().map(|r| r.as_ref().map(|l| first.calibrated_logit(l))).collect();
        let (want_second, asked, _) = detector::photo_second_opinion_with(&second, &band, &paths, &calibrated, 4);
        for second_threads in [1, 3] {
            let (got_first, got_second) = first.score_files_adaptive_with_second(&paths, 4, &exit, &second, &band, second_threads);
            assert_eq!(got_first.len(), paths.len());
            for (k, (got, want)) in got_first.iter().zip(&want_first).enumerate() {
                assert_eq!(bits(got), bits(want), "{band:?}: first stage of {}", paths[k].display());
            }
            let as_bits = |s: &[(usize, f32)]| s.iter().map(|&(i, p)| (i, p.to_bits())).collect::<Vec<_>>();
            assert_eq!(as_bits(&got_second), as_bits(&want_second), "{band:?}: second opinion");
        }
        assert!(want_first[17].is_none() && want_second.iter().all(|&(i, _)| i != 17), "the unreadable file is left out");
        if band.lo < -1e5 {
            assert_eq!(asked, paths.len() - 1, "the wide band asks about every photo that could be read");
            assert_eq!(want_second.len(), asked);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
