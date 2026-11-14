//! Frame features for the temporal-head experiment
//! (`docs/runbooks/temporal-head.md`): what `tintype dump-video-features`
//! writes and [`crate::temporal_head`] reads.
//!
//! Scoring a clip keeps one number per frame. Asking whether anything
//! *between* frames tells generated video from authentic video needs more:
//! the patch tokens of each sampled frame, taken with one view geometry for
//! the whole clip so that a cell of one frame and the same cell of the next
//! look at the same place; the time each frame was actually shown; and
//! enough about the frames in between to tell a head that reads motion from
//! one that reads cuts, repeated frames or the frame rate.
//!
//! A dump is a directory of four files:
//!
//! - `meta.json`: [`Meta`], how the dump was made and the record layout.
//! - `clips.tsv`: one row per clip that decoded ([`Clip`]), with the
//!   manifest's columns (grouping columns included) and the clip's cadence.
//! - `frames.tsv`: one row per sampled frame ([`Frame`]), with its position
//!   in the stream, its presentation time, its logits, and what the frames
//!   read since the previous sample did.
//! - `features.f32`: one record of `meta.record` little-endian `f32` per
//!   sampled frame. A record holds, for each block in `meta.blocks` in turn,
//!   the class token (`dim` values) and then the `grid * grid` pooled patch
//!   tokens of the whole-frame view, row by row (`dim` values each). Clip
//!   `c`'s records start at record `first_record` of its row in `clips.tsv`.
//!
//! Nothing here is used by `detect`, `score-video` or the server.
use crate::detector::{Detector, Device, ModelSpec, ViewSpec};
use crate::embed::{pool_grid, Embedder};
use crate::image_io::View;
use crate::ingest::{Item, GROUP_COLUMNS};
use crate::yuv::Trace;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Version of the layout described in the module documentation.
pub const FORMAT: u32 = 1;

/// A frame whose luma thumbnail is within this many 8-bit levels (mean
/// absolute difference) of the frame before it counts as a repeat of it.
pub const REPEAT_BELOW: f32 = 0.01;

const CLIP_COLUMNS: [&str; 21] = [
    "path", "label", "generator", "dataset", "parent_id", "scene_id", "source_id", "generator_family", "transform", "first_record", "frames", "width", "height", "fps", "duration", "frames_read", "gap_median", "gap_min", "gap_max",
    "median_diff", "repeats",
];
const FRAME_COLUMNS: [&str; 13] = ["path", "slot", "index", "pts", "fit_logit", "frame_logit", "view_logits", "gap_frames", "gap_s", "fit_diff", "max_diff", "sum_diff", "repeats"];

/// How a dump was made (`meta.json`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub format: u32,
    /// The checkpoint the tokens and logits come from.
    pub weights: String,
    pub weights_sha256: String,
    pub device: String,
    pub img: usize,
    pub dim: usize,
    /// Blocks whose tokens each record holds, in record order.
    pub blocks: Vec<usize>,
    /// Cells along one side of the pooled token map.
    pub grid: usize,
    /// `f32` values per frame record: `blocks.len() * (1 + grid * grid) * dim`.
    pub record: usize,
    /// Frames asked for per clip.
    pub frames: usize,
    pub max_seconds: f64,
    /// 0: frames spread over the clip. Otherwise the frames are pairs, the
    /// second frame of each this many frames after the first in the stream.
    pub follow: usize,
    /// Frames averaged into each sample (1 = none; the synthetic-blur control).
    pub blur: usize,
    /// The views scored on every frame, in the order of `view_logits`.
    pub views: Vec<ViewSpec>,
    /// Which of `views` is the whole-frame view the tokens come from.
    pub fit_view: usize,
    pub clips: usize,
    pub frame_records: u64,
    pub unreadable: usize,
    /// Seconds from the first clip to the last, decoding and inference together.
    pub wall_s: f64,
    /// Seconds inside the network (token and logit runs).
    pub model_s: f64,
    /// Seconds decoding and cropping, summed over the worker threads.
    pub decode_s: f64,
    pub threads: usize,
    pub repeat_below: f32,
}

impl Meta {
    /// Tokens per block in a record: the class token and the pooled cells.
    pub fn cells(&self) -> usize {
        1 + self.grid * self.grid
    }
}

/// One clip of a dump (`clips.tsv`).
#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    pub item: Item,
    /// Index of the clip's first record in `features.f32`.
    pub first_record: u64,
    /// Sampled frames (records) the clip has.
    pub frames: usize,
    pub width: u32,
    pub height: u32,
    /// Nominal frame rate from the container, 0 when it states none.
    pub fps: f64,
    pub duration: f64,
    pub stream: Stream,
}

/// What the frames read from a clip did, from the first to the last sample.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stream {
    /// Frames read.
    pub frames_read: usize,
    /// Median, smallest and largest time between consecutive frames, in
    /// seconds, from their presentation times (0 with fewer than two frames).
    pub gap_median: f64,
    pub gap_min: f64,
    pub gap_max: f64,
    /// Median thumbnail difference between consecutive frames, in 8-bit levels.
    pub median_diff: f32,
    /// Frames that repeat the frame before them ([`REPEAT_BELOW`]).
    pub repeats: usize,
}

impl Stream {
    pub fn of(trace: &Trace) -> Self {
        let median = |mut v: Vec<f64>| {
            v.sort_by(f64::total_cmp);
            if v.is_empty() { 0.0 } else { v[v.len() / 2] }
        };
        let gaps: Vec<f64> = trace.pts.windows(2).map(|w| w[1] - w[0]).collect();
        let diffs: Vec<f64> = trace.diff.iter().skip(1).map(|&d| d as f64).collect();
        Self {
            frames_read: trace.pts.len(),
            gap_min: if gaps.is_empty() { 0.0 } else { gaps.iter().copied().fold(f64::INFINITY, f64::min) },
            gap_max: gaps.iter().copied().fold(0.0, f64::max),
            gap_median: median(gaps),
            repeats: diffs.iter().filter(|&&d| d <= REPEAT_BELOW as f64).count(),
            median_diff: median(diffs) as f32,
        }
    }
}

/// One sampled frame of a dump (`frames.tsv`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    /// Position among the frames read from the clip, from 0.
    pub index: usize,
    /// Presentation time in seconds, as the decoder reported it.
    pub pts: f64,
    /// Logit of the whole-frame view (the view the tokens come from).
    pub fit_logit: f32,
    /// Mean logit of every view scored on the frame: the frame logit
    /// `score-video` writes.
    pub frame_logit: f32,
    pub view_logits: Vec<f32>,
    /// The rest describes the stretch from the previous sample of the clip
    /// to this one, and is zero for the first sample.
    ///
    /// Frames between the two samples in the stream (0: the same frame twice).
    pub gap_frames: i64,
    /// Seconds between their presentation times.
    pub gap_s: f64,
    /// Mean absolute difference of the two whole-frame views, 8-bit levels.
    pub fit_diff: f32,
    /// Largest and summed thumbnail difference between consecutive frames
    /// read in the stretch, and how many of those frames were repeats.
    pub max_diff: f32,
    pub sum_diff: f32,
    pub repeats: usize,
}

/// `(largest difference, summed difference, repeats)` over the frames read
/// after stream position `a` up to and including `b` (in either order).
pub fn stretch(trace: &Trace, a: usize, b: usize) -> (f32, f32, usize) {
    let (lo, hi) = (a.min(b), a.max(b).min(trace.diff.len().saturating_sub(1)));
    let d = if lo < hi { &trace.diff[lo + 1..=hi] } else { &[] };
    (d.iter().copied().fold(0.0, f32::max), d.iter().sum(), d.iter().filter(|&&x| x <= REPEAT_BELOW).count())
}

enum Store {
    File(std::sync::Mutex<std::fs::File>),
    Memory(Vec<Vec<f32>>),
}

/// A dump opened for reading. The rows are in memory; the tokens are read
/// from `features.f32` clip by clip, so a dump larger than memory is fine.
pub struct FeatureSet {
    pub meta: Meta,
    pub clips: Vec<Clip>,
    /// The sampled frames of each clip, in sampling order.
    pub frames: Vec<Vec<Frame>>,
    store: Store,
}

fn clip_line(c: &Clip) -> String {
    let it = &c.item;
    let mut f: Vec<String> = vec![it.path.display().to_string(), it.label.to_string(), it.generator.clone(), it.dataset.clone()];
    f.extend((0..GROUP_COLUMNS.len()).map(|k| it.group(k).unwrap_or("-").to_string()));
    let s = &c.stream;
    f.extend([c.first_record.to_string(), c.frames.to_string(), c.width.to_string(), c.height.to_string(), format!("{:.4}", c.fps), format!("{:.4}", c.duration)]);
    f.extend([s.frames_read.to_string(), format!("{:.6}", s.gap_median), format!("{:.6}", s.gap_min), format!("{:.6}", s.gap_max), format!("{:.4}", s.median_diff), s.repeats.to_string()]);
    f.join("\t")
}

fn frame_line(path: &str, slot: usize, f: &Frame) -> String {
    let views: Vec<String> = f.view_logits.iter().map(|v| format!("{v:.5}")).collect();
    format!("{path}\t{slot}\t{}\t{:.6}\t{:.5}\t{:.5}\t{}\t{}\t{:.6}\t{:.4}\t{:.4}\t{:.4}\t{}", f.index, f.pts, f.fit_logit, f.frame_logit, views.join(","), f.gap_frames, f.gap_s, f.fit_diff, f.max_diff, f.sum_diff, f.repeats)
}

/// Write the three text files of a dump (`features.f32` is written as the
/// records are made).
pub fn write_index(dir: &Path, meta: &Meta, clips: &[Clip], frames: &[Vec<Frame>]) -> Result<()> {
    std::fs::write(dir.join("meta.json"), serde_json::to_string_pretty(meta)? + "\n")?;
    let mut w = std::io::BufWriter::new(std::fs::File::create(dir.join("clips.tsv"))?);
    writeln!(w, "{}", CLIP_COLUMNS.join("\t"))?;
    for c in clips {
        writeln!(w, "{}", clip_line(c))?;
    }
    w.flush()?;
    let mut w = std::io::BufWriter::new(std::fs::File::create(dir.join("frames.tsv"))?);
    writeln!(w, "{}", FRAME_COLUMNS.join("\t"))?;
    for (c, fr) in clips.iter().zip(frames) {
        let path = c.item.path.display().to_string();
        for (slot, f) in fr.iter().enumerate() {
            writeln!(w, "{}", frame_line(&path, slot, f))?;
        }
    }
    Ok(w.flush()?)
}

impl FeatureSet {
    pub fn open(dir: &Path) -> Result<Self> {
        let read = |name: &str| std::fs::read_to_string(dir.join(name)).with_context(|| format!("reading {}", dir.join(name).display()));
        let meta: Meta = serde_json::from_str(&read("meta.json")?)?;
        anyhow::ensure!(meta.format == FORMAT, "{} is feature format {}, this build reads format {FORMAT}", dir.display(), meta.format);
        anyhow::ensure!(meta.record == meta.blocks.len() * meta.cells() * meta.dim, "{}: the record size does not match the blocks, grid and width", dir.display());
        let (clip_text, frame_text) = (read("clips.tsv")?, read("frames.tsv")?);
        let mut lines = clip_text.lines();
        anyhow::ensure!(lines.next() == Some(CLIP_COLUMNS.join("\t").as_str()), "{}: unexpected clips.tsv columns", dir.display());
        let mut clips = Vec::new();
        for l in lines.filter(|l| !l.is_empty()) {
            let f: Vec<&str> = l.split('\t').collect();
            anyhow::ensure!(f.len() == CLIP_COLUMNS.len(), "bad clips.tsv line: {l}");
            let mut item = Item { path: f[0].into(), label: f[1].parse()?, generator: f[2].into(), dataset: f[3].into(), extra: Vec::new() };
            for (k, v) in f[4..9].iter().enumerate().filter(|(_, v)| **v != "-") {
                item.set_group(k, v);
            }
            let stream = Stream { frames_read: f[15].parse()?, gap_median: f[16].parse()?, gap_min: f[17].parse()?, gap_max: f[18].parse()?, median_diff: f[19].parse()?, repeats: f[20].parse()? };
            clips.push(Clip { item, first_record: f[9].parse()?, frames: f[10].parse()?, width: f[11].parse()?, height: f[12].parse()?, fps: f[13].parse()?, duration: f[14].parse()?, stream });
        }
        let mut lines = frame_text.lines();
        anyhow::ensure!(lines.next() == Some(FRAME_COLUMNS.join("\t").as_str()), "{}: unexpected frames.tsv columns", dir.display());
        // Rows follow the clips, each clip's frames in sampling order.
        let mut frames: Vec<Vec<Frame>> = clips.iter().map(|c| Vec::with_capacity(c.frames)).collect();
        let mut at = 0usize;
        for l in lines.filter(|l| !l.is_empty()) {
            let f: Vec<&str> = l.split('\t').collect();
            anyhow::ensure!(f.len() == FRAME_COLUMNS.len(), "bad frames.tsv line: {l}");
            while at < clips.len() && frames[at].len() == clips[at].frames {
                at += 1;
            }
            anyhow::ensure!(at < clips.len() && clips[at].item.path.as_os_str() == f[0] && f[1].parse::<usize>()? == frames[at].len(), "frames.tsv does not follow clips.tsv at: {l}");
            let view_logits = f[6].split(',').filter(|v| !v.is_empty()).map(|v| v.parse::<f32>()).collect::<Result<_, _>>()?;
            frames[at].push(Frame { index: f[2].parse()?, pts: f[3].parse()?, fit_logit: f[4].parse()?, frame_logit: f[5].parse()?, view_logits, gap_frames: f[7].parse()?, gap_s: f[8].parse()?, fit_diff: f[9].parse()?, max_diff: f[10].parse()?, sum_diff: f[11].parse()?, repeats: f[12].parse()? });
        }
        anyhow::ensure!(clips.iter().zip(&frames).all(|(c, f)| c.frames == f.len()), "{}: frames.tsv is missing rows", dir.display());
        let file = std::fs::File::open(dir.join("features.f32")).with_context(|| format!("opening {}", dir.join("features.f32").display()))?;
        let need = clips.iter().map(|c| c.first_record + c.frames as u64).max().unwrap_or(0) * meta.record as u64 * 4;
        anyhow::ensure!(file.metadata()?.len() >= need, "{}: features.f32 is shorter than clips.tsv says", dir.display());
        Ok(Self { meta, clips, frames, store: Store::File(std::sync::Mutex::new(file)) })
    }

    /// A set held in memory: `tokens[c]` is clip `c`'s records, one after another.
    pub fn in_memory(meta: Meta, clips: Vec<Clip>, frames: Vec<Vec<Frame>>, tokens: Vec<Vec<f32>>) -> Self {
        Self { meta, clips, frames, store: Store::Memory(tokens) }
    }

    /// Clip `c`'s records: `[frames, meta.record]`.
    pub fn records(&self, c: usize) -> Result<Vec<f32>> {
        match &self.store {
            Store::Memory(t) => Ok(t[c].clone()),
            Store::File(f) => {
                let clip = &self.clips[c];
                let mut bytes = vec![0u8; clip.frames * self.meta.record * 4];
                {
                    let mut f = f.lock().unwrap();
                    f.seek(SeekFrom::Start(clip.first_record * self.meta.record as u64 * 4))?;
                    f.read_exact(&mut bytes)?;
                }
                Ok(bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect())
            }
        }
    }

    /// Clip `c`'s tokens of one stored block: `[frames, meta.cells(), dim]`,
    /// the class token first in each frame.
    pub fn block_tokens(&self, c: usize, block: usize) -> Result<Vec<f32>> {
        let at = self.meta.blocks.iter().position(|&b| b == block).ok_or_else(|| anyhow::anyhow!("block {block} is not in this dump (it holds {:?})", self.meta.blocks))?;
        let per = self.meta.cells() * self.meta.dim;
        let all = self.records(c)?;
        Ok(all.chunks_exact(self.meta.record).flat_map(|r| r[at * per..(at + 1) * per].iter().copied()).collect())
    }

    pub fn labels(&self) -> Vec<u8> {
        self.clips.iter().map(|c| c.item.label).collect()
    }
}

/// How `dump-video-features` samples and what it stores.
#[derive(Clone, Debug)]
pub struct DumpOptions {
    /// Frames per clip; `None` is the checkpoint's video profile.
    pub frames: Option<usize>,
    pub max_seconds: f64,
    pub grid: usize,
    /// Blocks to store; empty means the last block.
    pub blocks: Vec<usize>,
    /// See [`Meta::follow`].
    pub follow: usize,
    /// See [`Meta::blur`].
    pub blur: usize,
    /// Score the whole-frame view only (one view per frame instead of the
    /// profile's six): the frame logit is then that view's.
    pub fit_only: bool,
    pub device: Device,
    /// Decode threads.
    pub threads: usize,
}

struct Prepared {
    info: Option<crate::video::VideoInfo>,
    /// `(index, pts)` of each sample.
    at: Vec<(usize, f64)>,
    /// The whole-frame view of every sample, then every other view of every sample.
    fit: Vec<u8>,
    others: Vec<u8>,
    fit_diff: Vec<f32>,
    trace: Trace,
    seconds: f64,
}

/// Logits for any number of views from a detector whose GPU graph has a
/// fixed batch (the last batch is padded; the CPU engine takes what it gets).
fn logits_of(det: &Detector, x: &[u8], vl: usize, pad: bool) -> Vec<f32> {
    let mut out = Vec::with_capacity(x.len() / vl);
    let mut buf = Vec::new();
    for chunk in x.chunks(det.batch * vl) {
        let k = chunk.len() / vl;
        if pad && k < det.batch {
            buf.clear();
            buf.extend_from_slice(chunk);
            buf.resize(det.batch * vl, 0);
            out.extend_from_slice(&det.logits(&buf)[..k]);
        } else {
            out.extend(det.logits(chunk));
        }
    }
    out
}

/// Write a feature dump of every clip in `manifests` to the directory `out`.
///
/// Each clip is sampled the way `score-video` samples it (or in pairs, or
/// blurred: see [`DumpOptions`]) and every sample is scored with the video
/// profile's views. The tokens come from the profile's whole-frame view
/// only: it has one geometry for every frame of a clip, which the
/// texture-ranked native crops do not (they move from frame to frame).
pub fn dump(weights: &Path, manifests: &[PathBuf], out: &Path, o: &DumpOptions) -> Result<Meta> {
    let mut items = Vec::new();
    for m in manifests {
        items.extend(crate::ingest::read_manifest(m)?);
    }
    anyhow::ensure!(!items.is_empty(), "the manifests list no clips");
    let spec = ModelSpec::load_for(weights)?;
    let (views, profile_frames) = match &spec.video {
        Some(v) => (v.views.clone(), v.frames),
        None => (spec.views.clone(), 8),
    };
    let frames = o.frames.unwrap_or(profile_frames);
    anyhow::ensure!(frames >= 1, "at least one frame per clip");
    anyhow::ensure!(o.follow == 0 || frames % 2 == 0, "pairs need an even number of frames");
    let fit_at = views.iter().position(|v| matches!(v, ViewSpec::Fit { .. })).ok_or_else(|| anyhow::anyhow!("the video profile of {} has no whole-frame (`fit`) view to take aligned tokens from", weights.display()))?;
    // The whole-frame view first, then the others in profile order.
    let others: Vec<ViewSpec> = if o.fit_only { Vec::new() } else { views.iter().enumerate().filter(|(i, _)| *i != fit_at).map(|(_, v)| *v).collect() };
    let scored: Vec<ViewSpec> = std::iter::once(views[fit_at]).chain(others.iter().copied()).collect();
    let fit_view = View::from(views[fit_at]);
    let depth = crate::vit::VitConfig::by_name(&spec.arch, spec.img).map_or(0, |c| c.depth);
    let blocks = if o.blocks.is_empty() { vec![depth] } else { o.blocks.clone() };
    // One clip's worth of views per run.
    let emb = Embedder::load(weights, &spec, frames, o.device, &blocks)?;
    let (side, dim) = (emb.side(), emb.dim());
    anyhow::ensure!(o.grid >= 1 && side % o.grid == 0, "--grid must divide the {side}x{side} token map");
    let det = if others.is_empty() {
        None
    } else {
        let mut s = spec.clone();
        s.views = others.clone();
        s.early_exit = None;
        Some(Detector::load(weights, Some(s), frames * others.len(), o.device)?)
    };
    let pre = det.as_ref().map(|d| d.preprocessor());
    let vl = 3 * spec.img * spec.img;
    let sampling = crate::video::Sampling { frames, max_seconds: o.max_seconds, consecutive: false };
    std::fs::create_dir_all(out)?;
    let mut features = std::io::BufWriter::new(std::fs::File::create(out.join("features.f32"))?);
    let record = blocks.len() * (1 + o.grid * o.grid) * dim;

    let next = AtomicUsize::new(0);
    let start = std::time::Instant::now();
    let (mut model_s, mut decode_s) = (0f64, 0f64);
    let mut done: Vec<(usize, Clip, Vec<Frame>)> = Vec::new();
    let mut failed: Vec<(usize, String)> = Vec::new();
    let mut records = 0u64;
    let threads = o.threads.max(1);
    let (follow, blur, others_len) = (o.follow, o.blur, others.len() * vl);
    std::thread::scope(|s| -> Result<()> {
        let (tx, rx) = crossbeam_channel::bounded::<(usize, Result<Prepared>)>(threads * 2);
        for _ in 0..threads {
            let tx = tx.clone();
            let (next, items, pre) = (&next, &items, &pre);
            s.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= items.len() {
                    break;
                }
                let t = std::time::Instant::now();
                let path = &items[i].path;
                let prepared = crate::video::sample_frames_timed(path, sampling, follow, blur).map(|(samples, trace)| {
                    let mut p = Prepared { info: crate::video::probe(path).ok(), at: samples.iter().map(|s| (s.index, s.pts)).collect(), fit: Vec::with_capacity(samples.len() * vl), others: Vec::new(), fit_diff: Vec::with_capacity(samples.len()), trace, seconds: 0.0 };
                    for s in &samples {
                        let crop = fit_view.apply(&s.rgb);
                        let before = p.fit.len().checked_sub(vl).map(|at| &p.fit[at..]);
                        p.fit_diff.push(before.map_or(0.0, |b| b.iter().zip(&crop.data).map(|(a, b)| a.abs_diff(*b) as u64).sum::<u64>() as f32 / vl as f32));
                        p.fit.extend_from_slice(&crop.data);
                    }
                    if let Some(pre) = pre {
                        p.others = vec![0u8; samples.len() * others_len];
                        for (s, views) in samples.iter().zip(p.others.chunks_exact_mut(others_len)) {
                            pre.apply(&s.rgb, views);
                        }
                    }
                    p.seconds = t.elapsed().as_secs_f64();
                    p
                });
                if tx.send((i, prepared)).is_err() {
                    break;
                }
            });
        }
        drop(tx);
        for (i, prepared) in rx {
            let p = match prepared {
                Ok(p) => p,
                Err(e) => {
                    failed.push((i, e.to_string()));
                    continue;
                }
            };
            decode_s += p.seconds;
            let n = p.at.len();
            let t = std::time::Instant::now();
            let tapped = emb.embed(&p.fit)?;
            let other_logits = det.as_ref().map(|d| logits_of(d, &p.others, vl, o.device != Device::Cpu)).unwrap_or_default();
            model_s += t.elapsed().as_secs_f64();
            anyhow::ensure!(tapped.logits.len() == n && other_logits.len() == n * others.len(), "the network returned {} + {} logits for {n} frames", tapped.logits.len(), other_logits.len());
            let per = emb.tokens_per_view() * dim;
            let mut rows = Vec::with_capacity(n);
            for k in 0..n {
                for t in &tapped.tokens {
                    let tokens = &t[k * per..(k + 1) * per];
                    for v in tokens[..dim].iter().chain(&pool_grid(&tokens[dim..], side, dim, o.grid)) {
                        features.write_all(&v.to_le_bytes())?;
                    }
                }
                let mut view_logits = vec![tapped.logits[k]];
                view_logits.extend_from_slice(&other_logits[k * others.len()..(k + 1) * others.len()]);
                let mut f = Frame { index: p.at[k].0, pts: p.at[k].1, fit_logit: tapped.logits[k], frame_logit: view_logits.iter().sum::<f32>() / view_logits.len() as f32, view_logits, ..Default::default() };
                if k > 0 {
                    let (max_diff, sum_diff, repeats) = stretch(&p.trace, p.at[k - 1].0, p.at[k].0);
                    f = Frame { gap_frames: p.at[k].0 as i64 - p.at[k - 1].0 as i64, gap_s: p.at[k].1 - p.at[k - 1].1, fit_diff: p.fit_diff[k], max_diff, sum_diff, repeats, ..f };
                }
                rows.push(f);
            }
            let info = p.info.unwrap_or(crate::video::VideoInfo { width: 0, height: 0, duration: 0.0, fps: 0.0 });
            done.push((i, Clip { item: items[i].clone(), first_record: records, frames: n, width: info.width, height: info.height, fps: info.fps, duration: info.duration, stream: Stream::of(&p.trace) }, rows));
            records += n as u64;
        }
        Ok(())
    })?;
    features.flush()?;
    // Rows in manifest order, whatever order the clips finished in.
    done.sort_by_key(|d| d.0);
    failed.sort();
    let (clips, rows): (Vec<Clip>, Vec<Vec<Frame>>) = done.into_iter().map(|(_, c, f)| (c, f)).unzip();
    let meta = Meta {
        format: FORMAT,
        weights: weights.display().to_string(),
        weights_sha256: crate::integrity::sha256_file(weights)?,
        device: format!("{:?}", o.device).to_lowercase(),
        img: spec.img,
        dim,
        blocks,
        grid: o.grid,
        record,
        frames,
        max_seconds: o.max_seconds,
        follow: o.follow,
        blur: o.blur.max(1),
        views: scored,
        fit_view: 0,
        clips: clips.len(),
        frame_records: records,
        unreadable: failed.len(),
        wall_s: start.elapsed().as_secs_f64(),
        model_s,
        decode_s,
        threads,
        repeat_below: REPEAT_BELOW,
    };
    write_index(out, &meta, &clips, &rows)?;
    let mut w = std::io::BufWriter::new(std::fs::File::create(out.join("unreadable.tsv"))?);
    for (i, why) in &failed {
        writeln!(w, "{}\t{}", items[*i].path.display(), why.replace(['\t', '\n'], " "))?;
    }
    w.flush()?;
    Ok(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(blocks: Vec<usize>, grid: usize, dim: usize) -> Meta {
        Meta {
            format: FORMAT,
            weights: "runs/ws7/video.safetensors".into(),
            weights_sha256: "0".repeat(64),
            device: "cpu".into(),
            img: 224,
            dim,
            record: blocks.len() * (1 + grid * grid) * dim,
            blocks,
            grid,
            frames: 2,
            max_seconds: 30.0,
            follow: 0,
            blur: 1,
            views: vec![ViewSpec::Fit { resize: 256, crop: 224 }],
            fit_view: 0,
            clips: 2,
            frame_records: 3,
            unreadable: 0,
            wall_s: 1.0,
            model_s: 0.5,
            decode_s: 0.4,
            threads: 1,
            repeat_below: REPEAT_BELOW,
        }
    }

    #[test]
    fn stream_statistics_come_from_the_frames_read() {
        // 25 fps with one frame shown for twice as long; frames 2 and 3 repeat frame 1; a cut before frame 6.
        let trace = Trace { pts: vec![0.0, 0.04, 0.08, 0.12, 0.16, 0.24, 0.28], diff: vec![0.0, 2.0, 0.0, 0.0, 3.0, 2.0, 90.0] };
        let s = Stream::of(&trace);
        assert_eq!((s.frames_read, s.repeats), (7, 2));
        assert!((s.gap_median - 0.04).abs() < 1e-9 && (s.gap_min - 0.04).abs() < 1e-9 && (s.gap_max - 0.08).abs() < 1e-9, "{s:?}");
        assert_eq!(s.median_diff, 2.0);
        // Between samples at frames 1 and 4: the two repeats and the step of 3.
        assert_eq!(stretch(&trace, 1, 4), (3.0, 3.0, 2));
        // The same stretch whichever sample comes first; a sample against itself covers nothing.
        assert_eq!(stretch(&trace, 4, 1), (3.0, 3.0, 2));
        assert_eq!(stretch(&trace, 5, 5), (0.0, 0.0, 0));
        // The cut shows as the largest difference of its stretch.
        assert_eq!(stretch(&trace, 4, 6).0, 90.0);
        // One frame, or none: nothing to measure, and nothing divides by zero.
        assert_eq!(Stream::of(&Trace { pts: vec![0.0], diff: vec![0.0] }), Stream { frames_read: 1, ..Default::default() });
        assert_eq!(Stream::of(&Trace::default()), Stream::default());
    }

    #[test]
    fn a_dump_reads_back_as_it_was_written() {
        let dir = std::env::temp_dir().join(format!("tintype-test-features-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let m = meta(vec![9, 12], 2, 3);
        let item = |path: &str, label: u8, extra: &[&str]| Item { path: path.into(), label, generator: if label == 1 { "veo".into() } else { "real:phone".into() }, dataset: "set".into(), extra: extra.iter().map(|s| s.to_string()).collect() };
        let stream = Stream { frames_read: 50, gap_median: 0.04, gap_min: 0.04, gap_max: 0.04, median_diff: 1.5, repeats: 3 };
        let clips = vec![
            Clip { item: item("/clips/a b.mp4", 1, &["p1", "", "src"]), first_record: 1, frames: 2, width: 1280, height: 720, fps: 25.0, duration: 4.0, stream },
            Clip { item: item("/clips/c.mp4", 0, &[]), first_record: 0, frames: 1, width: 640, height: 360, fps: 0.0, duration: 0.0, stream: Stream::default() },
        ];
        let frames = vec![
            vec![
                Frame { index: 0, pts: 0.0, fit_logit: 1.5, frame_logit: 2.25, view_logits: vec![1.5, 3.0], ..Default::default() },
                Frame { index: 7, pts: 0.28, fit_logit: -0.5, frame_logit: 0.0, view_logits: vec![-0.5, 0.5], gap_frames: 7, gap_s: 0.28, fit_diff: 4.25, max_diff: 30.5, sum_diff: 41.0, repeats: 2 },
            ],
            vec![Frame { index: 3, pts: 0.125, fit_logit: -4.0, frame_logit: -4.0, view_logits: vec![-4.0], ..Default::default() }],
        ];
        write_index(&dir, &m, &clips, &frames).unwrap();
        // Three records of 30 values: record r holds r + i / 100.
        let values: Vec<f32> = (0..3 * m.record).map(|i| (i / m.record) as f32 + (i % m.record) as f32 / 100.0).collect();
        std::fs::write(dir.join("features.f32"), values.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>()).unwrap();

        let set = FeatureSet::open(&dir).unwrap();
        assert_eq!(set.meta, m);
        assert_eq!(set.clips, clips);
        assert_eq!(set.frames, frames);
        assert_eq!(set.labels(), vec![1, 0]);
        // Clip 0 starts at record 1 and has two; clip 1 is record 0.
        assert_eq!(set.records(0).unwrap(), values[m.record..]);
        assert_eq!(set.records(1).unwrap(), values[..m.record]);
        // Block 12 is the second half of each record: 5 tokens of 3 values.
        let b12 = set.block_tokens(0, 12).unwrap();
        assert_eq!(b12.len(), 2 * 5 * 3);
        assert_eq!(b12[..15], values[m.record + 15..2 * m.record]);
        assert!(set.block_tokens(0, 6).is_err());
        // A truncated feature file is refused.
        std::fs::write(dir.join("features.f32"), [0u8; 16]).unwrap();
        assert!(FeatureSet::open(&dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
