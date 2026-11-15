//! A complete-model inference fixture on the CPU engine: a fixed picture
//! through decoding, the photo model's own views and the released weights,
//! with the logits and score it gave when recorded. A change to decoding,
//! cropping, resizing, normalisation or the engine moves these numbers.
//!
//! The CPU engine is the one available everywhere (Accelerate on macOS, the
//! pure-Rust kernels on Linux and with `--features portable-cpu`); the values
//! below agree across all three to well inside the tolerance.
mod common;

use tintype::detector::{Detector, Device};

/// Per-view logits of the released photo model on the fixture, in view order,
/// recorded on macOS (Apple Silicon) with the CPU engine.
const EXPECTED_LOGITS: [f32; 6] = [-3.9661195, -3.9661195, -4.6595144, -4.8981724, -3.6314917, -3.6932852];
/// The default profile's score (calibrated probability) for those logits.
const EXPECTED_SCORE: f32 = 0.020435736;
/// SHA-256 of the decoded fixture's pixels: if this fails, the fixture or the
/// PNG decoder changed, not the model.
const EXPECTED_PIXELS_SHA256: &str = "9e51a7a3dd80af0af7edd8a5184e99818bdd931339fa835c41ea65e2f5ad04d2";

/// Logits differ by about 1e-4 between Accelerate and the pure-Rust kernels;
/// SIMD resizing may move a pixel by one level. Anything real is far larger.
const LOGIT_TOLERANCE: f32 = 0.05;

#[test]
fn photo_model_gives_the_recorded_logits_on_the_fixture() {
    let dir = common::scratch("inference");
    let png = dir.join("fixture.png");
    common::write_fixture(&png);

    let img = tintype::image_io::load(&png).unwrap();
    assert_eq!((img.w, img.h), (320, 256));
    let pixels = tintype::integrity::sha256_bytes(&img.data);

    let weights = common::photo_weights();
    let spec = tintype::detector::ModelSpec::load_for(&weights).unwrap();
    let views = spec.views.len();
    let det = Detector::load(&weights, Some(spec), views, Device::Cpu).unwrap();
    let (res, _) = det.score_files(std::slice::from_ref(&png), 1);
    let logits = res[0].clone().expect("the fixture decodes");
    let score = det.probability(&logits);
    let _ = std::fs::remove_dir_all(&dir);

    let report = format!("pixels sha256 {pixels}\nlogits {logits:?}\nscore {score}");
    assert_eq!(pixels, EXPECTED_PIXELS_SHA256, "{report}");
    assert_eq!(logits.len(), EXPECTED_LOGITS.len(), "{report}");
    for (i, (got, want)) in logits.iter().zip(EXPECTED_LOGITS).enumerate() {
        assert!((got - want).abs() <= LOGIT_TOLERANCE, "view {i}: logit {got}, recorded {want}\n{report}");
    }
    eprintln!("{report}");
    assert!((score - EXPECTED_SCORE).abs() <= 0.01, "score {score}, recorded {EXPECTED_SCORE}\n{report}");
}

/// The fixture's six photo views as the engines take them.
fn fixture_views(name: &str) -> (Vec<u8>, tintype::detector::ModelSpec, std::path::PathBuf) {
    let dir = common::scratch(name);
    let png = dir.join("fixture.png");
    common::write_fixture(&png);
    let img = tintype::image_io::load(&png).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let weights = common::photo_weights();
    let spec = tintype::detector::ModelSpec::load_for(&weights).unwrap();
    let det = Detector::load(&weights, Some(spec.clone()), spec.views.len(), Device::Cpu).unwrap();
    let mut views = vec![0u8; spec.views.len() * 3 * spec.img * spec.img];
    det.preprocessor().apply(&img, &mut views);
    (views, spec, weights)
}

#[test]
fn reading_patch_tokens_gives_the_same_logits_bit_for_bit() {
    let (views, spec, weights) = fixture_views("tokens-cpu");
    let det = Detector::load(&weights, Some(spec.clone()), spec.views.len(), Device::Cpu).unwrap();
    let plain = det.logits(&views);
    let emb = tintype::embed::Embedder::load(&weights, &spec, spec.views.len(), Device::Cpu, &[12, 6]).unwrap();
    let tapped = emb.embed(&views).unwrap();
    // Not a tolerance: the logit path is the scoring path, with the tokens read out beside it.
    assert_eq!(tapped.logits.iter().map(|v| v.to_bits()).collect::<Vec<_>>(), plain.iter().map(|v| v.to_bits()).collect::<Vec<_>>(), "{:?} vs {plain:?}", tapped.logits);
    for (got, want) in tapped.logits.iter().zip(EXPECTED_LOGITS) {
        assert!((got - want).abs() <= LOGIT_TOLERANCE, "logit {got}, recorded {want}");
    }
    // Six views of 1 + 14 x 14 tokens, 384 wide, for each of the two blocks asked for.
    assert_eq!((emb.depth(), emb.side(), emb.dim(), emb.tokens_per_view()), (12, 14, 384, 197));
    assert_eq!(tapped.tokens.len(), 2);
    for t in &tapped.tokens {
        assert_eq!(t.len(), 6 * 197 * 384);
        assert!(t.iter().all(|v| v.is_finite()));
    }
    assert!(tapped.tokens[0] != tapped.tokens[1]);
    // Views 0 and 1 of the fixture are the same pixels (the recorded logits agree), so their tokens are too.
    let per = 197 * 384;
    let worst = tapped.tokens[0][..per].iter().zip(&tapped.tokens[0][per..2 * per]).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max);
    assert!(worst < 1e-3, "identical views differ by {worst}");
    // Pooled to the 7 x 7 grid the feature dump stores by default.
    assert_eq!(tintype::embed::pool_grid(&tapped.tokens[0][384..per], 14, 384, 7).len(), 49 * 384);
}

/// How far two token sets are apart.
#[cfg(target_os = "macos")]
#[derive(Debug)]
struct Gap {
    /// Mean absolute difference over values.
    mean: f32,
    /// Over tokens (`dim` values each): the median, the 99th percentile and
    /// the largest relative difference `|a - b| / |b|`, and the smallest cosine.
    median_relative: f32,
    p99_relative: f32,
    worst_relative: f32,
    worst_cosine: f32,
}

#[cfg(target_os = "macos")]
fn token_gap(a: &[f32], b: &[f32], dim: usize) -> Gap {
    let norm = |v: &[f32]| v.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mut cosine = 1f32;
    let mut relative: Vec<f32> = Vec::new();
    for (x, y) in a.chunks_exact(dim).zip(b.chunks_exact(dim)) {
        let diff: Vec<f32> = x.iter().zip(y).map(|(p, q)| p - q).collect();
        relative.push(norm(&diff) / norm(y));
        cosine = cosine.min(x.iter().zip(y).map(|(p, q)| p * q).sum::<f32>() / (norm(x) * norm(y)));
    }
    relative.sort_by(f32::total_cmp);
    let n = relative.len();
    Gap { mean: a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum::<f32>() / a.len() as f32, median_relative: relative[n / 2], p99_relative: relative[n * 99 / 100], worst_relative: relative[n - 1], worst_cosine: cosine }
}

/// Needs a Metal device, so it is not part of the default run:
/// `cargo test --release --test inference -- --ignored gpu_and_cpu`.
///
/// Single precision on the GPU agrees with the CPU engine to rounding. Half
/// precision (the default scoring engine) does not, token by token: the
/// class token and the logit stay close, and so do most patch tokens, but a
/// few patch tokens come out far from their single-precision values. The
/// bounds below are several times what an Apple M-series GPU gave on the
/// fixture (the test prints the measurements); the half-precision bounds are
/// loose on purpose, and are the reason `dump-video-features` uses single
/// precision unless told otherwise.
#[cfg(target_os = "macos")]
#[test]
#[ignore]
fn gpu_and_cpu_patch_embeddings_agree_on_the_fixture() {
    let (views, spec, weights) = fixture_views("tokens-gpu");
    let blocks = [12, 6];
    let cpu = tintype::embed::Embedder::load(&weights, &spec, 6, Device::Cpu, &blocks).unwrap().embed(&views).unwrap();
    // A batch of four makes the second run a padded one.
    for (device, batch) in [(Device::GpuF32, 6), (Device::Gpu, 6), (Device::Gpu, 4)] {
        let gpu = tintype::embed::Embedder::load(&weights, &spec, batch, device, &blocks).unwrap().embed(&views).unwrap();
        assert_eq!(gpu.logits.len(), cpu.logits.len());
        let logit_gap = gpu.logits.iter().zip(&cpu.logits).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max);
        eprintln!("{device:?} batch {batch}: logits differ by at most {logit_gap}");
        assert!(logit_gap <= if device == Device::GpuF32 { 1e-3 } else { 0.05 }, "{device:?}: logits differ by {logit_gap}");
        for (k, block) in blocks.iter().enumerate() {
            assert_eq!(gpu.tokens[k].len(), cpu.tokens[k].len());
            let gap = token_gap(&gpu.tokens[k], &cpu.tokens[k], 384);
            eprintln!("{device:?} batch {batch}, block {block}: {gap:?}");
            if device == Device::GpuF32 {
                assert!(gap.worst_relative <= 2e-3 && gap.p99_relative <= 3e-4 && gap.worst_cosine >= 0.99999 && gap.mean <= 1e-4, "{device:?} block {block}: {gap:?}");
            } else {
                assert!(gap.median_relative <= 0.03 && gap.p99_relative <= 0.2 && gap.mean <= 0.05 && gap.worst_cosine >= 0.9, "{device:?} block {block}: {gap:?}");
            }
        }
    }
}
