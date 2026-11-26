# VP9 and VP8 decoding: the pure-Rust reader against libvpx

Tintype v0.4.0 could not read VP9: every VP9 MP4 and every WebM file was
reported as unreadable. The pure-Rust reader now decodes VP9 in MP4/MOV
(`vp09`) and in WebM/Matroska (`V_VP9`) with `filmcraft-vp9` 0.9.0, and VP8 in
WebM/Matroska with `filmcraft-vp8` 0.9.0 (the reader of the VP8 study, brought
onto main). On macOS AVFoundation refuses these files, so the existing
fallback hands them to this reader; elsewhere it is the only reader.

This is an engineering check on synthetic clips. It reports no detection
accuracy or speed, and the frame sampler, aggregation, models and cutoffs are
unchanged.

## What was checked

Eight clips made by [`make_clips.sh`](make_clips.sh) from FFmpeg's synthetic
sources with libvpx (CRF 30), 6.5 MB together:

| Clip | Container | Codec | Size | Frames | What it exercises |
|---|---|---|---|---:|---|
| `a_vp9_360p.mp4` | MP4 | VP9 | 640x360 | 120 | alt-ref frames in superframes, untagged |
| `b_vp9_720p_audio.webm` | WebM | VP9 | 1280x720 | 72 | two tile columns, a longer Opus track, untagged at 720p |
| `c_vp9_odd_bt709.webm` | WebM | VP9 | 347x251 | 75 | odd size, BT.709 tag, key frame every 30 |
| `d_vp9_720p_bt709.mp4` | MP4 | VP9 | 1280x720 | 60 | realtime encoder, BT.709 tag |
| `e_vp9_fullrange.mp4` | MP4 | VP9 | 480x270 | 75 | full range, SMPTE 170M |
| `f_vp9.mkv` | Matroska | VP9 | 320x240 | 60 | `.mkv` document type |
| `g_vp8_360p.webm` | WebM | VP8 | 640x360 | 90 | the VP8 reader |
| `h_vp9_10bit.mp4` | MP4 | VP9 profile 2 | 320x180 | 60 | 10-bit samples |

[`parity.py`](parity.py) ran two checks on Apple M6, macOS 27.2, with
FFmpeg 9.0.2:

1. **Pixels and times.** Every frame the reader returns, read through the
   ignored `swdec::reference` test, is hashed plane by plane (Y, Cb, Cr) and
   compared with FFmpeg's output from libvpx (`-c:v libvpx-vp9`, or
   `libvpx` for VP8) and from FFmpeg's own decoder (`vp9`, `vp8`). Its time
   is compared with the frame's presentation time from ffprobe. The 10-bit
   clip is compared after FFmpeg's 10-bit samples are reduced to 8 bits
   with the reader's rounding.
2. **Scores.** Each 8-bit clip of even size is decoded by libvpx and
   re-encoded as lossless H.264 (`-qp 0`, no B-frames) with the same
   timestamps and colour tags. The control is checked to decode to the same
   planes and times as the clip. Then the release binary's default path
   (`tintype detect --json`) scores both, once as by default and once with
   `--coverage uniform`, which scores and reports every sampled frame. On
   macOS the VP9 and VP8 clips go to the pure-Rust reader and the H.264
   controls to AVFoundation. Each control was deleted after scoring.

## Results ([parity.json](parity.json))

**Pixels and times: all 612 frames of the eight clips are identical**, plane
for plane, to both libvpx's and FFmpeg's own output, and every frame's time
equals ffprobe's exactly.

**Scores: the lossless controls score the same** as the clips they were
made from:

| Clip | Logit (clip = control) | Verdict | Per-frame logits and sampled times | Other fields |
|---|---:|---|---|---|
| `a_vp9_360p.mp4` | 2.0770774 | ai-generated | identical | identical |
| `b_vp9_720p_audio.webm` | 1.2895548 | likely-real | identical | `analysed_until_s` differs (see below) |
| `d_vp9_720p_bt709.mp4` | 1.5145973 | likely-real | identical | identical |
| `e_vp9_fullrange.mp4` | 0.6186797 | likely-real | identical | identical |
| `f_vp9.mkv` | 2.8740544 | ai-generated | identical | identical |
| `g_vp8_360p.webm` | 1.9700208 | ai-generated | identical | identical |

The synthetic clips have no real or generated class, so the verdicts say
only that both sides agree. No control could be made for two clips: x264
encodes 4:2:0 only at even sizes (`c`), and the reader reduces 10-bit
samples to 8 bits itself (`h`). Their pixels and times are covered by the
first check.

`TINTYPE_DECODER=rust` scores all eight clips exactly as the default path
does.

### Two things the check found

- **Millisecond times and sampling ties.** WebM stores times in
  milliseconds, so at 30 frames per second some sampling targets fall
  exactly half-way between two frames (1.25 s lies between 1.233 and
  1.267). The first version computed a WebM time as `ticks * 0.001`, one
  rounding step from the `ticks / 1000` that the MP4 reader and AVFoundation
  compute, and the tie went the other way. On `f_vp9.mkv` two of sixteen
  sampled frames differed and the logit was 2.8461 against the control's
  2.8741. The reader now divides once, and its times equal the MP4 reader's
  to the last bit (`webm_times_are_the_ones_an_mp4_of_the_same_instants_gives`).
- **Durations follow the container.** The WebM reader ends a clip one
  `DefaultDuration` (1/24 s exactly here) after its last frame, as the VP8
  study's reader did: 2.999667 s for `b`. FFmpeg gave the H.264 control's
  last sample 41 ms, so 2.999 s. Every sampled frame and logit is the
  same. Only `analysed_until_s`, which is computed from the duration,
  differs (2.8122 s against 2.8116 s).

## Colour

The matrix and range come from the container (`colr` or `vpcC` in MP4, the
`Colour` element in Matroska), then from the VP9 stream's own
`color_space` and `color_range`. A VP9 file that states no matrix gets the
rule the MP4 reader and the macOS decoder use: BT.709 from 720 pixels on the
longer side, BT.601 below. VP8 keeps the VP8 study's rule (BT.601 unless
tagged, per RFC 6386). Clip `b` (untagged, 720p) shows the rule. Its control
went through AVFoundation, which applied the same matrix.

## Reproduction

From a checkout, with FFmpeg and ffprobe built with libvpx and libx264:

```sh
docs/results/vp9-decode/make_clips.sh "$CLIPS"
cargo build --release --locked
python3 docs/results/vp9-decode/parity.py "$PWD" "$PWD/target/release/tintype" "$CLIPS" "$WORK" parity.json
```

`$WORK` holds one lossless control at a time, up to 16 MB here.
