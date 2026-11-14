//! Early exit prepares a frame and its views only when a stage gets to them.
//! This file holds one test so that it has the process, and with it the
//! work counters, to itself.
mod common;

use tintype::detector::{Detector, Device, EarlyExit, ExitStage, ModelSpec, ViewReuse};
use tintype::image_io::Rgb;
use tintype::stats::{CONVERTED_FRAMES, MATERIALISED_VIEWS, SCORED_VIEWS};
use tintype::video::Frame;
use tintype::yuv::Yuv;

/// A decoded frame with a textured luma plane and flat chroma, different for
/// every `seed`.
fn frame(w: usize, h: usize, seed: u32) -> Yuv {
    let luma = (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as u32, (i / w) as u32);
            let mut v = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ seed.wrapping_mul(0xC2B2_AE3D);
            v ^= v >> 15;
            v = v.wrapping_mul(0x2C1B_3C6D);
            (40 + (x * 150 / w as u32) + ((v >> 12) % (8 + 4 * seed))) as u8
        })
        .collect();
    Yuv::new(w, h, luma, vec![128; w.div_ceil(2) * h.div_ceil(2) * 2], false, false)
}

#[test]
fn a_clip_that_stops_early_has_only_the_frames_it_looked_at_converted_and_cut_up() {
    let weights = common::photo_weights();
    let mut det = Detector::load(&weights, Some(ModelSpec::load_for(&weights).unwrap()), 12, Device::Cpu).unwrap();
    det.reuse = ViewReuse::Never;
    let nv = det.spec.views.len();
    assert_eq!(nv, 6);
    // Four frames of 320 x 240 (no two views of a frame coincide). Looked at
    // frame by frame in the order 0, 3, 1, 2, with a check after the first
    // frame: `stop` always stops there, `go_on` never does.
    let frames: Vec<Yuv> = (0..4).map(|k| frame(320, 240, k + 1)).collect();
    let order: Vec<usize> = [0, 3, 1, 2].iter().flat_map(|f| (0..nv).map(move |v| f * nv + v)).collect();
    let stop = EarlyExit { order: order.clone(), stages: vec![ExitStage { views: nv, below: 0.0, above: 0.0 }] };
    let go_on = EarlyExit { order, stages: vec![ExitStage { views: nv, below: f32::INFINITY, above: f32::INFINITY }] };
    let counts = || (CONVERTED_FRAMES.get(), MATERIALISED_VIEWS.get(), SCORED_VIEWS.get());
    let since = |before: (u64, u64, u64)| {
        let now = counts();
        (now.0 - before.0, now.1 - before.1, now.2 - before.2)
    };
    let unconverted = |_: usize| Some(frames.iter().cloned().map(Frame::Yuv).collect::<Vec<_>>());
    let converted: Vec<Rgb> = frames.iter().map(Yuv::to_rgb).collect();

    // One clip that stops after its first frame: one frame converted, six views cut out and scored.
    let before = counts();
    let (stopped, _) = det.score_staged_frames(1, 1, &stop, 4, unconverted);
    assert_eq!(since(before), (1, 6, 6));
    let stopped = stopped[0].clone().expect("the clip is scored");
    assert_eq!(stopped.len(), nv);

    // One clip that goes on: everything is converted and cut out, once.
    let before = counts();
    let (all, _) = det.score_staged_frames(1, 1, &go_on, 4, unconverted);
    assert_eq!(since(before), (4, 24, 24));
    let all = all[0].clone().expect("the clip is scored");
    assert_eq!(all.len(), 4 * nv);

    // When a frame is converted and when its views are cut out changes no
    // logit: the same clip from frames converted up front, and its first
    // frame's views whether or not the clip stops there.
    let (from_rgb, _) = det.score_staged(1, 1, &go_on, 4, |_| Some(converted.clone()));
    let bits = |l: &[f32]| l.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&all), bits(from_rgb[0].as_ref().unwrap()));
    assert_eq!(bits(&stopped), bits(&all[..nv]));

    // Two clips on the CPU engine: the network sets the pace, so the workers
    // prepare everything up front, as they always did. Same stopping point.
    let before = counts();
    let (two, _) = det.score_staged_frames(2, 1, &stop, 4, unconverted);
    assert_eq!(since(before), (8, 48, 12));
    assert!(two.iter().all(|l| l.as_ref().is_some_and(|l| l.len() == nv)));
}
