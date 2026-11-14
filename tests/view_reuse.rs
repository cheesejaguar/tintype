//! Scoring identical views of an image once must not change anything a
//! caller sees: the same list of logits (one per view looked at, a repeated
//! view's score in each of its places), the same early-exit checks, the same
//! stopping point. These tests run the released photo model on the CPU
//! engine over the image sizes where views coincide, with reuse off, in its
//! default state and forced on, across batch sizes and through every scoring
//! path.
mod common;

use std::path::{Path, PathBuf};
use tintype::cpu_vit::RUN_SIZE_CHANGES_LOGITS;
use tintype::detector::{Detector, Device, EarlyExit, ExitStage, ModelSpec, ViewReuse};
use tintype::image_io::{self, Rgb};

/// Sizes where views coincide (below the crop size, exactly the crop size,
/// two and three window positions, a 256-pixel short side) and one where
/// they just do not (230 x 230).
const SHAPES: [(u32, u32); 6] = [(120, 90), (224, 224), (224, 300), (448, 224), (320, 256), (230, 230)];

/// A picture with gradients, a hard-edged shape and grain that gets stronger
/// to the right, from integer arithmetic only.
fn picture(w: u32, h: u32, seed: u32) -> image::RgbImage {
    image::RgbImage::from_fn(w, h, |x, y| {
        let mut v = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
        v ^= v >> 15;
        v = v.wrapping_mul(0x2C1B_3C6D);
        v ^= v >> 12;
        let strength = 4 + x * 28 / w.max(2);
        let grain = (v % (2 * strength + 1)) as i32 - strength as i32;
        let disc = (x as i32 - w as i32 / 3).pow(2) + (y as i32 - h as i32 / 2).pow(2) < (w.min(h) as i32 / 4).pow(2);
        let base = [(x * 255 / w.max(2)) as i32, (y * 255 / h.max(2)) as i32, ((x + y + 40 * seed) % 256) as i32];
        let c = |i: usize| (if disc { 255 - base[i] / 2 } else { base[i] } + grain).clamp(0, 255) as u8;
        image::Rgb([c(0), c(1), c(2)])
    })
}

fn write_pictures(dir: &Path) -> Vec<PathBuf> {
    SHAPES
        .iter()
        .enumerate()
        .map(|(k, &(w, h))| {
            let p = dir.join(format!("{w}x{h}.png"));
            picture(w, h, k as u32 + 1).save(&p).unwrap();
            p
        })
        .collect()
}

fn detector(batch: usize, reuse: ViewReuse) -> Detector {
    let weights = common::photo_weights();
    let mut det = Detector::load(&weights, Some(ModelSpec::load_for(&weights).unwrap()), batch, Device::Cpu).unwrap();
    det.reuse = reuse;
    det
}

fn bits(l: &[f32]) -> Vec<u32> {
    l.iter().map(|v| v.to_bits()).collect()
}

/// `got` against `want`: the same bits where reuse is exact, and within the
/// engine's own batch-size noise where it was forced on regardless.
fn assert_same(got: &[f32], want: &[f32], exact: bool, what: &str) {
    assert_eq!(got.len(), want.len(), "{what}: {got:?} against {want:?}");
    if exact {
        assert_eq!(bits(got), bits(want), "{what}: {got:?} against {want:?}");
    } else {
        for (g, w) in got.iter().zip(want) {
            assert!((g - w).abs() <= 1e-4, "{what}: {got:?} against {want:?}");
        }
    }
}

/// The standard looking order with a check after every view that never
/// stops: every view is looked at, one at a time.
fn never_stops() -> EarlyExit {
    EarlyExit { order: vec![2, 0, 1, 3, 4, 5], stages: (1..6).map(|views| ExitStage { views, below: f32::INFINITY, above: f32::INFINITY }).collect() }
}

/// A check after the first three views and none after: runs of three views.
fn in_threes() -> EarlyExit {
    EarlyExit { order: vec![2, 0, 1, 3, 4, 5], stages: vec![ExitStage { views: 3, below: f32::INFINITY, above: f32::INFINITY }] }
}

/// Whether forcing reuse on is exact for a detector with this batch size.
fn forced_is_exact(batch: usize) -> bool {
    !RUN_SIZE_CHANGES_LOGITS || batch == 1
}

#[test]
fn the_cpu_engine_depends_on_the_run_size_no_more_than_it_says() {
    // The default reuses scores everywhere on an engine that says a view's
    // logit does not depend on what it is run with: hold it to that.
    let vl = 3 * 224 * 224;
    let mut rng = tintype::rng::Rng::new(11);
    let views: Vec<Vec<u8>> = (0..6).map(|k| (0..vl).map(|i| ((i / 3 % 224) as u32 * (k + 1) + rng.below(48) as u32) as u8).collect()).collect();
    let one = detector(1, ViewReuse::Never);
    let alone: Vec<f32> = views.iter().map(|v| one.logits(v)[0]).collect();
    for batch in [2usize, 3, 6] {
        let det = detector(batch, ViewReuse::Never);
        for start in [0, 6 - batch] {
            let together = det.logits(&views[start..start + batch].concat());
            assert_same(&together, &alone[start..start + batch], !RUN_SIZE_CHANGES_LOGITS, &format!("batch {batch} from view {start}"));
        }
    }
    // The same run twice gives the same bits on every engine.
    let det = detector(3, ViewReuse::Never);
    let x = views[..3].concat();
    assert_eq!(bits(&det.logits(&x)), bits(&det.logits(&x)));
    // The same views in another order: where the engine says so, the position does not matter either.
    let (a, b) = (det.logits(&x), det.logits(&[&views[2][..], &views[0][..], &views[1][..]].concat()));
    assert_same(&[b[1], b[2], b[0]], &a, !RUN_SIZE_CHANGES_LOGITS, "three views in another order");
}

#[test]
fn every_view_scored_and_views_reused_give_the_same_logits_on_the_plain_path() {
    let dir = common::scratch("reuse-plain");
    let paths = write_pictures(&dir);
    let images: Vec<Rgb> = paths.iter().map(|p| image_io::load(p).unwrap()).collect();
    for batch in [1usize, 2, 7] {
        let every = detector(batch, ViewReuse::Never);
        // One worker, so that the batches are the same every time.
        let (want, _) = every.score_files(&paths, 1);
        let want: Vec<Vec<f32>> = want.into_iter().map(|r| r.expect("the picture decodes")).collect();
        for (reuse, exact) in [(ViewReuse::WhenExact, true), (ViewReuse::Always, forced_is_exact(batch))] {
            let det = detector(batch, reuse);
            let (got, _) = det.score_files(&paths, 1);
            let direct = det.score_images(&images);
            for (k, &(w, h)) in SHAPES.iter().enumerate() {
                let what = format!("{w}x{h}, batch {batch}, {reuse:?}");
                assert_same(got[k].as_ref().unwrap(), &want[k], exact, &what);
                // Already-decoded images take a different route to the same numbers
                // (their batches are cut differently, which matters only with Accelerate).
                assert_same(&direct[k], &want[k], !RUN_SIZE_CHANGES_LOGITS || batch == 1, &format!("{what}, decoded"));
                if reuse == ViewReuse::Always {
                    // Views with the same pixels carry one score.
                    let plan = det.preprocessor().plan(&images[k]);
                    for (v, &first) in image_io::first_uses(&plan).iter().enumerate() {
                        assert_eq!(got[k].as_ref().unwrap()[v].to_bits(), got[k].as_ref().unwrap()[first].to_bits(), "{what}: view {v} repeats view {first}");
                    }
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn early_exit_looks_at_the_same_views_and_stops_at_the_same_point_with_reuse() {
    let dir = common::scratch("reuse-staged");
    let paths = write_pictures(&dir);
    let images: Vec<Rgb> = paths.iter().map(|p| image_io::load(p).unwrap()).collect();
    let standard = ModelSpec::load_for(&common::photo_weights()).unwrap().early_exit.expect("the photo model has an early exit");
    for (name, exit) in [("standard", standard), ("never stops", never_stops()), ("in threes", in_threes())] {
        let single_views = exit.stages.len() == 5;
        for batch in [1usize, 6] {
            let every = detector(batch, ViewReuse::Never);
            // All files in one call, one worker: mixed batches. Then one file per call.
            let (want, _) = every.score_files_adaptive(&paths, 1, &exit);
            let want1: Vec<Vec<f32>> = paths.iter().map(|p| every.score_files_adaptive(std::slice::from_ref(p), 1, &exit).0.remove(0).unwrap()).collect();
            for (reuse, forced) in [(ViewReuse::WhenExact, false), (ViewReuse::Always, true)] {
                let det = detector(batch, reuse);
                let (got, _) = det.score_files_adaptive(&paths, 1, &exit);
                for (k, &(w, h)) in SHAPES.iter().enumerate() {
                    let what = format!("{w}x{h}, {name}, batch {batch}, {reuse:?}");
                    assert_same(got[k].as_ref().unwrap(), want[k].as_ref().unwrap(), !forced || forced_is_exact(batch), &format!("{what}, together"));
                    // Alone, with a check after every view, each run is one view and reuse is exact on every engine.
                    let exact_alone = !forced || single_views || forced_is_exact(batch);
                    let got1 = det.score_files_adaptive(std::slice::from_ref(&paths[k]), 1, &exit).0.remove(0).unwrap();
                    assert_same(&got1, &want1[k], exact_alone, &format!("{what}, alone"));
                    // The same image through the single-image entry point.
                    let plan = det.preprocessor().plan(&images[k]);
                    let same = image_io::first_uses(&plan);
                    let mut asked = Vec::new();
                    let stepwise = det.logits_adaptive_with(&exit, &same, |v, out| {
                        asked.push(v);
                        out.copy_from_slice(&plan[v].render(&images[k]).data)
                    });
                    assert_same(&stepwise, &want1[k], exact_alone, &format!("{what}, stepwise"));
                    // Nothing is prepared twice, and nothing that repeats an earlier view when scores are reused.
                    let mut distinct = asked.clone();
                    distinct.sort_unstable();
                    distinct.dedup();
                    assert_eq!(distinct.len(), asked.len(), "{what}: asked for {asked:?}");
                    if forced {
                        assert!(asked.iter().all(|&v| same[v] == v), "{what}: asked for {asked:?}, first uses {same:?}");
                    }
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reused_scores_follow_the_stopping_rule_computed_from_every_view() {
    // Score all six views of each picture one at a time (so that every engine
    // gives each its stand-alone logit), apply the stopping rule to them, and
    // compare with what early exit returns when it reuses scores.
    let dir = common::scratch("reuse-rule");
    let paths = write_pictures(&dir);
    let weights = common::photo_weights();
    let spec = ModelSpec::load_for(&weights).unwrap();
    let exit = spec.early_exit.clone().expect("the photo model has an early exit");
    let all = detector(1, ViewReuse::Never);
    let det = detector(6, ViewReuse::WhenExact);
    for (k, &(w, h)) in SHAPES.iter().enumerate() {
        let views = all.score_files(std::slice::from_ref(&paths[k]), 1).0.remove(0).unwrap();
        let want = exit.looked_at(&views, all.threshold_logit());
        let got = det.score_files_adaptive(std::slice::from_ref(&paths[k]), 1, &exit).0.remove(0).unwrap();
        assert_same(&got, &want, true, &format!("{w}x{h}"));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// Needs the Apple GPU: `cargo test --release --test view_reuse -- --ignored`.
#[cfg(target_os = "macos")]
#[test]
#[ignore]
fn the_gpu_engine_gives_a_view_the_same_logit_in_every_slot_and_with_reuse() {
    let dir = common::scratch("reuse-gpu");
    let paths = write_pictures(&dir);
    let weights = common::photo_weights();
    let spec = ModelSpec::load_for(&weights).unwrap();
    let vl = 3 * 224 * 224;
    let mut rng = tintype::rng::Rng::new(11);
    let views: Vec<Vec<u8>> = (0..8).map(|k| (0..vl).map(|i| ((i / 3 % 224) as u32 * (k + 1) + rng.below(48) as u32) as u8).collect()).collect();
    for device in [Device::Gpu, Device::GpuF32] {
        for batch in [6usize, 64] {
            let mut det = Detector::load(&weights, Some(spec.clone()), batch, device).unwrap();
            // A view alone in slot 0 of a zero-padded batch, then in other slots beside other views.
            for v in 0..3 {
                let mut x = vec![0u8; batch * vl];
                x[..vl].copy_from_slice(&views[v]);
                let alone = det.logits(&x)[0];
                for slot in [1, batch / 2, batch - 1] {
                    let mut x: Vec<u8> = (0..batch).flat_map(|s| views[(s + 3) % 8].iter().copied()).collect();
                    x[slot * vl..(slot + 1) * vl].copy_from_slice(&views[v]);
                    assert_eq!(det.logits(&x)[slot].to_bits(), alone.to_bits(), "{device:?} batch {batch}: view {v} in slot {slot}");
                }
            }
            // So reuse is exact on every path.
            let exit = never_stops();
            det.reuse = ViewReuse::Never;
            let (want, want_staged) = (det.score_files(&paths, 1).0, det.score_files_adaptive(&paths, 1, &exit).0);
            det.reuse = ViewReuse::WhenExact;
            let (got, got_staged) = (det.score_files(&paths, 1).0, det.score_files_adaptive(&paths, 1, &exit).0);
            for (k, &(w, h)) in SHAPES.iter().enumerate() {
                assert_same(got[k].as_ref().unwrap(), want[k].as_ref().unwrap(), true, &format!("{device:?} batch {batch} {w}x{h}"));
                assert_same(got_staged[k].as_ref().unwrap(), want_staged[k].as_ref().unwrap(), true, &format!("{device:?} batch {batch} {w}x{h}, early exit"));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
