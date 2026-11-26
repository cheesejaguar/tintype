#!/bin/sh
# Small VP9 (and VP8) verification clips from FFmpeg's synthetic sources.
# Usage: make_clips.sh OUT_DIR
set -eu
out="$1"
mkdir -p "$out"
cd "$out"
ff="ffmpeg -hide_banner -loglevel error -y"
# Moving test pattern with temporal noise, so inter frames carry real residuals.
noisy() { echo "testsrc2=size=$1:rate=$2:duration=$3,noise=alls=12:allf=t+u"; }

# A: VP9 in MP4, 640x360, 30 fps, 4 s, untagged, CRF 30 (good deadline: alt-ref, superframes).
$ff -f lavfi -i "$(noisy 640x360 30 4)" -c:v libvpx-vp9 -crf 30 -b:v 0 -row-mt 1 -pix_fmt yuv420p \
  -map_metadata -1 a_vp9_360p.mp4

# B: VP9 in WebM, 1280x720, 24 fps, 3 s, untagged, two tile columns, plus a longer Opus track.
$ff -f lavfi -i "$(noisy 1280x720 24 3)" -f lavfi -i 'sine=frequency=440:duration=4' \
  -map 0:v -map 1:a -c:v libvpx-vp9 -crf 30 -b:v 0 -row-mt 1 -tile-columns 1 -pix_fmt yuv420p \
  -c:a libopus -map_metadata -1 b_vp9_720p_audio.webm

# C: VP9 in WebM, odd size 347x251, 25 fps, 3 s, tagged BT.709 limited range, key frame every 30.
$ff -f lavfi -i 'mandelbrot=size=347x251:rate=25,trim=duration=3' -c:v libvpx-vp9 -crf 30 -b:v 0 -g 30 \
  -pix_fmt yuv420p -colorspace bt709 -color_primaries bt709 -color_trc bt709 -color_range tv \
  -map_metadata -1 c_vp9_odd_bt709.webm

# D: VP9 in MP4, 1280x720, 30 fps, 2 s, tagged BT.709, realtime deadline (no alt-ref).
$ff -f lavfi -i "$(noisy 1280x720 30 2)" -c:v libvpx-vp9 -crf 30 -b:v 0 -deadline realtime -cpu-used 8 \
  -pix_fmt yuv420p -colorspace bt709 -color_primaries bt709 -color_trc bt709 -color_range tv \
  -movflags +faststart -map_metadata -1 d_vp9_720p_bt709.mp4

# E: VP9 in MP4, 480x270, 25 fps, 3 s, full range BT.601.
$ff -f lavfi -i "$(noisy 480x270 25 3)" -c:v libvpx-vp9 -crf 30 -b:v 0 -pix_fmt yuv420p \
  -colorspace smpte170m -color_primaries smpte170m -color_trc smpte170m -color_range pc \
  -map_metadata -1 e_vp9_fullrange.mp4

# F: VP9 in Matroska (.mkv), 320x240, 30 fps, 2 s, untagged.
$ff -f lavfi -i "$(noisy 320x240 30 2)" -c:v libvpx-vp9 -crf 30 -b:v 0 -pix_fmt yuv420p \
  -map_metadata -1 f_vp9.mkv

# G: VP8 in WebM, 640x360, 30 fps, 3 s (the reader brought over from the VP8 study).
$ff -f lavfi -i "$(noisy 640x360 30 3)" -c:v libvpx -crf 30 -b:v 1M -pix_fmt yuv420p \
  -map_metadata -1 g_vp8_360p.webm

# H: VP9 profile 2 (10-bit) in MP4, 320x180, 30 fps, 2 s.
$ff -f lavfi -i "$(noisy 320x180 30 2)" -c:v libvpx-vp9 -crf 30 -b:v 0 -pix_fmt yuv420p10le \
  -map_metadata -1 h_vp9_10bit.mp4

ls -l "$out"
