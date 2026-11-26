Both one-second clips were created from FFmpeg's geometric test pattern.

```sh
ffmpeg -f lavfi -i 'testsrc2=size=96x64:rate=12:duration=1' \
  -c:v libx264 -pix_fmt yuv420p -movflags +faststart -map_metadata -1 video.mp4

ffmpeg -f lavfi -i 'testsrc2=size=98x66:rate=12:duration=1' \
  -c:v libx265 -x265-params 'pools=none:frame-threads=1:log-level=error' \
  -pix_fmt yuv420p -movflags +faststart -map_metadata -1 -tag:v hvc1 hevc.mp4
```

The VP9 and VP8 fixtures were made with FFmpeg 9.0.2 and libvpx. The reader
tests compare every decoded plane with libvpx's output for the same file,
`ffmpeg -c:v libvpx-vp9 -i FILE -fps_mode passthrough -f rawvideo -pix_fmt yuv420p -`
(`-c:v libvpx` for VP8).

`vp9.mp4` has twelve frames with a key frame every fourth, and alt-ref
frames hidden in superframes.

```sh
ffmpeg -f lavfi -i 'testsrc2=size=96x64:rate=12:duration=1' \
  -c:v libvpx-vp9 -crf 30 -b:v 0 -g 4 -threads 1 -pix_fmt yuv420p \
  -movflags +faststart -map_metadata -1 -fflags +bitexact -flags:v +bitexact vp9.mp4
```

`vp9.webm` is 35x27 (odd, for the chroma rounding), tagged BT.709, with
eight unevenly spaced frames beside an audio track longer than the video.

```sh
ffmpeg -f lavfi -i 'testsrc2=size=36x28:rate=8:duration=1' -f lavfi -i 'anullsrc=r=48000:cl=mono:d=2' \
  -map 0:v -map 1:a -vf 'crop=35:27:0:0:exact=1,setpts=if(lt(N\,4)\,N\,2*N-4)/(8*TB)' -fps_mode passthrough \
  -c:v libvpx-vp9 -crf 30 -b:v 0 -threads 1 -pix_fmt yuv420p -colorspace bt709 -c:a libopus \
  -fflags +bitexact -flags:v +bitexact -flags:a +bitexact -metadata encoder= vp9.webm
```

`vp9-444.mp4` is VP9 profile 1 (4:4:4), which the reader refuses.

```sh
ffmpeg -f lavfi -i 'testsrc2=size=16x16:rate=2:duration=1' \
  -c:v libvpx-vp9 -crf 30 -b:v 0 -threads 1 -pix_fmt yuv444p \
  -movflags +faststart -map_metadata -1 -fflags +bitexact -flags:v +bitexact vp9-444.mp4
```

`vp8.webm` comes from the VP8 study: eight frames, uneven presentation
spacing, and audio longer than the visible video.

```sh
ffmpeg -f lavfi -i 'testsrc2=size=34x26:rate=8:duration=1' -f lavfi -i 'anullsrc=r=48000:cl=mono:d=2' \
  -map 0:v -map 1:a -vf 'setpts=if(lt(N\,4)\,N\,2*N-4)/(8*TB)' -fps_mode passthrough \
  -c:v libvpx -threads 1 -deadline good -b:v 100k -c:a libopus -fflags +bitexact \
  -flags:v +bitexact -flags:a +bitexact -metadata encoder= vp8.webm
```
