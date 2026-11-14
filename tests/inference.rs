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

/// A spec may name a ViT-S/16 with fewer blocks (`vit_small_patch16_d8`): a
/// shallower student. Loaded over the released twelve-block weights it runs
/// their first blocks, which is only good for timing; the test is that the
/// name selects the depth and that twelve is the network as shipped.
#[test]
fn a_spec_can_name_a_network_with_fewer_blocks() {
    let dir = common::scratch("inference-depth");
    let png = dir.join("fixture.png");
    common::write_fixture(&png);
    let weights = common::photo_weights();
    let logits = |arch: &str| {
        let mut spec = tintype::detector::ModelSpec::load_for(&weights).unwrap();
        spec.arch = arch.into();
        let views = spec.views.len();
        let det = Detector::load(&weights, Some(spec), views, Device::Cpu).unwrap();
        det.score_files(std::slice::from_ref(&png), 1).0[0].clone().expect("the fixture decodes")
    };
    let shipped = logits("");
    assert_eq!(logits("vit_small_patch16_d12"), shipped);
    let eight = logits("vit_small_patch16_d8");
    assert!(eight.iter().all(|l| l.is_finite()) && eight.len() == shipped.len());
    assert!(eight.iter().zip(&shipped).any(|(a, b)| (a - b).abs() > 0.05), "eight blocks gave the twelve-block logits: {eight:?}");
    let mut spec = tintype::detector::ModelSpec::load_for(&weights).unwrap();
    spec.arch = "vit_small_patch16_d13".into();
    assert!(Detector::load(&weights, Some(spec), 1, Device::Cpu).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
