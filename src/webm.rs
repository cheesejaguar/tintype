//! WebM and Matroska reading for the pure-Rust reader: the first enabled
//! video track, VP9 (`filmcraft-vp9`) or VP8 (`filmcraft-vp8`), in 4:2:0.
//! The demuxer supplies packet presentation times; the decoders keep the
//! reference frames, hidden ones included. Ported from the VP8 study's reader
//! and extended to VP9.
use crate::video::VideoInfo;
use crate::yuv::{Source, Yuv};
use anyhow::{bail, ensure, Context, Result};
use matroska_demuxer::{FlagInterlaced, Frame, MatrixCoefficients, MatroskaFile, Primaries, Range, StereoMode, TrackType};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

type Demux = MatroskaFile<BufReader<File>>;

enum Decoder {
    Vp8(Box<filmcraft_vp8::Decoder>),
    Vp9(Box<filmcraft_vp9::Decoder>),
}

pub(super) struct Reader {
    demux: Demux,
    decoder: Decoder,
    packet: Frame,
    track: u64,
    /// Nanoseconds per timestamp unit.
    unit_ns: f64,
    pub(super) info: VideoInfo,
    /// What the file states: matrix coefficients and range, when it does.
    pub(super) declared: (Option<u16>, Option<bool>),
    bt709: bool,
    full_range: bool,
    /// Frames shown before this time (seconds) are skipped, all but the one
    /// on screen at it.
    start: Option<f64>,
    ready: VecDeque<(f64, Yuv)>,
    last_visible: Option<u64>,
    /// The demuxer has nothing more, or reading had to stop.
    ended: bool,
    /// Packets the decoder rejected, and the first reason given for any
    /// problem (a rejected packet, or what stopped reading).
    pub(super) errors: usize,
    pub(super) last_error: Option<String>,
}

/// Open `path` as WebM/Matroska if it starts with an EBML header; otherwise
/// return `previous`, the error the MP4 reader gave.
pub(super) fn open_if_matroska(path: &Path, start: Option<f64>, previous: anyhow::Error) -> Result<Reader> {
    let Ok(mut file) = File::open(path) else {
        return Err(previous);
    };
    let mut magic = [0; 4];
    if file.read_exact(&mut magic).is_err() || magic != [0x1a, 0x45, 0xdf, 0xa3] {
        return Err(previous);
    }
    file.rewind()?;
    open(file, start).map_err(|e| anyhow::anyhow!("reading {}: {e:#}", path.display()))
}

/// Seconds in `units` timestamp units of `unit_ns` nanoseconds, by one
/// division: the time the MP4 reader and AVFoundation give the same instant
/// (`ticks / timescale`), to the last bit. A product with a rounded
/// `unit_ns / 1e9` can land on the other side of a frame-sampling tie.
pub(super) fn seconds(units: f64, unit_ns: f64) -> f64 {
    units * unit_ns / 1e9
}

/// The ISO/IEC 23091-2 code of a stated matrix.
fn matrix_code(m: MatrixCoefficients) -> Option<u16> {
    use MatrixCoefficients::*;
    Some(match m {
        Unknown => return None,
        Identity => 0,
        Bt709 => 1,
        Fcc73682 => 4,
        Bt470bg => 5,
        Smpte170 => 6,
        Smpte240 => 7,
        YCoCg => 8,
        Bt2020Ncl => 9,
        Bt2020Cl => 10,
        SmpteSt2085 => 11,
        ChromaDerivedNcl => 12,
        ChromaDerivedCl => 13,
        Bt2100 => 14,
    })
}

/// The ISO/IEC 23091-2 code of stated primaries.
fn primaries_code(p: Primaries) -> Option<u16> {
    use Primaries::*;
    Some(match p {
        Unknown => return None,
        Bt709 => 1,
        Bt470m => 4,
        Bt601 => 5,
        Smpte170 => 6,
        Smpte240 => 7,
        Film => 8,
        Bt2020 => 9,
        SmpteSt428_1 => 10,
        SmpteRp432_2 => 11,
        SmpteEg432_2 => 12,
        JedecP22 => 22,
    })
}

fn open(file: File, start: Option<f64>) -> Result<Reader> {
    let mut demux = MatroskaFile::open(BufReader::new(file))?;
    ensure!(matches!(demux.ebml_header().doc_type(), "webm" | "matroska"), "unsupported EBML document type");
    let track = demux.tracks().iter().find(|t| t.flag_enabled() && t.track_type() == TrackType::Video).context("no enabled video track")?;
    let vp9 = match track.codec_id() {
        "V_VP9" => true,
        "V_VP8" => false,
        other => bail!("video codec {other} is not supported by this reader (VP9 and VP8 are)"),
    };
    ensure!(track.content_encodings().is_none_or(|e| e.is_empty()), "encoded Matroska track is unsupported");
    let video = track.video().context("video geometry missing")?;
    ensure!(video.alpha_mode().unwrap_or(0) == 0 && video.stereo_mode().is_none_or(|v| v == StereoMode::Mono), "alpha and stereo video are unsupported");
    ensure!(
        [video.pixel_crop_top(), video.pixel_crop_left(), video.pixel_crop_bottom(), video.pixel_crop_right()].into_iter().flatten().all(|n| n == 0),
        "cropped video track is unsupported"
    );
    ensure!(video.flag_interlaced() != FlagInterlaced::Interlaced, "interlaced video track is unsupported");
    let width = u32::try_from(video.pixel_width().get())?;
    let height = u32::try_from(video.pixel_height().get())?;
    // The same decoded picture budget as the local HEVC decoder.
    ensure!(u64::from(width) * u64::from(height) <= rust_h265::DecodeLimits::default().max_picture_pixels, "picture exceeds the local decoder limit");
    let colour = video.colour();
    let mut matrix = colour.and_then(|c| c.matrix_coefficients()).and_then(matrix_code);
    let primaries = colour.and_then(|c| c.primaries()).and_then(primaries_code);
    let mut range = match colour.and_then(|c| c.range()) {
        Some(Range::Full) => Some(true),
        Some(Range::Broadcast) => Some(false),
        _ => None,
    };
    let default_seconds = track.default_duration().map_or(0., |d| d.get() as f64 / 1e9);
    let track = track.track_number().get();
    let unit_ns = demux.info().timestamp_scale().get() as f64;
    let mut duration = seconds(demux.info().duration().unwrap_or(0.), unit_ns);
    let mut fps = if default_seconds > 0. { 1. / default_seconds } else { 0. };
    // The segment's end can include an initial presentation offset or another
    // track. A single-track file with timing in its header needs only its
    // first visible packet; otherwise the chosen track's interval is measured.
    let header_timing = demux.tracks().len() == 1 && duration.is_finite() && duration > 0. && fps > 0.;
    // VP8 says in its frame tag whether a frame is shown. A VP9 block holds
    // one shown frame, with any hidden ones packed beside it in a superframe.
    let shown = |p: &Frame| !p.is_invisible && if vp9 { !p.data.is_empty() } else { p.data.first().is_some_and(|b| b & 0x10 != 0) };
    let mut packet = Frame::default();
    let mut count = 0u64;
    let mut first_key = None;
    let (mut first, mut last, mut last_step, mut last_duration) = (0., 0., 0., 0.);
    while demux.next_frame(&mut packet)? {
        if packet.track != track {
            continue;
        }
        if vp9 && count == 0 && first_key.is_none() {
            first_key = filmcraft_vp9::keyframe_info(&packet.data);
        }
        if !shown(&packet) {
            continue;
        }
        let pts = seconds(packet.timestamp as f64, unit_ns);
        if count == 0 {
            first = pts;
        } else {
            ensure!(pts >= last, "out-of-order video timestamps");
            if pts > last {
                last_step = pts - last;
            }
        }
        last = pts;
        last_duration = seconds(packet.duration.unwrap_or(0) as f64, unit_ns);
        count += 1;
        if header_timing && duration > first {
            break;
        }
    }
    ensure!(count > 0, "no visible video packets");
    if header_timing && duration > first {
        duration -= first;
    } else {
        let tail = [last_duration, default_seconds, last_step].into_iter().find(|v| *v > 0.).unwrap_or(0.);
        duration = last - first + tail;
        if fps == 0. && duration > 0. {
            fps = count as f64 / duration;
        }
    }
    let bt709 = if vp9 {
        // The container's tags, then the stream's own, then what the MP4
        // reader assumes for an untagged file.
        if let Some(k) = first_key {
            ensure!(k.subsampling_x && k.subsampling_y, "not 4:2:0 video");
            matrix = matrix.or(super::vp9_matrix(k.color.color_space));
            range = range.or(Some(k.color.full_range));
        }
        super::assumed_bt709(matrix, primaries, width, height)
    } else {
        // VP8's colour space is YUV close to BT.601 (RFC 6386 §9.2).
        matrix.map_or(primaries == Some(1), |m| m == 1)
    };
    demux.seek(0)?;
    let decoder = if vp9 { Decoder::Vp9(Box::new(filmcraft_vp9::Decoder::with_threads(1))) } else { Decoder::Vp8(Box::new(filmcraft_vp8::Decoder::new())) };
    Ok(Reader {
        demux,
        decoder,
        packet: Frame::default(),
        track,
        unit_ns,
        info: VideoInfo { width, height, duration, fps },
        declared: (matrix, range),
        bt709,
        full_range: range.unwrap_or(false),
        start,
        ready: VecDeque::new(),
        last_visible: None,
        ended: false,
        errors: 0,
        last_error: None,
    })
}

impl Reader {
    /// Decode the next packet of the track and queue the frames it shows.
    /// False once the stream has ended.
    fn decode_packet(&mut self) -> Result<bool> {
        if !self.demux.next_frame(&mut self.packet)? {
            if let Decoder::Vp9(dec) = &mut self.decoder {
                for p in dec.flush() {
                    self.queue(super::vp9_planes(p)?)?;
                }
            }
            return Ok(false);
        }
        let f = &self.packet;
        if f.track != self.track {
            return Ok(true);
        }
        let (timestamp, invisible) = (f.timestamp, f.is_invisible);
        let pts = i64::try_from(timestamp)?;
        let decoded = match &mut self.decoder {
            Decoder::Vp8(dec) => {
                if let Some(k) = filmcraft_vp8::keyframe_info(&f.data) {
                    ensure!(k.width == self.info.width && k.height == self.info.height && k.horizontal_scale == 0 && k.vertical_scale == 0, "VP8 coded and display dimensions differ");
                }
                dec.decode(&f.data, pts).map(|p| p.into_iter().map(vp8_planes).collect::<Result<Vec<_>>>()).map_err(|e| e.to_string())
            }
            Decoder::Vp9(dec) => dec.decode(&f.data, pts).map(|p| p.into_iter().map(super::vp9_planes).collect::<Result<Vec<_>>>()).map_err(|e| e.to_string()),
        };
        match decoded {
            // VP8 frames in a block marked invisible are kept as references only.
            Ok(planes) if invisible && matches!(self.decoder, Decoder::Vp8(_)) => drop(planes?),
            Ok(planes) => {
                for p in planes? {
                    self.queue(p)?;
                }
            }
            Err(e) => {
                self.errors += 1;
                self.last_error.get_or_insert(e);
            }
        }
        Ok(true)
    }

    /// Queue a shown picture under the time of the packet that showed it.
    fn queue(&mut self, (shown_by, w, h, y, uv): super::Picture8) -> Result<()> {
        let timestamp = u64::try_from(shown_by)?;
        ensure!(self.last_visible.is_none_or(|last| timestamp >= last), "out-of-order video timestamps");
        self.last_visible = Some(timestamp);
        ensure!(y.len() == w * h, "decoder returned a short frame");
        self.ready.push_back((seconds(timestamp as f64, self.unit_ns), Yuv::new(w, h, y, uv, self.bt709, self.full_range)));
        Ok(())
    }

    fn pop(&mut self) -> Result<Option<(f64, Yuv)>> {
        loop {
            if let Some(f) = self.ready.pop_front() {
                return Ok(Some(f));
            }
            if self.ended {
                return Ok(None);
            }
            // The last call may still queue what the decoder held back.
            self.ended = !self.decode_packet()?;
        }
    }

    fn take_next(&mut self) -> Result<Option<(f64, Yuv)>> {
        let Some(start) = self.start.take().filter(|t| *t > 0.) else {
            return self.pop();
        };
        // The frame on screen at `start`: the last one shown at or before it.
        let Some(mut previous) = self.pop()? else {
            return Ok(None);
        };
        while previous.0 < start {
            let Some(current) = self.pop()? else {
                break;
            };
            if current.0 > start {
                self.ready.push_front(current);
                break;
            }
            previous = current;
        }
        Ok(Some(previous))
    }
}

/// A VP8 picture in the shape [`super::vp9_planes`] gives.
fn vp8_planes(p: filmcraft_vp8::Picture) -> Result<super::Picture8> {
    ensure!(p.color_space == 0, "reserved VP8 color space");
    let (w, h) = (p.width as usize, p.height as usize);
    let chroma = w.div_ceil(2) * h.div_ceil(2);
    ensure!(p.y.len() == w * h && p.u.len() == chroma && p.v.len() == chroma, "incomplete VP8 output planes");
    let mut uv = vec![0; chroma * 2];
    for ((pair, u), v) in uv.chunks_exact_mut(2).zip(p.u).zip(p.v) {
        pair[0] = u;
        pair[1] = v;
    }
    Ok((p.pts, w, h, p.y, uv))
}

impl Source for Reader {
    fn duration(&self) -> f64 {
        self.info.duration
    }

    /// A demuxer failure or a broken stream ends the clip; the reason is
    /// kept for [`super::Reader::errors`].
    fn next(&mut self) -> Option<(f64, Yuv)> {
        if self.ended && self.ready.is_empty() {
            return None;
        }
        self.take_next().unwrap_or_else(|e| {
            self.ended = true;
            self.ready.clear();
            self.errors += 1;
            self.last_error.get_or_insert(format!("{e:#}"));
            None
        })
    }
}
