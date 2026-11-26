//! Pure-Rust video reading: an MP4/MOV sample-table parser feeding the
//! `rust_h264`, `rust_h265` and `filmcraft-vp9` decoders, and a WebM/Matroska
//! reader ([`webm`]) feeding `filmcraft-vp9` and `filmcraft-vp8`. This is the
//! video decoder on platforms without AVFoundation, and on macOS for what
//! AVFoundation cannot read (VP9, WebM); `TINTYPE_DECODER=rust` selects it in
//! place of the system decoder for everything.
//!
//! It reads progressive MP4 and MOV files with one H.264, HEVC or VP9 video
//! track, and WebM and Matroska files with VP9 or VP8, all in 4:2:0.
//! Fragmented files, other containers and other codecs are reported as
//! unreadable. The HTTP upload preflight takes only H.264 and HEVC in MP4/MOV.
#[path = "webm.rs"]
mod webm;
use crate::image_io::Rgb;
use crate::video::VideoInfo;
use crate::yuv::{self, Source, Yuv};
use anyhow::{anyhow, bail, ensure, Context, Result};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Codec {
    H264,
    Hevc,
    Vp9,
}

struct Sample {
    offset: u64,
    size: u32,
    /// Presentation time in seconds.
    pts: f64,
    sync: bool,
}

struct Track {
    codec: Codec,
    /// `avcC`, `hvcC` or `vpcC` payload.
    config: Vec<u8>,
    /// `(colour primaries, matrix coefficients, full range)` from a `colr`
    /// box, when present.
    colr: Option<(u16, u16, Option<bool>)>,
    samples: Vec<Sample>,
    info: VideoInfo,
}

// ---------------------------------------------------------------- MP4 boxes

fn be32(d: &[u8], at: usize) -> Result<u32> {
    d.get(at..at + 4).map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]])).ok_or_else(|| anyhow!("truncated box"))
}

fn be64(d: &[u8], at: usize) -> Result<u64> {
    Ok(((be32(d, at)? as u64) << 32) | be32(d, at + 4)? as u64)
}

/// The child boxes of `d` as `(type, payload)`.
fn children(d: &[u8]) -> Vec<([u8; 4], &[u8])> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos + 8 <= d.len() {
        let size32 = u32::from_be_bytes([d[pos], d[pos + 1], d[pos + 2], d[pos + 3]]) as usize;
        let kind = [d[pos + 4], d[pos + 5], d[pos + 6], d[pos + 7]];
        let (head, size) = match size32 {
            0 => (8, d.len() - pos),
            1 if pos + 16 <= d.len() => (16, be64(d, pos + 8).unwrap_or(0) as usize),
            1 => break,
            n => (8, n),
        };
        if size < head || size > d.len() - pos {
            break;
        }
        out.push((kind, &d[pos + head..pos + size]));
        pos += size;
    }
    out
}

fn child<'a>(d: &'a [u8], kind: &[u8; 4]) -> Option<&'a [u8]> {
    children(d).into_iter().find(|(k, _)| k == kind).map(|(_, p)| p)
}

/// Read the `moov` box of a file into memory.
fn read_moov(f: &mut File, path: &Path, max_index: u64, upload: bool) -> Result<Vec<u8>> {
    let len = f.metadata()?.len();
    let mut pos = 0u64;
    let mut fragmented = false;
    let mut index = None;
    let mut boxes = 0;
    while len - pos >= 8 {
        boxes += 1;
        ensure!(!upload || boxes <= 8192, "video exceeds the top-level box limit");
        let mut h = [0u8; 16];
        f.seek(SeekFrom::Start(pos))?;
        f.read_exact(&mut h[..8])?;
        let mut size = u32::from_be_bytes([h[0], h[1], h[2], h[3]]) as u64;
        let mut head = 8u64;
        if size == 1 {
            f.read_exact(&mut h[8..16])?;
            size = u64::from_be_bytes(h[8..16].try_into().unwrap());
            head = 16;
        } else if size == 0 {
            size = len - pos;
        }
        ensure!(size >= head && size <= len - pos, "{} is not a readable MP4 or MOV file", path.display());
        match &h[4..8] {
            b"moov" => {
                ensure!(index.is_none(), "multiple movie boxes are unsupported for uploads");
                ensure!(size - head <= max_index, "{} exceeds the MP4 index size limit", path.display());
                let mut moov = vec![0u8; (size - head) as usize];
                f.read_exact(&mut moov)?;
                if !upload {
                    return Ok(moov);
                }
                index = Some(moov);
            }
            b"moof" => fragmented = true,
            _ => {}
        }
        pos += size;
    }
    if fragmented {
        bail!("{} is a fragmented MP4, which this reader does not support", path.display());
    }
    if let Some(moov) = index {
        ensure!(pos == len, "truncated top-level box");
        return Ok(moov);
    }
    bail!("{} is not an MP4, MOV, WebM or Matroska file (other containers need the macOS system decoder)", path.display())
}

/// A full box's entries after its version/flags word and entry count.
fn table(d: &[u8]) -> Result<(u8, usize, &[u8])> {
    ensure!(d.len() >= 8, "truncated table");
    Ok((d[0], be32(d, 4)? as usize, &d[8..]))
}

fn parse_track(trak: &[u8], movie_timescale: u32, max_samples: usize, upload: bool) -> Result<Option<Track>> {
    let Some(mdia) = child(trak, b"mdia") else { return Ok(None) };
    let is_video = child(mdia, b"hdlr").is_some_and(|h| h.get(8..12) == Some(b"vide"));
    if !is_video {
        return Ok(None);
    }
    let mdhd = child(mdia, b"mdhd").ok_or_else(|| anyhow!("no media header"))?;
    let (timescale, media_duration) = if mdhd.first() == Some(&1) { (be32(mdhd, 20)?, be64(mdhd, 24)?) } else { (be32(mdhd, 12)?, be32(mdhd, 16)? as u64) };
    ensure!(timescale > 0, "zero timescale");
    let stbl = child(mdia, b"minf").and_then(|m| child(m, b"stbl")).ok_or_else(|| anyhow!("no sample table"))?;

    // Sample description: the first entry decides the codec.
    let stsd = child(stbl, b"stsd").ok_or_else(|| anyhow!("no sample description"))?;
    let entries = children(stsd.get(8..).unwrap_or(&[]));
    let Some((kind, entry)) = entries.first() else { bail!("empty sample description") };
    ensure!(!upload || entries.len() == 1, "multiple sample descriptions are unsupported for uploads");
    let codec = match kind {
        b"avc1" | b"avc3" => Codec::H264,
        b"hvc1" | b"hev1" => Codec::Hevc,
        b"vp09" => Codec::Vp9,
        other => bail!("video codec `{}` is not supported by this reader (H.264, HEVC and VP9 are)", String::from_utf8_lossy(other)),
    };
    ensure!(entry.len() > 78, "truncated sample entry");
    let (width, height) = (u16::from_be_bytes([entry[24], entry[25]]) as u32, u16::from_be_bytes([entry[26], entry[27]]) as u32);
    let inner = &entry[78..];
    let config_box = match codec {
        Codec::H264 => b"avcC",
        Codec::Hevc => b"hvcC",
        Codec::Vp9 => b"vpcC",
    };
    let config = child(inner, config_box).ok_or_else(|| anyhow!("no decoder configuration"))?.to_vec();
    let colr = child(inner, b"colr").and_then(|c| match c.get(..4)? {
        b"nclx" if c.len() >= 11 => Some((u16::from_be_bytes([c[4], c[5]]), u16::from_be_bytes([c[8], c[9]]), Some(c[10] & 0x80 != 0))),
        b"nclc" if c.len() >= 10 => Some((u16::from_be_bytes([c[4], c[5]]), u16::from_be_bytes([c[8], c[9]]), None)),
        _ => None,
    });

    // Sample sizes.
    let stsz = child(stbl, b"stsz").ok_or_else(|| anyhow!("no sample sizes (compact size tables are not supported)"))?;
    let (uniform, count) = (be32(stsz, 4)?, be32(stsz, 8)? as usize);
    ensure!(count <= max_samples, "video exceeds the sample count limit");
    let sizes: Vec<u32> = if uniform != 0 { vec![uniform; count] } else { (0..count).map(|i| be32(stsz, 12 + 4 * i)).collect::<Result<_>>()? };

    // Chunk offsets and the samples-per-chunk runs.
    let offsets: Vec<u64> = if let Some(co) = child(stbl, b"stco") {
        let (_, n, e) = table(co)?;
        ensure!(!upload || n <= max_samples, "video exceeds the chunk count limit");
        (0..n).map(|i| be32(e, 4 * i).map(u64::from)).collect::<Result<_>>()?
    } else {
        let (_, n, e) = table(child(stbl, b"co64").ok_or_else(|| anyhow!("no chunk offsets"))?)?;
        ensure!(!upload || n <= max_samples, "video exceeds the chunk count limit");
        (0..n).map(|i| be64(e, 8 * i)).collect::<Result<_>>()?
    };
    let (_, runs, stsc) = table(child(stbl, b"stsc").ok_or_else(|| anyhow!("no sample-to-chunk table"))?)?;
    ensure!(!upload || runs <= max_samples, "video exceeds the chunk run limit");
    let mut sample_offsets = Vec::with_capacity(count);
    for r in 0..runs {
        let first = be32(stsc, 12 * r)? as usize;
        let per_chunk = be32(stsc, 12 * r + 4)? as usize;
        ensure!(!upload || be32(stsc, 12 * r + 8)? == 1, "invalid sample description index");
        let end = if r + 1 < runs { be32(stsc, 12 * (r + 1))? as usize } else { offsets.len() + 1 };
        ensure!(first >= 1 && end >= first && per_chunk <= count, "malformed sample-to-chunk table");
        for chunk in first..end.min(offsets.len() + 1) {
            // Every sample is placed: later chunks and runs have nothing to add.
            if sample_offsets.len() == count {
                break;
            }
            let mut at = offsets[chunk - 1];
            for _ in 0..per_chunk {
                let Some(&size) = sizes.get(sample_offsets.len()) else { break };
                sample_offsets.push(at);
                at = at.checked_add(u64::from(size)).ok_or_else(|| anyhow!("sample offset overflow"))?;
            }
        }
    }
    ensure!(sample_offsets.len() == count, "sample tables disagree ({} offsets for {count} samples)", sample_offsets.len());

    // Decode times, then composition offsets.
    let (_, n, stts) = table(child(stbl, b"stts").ok_or_else(|| anyhow!("no time table"))?)?;
    let ctts = match child(stbl, b"ctts") {
        Some(c) => Some(table(c)?),
        None => None,
    };
    let times = sample_times(stts, n, ctts.map(|(_, n, e)| (e, n)), count)?;

    // The edit list maps media time to presentation time: an initial empty
    // edit delays the track, and the first real edit's media time is where
    // presentation starts.
    let mut shift = 0f64;
    if let Some(elst) = child(trak, b"edts").and_then(|e| child(e, b"elst")) {
        let (version, n, e) = table(elst)?;
        for i in 0..n {
            let (duration, media_time) = if version == 1 { (be64(e, 20 * i)? as f64, be64(e, 20 * i + 8)? as i64) } else { (be32(e, 12 * i)? as f64, be32(e, 12 * i + 4)? as i32 as i64) };
            if media_time < 0 {
                shift += duration / movie_timescale.max(1) as f64;
            } else {
                shift -= media_time as f64 / timescale as f64;
                break;
            }
        }
    }

    let sync: Option<std::collections::HashSet<u32>> = match child(stbl, b"stss") {
        Some(s) => {
            let (_, n, e) = table(s)?;
            Some((0..n).map(|i| be32(e, 4 * i)).collect::<Result<_>>()?)
        }
        None => None,
    };
    let samples = (0..count)
        .map(|i| Sample { offset: sample_offsets[i], size: sizes[i], pts: times[i] as f64 / timescale as f64 + shift, sync: sync.as_ref().is_none_or(|s| s.contains(&(i as u32 + 1))) })
        .collect();
    let seconds = media_duration as f64 / timescale as f64;
    // The clip's duration is the video track's own (what the macOS reader
    // reports), not the movie's, which also covers a longer audio track.
    let track_duration = match child(trak, b"tkhd") {
        Some(t) if t.first() == Some(&1) => be64(t, 28)? as f64,
        Some(t) => be32(t, 20)? as f64,
        None => 0.0,
    } / movie_timescale.max(1) as f64;
    let info = VideoInfo { width, height, duration: track_duration, fps: if seconds > 0.0 { count as f64 / seconds } else { 0.0 } };
    Ok(Some(Track { codec, config, colr, samples, info }))
}

/// Presentation time of each of a track's `count` samples, in its timescale,
/// from the decode-time runs (`stts`: `n` entries of run length and delta)
/// and the optional composition offsets (`ctts`: entries and their number).
///
/// The run lengths come from the file and may each claim four billion
/// samples: only as many as the track has are ever walked, so the work is
/// bounded by `count` whatever the tables say.
fn sample_times(stts: &[u8], n: usize, ctts: Option<(&[u8], usize)>, count: usize) -> Result<Vec<i64>> {
    let mut times: Vec<i64> = Vec::with_capacity(count);
    let mut t = 0i64;
    for i in 0..n {
        if times.len() == count {
            break;
        }
        let (run, delta) = (be32(stts, 8 * i)? as usize, be32(stts, 8 * i + 4)? as i64);
        for _ in 0..run.min(count - times.len()) {
            times.push(t);
            t = t.saturating_add(delta);
        }
    }
    ensure!(times.len() == count, "time table covers {} of {count} samples", times.len());
    if let Some((e, n)) = ctts {
        let mut i = 0usize;
        for r in 0..n {
            if i >= count {
                break;
            }
            // Signed in version 1; writers also put negative values in version 0.
            let (run, off) = (be32(e, 8 * r)? as usize, be32(e, 8 * r + 4)? as i32 as i64);
            let run = run.min(count - i);
            for v in &mut times[i..i + run] {
                *v = v.saturating_add(off);
            }
            i += run;
        }
    }
    Ok(times)
}

fn read_track(f: &mut File, path: &Path, upload: bool) -> Result<Track> {
    let (max_index, max_samples) = if upload { (4 << 20, 8_192) } else { (256 << 20, 10_000_000) };
    let moov = read_moov(f, path, max_index, upload)?;
    let mvhd = child(&moov, b"mvhd").ok_or_else(|| anyhow!("{} has no movie header", path.display()))?;
    let (timescale, duration) = if mvhd.first() == Some(&1) { (be32(mvhd, 20)?, be64(mvhd, 24)?) } else { (be32(mvhd, 12)?, be32(mvhd, 16)? as u64) };
    let mut first_error = None;
    let mut first_track = None;
    for (kind, trak) in children(&moov) {
        if &kind != b"trak" {
            continue;
        }
        match parse_track(trak, timescale, max_samples, upload) {
            Ok(Some(mut t)) => {
                if t.info.duration <= 0.0 && timescale > 0 {
                    t.info.duration = duration as f64 / timescale as f64;
                }
                if !upload {
                    return Ok(t);
                }
                ensure!(first_track.is_none(), "multiple video tracks are unsupported for uploads");
                first_track = Some(t);
            }
            Ok(None) => {}
            Err(e) if upload => return Err(e).with_context(|| format!("reading {}", path.display())),
            Err(e) => first_error = first_error.or(Some(e)),
        }
    }
    if let Some(track) = first_track {
        return Ok(track);
    }
    match first_error {
        Some(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        None => bail!("no video track in {}", path.display()),
    }
}

// ------------------------------------------------------------------ decode

enum Decoder {
    H264 { dec: Box<rust_h264::decoder::OrderedDecoder<f64>>, length_size: usize },
    Hevc { dec: Box<rust_h265::Decoder>, length_size: u8 },
    /// Each sample is one frame or superframe; its index is passed as the
    /// picture's time, so every picture is matched to the sample that showed it.
    Vp9(Box<filmcraft_vp9::Decoder>),
}

/// The matrix-coefficients code (ISO/IEC 23091-2) a VP9 `color_space`
/// stands for, as FFmpeg reads it; `None` when the stream does not say.
fn vp9_matrix(color_space: u8) -> Option<u16> {
    match color_space {
        1 => Some(5), // BT.601
        2 => Some(1), // BT.709
        3 => Some(6), // SMPTE 170M
        4 => Some(7), // SMPTE 240M
        5 => Some(9), // BT.2020, non-constant luminance
        7 => Some(0), // sRGB
        _ => None,    // unknown or reserved
    }
}

/// Whether frames are converted with the BT.709 matrix: the one the file
/// states, else the one that goes with its stated primaries, else what the
/// macOS decoder assumes for a file that states neither: BT.709 from 720
/// pixels on the longer side and BT.601 below.
fn assumed_bt709(matrix: Option<u16>, primaries: Option<u16>, width: u32, height: u32) -> bool {
    match (matrix, primaries) {
        (Some(m), _) => m == 1,
        (None, Some(p)) => p == 1,
        (None, None) => width.max(height) >= 720,
    }
}

/// The bytes of one sample, after checking that the file holds them.
fn read_sample(file: &mut File, file_len: u64, s: &Sample, index: usize) -> Result<Vec<u8>> {
    let size = s.size as usize;
    // Nothing is allocated for a sample the file is too short to hold.
    ensure!(size <= 256 << 20 && s.offset.saturating_add(size as u64) <= file_len, "sample {index} lies outside the file");
    let mut buf = vec![0u8; size];
    file.seek(SeekFrom::Start(s.offset))?;
    file.read_exact(&mut buf)?;
    Ok(buf)
}

/// Interleave two chroma planes into `ceil(w / 2) * 2` bytes per row over
/// `ceil(h / 2)` rows, repeating the last sample where the planes are short.
fn interleave(w: usize, h: usize, cw_src: usize, ch_src: usize, at: impl Fn(usize) -> (u8, u8)) -> Vec<u8> {
    let (cw, ch) = (w.div_ceil(2), h.div_ceil(2));
    let mut uv = vec![128u8; cw * ch * 2];
    if cw_src == 0 || ch_src == 0 {
        return uv;
    }
    for r in 0..ch {
        let sr = r.min(ch_src - 1);
        for c in 0..cw {
            let (u, v) = at(sr * cw_src + c.min(cw_src - 1));
            uv[(r * cw + c) * 2] = u;
            uv[(r * cw + c) * 2 + 1] = v;
        }
    }
    uv
}

/// Reduce a sample of `depth` bits to 8, rounding.
fn to8(v: u16, depth: u8) -> u8 {
    let s = depth.saturating_sub(8) as u32;
    if s == 0 {
        v.min(255) as u8
    } else {
        ((v as u32 + (1 << (s - 1))) >> s).min(255) as u8
    }
}

/// A pure-Rust reader for one clip: MP4/MOV, or WebM/Matroska.
pub struct Reader(Kind);

enum Kind {
    Mp4(Box<Mp4Reader>),
    Webm(Box<webm::Reader>),
}

struct Mp4Reader {
    file: File,
    file_len: u64,
    track: Track,
    decoder: Decoder,
    next_sample: usize,
    /// Presentation times of the samples being decoded, ascending: the
    /// times of the HEVC decoder's frames, which come out in display order.
    times: Vec<f64>,
    emitted: usize,
    ready: VecDeque<(Option<f64>, Yuv)>,
    flushed: bool,
    /// Frames ending at or before this time are skipped.
    start: Option<f64>,
    bt709: bool,
    full_range: bool,
    /// What the file states: matrix coefficients and range, when it does.
    declared: (Option<u16>, Option<bool>),
    errors: usize,
    /// The first reason the decoder gave for rejecting part of the stream.
    last_error: Option<String>,
}

/// Open a clip, read from the start or, with `start`, from the frame shown
/// at that time (seconds).
pub fn open(path: &Path, start: Option<f64>) -> Result<Reader> {
    match open_mp4(path, start) {
        Ok(r) => Ok(Reader(Kind::Mp4(Box::new(r)))),
        // A file that is not MP4 is tried as WebM/Matroska whatever its name.
        Err(e) => webm::open_if_matroska(path, start, e).map(|r| Reader(Kind::Webm(Box::new(r)))),
    }
}

fn open_mp4(path: &Path, start: Option<f64>) -> Result<Mp4Reader> {
    let mut file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let track = read_track(&mut file, path, false)?;
    ensure!(!track.samples.is_empty(), "no frames in {}", path.display());
    let file_len = file.metadata()?.len();
    // Colour tags: the container's `colr` box, else the stream's own.
    // The value 2 means "unspecified" in both fields.
    let stated = |v: u16| Some(v).filter(|&v| v != 2);
    let (mut primaries, mut matrix, mut full_range) = (track.colr.and_then(|c| stated(c.0)), track.colr.and_then(|c| stated(c.1)), track.colr.and_then(|c| c.2));
    let mut first = 0usize;
    let decoder = match track.codec {
        Codec::H264 => {
            let cfg = rust_h264::nal::parse_avcc_config(&track.config).map_err(|e| anyhow!("{}: {e}", path.display()))?;
            let mut dec = rust_h264::decoder::OrderedDecoder::<f64>::new();
            if let Some(info) = cfg.sps_nals.first().and_then(|n| rust_h264::nal::parse_sps_info(n).ok()) {
                ensure!(info.chroma_format <= 1, "{} is not 4:2:0 video", path.display());
                primaries = primaries.or(stated(info.primaries as u16));
                matrix = matrix.or(stated(info.matrix as u16));
                full_range = full_range.or(Some(info.full_range));
            }
            for nal in cfg.sps_nals.iter().chain(&cfg.pps_nals) {
                let _ = dec.decode_nal_with_meta(nal, 0.0);
            }
            // Frames carry their own times through this decoder, so a seek
            // can start at the last sync sample before the wanted time.
            if let Some(t) = start {
                first = track.samples.iter().rposition(|s| s.sync && s.pts <= t).unwrap_or(0);
            }
            Decoder::H264 { dec: Box::new(dec), length_size: cfg.length_size }
        }
        Codec::Hevc => {
            let c = &track.config;
            ensure!(c.len() >= 23, "{} has a truncated decoder configuration", path.display());
            let mut dec = rust_h265::Decoder::new().with_display_order();
            let mut at = 23usize;
            for _ in 0..c[22] {
                ensure!(at + 3 <= c.len(), "truncated decoder configuration");
                let n = u16::from_be_bytes([c[at + 1], c[at + 2]]) as usize;
                at += 3;
                for _ in 0..n {
                    ensure!(at + 2 <= c.len(), "truncated decoder configuration");
                    let len = u16::from_be_bytes([c[at], c[at + 1]]) as usize;
                    at += 2;
                    ensure!(at + len <= c.len(), "truncated decoder configuration");
                    if let Some(nal) = rust_h265::parse_nal(&c[at..at + len]) {
                        if nal.nal_unit_type == rust_h265::NalUnitType::Sps && (matrix.is_none() || full_range.is_none()) {
                            if let Some((p, m, fr)) = hevc_colour(&nal.rbsp) {
                                primaries = primaries.or(p.and_then(stated));
                                matrix = matrix.or(m.and_then(stated));
                                full_range = full_range.or(Some(fr));
                            }
                        }
                        let _ = dec.decode_nal(&nal);
                    }
                    at += len;
                }
            }
            Decoder::Hevc { dec: Box::new(dec), length_size: (c[21] & 3) + 1 }
        }
        Codec::Vp9 => {
            // VPCodecConfigurationBox, version 1: profile and level, then bit
            // depth (4 bits), chroma subsampling (3) and full range (1), then
            // colour primaries, transfer and matrix codes. 0 and 1 are the two
            // 4:2:0 chroma sitings.
            let c = &track.config;
            if c.first() == Some(&1) && c.len() >= 12 {
                ensure!((c[6] >> 1) & 7 <= 1, "{} is not 4:2:0 video", path.display());
                primaries = primaries.or(stated(c[7].into()));
                matrix = matrix.or(stated(c[9].into()));
                full_range = full_range.or(Some(c[6] & 1 == 1));
            }
            // Then the stream's own colour signalling, from its first key frame.
            if let Some(k) = filmcraft_vp9::keyframe_info(&read_sample(&mut file, file_len, &track.samples[0], 0)?) {
                ensure!(k.subsampling_x && k.subsampling_y, "{} is not 4:2:0 video", path.display());
                matrix = matrix.or(vp9_matrix(k.color.color_space));
                full_range = full_range.or(Some(k.color.full_range));
            }
            // A seek starts at the last key frame at or before the wanted
            // time. The sync table says where key frames are, and each
            // candidate is checked, since without one every sample is sync.
            if let Some(t) = start {
                first = (0..track.samples.len())
                    .rev()
                    .filter(|&i| track.samples[i].sync && track.samples[i].pts <= t)
                    .find(|&i| read_sample(&mut file, file_len, &track.samples[i], i).is_ok_and(|d| filmcraft_vp9::is_keyframe(&d)))
                    .unwrap_or(0);
            }
            Decoder::Vp9(Box::new(filmcraft_vp9::Decoder::with_threads(1)))
        }
    };
    let bt709 = assumed_bt709(matrix, primaries, track.info.width, track.info.height);
    let mut times: Vec<f64> = track.samples[first..].iter().map(|s| s.pts).collect();
    times.sort_by(f64::total_cmp);
    Ok(Mp4Reader { file, file_len, track, decoder, next_sample: first, times, emitted: 0, ready: VecDeque::new(), flushed: false, start, bt709, full_range: full_range.unwrap_or(false), declared: (matrix, full_range), errors: 0, last_error: None })
}

struct Bits<'a> {
        d: &'a [u8],
        pos: usize,
    }
    impl Bits<'_> {
        fn u(&mut self, n: usize) -> Option<u32> {
            let mut v = 0u32;
            for _ in 0..n {
                let byte = *self.d.get(self.pos / 8)?;
                v = (v << 1) | ((byte >> (7 - self.pos % 8)) & 1) as u32;
                self.pos += 1;
            }
            Some(v)
        }
        fn ue(&mut self) -> Option<u32> {
            let mut zeros = 0;
            while self.u(1)? == 0 {
                zeros += 1;
                if zeros > 31 {
                    return None;
                }
            }
            Some((1u32 << zeros) - 1 + self.u(zeros)?)
        }
        fn se(&mut self) -> Option<i32> {
            let k = self.ue()?;
            Some(if k % 2 == 1 { k.div_ceil(2) as i32 } else { -((k / 2) as i32) })
        }
        fn skip(&mut self, n: usize) -> Option<()> {
            self.pos += n;
            (self.pos <= self.d.len() * 8).then_some(())
        }
    }
fn hevc_sps_prefix(rbsp: &[u8]) -> Option<(Bits<'_>, usize)> {
    let mut b = Bits { d: rbsp, pos: 0 };
    b.skip(4)?;
    let sub_layers = b.u(3)? as usize;
    b.skip(1)?;
    // profile_tier_level
    b.skip(96)?;
    let present: Vec<(bool, bool)> = (0..sub_layers).map(|_| Some((b.u(1)? == 1, b.u(1)? == 1))).collect::<Option<_>>()?;
    if sub_layers > 0 {
        b.skip(2 * (8 - sub_layers))?;
    }
    for (profile, level) in present {
        b.skip(if profile { 88 } else { 0 } + if level { 8 } else { 0 })?;
    }
    b.ue()?;
    Some((b, sub_layers))
}

/// `(colour primaries and matrix coefficients when stated, full range)` from
/// an HEVC sequence parameter set, or `None` when it cannot be read that far.
fn hevc_colour(rbsp: &[u8]) -> Option<(Option<u16>, Option<u16>, bool)> {
    let (mut b, sub_layers) = hevc_sps_prefix(rbsp)?;
    if b.ue()? == 3 {
        b.skip(1)?;
    }
    b.ue()?;
    b.ue()?;
    if b.u(1)? == 1 {
        for _ in 0..4 {
            b.ue()?;
        }
    }
    b.ue()?;
    b.ue()?;
    let log2_max_poc = b.ue()? + 4;
    let ordering = b.u(1)? == 1;
    for _ in (if ordering { 0 } else { sub_layers })..=sub_layers {
        b.ue()?;
        b.ue()?;
        b.ue()?;
    }
    for _ in 0..6 {
        b.ue()?;
    }
    if b.u(1)? == 1 && b.u(1)? == 1 {
        // scaling_list_data
        for size in 0..4 {
            let mut m = 0;
            while m < 6 {
                if b.u(1)? == 0 {
                    b.ue()?;
                } else {
                    let coefs = 64.min(1 << (4 + (size << 1)));
                    if size > 1 {
                        b.se()?;
                    }
                    for _ in 0..coefs {
                        b.se()?;
                    }
                }
                m += if size == 3 { 3 } else { 1 };
            }
        }
    }
    b.skip(2)?;
    if b.u(1)? == 1 {
        b.skip(8)?;
        b.ue()?;
        b.ue()?;
        b.skip(1)?;
    }
    // Short-term reference picture sets.
    let sets = b.ue()? as usize;
    if sets > 64 {
        return None;
    }
    let mut deltas: Vec<usize> = Vec::with_capacity(sets);
    for i in 0..sets {
        if i > 0 && b.u(1)? == 1 {
            b.skip(1)?;
            b.ue()?;
            let reference = deltas[i - 1];
            let mut n = 0;
            for _ in 0..=reference {
                let used = b.u(1)? == 1;
                if used || b.u(1)? == 1 {
                    n += 1;
                }
            }
            deltas.push(n);
        } else {
            let (neg, pos) = (b.ue()? as usize, b.ue()? as usize);
            if neg + pos > 32 {
                return None;
            }
            for _ in 0..neg + pos {
                b.ue()?;
                b.skip(1)?;
            }
            deltas.push(neg + pos);
        }
    }
    if b.u(1)? == 1 {
        for _ in 0..b.ue()? {
            b.skip(log2_max_poc as usize + 1)?;
        }
    }
    b.skip(2)?;
    if b.u(1)? == 0 {
        // No VUI: nothing stated.
        return Some((None, None, false));
    }
    if b.u(1)? == 1 && b.u(8)? == 255 {
        b.skip(32)?;
    }
    if b.u(1)? == 1 {
        b.skip(1)?;
    }
    if b.u(1)? == 0 {
        return Some((None, None, false));
    }
    b.skip(3)?;
    let full_range = b.u(1)? == 1;
    if b.u(1)? == 0 {
        return Some((None, None, full_range));
    }
    let primaries = b.u(8)? as u16;
    b.skip(8)?;
    Some((Some(primaries), Some(b.u(8)? as u16), full_range))
}

impl Reader {
    pub fn info(&self) -> VideoInfo {
        match &self.0 {
            Kind::Mp4(r) => r.track.info,
            Kind::Webm(r) => r.info,
        }
    }

    /// The matrix coefficients and range the file states, when it does.
    pub fn declared(&self) -> (Option<u16>, Option<bool>) {
        match &self.0 {
            Kind::Mp4(r) => r.declared,
            Kind::Webm(r) => r.declared,
        }
    }

    /// NAL units or packets the decoder rejected so far, and the first
    /// reason it gave.
    pub fn errors(&self) -> (usize, Option<&str>) {
        match &self.0 {
            Kind::Mp4(r) => (r.errors, r.last_error.as_deref()),
            Kind::Webm(r) => (r.errors, r.last_error.as_deref()),
        }
    }

    /// Why nothing could be decoded, for an error message.
    fn why_empty(&self, path: &Path) -> anyhow::Error {
        match self.errors().1 {
            Some(e) => anyhow!("{} cannot be decoded by the pure-Rust reader: {e}", path.display()),
            None => anyhow!("no frames decoded from {}", path.display()),
        }
    }
}

impl Source for Reader {
    fn duration(&self) -> f64 {
        self.info().duration
    }

    fn next(&mut self) -> Option<(f64, Yuv)> {
        match &mut self.0 {
            Kind::Mp4(r) => r.next(),
            Kind::Webm(r) => r.next(),
        }
    }
}

impl Mp4Reader {
    /// Feed the next sample (or flush) and queue the frames that come out.
    fn advance(&mut self) -> Result<()> {
        let index = self.next_sample;
        let (data, pts) = match self.track.samples.get(index) {
            Some(s) => {
                let buf = read_sample(&mut self.file, self.file_len, s, index)?;
                self.next_sample += 1;
                (Some(buf), s.pts)
            }
            None => {
                self.flushed = true;
                (None, 0.0)
            }
        };
        // (time when the decoder carries it, width, height, luma, interleaved chroma)
        let mut out: Vec<(Option<f64>, usize, usize, Vec<u8>, Vec<u8>)> = Vec::new();
        let mut errors = 0usize;
        match &mut self.decoder {
            Decoder::H264 { dec, length_size } => {
                let mut frames = Vec::new();
                match &data {
                    Some(buf) => {
                        for nal in rust_h264::nal::parse_avcc(buf, *length_size) {
                            match dec.decode_nal_with_meta(&nal, pts) {
                                Ok(f) => frames.extend(f),
                                Err(e) => {
                                    self.last_error.get_or_insert_with(|| e.to_string());
                                    errors += 1
                                }
                            }
                        }
                    }
                    None => frames = dec.flush_with_meta(),
                }
                for (f, t) in frames {
                    ensure!(f.chroma_format == rust_h264::decoder::ChromaFormat::Yuv420, "not 4:2:0 video");
                    let (w, h) = (f.width as usize, f.height as usize);
                    let (cw, ch) = (f.chroma_width() as usize, f.chroma_height() as usize);
                    let (y, uv) = if f.bit_depth > 8 {
                        ((0..w * h).map(|i| to8(f.sample(&f.y, i), f.bit_depth)).collect(), interleave(w, h, cw, ch, |i| (to8(f.sample(&f.u, i), f.bit_depth), to8(f.sample(&f.v, i), f.bit_depth))))
                    } else {
                        let uv = interleave(w, h, cw, ch, |i| (f.u[i], f.v[i]));
                        (f.y, uv)
                    };
                    out.push((Some(t), w, h, y, uv));
                }
            }
            Decoder::Hevc { dec, length_size } => {
                let mut frames = Vec::new();
                match &data {
                    Some(buf) => {
                        for nal in rust_h265::parse_hvcc(buf, *length_size) {
                            if let Err(e) = dec.decode_nal(&nal) {
                                self.last_error.get_or_insert_with(|| format!("{e:?}"));
                                errors += 1;
                            }
                            while let Some(f) = dec.next_output() {
                                frames.push(f);
                            }
                        }
                    }
                    None => {
                        while let Some(f) = dec.flush() {
                            frames.push(f);
                        }
                    }
                }
                for f in frames {
                    ensure!(f.chroma_format_idc == 1, "not 4:2:0 video");
                    let (w, h) = (f.width as usize, f.height as usize);
                    let (cw, ch) = (w.div_ceil(2), h.div_ceil(2));
                    use rust_h265::PixelData::{U16, U8};
                    let (y, uv) = match (f.y, f.u, f.v) {
                        (U8(y), U8(u), U8(v)) if u.len() >= cw * ch && v.len() >= cw * ch => {
                            let uv = interleave(w, h, cw, ch, |i| (u[i], v[i]));
                            (y, uv)
                        }
                        (U16(y), U16(u), U16(v)) if u.len() >= cw * ch && v.len() >= cw * ch => (y.iter().map(|&s| to8(s, f.bit_depth)).collect(), interleave(w, h, cw, ch, |i| (to8(u[i], f.bit_depth), to8(v[i], f.bit_depth)))),
                        _ => bail!("decoder returned planes of unexpected shape"),
                    };
                    out.push((None, w, h, y, uv));
                }
            }
            Decoder::Vp9(dec) => {
                let pictures = match &data {
                    Some(buf) => dec.decode(buf, index as i64).unwrap_or_else(|e| {
                        self.last_error.get_or_insert_with(|| e.to_string());
                        errors += 1;
                        Vec::new()
                    }),
                    None => dec.flush(),
                };
                for p in pictures {
                    let (shown_by, w, h, y, uv) = vp9_planes(p)?;
                    let t = usize::try_from(shown_by).ok().and_then(|i| self.track.samples.get(i)).map(|s| s.pts);
                    out.push((t, w, h, y, uv));
                }
            }
        }
        self.errors += errors;
        for (t, w, h, y, uv) in out {
            ensure!(y.len() == w * h, "decoder returned a short frame");
            self.ready.push_back((t, Yuv::new(w, h, y, uv, self.bt709, self.full_range)));
        }
        Ok(())
    }
}

/// A VP8 or VP9 picture, 8-bit: the time passed to the decoder with the
/// chunk that showed it, width, height, luma and interleaved chroma.
type Picture8 = (i64, usize, usize, Vec<u8>, Vec<u8>);

/// A VP9 picture reduced to 8 bits. Only 4:2:0 is accepted.
fn vp9_planes(p: filmcraft_vp9::Picture) -> Result<Picture8> {
    ensure!(p.subsampling_x && p.subsampling_y, "not 4:2:0 video");
    let (w, h) = (p.width as usize, p.height as usize);
    let (cw, ch) = (p.chroma_width as usize, p.chroma_height as usize);
    let depth = p.bit_depth as u8;
    use filmcraft_vp9::Plane::{U16, U8};
    let (y, uv) = match (p.y, p.u, p.v) {
        (U8(y), U8(u), U8(v)) if u.len() >= cw * ch && v.len() >= cw * ch => {
            let uv = interleave(w, h, cw, ch, |i| (u[i], v[i]));
            (y, uv)
        }
        (U16(y), U16(u), U16(v)) if u.len() >= cw * ch && v.len() >= cw * ch => (y.iter().map(|&s| to8(s, depth)).collect(), interleave(w, h, cw, ch, |i| (to8(u[i], depth), to8(v[i], depth)))),
        _ => bail!("decoder returned planes of unexpected shape"),
    };
    Ok((p.pts, w, h, y, uv))
}

impl Source for Mp4Reader {
    fn duration(&self) -> f64 {
        self.track.info.duration
    }

    fn next(&mut self) -> Option<(f64, Yuv)> {
        loop {
            if let Some((t, f)) = self.ready.pop_front() {
                let i = self.emitted;
                self.emitted += 1;
                let pts = t.or_else(|| self.times.get(i).copied()).or_else(|| self.times.last().copied())?;
                // Skip frames that end before the wanted start.
                if let Some(start) = self.start {
                    let ends = self.times.iter().copied().find(|&n| n > pts).unwrap_or(f64::INFINITY);
                    if ends <= start {
                        continue;
                    }
                }
                return Some((pts, f));
            }
            if self.flushed || self.advance().is_err() {
                return None;
            }
        }
    }
}

pub fn probe(path: &Path) -> Result<VideoInfo> {
    let mut f = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    read_track(&mut f, path, false).map(|t| t.info).or_else(|e| webm::open_if_matroska(path, None, e).map(|r| r.info))
}

/// Preflight a bounded HTTP upload before passing it to a video decoder.
pub(crate) fn validate_upload(path: &Path) -> Result<()> {
    const MAX_PIXELS: u64 = 2_200_000;
    let mut file = File::open(path)?;
    let file_len = file.metadata()?.len();
    let track = read_track(&mut file, path, true)?;
    let dimensions_ok = |w: u32, h: u32| {
        w > 0 && h > 0 && w.max(h) <= 4096 && w.max(h) <= w.min(h) * 32
            && u64::from(w) * u64::from(h) <= MAX_PIXELS
    };
    ensure!(dimensions_ok(track.info.width, track.info.height), "video exceeds the dimension limit");
    ensure!(!track.samples.is_empty(), "video has no samples");

    let mut hevc = rust_h265::Decoder::with_limits(rust_h265::DecodeLimits {
        max_picture_pixels: MAX_PIXELS,
        max_picture_memory_bytes: 64 << 20,
    });
    let avc_sps = |sps: &rust_h264::nal::NalUnit<'_>| -> Result<()> {
        let info = rust_h264::nal::parse_sps_info(sps)?;
        ensure!(dimensions_ok(info.coded_width, info.coded_height), "H.264 coded frame exceeds the dimension limit");
        ensure!(dimensions_ok(info.width, info.height) && info.chroma_format == 1, "unsupported H.264 frame dimensions or chroma");
        ensure!(info.reference_frames <= 16 && info.max_reorder_frames.is_none_or(|n| n <= 16), "H.264 stream exceeds the reference frame limit");
        Ok(())
    };
    let mut hevc_sps = |bytes: &[u8]| -> Result<bool> {
        if let Some(nal) = rust_h265::parse_nal(bytes) {
            if nal.nal_unit_type == rust_h265::NalUnitType::Sps && nal.nuh_layer_id == 0 {
                let (mut bits, _) = hevc_sps_prefix(&nal.rbsp).context("invalid HEVC stream parameters")?;
                ensure!(bits.ue() == Some(1), "HEVC stream must use 4:2:0 chroma");
                let (w, h) = bits.ue().zip(bits.ue()).context("invalid HEVC frame dimensions")?;
                ensure!(dimensions_ok(w, h), "HEVC coded frame exceeds the dimension limit");
                if bits.u(1).context("invalid HEVC conformance window")? == 1 {
                    let crop = |size: u32, a: Option<u32>, b: Option<u32>| {
                        a?.checked_add(b?)?.checked_mul(2).and_then(|n| size.checked_sub(n))
                    };
                    let visible = crop(w, bits.ue(), bits.ue()).zip(crop(h, bits.ue(), bits.ue()));
                    ensure!(visible.is_some_and(|(w, h)| dimensions_ok(w, h)), "HEVC visible frame exceeds the dimension limit");
                }
                hevc.decode_nal(&nal).map_err(|e| anyhow!("invalid HEVC stream parameters: {e:?}"))?;
                return Ok(true);
            }
        }
        Ok(false)
    };
    let length_size = match track.codec {
        Codec::H264 => {
            let config = rust_h264::nal::parse_avcc_config(&track.config).map_err(|e| anyhow!("{e}"))?;
            ensure!(!config.sps_nals.is_empty(), "H.264 stream has no initial sequence parameters");
            for sps in &config.sps_nals {
                avc_sps(sps)?;
            }
            ensure!(config.pps_nals.iter().all(|nal| nal.nal_unit_type == rust_h264::nal::NalUnitType::Pps), "invalid H.264 picture parameter set");
            config.length_size
        }
        Codec::Hevc => {
            let config = &track.config;
            let arrays = *config.get(22).context("truncated HEVC decoder configuration")?;
            let mut at = 23;
            let mut has_sps = false;
            for _ in 0..arrays {
                let kind = *config.get(at).context("truncated HEVC array")? & 63;
                ensure!(matches!(kind, 32..=34 | 39 | 40), "invalid HEVC parameter type");
                let count = u16::from_be_bytes(config.get(at + 1..at + 3).context("truncated HEVC array")?.try_into()?);
                at += 3;
                for _ in 0..count {
                    let len = u16::from_be_bytes(config.get(at..at + 2).context("truncated HEVC parameter length")?.try_into()?) as usize;
                    at += 2;
                    let nal = config.get(at..at + len).context("truncated HEVC parameter")?;
                    ensure!(nal.first().is_some_and(|b| (b >> 1) & 63 == kind), "HEVC parameter type disagrees with its array");
                    has_sps |= hevc_sps(nal)?;
                    at += len;
                }
            }
            ensure!(has_sps, "HEVC stream has no initial sequence parameters");
            usize::from(config[21] & 3) + 1
        }
        // A VP9 inter frame can state a new frame size of its own, so bounding
        // a stream's pictures needs every frame header read; uploads do not
        // take VP9 until that is done.
        Codec::Vp9 => bail!("VP9 video is not supported for uploads"),
    };

    let mut remaining = file_len;
    let mut buf = Vec::new();
    let mut nal_budget = 131_072;
    for sample in track.samples {
        let len = u64::from(sample.size);
        ensure!(len <= 8 << 20 && len <= remaining && sample.offset <= file_len - len, "invalid or excessive video sample data");
        remaining -= len;
        buf.resize(len as usize, 0);
        file.seek(SeekFrom::Start(sample.offset))?;
        file.read_exact(&mut buf)?;
        let mut data = buf.as_slice();
        while !data.is_empty() {
            ensure!(nal_budget > 0, "video exceeds the NAL unit limit");
            nal_budget -= 1;
            let length = data.get(..length_size).context("truncated video NAL length")?;
            let len = length.iter().fold(0usize, |len, &b| (len << 8) | usize::from(b));
            let (nal, rest) = data[length_size..].split_at_checked(len).context("truncated video NAL unit")?;
            ensure!(!nal.is_empty(), "empty video NAL unit");
            match track.codec {
                Codec::H264 if nal[0] & 31 == 7 => {
                    // This parser takes the length prefix as well as the NAL.
                    let nals = rust_h264::nal::parse_avcc(&data[..length_size + len], length_size);
                    avc_sps(nals.first().context("invalid H.264 parameter set")?)?;
                }
                Codec::Hevc if (nal[0] >> 1) & 63 == 33 => { hevc_sps(nal)?; }
                _ => {}
            }
            data = rest;
        }
    }
    Ok(())
}

pub fn frames_uniform(path: &Path, n: usize, max_seconds: f64) -> Result<Vec<Rgb>> {
    frames_uniform_range(path, n, max_seconds, 0, n)
}

/// Frames `from..to` of the `n` that [`frames_uniform`] picks, reading no
/// further than the last one wanted.
pub fn frames_uniform_range(path: &Path, n: usize, max_seconds: f64, from: usize, to: usize) -> Result<Vec<Rgb>> {
    let mut r = open(path, None)?;
    yuv::uniform_range(&mut r, n, max_seconds, from, to, path).map_err(|_| r.why_empty(path))
}

/// As [`frames_uniform`], with each frame's presentation time and position
/// in the stream and a trace of every frame read (see [`yuv::uniform_timed`]).
pub fn frames_uniform_timed(path: &Path, n: usize, max_seconds: f64, follow: usize, blur: usize) -> Result<(Vec<yuv::Timed>, yuv::Trace)> {
    let mut r = open(path, None)?;
    yuv::uniform_timed(&mut r, n, max_seconds, follow, blur, path).map_err(|_| r.why_empty(path))
}

/// The frames nearest to `targets` (seconds after the first frame,
/// ascending), each with the time of the frame used; `None` for a target the
/// clip ended before. See [`yuv::at_times`].
pub fn frames_at(path: &Path, targets: &[f64]) -> Result<Vec<Option<(f64, Rgb)>>> {
    let mut r = open(path, None)?;
    yuv::at_times(&mut r, targets, path).map_err(|_| r.why_empty(path))
}

/// Times of the clip's hard cuts, from a pass over every frame ([`yuv::cuts`]).
pub fn cut_times(path: &Path) -> Result<Vec<f64>> {
    Ok(yuv::cuts(&mut open(path, None)?))
}

/// As [`frames_uniform_range`], with the frames left as decoded.
pub fn frames_uniform_range_unconverted(path: &Path, n: usize, max_seconds: f64, from: usize, to: usize) -> Result<Vec<yuv::Yuv>> {
    let mut r = open(path, None)?;
    yuv::uniform_range_with(&mut r, n, max_seconds, from, to, path, yuv::Yuv::clone).map_err(|_| r.why_empty(path))
}

pub fn frames_first(path: &Path, n: usize) -> Result<Vec<Rgb>> {
    let mut r = open(path, None)?;
    yuv::first(&mut r, n, true, path).map_err(|_| r.why_empty(path))
}

/// The first frame shown at or after time `t` (seconds).
pub fn frame_at(path: &Path, t: f64) -> Result<Rgb> {
    let mut r = open(path, Some(t))?;
    match r.next() {
        Some((_, f)) => Ok(f.to_rgb()),
        None if r.errors().1.is_some() => Err(r.why_empty(path)),
        None => bail!("no frame at {t:.2}s in {}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_video_preflight_rejects_an_excessive_uniform_sample_count_and_fragments() {
        let fixture = include_bytes!("../tests/fixtures/video.mp4");
        let check = |bytes: &[u8]| {
            use std::io::Write;
            let mut file = tempfile::NamedTempFile::new().unwrap();
            file.write_all(bytes).unwrap();
            validate_upload(file.path())
        };
        check(fixture).unwrap();

        let mut bytes = fixture.to_vec();
        let size_box = bytes.windows(4).position(|part| part == b"stsz").unwrap() + 4;
        bytes[size_box + 4..size_box + 8].copy_from_slice(&1u32.to_be_bytes());
        bytes[size_box + 8..size_box + 12].copy_from_slice(&1_000_000u32.to_be_bytes());
        let error = check(&bytes).unwrap_err();
        assert!(format!("{error:#}").contains("sample count limit"));

        let mut bytes = fixture.to_vec();
        bytes.extend_from_slice(b"\x00\x00\x00\x08moof");
        assert!(check(&bytes).unwrap_err().to_string().contains("fragmented MP4"));
    }

    #[test]
    fn http_video_preflight_accepts_hevc_with_a_conformance_window() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hevc.mp4");
        validate_upload(&path).unwrap();
        let decoded = frames_uniform(&path, 4, 30.0).unwrap();
        assert_eq!(decoded.len(), 4);
        assert!(decoded.iter().all(|frame| (frame.w, frame.h) == (98, 66)));
    }

    #[test]
    fn hostile_run_lengths_cost_no_more_than_the_track_has_samples() {
        let entry = |run: u32, value: u32| [run.to_be_bytes(), value.to_be_bytes()].concat();
        // Three samples, and tables whose every run claims four billion.
        let stts = [entry(u32::MAX, 10), entry(u32::MAX, 10), entry(u32::MAX, 10)].concat();
        let ctts = [entry(1, 5), entry(u32::MAX, 2), entry(u32::MAX, 9)].concat();
        let started = std::time::Instant::now();
        let times = sample_times(&stts, 3, Some((&ctts, 3)), 3).unwrap();
        assert_eq!(times, vec![5, 12, 22]);
        assert!(started.elapsed().as_millis() < 500, "walked the claimed runs: {:?}", started.elapsed());
        // Ordinary tables still read as before.
        let stts = [entry(2, 10), entry(2, 20)].concat();
        assert_eq!(sample_times(&stts, 2, None, 4).unwrap(), vec![0, 10, 20, 40]);
        // A table that covers too few samples is an error, not a short track.
        assert!(sample_times(&stts, 2, None, 5).is_err());
    }

    fn boxed(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut b = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
        b.extend_from_slice(kind);
        b.extend_from_slice(payload);
        b
    }

    #[test]
    fn child_boxes_are_found_and_truncation_is_tolerated() {
        let mut d = boxed(b"free", &[1, 2, 3]);
        d.extend(boxed(b"mdia", b"payload"));
        assert_eq!(child(&d, b"mdia"), Some(&b"payload"[..]));
        assert_eq!(child(&d, b"stbl"), None);
        // A box that claims more bytes than there are ends the walk.
        d.extend_from_slice(&[0, 0, 1, 0, b'm', b'o', b'o', b'v', 9]);
        assert_eq!(children(&d).len(), 2);
        let mut d = boxed(b"free", &[1, 2, 3]);
        d.extend([0, 0, 0, 1, b'm', b'o', b'o', b'v']);
        d.extend(u64::MAX.to_be_bytes());
        assert_eq!(children(&d).len(), 1);
    }

    #[test]
    fn samples_are_reduced_to_eight_bits_with_rounding() {
        assert_eq!(to8(200, 8), 200);
        assert_eq!(to8(1023, 10), 255);
        assert_eq!(to8(514, 10), 129);
        assert_eq!(to8(513, 10), 128);
    }

    #[test]
    fn chroma_is_interleaved_and_padded_for_odd_sizes() {
        // 3x3 luma with a 1x1 chroma plane: both chroma rows and columns repeat it.
        assert_eq!(interleave(3, 3, 1, 1, |_| (7, 9)), vec![7, 9, 7, 9, 7, 9, 7, 9]);
        assert_eq!(interleave(4, 2, 2, 1, |i| (i as u8, 10 + i as u8)), vec![0, 10, 1, 11]);
    }

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
    }

    /// Every frame `r` gives: the SHA-256 of the frames laid out as FFmpeg's
    /// `-f rawvideo -pix_fmt yuv420p` writes them (Y, Cb, Cr planes, frame
    /// after frame), and the frames' times and pictures.
    fn decoded(r: &mut Reader) -> (String, Vec<(f64, Yuv)>) {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        let mut frames = Vec::new();
        while let Some((t, f)) = r.next() {
            let (y, uv) = f.samples();
            hash.update(y);
            hash.update(uv.iter().step_by(2).copied().collect::<Vec<u8>>());
            hash.update(uv.iter().skip(1).step_by(2).copied().collect::<Vec<u8>>());
            frames.push((t, f));
        }
        (hash.finalize().iter().map(|b| format!("{b:02x}")).collect(), frames)
    }

    // The expected digests are libvpx's, through FFmpeg:
    // `ffmpeg -c:v libvpx-vp9 -i FILE -fps_mode passthrough -f rawvideo -pix_fmt yuv420p - | shasum -a 256`
    // (`-c:v libvpx` for VP8). FFmpeg's own decoders give the same.

    #[test]
    fn vp9_in_mp4_matches_libvpx_frame_for_frame() {
        // Twelve frames at 12 per second, a key frame every fourth, alt-ref frames hidden in superframes.
        let mut r = open(&fixture("vp9.mp4"), None).unwrap();
        let info = r.info();
        assert_eq!((info.width, info.height, info.duration, info.fps), (96, 64, 1.0, 12.0));
        let (digest, frames) = decoded(&mut r);
        assert_eq!(digest, "0d20acce6eea4caf5f866d7f5f8ca149928f9621733bff85d08db1337cbc3e7d");
        assert_eq!(frames.iter().map(|f| f.0).collect::<Vec<_>>(), (0..12).map(|i| i as f64 / 12.0).collect::<Vec<_>>());
        assert_eq!(r.errors(), (0, None));
        // Untagged and smaller than 720 pixels: BT.601, and the stream's own limited range.
        assert_eq!(r.declared(), (None, Some(false)));
        assert_eq!(frames[0].1.tags(), (false, false));
        assert_eq!(probe(&fixture("vp9.mp4")).unwrap().fps, 12.0);
    }

    #[test]
    fn a_vp9_seek_starts_at_a_key_frame_and_gives_the_frame_on_screen() {
        let path = fixture("vp9.mp4");
        let (_, all) = decoded(&mut open(&path, None).unwrap());
        // Key frames are frames 0, 4 and 8.
        for (t, key, shown) in [(0.30, 0, 3), (0.34, 4, 4), (0.95, 8, 11)] {
            let mut r = open(&path, Some(t)).unwrap();
            let Kind::Mp4(mp4) = &r.0 else { panic!("not read as MP4") };
            assert_eq!(mp4.next_sample, key, "at {t}s");
            let (pts, frame) = r.next().unwrap();
            assert_eq!(pts, all[shown].0, "at {t}s");
            assert!(frame.same_as(&all[shown].1), "at {t}s");
            assert_eq!(r.errors(), (0, None));
            assert_eq!(frame_at(&path, t).unwrap().data, all[shown].1.to_rgb().data);
        }
    }

    #[test]
    fn vp9_in_webm_matches_libvpx_on_the_video_timeline() {
        // 35x27, eight unevenly spaced frames, and a two-second audio track that the duration must ignore.
        let path = fixture("vp9.webm");
        let mut r = open(&path, None).unwrap();
        let info = r.info();
        assert_eq!((info.width, info.height, info.duration, info.fps), (35, 27, 1.375, 8.0));
        let (digest, frames) = decoded(&mut r);
        assert_eq!(digest, "5d4b1c521352fe68f59d326941e8df5f58ad928f54b1ddc80dfbd7f1df32fabb");
        assert_eq!(frames.iter().map(|f| f.0).collect::<Vec<_>>(), [0.0, 0.125, 0.25, 0.375, 0.5, 0.75, 1.0, 1.25]);
        assert_eq!(r.errors(), (0, None));
        // The stream says BT.709; the range is the stream's.
        assert_eq!(r.declared(), (Some(1), Some(false)));
        assert_eq!(frames[0].1.tags(), (true, false));
        // Seeking gives the frame on screen at the time asked for.
        let mut seek = open(&path, Some(0.74)).unwrap();
        assert_eq!(seek.next().unwrap().0, 0.5);
        assert_eq!(seek.next().unwrap().0, 0.75);
    }

    #[test]
    fn vp8_in_webm_matches_libvpx_whatever_the_file_is_called() {
        use std::io::Write;
        // The VP8 study's fixture, under an MP4 name: the reader goes by the contents.
        let mut renamed = tempfile::Builder::new().suffix(".mp4").tempfile().unwrap();
        renamed.write_all(include_bytes!("../tests/fixtures/vp8.webm")).unwrap();
        let mut r = open(renamed.path(), None).unwrap();
        let info = r.info();
        assert_eq!((info.width, info.height, info.duration, info.fps), (34, 26, 1.375, 8.0));
        let (digest, frames) = decoded(&mut r);
        assert_eq!(digest, "1fddae316e5fd5808f8f2ac7ee269eab421c2d0f4eef8332c2ef2f4b0d9c4ebf");
        assert_eq!(frames.iter().map(|f| f.0).collect::<Vec<_>>(), [0.0, 0.125, 0.25, 0.375, 0.5, 0.75, 1.0, 1.25]);
        assert_eq!(r.errors(), (0, None));
        // Untagged VP8 is BT.601 (RFC 6386), not the size rule.
        assert_eq!(frames[0].1.tags(), (false, false));
        let mut seek = open(renamed.path(), Some(0.74)).unwrap();
        assert_eq!(seek.next().unwrap().0, 0.5);
    }

    #[test]
    fn other_matroska_codecs_and_vp9_other_than_4_2_0_are_refused_by_name() {
        use std::io::Write;
        let mut bytes = include_bytes!("../tests/fixtures/vp8.webm").to_vec();
        let at = bytes.windows(5).position(|w| w == b"V_VP8").unwrap();
        bytes[at..at + 5].copy_from_slice(b"V_AV1");
        let mut other = tempfile::NamedTempFile::new().unwrap();
        other.write_all(&bytes).unwrap();
        assert!(format!("{:#}", probe(other.path()).unwrap_err()).contains("V_AV1"));
        // VP9 profile 1, 4:4:4.
        assert!(format!("{:#}", open(&fixture("vp9-444.mp4"), None).err().unwrap()).contains("not 4:2:0"));
    }

    #[test]
    fn http_video_preflight_refuses_vp9() {
        assert!(format!("{:#}", validate_upload(&fixture("vp9.mp4")).unwrap_err()).contains("VP9 video is not supported for uploads"));
    }

    #[test]
    fn webm_times_are_the_ones_an_mp4_of_the_same_instants_gives() {
        // Millisecond units against the 1/16000 timescale FFmpeg writes for them in MP4.
        assert!((0..200_000u64).all(|ms| webm::seconds(ms as f64, 1e6) == (ms * 16) as f64 / 16_000.0));
    }

    #[test]
    fn vp9_colour_codes_follow_ffmpeg() {
        assert_eq!([0, 1, 2, 3, 4, 5, 6, 7].map(vp9_matrix), [None, Some(5), Some(1), Some(6), Some(7), Some(9), None, Some(0)]);
        // The untagged rule the macOS decoder follows.
        assert!(assumed_bt709(None, None, 1280, 720) && !assumed_bt709(None, None, 640, 360));
        assert!(assumed_bt709(None, Some(1), 640, 360) && !assumed_bt709(Some(6), Some(1), 1280, 720));
    }
}

#[cfg(all(test, target_os = "macos"))]
mod against_system_decoder {
    /// Needs a real clip: `TINTYPE_TEST_VIDEO=clip.mp4 cargo test --release -- --ignored`.
    #[test]
    #[ignore]
    fn frames_match_the_system_decoder() {
        let path = std::path::PathBuf::from(std::env::var("TINTYPE_TEST_VIDEO").expect("set TINTYPE_TEST_VIDEO"));
        let c = crate::avf::compare_with_pure_rust(&path, 120).unwrap();
        assert!(c.frames > 0, "{c:?}");
        assert_eq!((c.identical, c.produced.0), (c.frames, c.produced.1), "{c:?}");
        assert!(c.max_time_difference < 1e-3, "{c:?}");
    }
}

#[cfg(test)]
mod reference {
    /// Per-frame digests of what the reader decodes, for checking it against
    /// another decoder. Needs real clips: `TINTYPE_TEST_VIDEOS=clips.txt
    /// TINTYPE_DIGESTS=out.tsv cargo test --release --lib swdec::reference --
    /// --ignored`. Writes one line per frame: clip, index, presentation time,
    /// width, height, the SHA-256 of the Y, Cb and Cr planes (8-bit planar,
    /// as FFmpeg's `yuv420p` lays them out) and the colour tags.
    #[test]
    #[ignore]
    fn frame_digests() {
        use crate::yuv::Source;
        use sha2::{Digest, Sha256};
        use std::io::Write;
        let hex = |d: &[u8]| Sha256::digest(d).iter().map(|b| format!("{b:02x}")).collect::<String>();
        let list = std::fs::read_to_string(std::env::var("TINTYPE_TEST_VIDEOS").expect("set TINTYPE_TEST_VIDEOS")).unwrap();
        let mut out = std::fs::File::create(std::env::var("TINTYPE_DIGESTS").expect("set TINTYPE_DIGESTS")).unwrap();
        for path in list.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let mut r = super::open(std::path::Path::new(path), None).unwrap();
            let mut index = 0;
            while let Some((pts, f)) = r.next() {
                let (y, uv) = f.samples();
                let (u, v): (Vec<u8>, Vec<u8>) = uv.chunks_exact(2).map(|p| (p[0], p[1])).unzip();
                let (bt709, full_range) = f.tags();
                writeln!(out, "{path}\t{index}\t{pts:?}\t{}\t{}\t{}\t{}\t{}\t{bt709}\t{full_range}", f.w, f.h, hex(y), hex(&u), hex(&v)).unwrap();
                index += 1;
            }
            assert_eq!(r.errors(), (0, None), "{path}");
        }
    }
}

#[cfg(test)]
mod continuation {
    /// Decoder-continuation benchmark for the pure-Rust reader. Needs real clips:
    /// `TINTYPE_TEST_VIDEOS=clips.txt cargo test --release --lib swdec -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn reading_on_against_reopening_for_the_second_half() {
        crate::yuv::report_continuation("pure-Rust reader", |path| crate::yuv::continuation_times(|| super::open(path, None), 16, 30.0));
    }
}
