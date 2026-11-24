//! EXPERIMENTAL, research only: the YuNet face detector on the CPU.
//!
//! YuNet (`face_detection_yunet_2023mar.onnx` of the OpenCV Zoo, MIT
//! licence) is a small convolutional detector: a 3x3 stride-2 stem, then
//! pairs of a 1x1 convolution and a 3x3 depthwise one with ReLU, a
//! three-level feature pyramid at strides 8, 16 and 32, and per level a
//! class score, an objectness score and a box for every cell. This module
//! reads the weights straight from that ONNX file, runs the network in
//! plain Rust on one thread, and post-processes as OpenCV's
//! `FaceDetectorYN` does (`modules/objdetect/src/face_detect.cpp`):
//!
//! * the input is the image's pixels in BGR order as floats 0..255, no mean
//!   and no scale, zero-padded on the right and at the bottom to multiples
//!   of 32 (`setInputSize` then `padWithDivisor`);
//! * every cell of every level, padding included, gives a score
//!   `sqrt(cls * obj)`; cells at or above the score threshold give a box
//!   `cx = (col + dx) * stride`, `cy = (row + dy) * stride`,
//!   `w = exp(dw) * stride`, `h = exp(dh) * stride`;
//! * with more than one such box, `dnn::NMSBoxes` runs on the boxes
//!   truncated to whole pixels: strictly above the score threshold, by
//!   falling score, kept while the overlap with every kept box is at most
//!   the NMS threshold.
//!
//! The five landmarks are not computed. Nothing in `detect`, `batch` or
//! `serve` uses this; `tintype face-stage --face-finder yunet` and `tintype
//! yunet-boxes` do.
use crate::image_io::Rgb;
use anyhow::{bail, ensure, Context, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

/// The settings of the Python reference (`yunet_common.py`).
pub const SCORE_THRESHOLD: f32 = 0.9;
pub const NMS_THRESHOLD: f32 = 0.3;
pub const TOP_K: usize = 5000;
/// The photo is looked at with its longer side scaled down to this.
pub const DETECT_SIDE: u32 = 640;

const STRIDES: [usize; 3] = [8, 16, 32];
const DIVISOR: usize = 32;

/// A face: the box's top left corner, width and height in pixels of the
/// image that was looked at, and the score.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub score: f32,
}

/// One convolution of the ONNX graph: its weight's shape and values, and its bias.
#[derive(Clone, Debug)]
pub struct Conv {
    pub dims: [usize; 4],
    pub w: Vec<f32>,
    pub b: Vec<f32>,
}

/// A 1x1 convolution followed by a 3x3 depthwise one (padding 1).
struct Unit {
    cin: usize,
    cout: usize,
    pw: Vec<f32>,
    pb: Vec<f32>,
    dw: Vec<f32>,
    db: Vec<f32>,
}

pub struct YuNet {
    /// 16 by 27: the stem's weights with the taps as `channel, row, column`.
    stem: Conv,
    /// Eleven units of the backbone, then the neck's three (levels 32, 16, 8).
    units: Vec<Unit>,
    /// Per level (strides 8, 16, 32) six channels: class, objectness, box.
    heads: Vec<Unit>,
    pub score_threshold: f32,
    pub nms_threshold: f32,
    pub top_k: usize,
}

// ---- reading the ONNX file ----

enum Val<'a> {
    Int(u64),
    Bytes(&'a [u8]),
    Fixed32(u32),
}

fn varint(buf: &mut &[u8]) -> Result<u64> {
    let mut v = 0u64;
    for shift in (0..64).step_by(7) {
        let (&b, rest) = buf.split_first().context("the file ends inside a number")?;
        *buf = rest;
        v |= ((b & 0x7f) as u64) << shift;
        if b < 0x80 {
            return Ok(v);
        }
    }
    bail!("a number is too long")
}

/// The fields of one protobuf message.
fn fields(mut buf: &[u8]) -> Result<Vec<(u64, Val<'_>)>> {
    let mut out = Vec::new();
    while !buf.is_empty() {
        let key = varint(&mut buf)?;
        let field = key >> 3;
        let n = match key & 7 {
            0 => {
                out.push((field, Val::Int(varint(&mut buf)?)));
                0
            }
            1 => 8,
            2 => {
                let n = varint(&mut buf)? as usize;
                ensure!(buf.len() >= n, "the file ends inside a field");
                out.push((field, Val::Bytes(&buf[..n])));
                n
            }
            5 => {
                ensure!(buf.len() >= 4, "the file ends inside a field");
                out.push((field, Val::Fixed32(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))));
                4
            }
            other => bail!("unknown protobuf wire type {other}"),
        };
        ensure!(buf.len() >= n, "the file ends inside a field");
        buf = &buf[n..];
    }
    Ok(out)
}

/// The convolutions of an ONNX model in the order of its graph, each with
/// its weight and bias from the model's initializers.
pub fn onnx_convs(bytes: &[u8]) -> Result<Vec<Conv>> {
    let model = fields(bytes)?;
    // ModelProto.graph = 7; GraphProto.node = 1, .initializer = 5.
    let graph = model.iter().find_map(|(f, v)| match v {
        Val::Bytes(b) if *f == 7 => Some(*b),
        _ => None,
    });
    let graph = fields(graph.context("no graph in the ONNX file")?)?;
    let mut tensors: HashMap<String, (Vec<usize>, Vec<f32>)> = HashMap::new();
    let mut convs: Vec<Vec<String>> = Vec::new();
    for (f, v) in &graph {
        let Val::Bytes(b) = v else { continue };
        if *f == 1 {
            // NodeProto.input = 1, .op_type = 4.
            let (mut inputs, mut op) = (Vec::new(), "");
            for (f, v) in fields(b)? {
                match (f, v) {
                    (1, Val::Bytes(s)) => inputs.push(String::from_utf8_lossy(s).into_owned()),
                    (4, Val::Bytes(s)) => op = std::str::from_utf8(s).unwrap_or(""),
                    _ => {}
                }
            }
            if op == "Conv" {
                convs.push(inputs);
            }
        } else if *f == 5 {
            // TensorProto.dims = 1, .data_type = 2, .float_data = 4, .name = 8, .raw_data = 9.
            let (mut dims, mut kind, mut name, mut data) = (Vec::new(), 0, String::new(), Vec::new());
            for (f, v) in fields(b)? {
                match (f, v) {
                    (1, Val::Int(d)) => dims.push(d as usize),
                    (1, Val::Bytes(mut packed)) => {
                        while !packed.is_empty() {
                            dims.push(varint(&mut packed)? as usize);
                        }
                    }
                    (2, Val::Int(k)) => kind = k,
                    (4, Val::Fixed32(x)) => data.push(f32::from_bits(x)),
                    (4 | 9, Val::Bytes(raw)) => data.extend(raw.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))),
                    (8, Val::Bytes(s)) => name = String::from_utf8_lossy(s).into_owned(),
                    _ => {}
                }
            }
            // 1 is FLOAT; the graph's other initializers are shapes.
            if kind == 1 {
                tensors.insert(name, (dims, data));
            }
        }
    }
    convs
        .iter()
        .map(|inputs| {
            ensure!(inputs.len() == 3, "a convolution without a bias");
            let (dims, w) = tensors.get(&inputs[1]).with_context(|| format!("no weight `{}`", inputs[1]))?;
            let (_, b) = tensors.get(&inputs[2]).with_context(|| format!("no bias `{}`", inputs[2]))?;
            ensure!(dims.len() == 4 && w.len() == dims.iter().product::<usize>() && b.len() == dims[0], "a convolution weight of an unexpected shape");
            Ok(Conv { dims: [dims[0], dims[1], dims[2], dims[3]], w: w.clone(), b: b.clone() })
        })
        .collect()
}

impl YuNet {
    /// Read `face_detection_yunet_2023mar.onnx`.
    pub fn load(path: &Path) -> Result<YuNet> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        Self::from_convs(onnx_convs(&bytes).with_context(|| format!("reading the ONNX graph of {}", path.display()))?).with_context(|| format!("{} is not YuNet 2023mar", path.display()))
    }

    /// The network from its convolutions in graph order: the stem, 28 for
    /// the backbone's and neck's units, then per kind (class, box,
    /// objectness, landmarks) two for each of the three levels.
    pub fn from_convs(convs: Vec<Conv>) -> Result<YuNet> {
        ensure!(convs.len() >= 47, "{} convolutions, YuNet has 53", convs.len());
        ensure!(convs[0].dims == [16, 3, 3, 3], "the first convolution is not 3x3 from 3 to 16 channels");
        let widths: [(usize, usize); 14] = [(16, 16), (16, 16), (16, 32), (32, 32), (32, 64), (64, 64), (64, 64), (64, 64), (64, 64), (64, 64), (64, 64), (64, 64), (64, 64), (64, 64)];
        let pair = |i: usize, cin: usize, cout: usize| -> Result<(&Conv, &Conv)> {
            let (p, d) = (&convs[i], &convs[i + 1]);
            ensure!(p.dims == [cout, cin, 1, 1] && d.dims == [cout, 1, 3, 3], "convolutions {i} and {} have shapes {:?} and {:?}", i + 1, p.dims, d.dims);
            Ok((p, d))
        };
        let mut units = Vec::new();
        for (u, &(cin, cout)) in widths.iter().enumerate() {
            let (p, d) = pair(1 + 2 * u, cin, cout)?;
            units.push(Unit { cin, cout, pw: p.w.clone(), pb: p.b.clone(), dw: d.w.clone(), db: d.b.clone() });
        }
        let mut heads = Vec::new();
        for level in 0..3 {
            let mut head = Unit { cin: 64, cout: 6, pw: Vec::new(), pb: Vec::new(), dw: Vec::new(), db: Vec::new() };
            // Class at 29, box at 35, objectness at 41; stored as class, objectness, box.
            for (first, k) in [(29, 1), (41, 1), (35, 4)] {
                let (p, d) = pair(first + 2 * level, 64, k)?;
                head.pw.extend(&p.w);
                head.pb.extend(&p.b);
                head.dw.extend(&d.w);
                head.db.extend(&d.b);
            }
            heads.push(head);
        }
        Ok(YuNet { stem: convs.into_iter().next().unwrap(), units, heads, score_threshold: SCORE_THRESHOLD, nms_threshold: NMS_THRESHOLD, top_k: TOP_K })
    }
}

// ---- the layers ----

thread_local! {
    static SPARE: RefCell<Vec<Vec<f32>>> = const { RefCell::new(Vec::new()) };
}

/// A buffer of `n` values, every one of which the caller writes.
fn take(n: usize) -> Vec<f32> {
    let mut v = SPARE.with(|s| {
        let mut s = s.borrow_mut();
        // The smallest spare one that is large enough, else any.
        match s.iter().enumerate().filter(|(_, v)| v.capacity() >= n).min_by_key(|(_, v)| v.capacity()) {
            Some((i, _)) => s.swap_remove(i),
            None => s.pop().unwrap_or_default(),
        }
    });
    v.resize(n, 0.0);
    v
}

fn give(v: Vec<f32>) {
    SPARE.with(|s| {
        let mut s = s.borrow_mut();
        if s.len() < 12 {
            s.push(v);
        }
    });
}

/// `dst[cout, n] += w[cout, cin] * x[cin, n]` on this thread.
fn matmul(dst: &mut [f32], w: &[f32], x: &[f32], cout: usize, cin: usize, n: usize) {
    assert!(dst.len() >= cout * n && w.len() >= cout * cin && x.len() >= cin * n);
    if n == 0 {
        return;
    }
    // As the product of the tall `x` transposed (one row a pixel) with `w` transposed.
    unsafe {
        gemm::gemm(n, cout, cin, dst.as_mut_ptr(), n as isize, 1, true, x.as_ptr(), n as isize, 1, w.as_ptr(), cin as isize, 1, 1.0f32, 1.0f32, false, false, false, gemm::Parallelism::None);
    }
}

/// The stem: 3x3, stride 2, padding 1, from the image's pixels in BGR order
/// (zero beyond its right and bottom edges, up to `ph` by `pw`) to 16
/// channels of `ph / 2` by `pw / 2`, with ReLU.
fn stem(conv: &Conv, img: &Rgb, ph: usize, pw: usize) -> Vec<f32> {
    const BAND: usize = 8;
    let (w, h) = (img.w as usize, img.h as usize);
    let (oh, ow) = (ph / 2, pw / 2);
    let n = oh * ow;
    let mut out = take(16 * n);
    for (co, plane) in out.chunks_exact_mut(n.max(1)).enumerate() {
        plane.fill(conv.b[co]);
    }
    let mut col = take(27 * BAND * ow);
    for oy0 in (0..oh).step_by(BAND) {
        let rows = BAND.min(oh - oy0);
        let cols = rows * ow;
        for tap in 0..27 {
            let (ci, ky, kx) = (tap / 9, tap / 3 % 3, tap % 3);
            let line = &mut col[tap * cols..(tap + 1) * cols];
            for r in 0..rows {
                let dst = &mut line[r * ow..(r + 1) * ow];
                let sy = 2 * (oy0 + r) + ky;
                if sy < 1 || sy > h {
                    dst.fill(0.0);
                    continue;
                }
                let src = &img.data[(sy - 1) * w * 3..sy * w * 3];
                // Column `2 * ox + kx - 1` of the image, where it has one.
                let lo = (kx == 0) as usize;
                let hi = ow.min((w + 2 - kx) / 2).max(lo);
                dst[..lo].fill(0.0);
                dst[hi..].fill(0.0);
                // The network's channels are blue, green, red.
                let ch = 2 - ci;
                for (ox, d) in dst[lo..hi].iter_mut().enumerate() {
                    *d = src[(2 * (ox + lo) + kx - 1) * 3 + ch] as f32;
                }
            }
        }
        unsafe {
            gemm::gemm(cols, 16, 27, out.as_mut_ptr().add(oy0 * ow), n as isize, 1, true, col.as_ptr(), cols as isize, 1, conv.w.as_ptr(), 27, 1, 1.0f32, 1.0f32, false, false, false, gemm::Parallelism::None);
        }
    }
    give(col);
    for v in out.iter_mut() {
        *v = v.max(0.0);
    }
    out
}

/// A 3x3 depthwise convolution with padding 1 over `c` planes of `h` by `w`.
fn depthwise(x: &[f32], out: &mut [f32], c: usize, h: usize, w: usize, weight: &[f32], bias: &[f32], relu: bool) {
    let n = h * w;
    assert!(x.len() >= c * n && out.len() >= c * n && weight.len() >= c * 9 && bias.len() >= c);
    if n == 0 {
        return;
    }
    for ch in 0..c {
        let (x, k) = (&x[ch * n..(ch + 1) * n], &weight[ch * 9..ch * 9 + 9]);
        for (y, o) in out[ch * n..(ch + 1) * n].chunks_exact_mut(w).enumerate() {
            o.fill(bias[ch]);
            for ky in 0..3 {
                if y + ky < 1 || y + ky > h {
                    continue;
                }
                let r = &x[(y + ky - 1) * w..(y + ky) * w];
                let (k0, k1, k2) = (k[ky * 3], k[ky * 3 + 1], k[ky * 3 + 2]);
                if w >= 3 {
                    for (((o, &a), &b), &c) in o[1..w - 1].iter_mut().zip(&r[..w - 2]).zip(&r[1..w - 1]).zip(&r[2..]) {
                        *o += k0 * a + k1 * b + k2 * c;
                    }
                }
                o[0] += k1 * r[0];
                if w >= 2 {
                    o[0] += k2 * r[1];
                    o[w - 1] += k0 * r[w - 2] + k1 * r[w - 1];
                }
            }
            if relu {
                for v in o.iter_mut() {
                    *v = v.max(0.0);
                }
            }
        }
    }
}

/// A unit on `cin` planes of `h` by `w`.
fn unit(u: &Unit, x: &[f32], h: usize, w: usize, relu: bool) -> Vec<f32> {
    let n = h * w;
    let mut mid = take(u.cout * n);
    for (co, plane) in mid.chunks_exact_mut(n.max(1)).enumerate() {
        plane.fill(u.pb[co]);
    }
    matmul(&mut mid, &u.pw, x, u.cout, u.cin, n);
    let mut out = take(u.cout * n);
    depthwise(&mid, &mut out, u.cout, h, w, &u.dw, &u.db, relu);
    give(mid);
    out
}

/// 2x2 maximum with stride 2 (`h` and `w` even).
fn maxpool(x: &[f32], c: usize, h: usize, w: usize) -> Vec<f32> {
    let (oh, ow) = (h / 2, w / 2);
    let mut out = take(c * oh * ow);
    for ch in 0..c {
        for y in 0..oh {
            let (a, b) = (&x[ch * h * w + 2 * y * w..][..w], &x[ch * h * w + (2 * y + 1) * w..][..w]);
            let o = &mut out[ch * oh * ow + y * ow..][..ow];
            for ((o, a), b) in o.iter_mut().zip(a.chunks_exact(2)).zip(b.chunks_exact(2)) {
                *o = a[0].max(a[1]).max(b[0].max(b[1]));
            }
        }
    }
    out
}

/// `fine += coarse` doubled in both directions (nearest neighbour).
fn add_upsampled(fine: &mut [f32], coarse: &[f32], c: usize, h: usize, w: usize) {
    let (ch_, cw) = (h / 2, w / 2);
    for ch in 0..c {
        for y in 0..h {
            let src = &coarse[ch * ch_ * cw + (y / 2) * cw..][..cw];
            let dst = &mut fine[ch * h * w + y * w..][..w];
            for (d, &s) in dst.chunks_exact_mut(2).zip(src) {
                d[0] += s;
                d[1] += s;
            }
        }
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// The boxes of one level's cells at or above `threshold`, row by row, from
/// the head's six planes (class and objectness before the sigmoid, then the
/// box), as `FaceDetectorYNImpl::postProcess` decodes them.
pub fn decode(head: &[f32], rows: usize, cols: usize, stride: usize, threshold: f32, out: &mut Vec<Face>) {
    let n = rows * cols;
    let s = stride as f32;
    for r in 0..rows {
        for c in 0..cols {
            let i = r * cols + c;
            let score = (sigmoid(head[i]).clamp(0.0, 1.0) * sigmoid(head[n + i]).clamp(0.0, 1.0)).sqrt();
            if score < threshold {
                continue;
            }
            let (cx, cy) = ((c as f32 + head[2 * n + i]) * s, (r as f32 + head[3 * n + i]) * s);
            let (w, h) = (head[4 * n + i].exp() * s, head[5 * n + i].exp() * s);
            out.push(Face { x: cx - w / 2.0, y: cy - h / 2.0, w, h, score });
        }
    }
}

/// What `FaceDetectorYN` returns for the decoded `faces`: one box or none
/// as they are; otherwise OpenCV's `dnn::NMSBoxes` on the boxes truncated
/// to whole pixels, which keeps scores strictly above `score_threshold`,
/// the `top_k` highest of them, and in falling order of score every box
/// whose overlap (intersection over union) with each box kept so far is at
/// most `nms_threshold`.
pub fn nms(faces: &[Face], score_threshold: f32, nms_threshold: f32, top_k: usize) -> Vec<Face> {
    if faces.len() <= 1 {
        return faces.to_vec();
    }
    let boxes: Vec<[i64; 4]> = faces.iter().map(|f| [f.x as i32 as i64, f.y as i32 as i64, f.w as i32 as i64, f.h as i32 as i64]).collect();
    let mut order: Vec<usize> = (0..faces.len()).filter(|&i| faces[i].score > score_threshold).collect();
    // Stable: equal scores stay in decoding order.
    order.sort_by(|&a, &b| faces[b].score.partial_cmp(&faces[a].score).unwrap_or(std::cmp::Ordering::Equal));
    if top_k > 0 {
        order.truncate(top_k);
    }
    let overlap = |a: &[i64; 4], b: &[i64; 4]| -> f32 {
        let (aa, ab) = (a[2] * a[3], b[2] * b[3]);
        if aa + ab <= 0 {
            return 1.0;
        }
        let (iw, ih) = ((a[0] + a[2]).min(b[0] + b[2]) - a[0].max(b[0]), (a[1] + a[3]).min(b[1] + b[3]) - a[1].max(b[1]));
        let inter = if iw > 0 && ih > 0 { (iw * ih) as f64 } else { 0.0 };
        1.0 - (1.0 - inter / ((aa + ab) as f64 - inter)) as f32
    };
    let mut kept: Vec<usize> = Vec::new();
    for i in order {
        if kept.iter().all(|&k| overlap(&boxes[i], &boxes[k]) <= nms_threshold) {
            kept.push(i);
        }
    }
    kept.into_iter().map(|i| faces[i]).collect()
}

/// The first of the faces with the largest area (Python's `max` over
/// `w * h`).
pub fn largest(faces: &[Face]) -> Option<Face> {
    let mut best: Option<Face> = None;
    for f in faces {
        if best.is_none_or(|b| f.w * f.h > b.w * b.h) {
            best = Some(*f);
        }
    }
    best
}

impl YuNet {
    /// The head's planes for the three levels, with each level's rows and columns.
    fn forward(&self, img: &Rgb) -> Vec<(Vec<f32>, usize, usize)> {
        let pad = |v: u32| (v as usize - 1) / DIVISOR * DIVISOR + DIVISOR;
        let (ph, pw) = (pad(img.h), pad(img.w));
        let u = &self.units;
        let (h, w) = (ph / 2, pw / 2);
        let x = stem(&self.stem, img, ph, pw);
        let y = unit(&u[0], &x, h, w, true);
        give(x);
        let mut x = maxpool(&y, 16, h, w);
        give(y);
        let (h, w) = (h / 2, w / 2);
        for unit_ in &u[1..5] {
            let y = unit(unit_, &x, h, w, true);
            give(std::mem::replace(&mut x, y));
        }
        // Two units after each of three poolings: strides 8, 16, 32.
        let mut levels: Vec<(Vec<f32>, usize, usize)> = Vec::new();
        let (mut h, mut w) = (h, w);
        for l in 0..3 {
            let pooled = maxpool(if l == 0 { &x } else { &levels[l - 1].0 }, 64, h, w);
            (h, w) = (h / 2, w / 2);
            let a = unit(&u[5 + 2 * l], &pooled, h, w, true);
            give(pooled);
            levels.push((unit(&u[6 + 2 * l], &a, h, w, true), h, w));
            give(a);
        }
        give(x);
        // The neck, from the coarsest level down.
        let mut out: Vec<(Vec<f32>, usize, usize)> = Vec::new();
        let mut above: Option<Vec<f32>> = None;
        for l in (0..3).rev() {
            let (mut feat, h, w) = std::mem::take(&mut levels[l]);
            if let Some(above) = above.take() {
                add_upsampled(&mut feat, &above, 64, h, w);
                give(above);
            }
            let p = unit(&u[13 - l], &feat, h, w, true);
            give(feat);
            out.push((unit(&self.heads[l], &p, h, w, false), h, w));
            above = Some(p);
        }
        give(above.unwrap());
        out.reverse();
        out
    }

    /// The faces of `img` as `cv2.FaceDetectorYN.detect` gives them after
    /// `setInputSize` to the image's size: in falling order of score, in
    /// the image's pixels.
    pub fn detect(&self, img: &Rgb) -> Vec<Face> {
        if img.w == 0 || img.h == 0 {
            return Vec::new();
        }
        let mut faces = Vec::new();
        for ((head, rows, cols), stride) in self.forward(img).into_iter().zip(STRIDES) {
            decode(&head, rows, cols, stride, self.score_threshold, &mut faces);
            give(head);
        }
        nms(&faces, self.score_threshold, self.nms_threshold, self.top_k)
    }

    /// The largest face of `img`, looked for on a copy scaled down so that
    /// its longer side is `long_side` (never up), as `(x, y, w, h, score)`
    /// in `img`'s pixels: `largest_face` of the Python reference.
    pub fn largest_face(&self, img: &Rgb, long_side: u32) -> Option<[f32; 5]> {
        match shrink(img, long_side) {
            Some(small) => self.largest_in(&small, img.w, img.h),
            None => self.largest_in(img, img.w, img.h),
        }
    }

    /// The largest face of `small`, a copy of a `width` by `height` image,
    /// in that image's pixels.
    pub fn largest_in(&self, small: &Rgb, width: u32, height: u32) -> Option<[f32; 5]> {
        let f = largest(&self.detect(small))?;
        let (sx, sy) = ((width as f64 / small.w as f64) as f32, (height as f64 / small.h as f64) as f32);
        Some([f.x * sx, f.y * sy, f.w * sx, f.h * sy, f.score])
    }
}

// ---- the copy the detector looks at ----

/// `img` scaled down so that its longer side is `long_side`, the other side
/// rounded half to even (Python's `round`); `None` when it is that small
/// already.
pub fn shrink(img: &Rgb, long_side: u32) -> Option<Rgb> {
    let scale = (long_side as f64 / img.w.max(img.h) as f64).min(1.0);
    if scale == 1.0 || img.w == 0 || img.h == 0 {
        return None;
    }
    let side = |v: u32| ((v as f64 * scale).round_ties_even() as u32).max(1);
    Some(resize_area(img, side(img.w), side(img.h)))
}

/// Where the source pixels of each destination pixel start in `taps`, and
/// each tap's source index and weight (`computeResizeAreaTab`).
fn area_taps(ssize: usize, dsize: usize) -> (Vec<usize>, Vec<(usize, f32)>) {
    let scale = 1.0 / (dsize as f64 / ssize as f64);
    let (mut starts, mut taps) = (Vec::with_capacity(dsize + 1), Vec::new());
    for d in 0..dsize {
        starts.push(taps.len());
        let fs1 = d as f64 * scale;
        let fs2 = fs1 + scale;
        let cell = scale.min(ssize as f64 - fs1);
        let s2 = (fs2.floor() as usize).min(ssize - 1);
        let s1 = (fs1.ceil() as usize).min(s2);
        if s1 as f64 - fs1 > 1e-3 {
            taps.push((s1 - 1, ((s1 as f64 - fs1) / cell) as f32));
        }
        for s in s1..s2 {
            taps.push((s, (1.0 / cell) as f32));
        }
        if fs2 - s2 as f64 > 1e-3 {
            taps.push((s2, ((fs2 - s2 as f64).min(1.0).min(cell) / cell) as f32));
        }
    }
    starts.push(taps.len());
    (starts, taps)
}

/// `cv2.resize(img, (dw, dh), interpolation=cv2.INTER_AREA)` for a size no
/// larger than the image's: every destination pixel is the mean of the
/// source area it covers, pixels on the area's edge counted by the share
/// that is inside.
pub fn resize_area(img: &Rgb, dw: u32, dh: u32) -> Rgb {
    assert!(dw <= img.w && dh <= img.h && dw > 0 && dh > 0, "the area resize only scales down");
    if (dw, dh) == (img.w, img.h) {
        return img.clone();
    }
    let (sw, sh, dw, dh) = (img.w as usize, img.h as usize, dw as usize, dh as usize);
    let mut out = Rgb::new(dw as u32, dh as u32);
    let to_u8 = |v: f32| v.round_ties_even().clamp(0.0, 255.0) as u8;
    // Whole-number factors go through OpenCV's "fast" path: integer sums.
    if sw % dw == 0 && sh % dh == 0 {
        let (fx, fy) = (sw / dw, sh / dh);
        let inv = 1.0f32 / (fx * fy) as f32;
        for y in 0..dh {
            for x in 0..dw {
                for c in 0..3 {
                    let mut sum = 0u32;
                    for yy in 0..fy {
                        let row = &img.data[((y * fy + yy) * sw + x * fx) * 3 + c..];
                        sum += (0..fx).map(|xx| row[xx * 3] as u32).sum::<u32>();
                    }
                    out.data[(y * dw + x) * 3 + c] = if (fx, fy) == (2, 2) { ((sum + 2) >> 2) as u8 } else { to_u8(sum as f32 * inv) };
                }
            }
        }
        return out;
    }
    let (xs, xt) = area_taps(sw, dw);
    let (ys, yt) = area_taps(sh, dh);
    let n = dw * 3;
    let (mut buf, mut sum) = (vec![0f32; n], vec![0f32; n]);
    let mut have = usize::MAX;
    for y in 0..dh {
        for (k, &(sy, beta)) in yt[ys[y]..ys[y + 1]].iter().enumerate() {
            // A source row shared by two destination rows is summed once.
            if sy != have {
                let src = &img.data[sy * sw * 3..(sy + 1) * sw * 3];
                for (x, b) in buf.chunks_exact_mut(3).enumerate() {
                    let mut acc = [0f32; 3];
                    for &(sx, alpha) in &xt[xs[x]..xs[x + 1]] {
                        let p = &src[sx * 3..sx * 3 + 3];
                        for c in 0..3 {
                            acc[c] = muladd(p[c] as f32, alpha, acc[c]);
                        }
                    }
                    b.copy_from_slice(&acc);
                }
                have = sy;
            }
            if k == 0 {
                for (s, &b) in sum.iter_mut().zip(&buf) {
                    *s = beta * b;
                }
            } else {
                for (s, &b) in sum.iter_mut().zip(&buf) {
                    *s = muladd(beta, b, *s);
                }
            }
        }
        for (o, &s) in out.data[y * n..(y + 1) * n].iter_mut().zip(&sum) {
            *o = to_u8(s);
        }
    }
    out
}

/// `a * b + c` as OpenCV's build computes it in the area resize.
#[inline(always)]
fn muladd(a: f32, b: f32, c: f32) -> f32 {
    a * b + c
}

#[cfg(test)]
mod tests {
    use super::*;

    fn logit(p: f32) -> f32 {
        (p / (1.0 - p)).ln()
    }

    #[test]
    fn a_cell_decodes_to_its_box_and_score() {
        // Three rows of four cells at stride 8; only the cell in row 2, column 3 scores.
        let (rows, cols, n) = (3, 4, 12);
        let mut head = vec![0.0f32; 6 * n];
        head[..2 * n].fill(-20.0);
        let i = 2 * cols + 3;
        head[i] = logit(0.99);
        head[n + i] = logit(0.95);
        head[2 * n + i] = 0.5;
        head[3 * n + i] = 0.25;
        head[4 * n + i] = 2f32.ln();
        head[5 * n + i] = 0.0;
        let mut out = Vec::new();
        decode(&head, rows, cols, 8, 0.9, &mut out);
        assert_eq!(out.len(), 1);
        let f = out[0];
        // Centre ((3 + 0.5) * 8, (2 + 0.25) * 8) = (28, 18), 16 wide and 8 high.
        for (got, want) in [(f.x, 20.0), (f.y, 14.0), (f.w, 16.0), (f.h, 8.0), (f.score, (0.99f32 * 0.95).sqrt())] {
            assert!((got - want).abs() < 1e-4, "{got} for {want}");
        }
    }

    #[test]
    fn the_score_is_the_geometric_mean_and_the_threshold_keeps_equal_scores() {
        // Class 0.9 and objectness 0.8 give 0.849: under 0.9, over 0.84.
        let head = [logit(0.9), logit(0.8), 0.0, 0.0, 0.0, 0.0];
        let mut out = Vec::new();
        decode(&head, 1, 1, 16, 0.9, &mut out);
        assert!(out.is_empty());
        decode(&head, 1, 1, 16, 0.84, &mut out);
        assert_eq!(out.len(), 1);
        assert!((out[0].score - 0.72f32.sqrt()).abs() < 1e-5);
        // A box of one stride centred on the cell's corner.
        assert_eq!((out[0].x, out[0].y, out[0].w, out[0].h), (-8.0, -8.0, 16.0, 16.0));
    }

    #[test]
    fn cells_are_decoded_row_by_row() {
        let mut head = vec![20.0f32; 6 * 6];
        head[2 * 6..].fill(0.0);
        let mut out = Vec::new();
        decode(&head, 2, 3, 32, 0.9, &mut out);
        let centres: Vec<(f32, f32)> = out.iter().map(|f| (f.x + 16.0, f.y + 16.0)).collect();
        assert_eq!(centres, [(0.0, 0.0), (32.0, 0.0), (64.0, 0.0), (0.0, 32.0), (32.0, 32.0), (64.0, 32.0)]);
    }

    fn face(x: f32, y: f32, w: f32, h: f32, score: f32) -> Face {
        Face { x, y, w, h, score }
    }

    #[test]
    fn overlapping_boxes_give_way_to_the_higher_score() {
        let faces = [face(10.0, 10.0, 100.0, 100.0, 0.92), face(12.0, 8.0, 100.0, 100.0, 0.97), face(300.0, 300.0, 50.0, 50.0, 0.95), face(20.0, 20.0, 100.0, 100.0, 0.91)];
        let kept = nms(&faces, 0.9, 0.3, 5000);
        // In falling order of score; the first and last overlap the second by more than 0.3.
        assert_eq!(kept, [faces[1], faces[2]]);
    }

    #[test]
    fn boxes_that_overlap_by_no_more_than_the_threshold_are_both_kept() {
        // 100 by 100 boxes 54 apart: 46 * 100 shared of 15,400, an overlap of 0.2987.
        let near = [face(0.0, 0.0, 100.0, 100.0, 0.99), face(54.0, 0.0, 100.0, 100.0, 0.95)];
        assert_eq!(nms(&near, 0.9, 0.3, 5000).len(), 2);
        // 53 apart: 4,700 of 15,300, 0.3072.
        let nearer = [near[0], face(53.0, 0.0, 100.0, 100.0, 0.95)];
        assert_eq!(nms(&nearer, 0.9, 0.3, 5000), [near[0]]);
        // Exactly at the threshold is kept: 50 apart shares a third, with the threshold a third.
        let third = [near[0], face(50.0, 0.0, 100.0, 100.0, 0.95)];
        assert_eq!(nms(&third, 0.9, 1.0f32 - (1.0f64 - 5000.0 / 15000.0) as f32, 5000).len(), 2);
    }

    #[test]
    fn the_overlap_is_of_boxes_truncated_to_whole_pixels() {
        // As floats these are 53.1 apart and overlap by 0.3063; truncated they are 54 apart: 0.2987.
        let faces = [face(0.9, 0.0, 100.0, 100.0, 0.99), face(54.0, 0.0, 100.0, 100.0, 0.95)];
        assert_eq!(nms(&faces, 0.9, 0.3, 5000).len(), 2);
        // Boxes under a pixel have no area and count as the same box.
        let specks = [face(5.0, 5.0, 0.5, 0.5, 0.99), face(400.0, 400.0, 0.7, 0.7, 0.95)];
        assert_eq!(nms(&specks, 0.9, 0.3, 5000), [specks[0]]);
    }

    #[test]
    fn a_single_box_is_returned_as_it_is_and_several_must_beat_the_threshold() {
        // OpenCV only runs the suppression, with its strict comparison, on more than one box.
        let at = face(0.0, 0.0, 10.0, 10.0, 0.9);
        assert_eq!(nms(&[at], 0.9, 0.3, 5000), [at]);
        let above = face(200.0, 0.0, 10.0, 10.0, 0.93);
        assert_eq!(nms(&[at, above], 0.9, 0.3, 5000), [above]);
        assert!(nms(&[], 0.9, 0.3, 5000).is_empty());
    }

    #[test]
    fn only_the_highest_scores_go_into_the_suppression_and_ties_keep_their_order() {
        let faces: Vec<Face> = (0..5).map(|i| face(100.0 * i as f32, 0.0, 20.0, 20.0, [0.95, 0.99, 0.95, 0.91, 0.97][i])).collect();
        assert_eq!(nms(&faces, 0.9, 0.3, 3), [faces[1], faces[4], faces[0]]);
        assert_eq!(nms(&faces, 0.9, 0.3, 0).len(), 5);
        assert_eq!(nms(&faces, 0.9, 0.3, 5000), [faces[1], faces[4], faces[0], faces[2], faces[3]]);
    }

    #[test]
    fn the_largest_face_is_the_first_of_the_largest() {
        let faces = [face(0.0, 0.0, 10.0, 20.0, 0.99), face(50.0, 0.0, 30.0, 20.0, 0.95), face(90.0, 0.0, 20.0, 30.0, 0.93)];
        assert_eq!(largest(&faces), Some(faces[1]));
        assert_eq!(largest(&[]), None);
    }

    /// The unit computed tap by tap.
    fn unit_slowly(u: &Unit, x: &[f32], h: usize, w: usize, relu: bool) -> Vec<f32> {
        let n = h * w;
        let mut mid = vec![0f32; u.cout * n];
        for co in 0..u.cout {
            for i in 0..n {
                mid[co * n + i] = u.pb[co] + (0..u.cin).map(|ci| u.pw[co * u.cin + ci] * x[ci * n + i]).sum::<f32>();
            }
        }
        let mut out = vec![0f32; u.cout * n];
        for co in 0..u.cout {
            for y in 0..h as i64 {
                for xx in 0..w as i64 {
                    let mut v = u.db[co];
                    for ky in 0..3 {
                        for kx in 0..3 {
                            let (sy, sx) = (y + ky - 1, xx + kx - 1);
                            if sy >= 0 && sy < h as i64 && sx >= 0 && sx < w as i64 {
                                v += u.dw[co * 9 + (ky * 3 + kx) as usize] * mid[co * n + (sy * w as i64 + sx) as usize];
                            }
                        }
                    }
                    out[co * n + (y * w as i64 + xx) as usize] = if relu { v.max(0.0) } else { v };
                }
            }
        }
        out
    }

    fn noise(n: usize, seed: u32) -> Vec<f32> {
        let mut s = seed;
        (0..n)
            .map(|_| {
                s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (s >> 8) as f32 / (1 << 24) as f32 - 0.5
            })
            .collect()
    }

    #[test]
    fn a_unit_is_a_pointwise_then_a_padded_depthwise_convolution() {
        let u = Unit { cin: 5, cout: 7, pw: noise(35, 1), pb: noise(7, 2), dw: noise(63, 3), db: noise(7, 4) };
        for (h, w) in [(6, 9), (1, 1), (1, 4), (3, 1), (2, 2), (4, 3)] {
            let x = noise(5 * h * w, 5);
            for relu in [false, true] {
                let (fast, slow) = (unit(&u, &x, h, w, relu), unit_slowly(&u, &x, h, w, relu));
                assert_eq!(fast.len(), slow.len());
                for (a, b) in fast.iter().zip(&slow) {
                    assert!((a - b).abs() < 1e-5, "{h}x{w}: {a} against {b}");
                }
            }
        }
    }

    #[test]
    fn the_stem_reads_blue_green_red_with_stride_two_and_zero_padding() {
        let mut img = Rgb::new(5, 3);
        img.data = (0..45).map(|i| (i * 5 % 251) as u8).collect();
        let conv = Conv { dims: [16, 3, 3, 3], w: noise(16 * 27, 9), b: noise(16, 10) };
        // Padded to 32 by 32, so 16 by 16 out.
        let out = stem(&conv, &img, 32, 32);
        assert_eq!(out.len(), 16 * 256);
        let pixel = |y: i64, x: i64, c: usize| if y >= 0 && y < 3 && x >= 0 && x < 5 { img.data[((y * 5 + x) * 3) as usize + (2 - c)] as f32 } else { 0.0 };
        for co in [0, 7, 15] {
            for (oy, ox) in [(0, 0), (0, 2), (1, 1), (1, 2), (2, 3), (5, 5)] {
                let mut want = conv.b[co];
                for ci in 0..3 {
                    for ky in 0..3 {
                        for kx in 0..3 {
                            want += conv.w[co * 27 + ci * 9 + ky * 3 + kx] * pixel(2 * oy + ky as i64 - 1, 2 * ox + kx as i64 - 1, ci);
                        }
                    }
                }
                let got = out[co * 256 + (oy * 16 + ox) as usize];
                assert!((got - want.max(0.0)).abs() < 1e-2, "channel {co} at ({oy}, {ox}): {got} for {}", want.max(0.0));
            }
        }
    }

    #[test]
    fn pooling_takes_the_maximum_of_each_two_by_two() {
        let x = [1.0, 5.0, 2.0, 0.0, 3.0, 4.0, 9.0, -1.0, -3.0, -2.0, 0.5, 0.25, -4.0, -8.0, 0.125, 0.0];
        assert_eq!(maxpool(&x, 1, 4, 4), [5.0, 9.0, -2.0, 0.5]);
        let mut fine = vec![1.0f32; 16];
        add_upsampled(&mut fine, &[10.0, 20.0, 30.0, 40.0], 1, 4, 4);
        assert_eq!(fine, [11.0, 11.0, 21.0, 21.0, 11.0, 11.0, 21.0, 21.0, 31.0, 31.0, 41.0, 41.0, 31.0, 31.0, 41.0, 41.0]);
    }

    /// A network whose weights are all zero: every layer gives its bias.
    fn blank(class_bias: f32) -> YuNet {
        let conv = |cout: usize, cin: usize, k: usize, bias: f32| Conv { dims: [cout, cin, k, k], w: vec![0.0; cout * cin * k * k], b: vec![bias; cout] };
        let mut convs = vec![conv(16, 3, 3, 0.0)];
        for (cin, cout) in [(16, 16), (16, 16), (16, 32), (32, 32), (32, 64)].into_iter().chain(std::iter::repeat_n((64, 64), 9)) {
            convs.extend([conv(cout, cin, 1, 0.0), conv(cout, 1, 3, 0.0)]);
        }
        for (k, bias) in [(1, class_bias), (4, 0.0), (1, 20.0), (10, 0.0)] {
            for _ in 0..3 {
                convs.extend([conv(k, 64, 1, 0.0), conv(k, 1, 3, bias)]);
            }
        }
        YuNet::from_convs(convs).unwrap()
    }

    #[test]
    fn every_cell_of_the_padded_image_is_decoded_at_three_strides() {
        // 100 by 70 is padded to 128 by 96: 16 * 12 + 8 * 6 + 4 * 3 cells.
        let img = Rgb::new(100, 70);
        let net = blank(20.0);
        let levels = net.forward(&img);
        assert_eq!(levels.iter().map(|l| (l.1, l.2)).collect::<Vec<_>>(), [(12, 16), (6, 8), (3, 4)]);
        let mut faces = Vec::new();
        for ((head, rows, cols), stride) in levels.into_iter().zip(STRIDES) {
            decode(&head, rows, cols, stride, 0.9, &mut faces);
        }
        assert_eq!(faces.len(), 252);
        // Each a box of one stride centred on its cell's corner; the last is at stride 32.
        assert_eq!(faces[0], face(-4.0, -4.0, 8.0, 8.0, faces[0].score));
        assert_eq!(faces[251], face(80.0, 48.0, 32.0, 32.0, faces[0].score));
        // All score alike, so the stride-8 boxes come first and, touching but not overlapping, all stay.
        let kept = net.detect(&img);
        assert!(kept.len() >= 192 && kept[..192].iter().all(|f| f.w == 8.0));
        assert_eq!(largest(&kept).map(|f| f.w), Some(32.0));
        // With no cell at the threshold there is no face.
        assert!(blank(-20.0).detect(&img).is_empty());
    }

    #[test]
    fn a_wrong_graph_is_refused() {
        let conv = Conv { dims: [16, 3, 3, 3], w: vec![0.0; 432], b: vec![0.0; 16] };
        assert!(YuNet::from_convs(vec![conv.clone(); 3]).is_err());
        assert!(YuNet::from_convs(vec![conv; 53]).is_err());
    }

    /// A protobuf field with a length.
    fn pb(field: u8, body: &[u8]) -> Vec<u8> {
        assert!(body.len() < 1 << 14);
        let mut v = vec![field << 3 | 2];
        if body.len() < 128 {
            v.push(body.len() as u8);
        } else {
            v.extend([body.len() as u8 | 0x80, (body.len() >> 7) as u8]);
        }
        v.extend(body);
        v
    }

    #[test]
    fn convolutions_are_read_from_an_onnx_graph_in_node_order() {
        let tensor = |name: &str, dims: &[u8], values: &[f32], raw: bool| {
            let mut t: Vec<u8> = dims.iter().flat_map(|&d| [0x08, d]).collect();
            t.extend([0x10, 1]);
            t.extend(pb(8, name.as_bytes()));
            let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
            t.extend(pb(if raw { 9 } else { 4 }, &bytes));
            pb(5, &t)
        };
        let node = |op: &str, inputs: &[&str]| {
            let mut n: Vec<u8> = inputs.iter().flat_map(|i| pb(1, i.as_bytes())).collect();
            n.extend(pb(4, op.as_bytes()));
            pb(1, &n)
        };
        let mut graph = node("Conv", &["x", "w2", "b2"]);
        graph.extend(node("Relu", &["y"]));
        graph.extend(node("Conv", &["y", "w1", "b1"]));
        graph.extend(tensor("w1", &[1, 2, 1, 1], &[1.5, -2.0], true));
        graph.extend(tensor("b1", &[1], &[0.25], false));
        graph.extend(tensor("w2", &[2, 1, 1, 1], &[3.0, 4.0], false));
        graph.extend(tensor("b2", &[2], &[5.0, 6.0], true));
        let mut model = vec![0x08, 7];
        model.extend(pb(7, &graph));
        let convs = onnx_convs(&model).unwrap();
        assert_eq!(convs.len(), 2);
        assert_eq!((convs[0].dims, &convs[0].w[..], &convs[0].b[..]), ([2, 1, 1, 1], &[3.0, 4.0][..], &[5.0, 6.0][..]));
        assert_eq!((convs[1].dims, &convs[1].w[..], &convs[1].b[..]), ([1, 2, 1, 1], &[1.5, -2.0][..], &[0.25][..]));
        // A convolution whose weight is missing is an error.
        assert!(onnx_convs(&pb(7, &node("Conv", &["x", "nowhere", "b"]))).is_err());
    }

    #[test]
    fn the_area_resize_averages_what_each_pixel_covers() {
        // Halving: the rounded mean of each 2x2.
        let mut img = Rgb::new(4, 2);
        for (i, v) in [10u8, 20, 30, 41, 1, 2, 3, 5].into_iter().enumerate() {
            img.data[i * 3..i * 3 + 3].copy_from_slice(&[v, v / 2, 255 - v]);
        }
        let half = resize_area(&img, 2, 1);
        assert_eq!(half.data, [8, 4, 247, 20, 10, 235]);
        // Three to two: the middle pixel is shared half and half.
        let mut row = Rgb::new(3, 1);
        row.data = vec![30, 0, 0, 60, 0, 0, 120, 0, 0];
        let two = resize_area(&row, 2, 1);
        assert_eq!((two.data[0], two.data[3]), (40, 100));
        // By three in both directions: the mean of nine.
        let mut nine = Rgb::new(3, 3);
        for (i, p) in nine.data.chunks_exact_mut(3).enumerate() {
            p.fill(i as u8 * 10);
        }
        assert_eq!(resize_area(&nine, 1, 1).data, [40, 40, 40]);
        // A flat image stays flat at an awkward ratio.
        let mut flat = Rgb::new(37, 23);
        flat.data.fill(200);
        assert!(resize_area(&flat, 16, 10).data.iter().all(|&v| v == 200));
    }

    #[test]
    fn the_copy_looked_at_has_the_longer_side_asked_for() {
        let img = Rgb::new(1024, 683);
        let small = shrink(&img, 640).unwrap();
        // 683 * 0.625 = 426.875.
        assert_eq!((small.w, small.h), (640, 427));
        // Halves round to even, as Python's `round`: 100 * 0.625 = 62.5.
        assert_eq!(shrink(&Rgb::new(1024, 100), 640).map(|s| s.h), Some(62));
        assert!(shrink(&Rgb::new(640, 480), 640).is_none());
        assert!(shrink(&Rgb::new(300, 200), 640).is_none());
    }
}
