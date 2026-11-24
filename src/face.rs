//! EXPERIMENTAL, research only: a "face stage" measurement prototype.
//!
//! Per photo: find the largest face (Apple Vision, macOS only), cut a square
//! around it with a margin, and score the square with a face model (a DINOv2
//! ViT-B/14 with registers at 224 px, one logit). Nothing in `detect`,
//! `batch` or `serve` uses this; it exists so that the stage's agreement
//! with its PyTorch reference and its speed can be measured (`tintype
//! face-stage`). The cutoff is not calibrated for release.
use crate::detector::{Detector, Device, ModelSpec, Timing};
use crate::image_io::{self, Rgb};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// The face model's logit above which a face is flagged (research value).
pub const FACE_CUTOFF: f32 = 6.328;

/// Margin around the face box, as a share of its longer side, on each side.
pub const FACE_MARGIN: f64 = 0.3;

/// Squares smaller than this on either side are not scored.
pub const MIN_CROP: u32 = 32;

/// How the face model is run: the shorter side resized to 224 with Pillow's
/// bicubic filter, the centre 224 square, ImageNet normalisation. The same
/// network shape and preprocessing as the edit check.
pub fn spec() -> ModelSpec {
    ModelSpec::edit_check()
}

/// The face model loaded for batches of up to `batch` squares.
pub fn load(weights: &Path, batch: usize, device: Device) -> Result<Detector> {
    Detector::load(weights, Some(spec()), batch.max(1), device)
}

/// A face box as Vision gives it: fractions of the image's width and height,
/// the origin at the bottom left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl FaceBox {
    fn area(&self) -> f64 {
        self.w * self.h
    }
}

/// The square around `face` in a `width` by `height` image, as `(left, top,
/// width, height)` in pixels from the top left: its side is the box's longer
/// side in pixels times `1 + 2 * FACE_MARGIN`, centred on the box, clipped
/// to the image and widened to whole pixels. `None` when what is left is
/// under [`MIN_CROP`] on a side. The arithmetic of `tools/facecrop.swift`.
pub fn crop_rect(face: &FaceBox, width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = (width as f64, height as f64);
    let side = (face.w * w).max(face.h * h) * (1.0 + 2.0 * FACE_MARGIN);
    let (cx, cy) = ((face.x + face.w / 2.0) * w, (face.y + face.h / 2.0) * h);
    // Clip, then widen to whole pixels (CGRect `intersection` and `integral`).
    let span = |c: f64, full: f64| {
        let (lo, hi) = ((c - side / 2.0).max(0.0), (c + side / 2.0).min(full));
        (hi > lo).then(|| (lo.floor(), hi.ceil().min(full)))
    };
    let ((x0, x1), (y0, y1)) = (span(cx, w)?, span(cy, h)?);
    let (cw, ch) = ((x1 - x0) as u32, (y1 - y0) as u32);
    // Vision counts rows from the bottom; pixels are stored from the top.
    (cw >= MIN_CROP && ch >= MIN_CROP).then(|| (x0 as u32, (h - y1) as u32, cw, ch))
}

/// The largest face in `img`, if Vision finds one.
#[cfg(target_os = "macos")]
pub fn largest_face(img: &Rgb) -> Result<Option<FaceBox>> {
    vision::largest_face(img)
}

/// The largest face of the file at `path`, read by Core Image, with the
/// image's width and height as Core Image has them.
#[cfg(target_os = "macos")]
pub fn largest_face_in_file(path: &Path) -> Result<Option<(FaceBox, u32, u32)>> {
    vision::largest_face_in_file(path)
}

#[cfg(not(target_os = "macos"))]
pub fn largest_face_in_file(_path: &Path) -> Result<Option<(FaceBox, u32, u32)>> {
    anyhow::bail!("face detection uses Apple's Vision framework and needs macOS")
}

#[cfg(not(target_os = "macos"))]
pub fn largest_face(_img: &Rgb) -> Result<Option<FaceBox>> {
    anyhow::bail!("face detection uses Apple's Vision framework and needs macOS")
}

/// `VNDetectFaceRectanglesRequest`, on pixels this crate decoded or on the
/// file as Core Image reads it.
#[cfg(target_os = "macos")]
mod vision {
    use super::FaceBox;
    use crate::image_io::Rgb;
    use anyhow::{anyhow, bail, Result};
    use std::ffi::c_void;
    use objc2::rc::{autoreleasepool, Retained};
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2::msg_send;
    use objc2_core_foundation::CGRect;
    use objc2_foundation::{NSArray, NSDictionary, NSError, NSString, NSURL};

    #[link(name = "Vision", kind = "framework")]
    extern "C" {}
    #[link(name = "CoreImage", kind = "framework")]
    extern "C" {}

    fn class(name: &std::ffi::CStr) -> Result<&'static AnyClass> {
        AnyClass::get(name).ok_or_else(|| anyhow!("the Vision framework has no class {name:?}"))
    }

    /// A Core Graphics object, for the Objective-C runtime's type check.
    #[repr(C)]
    pub struct CGImage {
        _opaque: [u8; 0],
    }
    unsafe impl objc2::RefEncode for CGImage {
        const ENCODING_REF: objc2::Encoding = objc2::Encoding::Pointer(&objc2::Encoding::Struct("CGImage", &[]));
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        static kCGColorSpaceSRGB: *const c_void;
        fn CGColorSpaceCreateWithName(name: *const c_void) -> *mut c_void;
        fn CGColorSpaceRelease(space: *mut c_void);
        fn CGDataProviderCreateWithData(info: *mut c_void, data: *const c_void, size: usize, release: *const c_void) -> *mut c_void;
        fn CGDataProviderRelease(provider: *mut c_void);
        #[allow(clippy::too_many_arguments)]
        fn CGImageCreate(width: usize, height: usize, bits_per_component: usize, bits_per_pixel: usize, bytes_per_row: usize, space: *mut c_void, bitmap_info: u32, provider: *mut c_void, decode: *const f64, interpolate: bool, intent: i32) -> *mut CGImage;
        fn CGImageRelease(image: *mut CGImage);
    }
    /// The largest face in `img`, shown to Vision as a `CGImage` over its
    /// own bytes (8-bit RGB tagged sRGB; nothing is copied).
    pub fn largest_face(img: &Rgb) -> Result<Option<FaceBox>> {
        let (w, h) = (img.w as usize, img.h as usize);
        if w == 0 || h == 0 {
            return Ok(None);
        }
        autoreleasepool(|_| unsafe {
            let space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
            let provider = CGDataProviderCreateWithData(std::ptr::null_mut(), img.data.as_ptr() as *const c_void, img.data.len(), std::ptr::null());
            // 8 bits a component, 24 a pixel, no alpha, default rendering intent.
            let image = CGImageCreate(w, h, 8, 24, w * 3, space, 0, provider, std::ptr::null(), false, 0);
            CGDataProviderRelease(provider);
            CGColorSpaceRelease(space);
            if image.is_null() {
                bail!("cannot wrap the pixels for Vision");
            }
            let options = NSDictionary::<NSString, AnyObject>::new();
            let found = (|| {
                let handler: Retained<AnyObject> = msg_send![msg_send![class(c"VNImageRequestHandler")?, alloc], initWithCGImage: image, options: &*options];
                largest(&handler)
            })();
            CGImageRelease(image);
            found
        })
    }

    /// The largest face of the image file at `path` as Core Image reads it
    /// (what `tools/facecrop.swift` hands to Vision), with the width and
    /// height Core Image gives the image.
    pub fn largest_face_in_file(path: &std::path::Path) -> Result<Option<(FaceBox, u32, u32)>> {
        autoreleasepool(|_| unsafe {
            let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
            let image: Option<Retained<AnyObject>> = msg_send![class(c"CIImage")?, imageWithContentsOfURL: &*url];
            let Some(image) = image else { bail!("Core Image cannot read {}", path.display()) };
            let extent: CGRect = msg_send![&image, extent];
            let options = NSDictionary::<NSString, AnyObject>::new();
            let handler: Retained<AnyObject> = msg_send![msg_send![class(c"VNImageRequestHandler")?, alloc], initWithCIImage: &*image, options: &*options];
            Ok(largest(&handler)?.map(|b| (b, extent.size.width as u32, extent.size.height as u32)))
        })
    }

    /// Run the face request through `handler` and pick the largest box.
    unsafe fn largest(handler: &AnyObject) -> Result<Option<FaceBox>> {
        {
            let request: Retained<AnyObject> = msg_send![class(c"VNDetectFaceRectanglesRequest")?, new];
            let requests = NSArray::from_retained_slice(&[request.clone()]);
            let done: Result<(), Retained<NSError>> = msg_send![handler, performRequests: &*requests, error: _];
            if let Err(e) = done {
                bail!("Vision: {}", e.localizedDescription());
            }
            let found: Option<Retained<NSArray<AnyObject>>> = msg_send![&request, results];
            let mut best: Option<FaceBox> = None;
            for face in found.iter().flat_map(|a| a.iter()) {
                let r: CGRect = msg_send![&face, boundingBox];
                let b = FaceBox { x: r.origin.x, y: r.origin.y, w: r.size.width, h: r.size.height };
                // The first of equally large faces, as Swift's `max(by:)` gives.
                if best.is_none_or(|a| b.area() > a.area()) {
                    best = Some(b);
                }
            }
            Ok(best)
        }
    }
}

/// What became of one photo of [`run`].
#[derive(Clone, Debug)]
pub struct Row {
    pub path: PathBuf,
    /// `ok`, `noface`, `small` (a square under [`MIN_CROP`]), `error`
    /// (Vision or Core Image failed, or read another size) or `unreadable`.
    pub status: &'static str,
    pub logit: Option<f32>,
    /// The square that was scored: left, top, width, height.
    pub crop: Option<(u32, u32, u32, u32)>,
    /// The logit of the same square after a JPEG round trip, when asked for.
    pub reencoded: Option<f32>,
}

/// What Vision is shown when a face is looked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceInput {
    /// The file as Core Image reads it: what `tools/facecrop.swift` does,
    /// and the boxes it gets. Costs a second decode of the photo.
    File,
    /// The pixels this crate decoded (one decode). Vision then gives
    /// slightly different boxes for the same photo: a JPEG decodes to
    /// slightly different pixels here than in Core Image.
    Pixels,
}

/// Options of [`run`].
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Find and cut out the largest face first; `None` takes each file to
    /// be a face square already.
    pub detect: Option<FaceInput>,
    /// Also score each square after encoding it as JPEG at this quality and
    /// decoding it again (the reference crops were stored as JPEG). The
    /// flag stays on the square as cut; this only adds a column.
    pub reencode: Option<u8>,
    /// With [`FaceInput::Pixels`]: look for the face on a copy of the photo
    /// scaled down so that its longer side is this many pixels (photos
    /// already that small are looked at as they are). The square is still
    /// cut from the photo at full size.
    pub detect_side: Option<u32>,
    pub threads: usize,
}

impl Row {
    pub fn flagged(&self) -> bool {
        self.logit.is_some_and(|z| z > FACE_CUTOFF)
    }
}

/// Seconds spent in each part of [`run`]. `find_s` is summed over worker
/// threads (decoding, detection and cropping); the others are wall time.
#[derive(Clone, Copy, Debug, Default)]
pub struct Spent {
    pub wall_s: f64,
    pub model_s: f64,
    pub find_s: f64,
    pub scored: usize,
}

/// Score `paths` with the face model. Without `detect` each file is taken
/// to be a face square already and goes straight to the model; with it,
/// the largest face of each photo is found and cut out first.
pub fn run(det: &Detector, paths: &[PathBuf], options: Options) -> Result<(Vec<Row>, Spent)> {
    let start = std::time::Instant::now();
    let threads = options.threads;
    let Some(input) = options.detect else {
        let (res, t): (_, Timing) = det.score_files(paths, threads);
        let rows: Vec<Row> = paths.iter().zip(res).map(|(p, r)| Row { path: p.clone(), status: if r.is_some() { "ok" } else { "unreadable" }, logit: r.map(|l| l[0]), crop: None, reencoded: None }).collect();
        let scored = rows.iter().filter(|r| r.logit.is_some()).count();
        return Ok((rows, Spent { wall_s: start.elapsed().as_secs_f64(), model_s: t.model_s, find_s: 0.0, scored }));
    };
    use rayon::prelude::*;
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads.max(1)).build().context("starting the face-finding threads")?;
    let mut rows = Vec::with_capacity(paths.len());
    let mut spent = Spent::default();
    // A few batches of photos at a time, so that whole photos are not all held at once.
    for chunk in paths.chunks((det.batch * 4).max(threads)) {
        let found: Vec<(Row, Option<Rgb>, f64)> = pool.install(|| {
            chunk
                .par_iter()
                .map(|p| {
                    let t = std::time::Instant::now();
                    let mut row = Row { path: p.clone(), status: "unreadable", logit: None, crop: None, reencoded: None };
                    let mut square = None;
                    if let Ok(img) = image_io::load(p) {
                        let face = match input {
                            FaceInput::Pixels => match options.detect_side.filter(|&side| side > 0 && side < img.w.max(img.h)) {
                                // The box is in fractions of the image, so it needs no mapping back.
                                Some(side) => {
                                    let scale = side as f64 / img.w.max(img.h) as f64;
                                    let small = |v: u32| ((v as f64 * scale).round() as u32).max(1);
                                    largest_face(&img.resize(small(img.w), small(img.h), image_io::Filter::Bilinear))
                                }
                                None => largest_face(&img),
                            },
                            // Core Image must see the image the square is cut from.
                            FaceInput::File => largest_face_in_file(p).and_then(|f| match f {
                                Some((_, w, h)) if (w, h) != (img.w, img.h) => anyhow::bail!("Core Image reads {w}x{h}, this crate {}x{}", img.w, img.h),
                                f => Ok(f.map(|f| f.0)),
                            }),
                        };
                        match face {
                            Ok(Some(face)) => match crop_rect(&face, img.w, img.h) {
                                Some((x, y, w, h)) => {
                                    row.status = "ok";
                                    row.crop = Some((x, y, w, h));
                                    square = Some(img.crop(x as i64, y as i64, w, h));
                                }
                                None => row.status = "small",
                            },
                            Ok(None) => row.status = "noface",
                            Err(_) => row.status = "error",
                        }
                    }
                    (row, square, t.elapsed().as_secs_f64())
                })
                .collect()
        });
        let mut squares: Vec<Rgb> = found.iter().filter_map(|(_, s, _)| s.as_ref().map(|s| s.crop(0, 0, s.w, s.h))).collect();
        let cut = squares.len();
        if let Some(quality) = options.reencode {
            // A square that cannot make the round trip is scored as it is.
            let again: Vec<Rgb> = pool.install(|| squares.par_iter().map(|s| s.jpeg_roundtrip(quality).unwrap_or_else(|_| s.crop(0, 0, s.w, s.h))).collect());
            squares.extend(again);
        }
        let t = std::time::Instant::now();
        let logits: Vec<f32> = det.score_images(&squares).into_iter().map(|l| l[0]).collect();
        spent.model_s += t.elapsed().as_secs_f64();
        let mut k = 0;
        for (mut row, square, took) in found {
            spent.find_s += took;
            if square.is_some() {
                row.logit = Some(logits[k]);
                row.reencoded = options.reencode.map(|_| logits[cut + k]);
                k += 1;
                spent.scored += 1;
            }
            rows.push(row);
        }
    }
    spent.wall_s = start.elapsed().as_secs_f64();
    Ok((rows, spent))
}

/// The rows as a table: `path status logit flagged [left top width height
/// [reencoded logit]]`.
pub fn table(rows: &[Row]) -> String {
    let mut out = String::new();
    for r in rows {
        let logit = r.logit.map_or_else(|| "nan".to_string(), |z| format!("{z:.6}"));
        out.push_str(&format!("{}\t{}\t{logit}\t{}", r.path.display(), r.status, r.flagged() as u8));
        if let Some((x, y, w, h)) = r.crop {
            out.push_str(&format!("\t{x}\t{y}\t{w}\t{h}"));
            if let Some(z) = r.reencoded {
                out.push_str(&format!("\t{z:.6}"));
            }
        }
        out.push('\n');
    }
    out
}

/// Paths from a manifest: the first tab-separated column of each line.
pub fn manifest_paths(manifest: &Path) -> Result<Vec<PathBuf>> {
    let text = std::fs::read_to_string(manifest).with_context(|| format!("reading {}", manifest.display()))?;
    Ok(text.lines().filter_map(|l| l.split('\t').next()).filter(|p| !p.is_empty()).map(PathBuf::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_square_is_centred_on_the_face_with_its_margin() {
        // A 100 by 200 pixel box in a 1000 by 800 image, centred at (450, 300) from the bottom left.
        let face = FaceBox { x: 0.4, y: 0.25, w: 0.1, h: 0.25 };
        // Side 200 * 1.6 = 320: columns 290..610, rows (from the bottom) 140..460, so 340 from the top.
        assert_eq!(crop_rect(&face, 1000, 800), Some((290, 340, 320, 320)));
    }

    #[test]
    fn the_square_is_clipped_to_the_image_and_widened_to_whole_pixels() {
        // Near the top right corner of a 1024-pixel image, a 128-pixel box centred at (896, 832) from the bottom left.
        let face = FaceBox { x: 0.8125, y: 0.75, w: 0.125, h: 0.125 };
        // Side 204.8: columns 793.6..998.4, rows from the bottom 729.6..934.4, widened outwards.
        assert_eq!(crop_rect(&face, 1024, 1024), Some((793, 89, 206, 206)));
        // In the corner itself the square is cut off on two sides.
        let corner = FaceBox { x: 0.875, y: 0.875, w: 0.125, h: 0.125 };
        assert_eq!(crop_rect(&corner, 1024, 1024), Some((857, 0, 167, 167)));
        // At the bottom left, rows count from the top of the image.
        let low = FaceBox { x: 0.0, y: 0.0, w: 0.125, h: 0.125 };
        assert_eq!(crop_rect(&low, 1024, 512), Some((0, 512 - 135, 167, 135)));
    }

    #[test]
    fn vision_counts_rows_from_the_bottom_and_pixels_from_the_top() {
        // A face in the upper left of a tall image: Vision's `y` is the
        // distance of the box's lower edge from the bottom, as a fraction.
        let face = FaceBox { x: 0.125, y: 0.75, w: 0.25, h: 0.125 };
        // 128 by 128 pixels in 512 by 1024, centred 128 from the left and
        // 832 from the bottom, so 192 from the top; side 204.8.
        let (left, top, w, h) = crop_rect(&face, 512, 1024).unwrap();
        assert_eq!((left, top, w, h), (25, 89, 206, 206));
        // The square's centre is the face's, measured from the top.
        assert_eq!((left + w / 2, top + h / 2), (128, 192));
        // The same box mirrored top to bottom lands as far from the bottom edge.
        let mirrored = FaceBox { y: 1.0 - face.y - face.h, ..face };
        assert_eq!(crop_rect(&mirrored, 512, 1024), Some((25, 1024 - 89 - 206, 206, 206)));
    }

    #[test]
    fn a_square_under_the_smallest_size_is_not_scored() {
        let tiny = FaceBox { x: 0.5, y: 0.5, w: 0.015625, h: 0.015625 };
        assert_eq!(crop_rect(&tiny, 1024, 1024), None);
        // 20 pixels of box give exactly 32 of square.
        let just = FaceBox { x: 0.5, y: 0.5, w: 0.01953125, h: 0.01953125 };
        assert_eq!(crop_rect(&just, 1024, 1024), Some((506, 1024 - 538, 32, 32)));
    }

    #[test]
    fn the_face_model_runs_as_the_edit_check_does() {
        let s = spec();
        assert_eq!((s.img, s.arch.as_str(), s.views.len()), (224, "vit_base_patch14_reg4_dinov2", 1));
    }
}
