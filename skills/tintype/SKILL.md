---
name: tintype
description: >
  Screen local images and videos for signs of AI generation with Tintype. Use
  when asked to assess a media file, triage a folder, or interpret Tintype
  JSON. The score measures model evidence and video results describe the
  sampled portion of the clip.
compatibility: Requires the Tintype CLI and compatible local model weights.
license: MIT OR Apache-2.0
---

# Tintype media screening

Run `tintype --version` to check that the CLI is available. If it is missing,
offer the project installer and explain that the bundled checkpoints are
CC BY-NC-SA 4.0. Follow the user's environment rules for installing tools.

```sh
curl -fsSL https://cheesejaguar.github.io/tintype/install.sh | sh
```

Use paths supplied by the user. Quote each path and put flags before `--`.
Detection runs locally and accepts a photo or video. Batch scans a directory
recursively and emits one JSON object per file.

```sh
tintype detect --json -- "/path/to/media.mp4"
tintype batch --json -- "/path/to/folder"
```

For each relevant file, report the verdict and score, and for video report
the sampled time span and whether decoding completed. A score above 0.5
flags media at the chosen operating point. A score of 0.8 expresses model
evidence on that scale. Report an error row as unreadable and
`verdict: "inconclusive"` as incomplete analysis. Exit code 2 indicates
some input errors and the other rows are still usable.

The default samples video frames across the first 30 seconds and may stop
early. If the user asks about a long clip as a whole, use
`tintype detect --json --max-seconds 0 --frames 64 --all-views -- "$path"`.
Even a full-length run samples frames, so a short edited segment can fall
between them.

Treat flags as leads for review. Mention any input-specific limitation that
affects interpretation, such as heavy recompression, face swaps, or short
generated sections between samples. Use the original media where available.
The exact per-set results are in the
[benchmark report](https://github.com/cheesejaguar/tintype/blob/main/docs/BENCHMARK.md).
