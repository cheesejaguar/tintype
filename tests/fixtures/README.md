Both one-second clips were created from FFmpeg's geometric test pattern.

```sh
ffmpeg -f lavfi -i 'testsrc2=size=96x64:rate=12:duration=1' \
  -c:v libx264 -pix_fmt yuv420p -movflags +faststart -map_metadata -1 video.mp4

ffmpeg -f lavfi -i 'testsrc2=size=98x66:rate=12:duration=1' \
  -c:v libx265 -x265-params 'pools=none:frame-threads=1:log-level=error' \
  -pix_fmt yuv420p -movflags +faststart -map_metadata -1 -tag:v hvc1 hevc.mp4
```
