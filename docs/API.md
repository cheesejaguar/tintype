# Self-hosted HTTP API

`tintype serve` loads the photo and video models once, then processes uploads on the host. It uses the same default photo and video profiles as `tintype detect`. For a media library already on the host, `tintype batch --json ./media` streams results without HTTP or uploads.

## Start the server

```sh
tintype serve --bind 127.0.0.1:8080
curl --fail-with-body http://127.0.0.1:8080/healthz
```

The address defaults to `127.0.0.1:8080`. When the model worker is ready, `GET /healthz` returns JSON with `status` set to `"ok"` and `version` set to the running Tintype version. `HEAD /healthz` returns the same status and headers with an empty body. Health requests need no credentials.

`--device auto` uses Apple Metal when available and the CPU elsewhere. Use `--device cpu` to select the CPU explicitly. `--weights` and `--video-weights` select model files. The [distribution guide](DISTRIBUTION.md) covers containers, model discovery, and the licences for bundled and separately supplied weights.

Set `TINTYPE_API_TOKEN` to require a bearer token for inference. A token is required when binding outside loopback. For a remote deployment, terminate TLS at a reverse proxy, keep the backend on a private interface, and restrict access and request rates at the proxy.

```sh
export TINTYPE_API_TOKEN="$(openssl rand -hex 32)"
tintype serve --bind 0.0.0.0:8080 --device cpu
```

Include `-H "Authorization: Bearer $TINTYPE_API_TOKEN"` in authenticated requests. The token is compared exactly, and the `Bearer` scheme is case-insensitive.

## Detect one file

Send a `multipart/form-data` upload containing one file field named `file`. Inference takes uploaded bytes directly. The filename is echoed in the result and selects video decoding for `.mp4`, `.mov`, and `.m4v`.

```sh
curl --fail-with-body -F 'file=@photo.jpg' http://127.0.0.1:8080/v1/detect
```

A successful photo response looks like this. Values are illustrative.

```json
{
  "name": "photo.jpg",
  "kind": "image",
  "profile": "default",
  "score": 0.71,
  "verdict": "ai-generated",
  "logit": 2.41,
  "threshold_logit": 1.52,
  "views": 6
}
```

The detector flags scores above `0.5` and reports `likely-real` otherwise. Successful rows also carry the raw `logit` and `threshold_logit` used by the model profile. Treat the result as one signal when reviewing provenance.

`score` is a detector score, not a probability that the file is AI-generated. The calibration places `0.5` at a false-positive target on held-out authentic media and fits the slope with both classes weighted equally. The share of flags that are correct depends on how common AI-generated files are among your inputs: `precision = recall·π / (recall·π + FPR·(1 − π))` for a share π. At the rates of the newest blind photo batch (89.7% recall, 1.21% of authentic photos flagged) and π = 1%, about 43% of flags would be correct, if those rates transferred. Reaching 90% at 90% recall takes a false-positive rate of about 0.10%.

For video, `profile` is `video`. The response reports `frames`, `frames_expected`, `duration_s`, `analysed_until_s`, and `complete`. `analysed_until_s` is the latest sampling target represented in the score. The decoder can use the nearest frame to each target. Both timestamps may be `null` if the information is unavailable. `complete` means the model obtained the evidence its sampling and early-exit profile required. A decodable clip with insufficient coverage receives the `inconclusive` verdict. The server samples at most the first 30 seconds of each clip.

## Review fields (opt-in)

By default no response has a `review` field. Start the server with a review band to add one to every scored result:

```sh
tintype serve --review-margin 1.0                              # by margin
tintype serve --review-margin 1.0 --review-disagreement 0.5    # and by disagreement among views
```

The same flags work on `tintype detect` and `tintype batch`. A model spec can carry the band instead, as `"review": {"margin": 1.0, "view_disagreement": 0.5}` at the top level for the default photo profile and inside `"video"` for clips; a flag overrides the stored value of the same name. The released specs have no band.

```json
{
  "name": "photo.jpg",
  "kind": "image",
  "profile": "default",
  "score": 0.61,
  "verdict": "ai-generated",
  "logit": 2.37,
  "threshold_logit": 1.69,
  "views": 6,
  "review": true,
  "review_reason": "margin"
}
```

| Field | Present | Meaning |
| --- | --- | --- |
| `review` | On every scored result when a band is in use; never otherwise | `true` when the result should go to a person |
| `review_reason` | Only when `review` is `true` | `"margin"`: the calibrated logit, `ln(score / (1 − score))`, is strictly within the margin of 0. `"view_disagreement"`: at least the stated share of the views scored fall on the other side of `threshold_logit` from their mean |

These fields are additive. `verdict`, `score`, `logit`, HTTP status and CLI exit status are the same with and without a band, and no extra inference runs: the decision uses the view or frame logits already computed.

- **Precedence.** When both reasons apply, `review_reason` is `"margin"`.
- **Margin units.** Calibrated logits around the active profile's threshold: `logit_scale · (logit − threshold_logit)`. A margin of 0 marks nothing; the band is open, so a result exactly at the margin is outside it.
- **Disagreement with early exit.** Only the views that were looked at exist. A photo decided on its first view has one logit and is never marked for disagreement. With `--all-views` every view counts.
- **Video.** The margin is on the clip's calibrated logit, and disagreement is the share of scored frame logits (each the mean of that frame's views) on the other side of the threshold. `inconclusive` keeps its meaning (incomplete coverage) and is still reported in `verdict`; `review` is a separate field and is computed from the frames that were read.
- **Second opinion** (`detect` only). The margin is measured on the blended score. Disagreement is among the first stage's views, against the first stage's threshold.
- **Profiles.** A band stored in a spec is in the default profile's calibrated logits, so `--strict`, `--sensitive` and `--fast` ignore it unless a flag gives one.
- **Errors.** A file that could not be scored has no verdict and no review fields.

A band is a policy fitted on calibration data once the rest of the system is fixed, and an AI-generated file sent to review counts as a miss for automatic detection. `tintype report --operating-points --review-margin …` reports coverage, review rates and recall on that basis; see the [calibration and abstention runbook](runbooks/calibration-abstention.md).

## Detect a batch

Repeat the field `file` up to 16 times. The `results` array follows upload order, including duplicate filenames and per-file errors. A structurally valid batch returns HTTP 200 even if individual files fail. Use the array index to associate results with inputs, and inspect each row before using a score. For large batches, use HTTP `Expect: 100-continue` so the server can reject unauthorized, oversized, or overloaded requests before the client sends the body.

```sh
curl --fail-with-body \
  -H 'Expect: 100-continue' \
  -F 'file=@photo.jpg' \
  -F 'file=@clip.mp4' \
  http://127.0.0.1:8080/v1/batch
```

```json
{
  "results": [
    {"name":"photo.jpg","kind":"image","profile":"default","score":0.71,"verdict":"ai-generated","logit":2.41,"threshold_logit":1.52,"views":6},
    {"name":"clip.mp4","error":{"code":"unreadable_media","message":"media is unsupported, exceeds decode limits, or could not be decoded"}}
  ]
}
```

## Limits and errors

| Limit | Value |
| --- | --- |
| Request body | 64 MiB including multipart metadata |
| Filename | 1–255 UTF-8 bytes |
| Images | JPEG, PNG, WebP, GIF, BMP, TIFF. Up to 12 million pixels, 8,192 pixels per side, and a 32:1 aspect ratio |
| Videos | Progressive MP4/MOV/M4V with one H.264 or HEVC video track in 4:2:0. Up to 2.2 million pixels per frame, 4,096 pixels per side, and a 32:1 aspect ratio. Container index up to 4 MiB, with up to 8,192 samples of up to 8 MiB each |
| Concurrent inference requests | Four across uploading, waiting, and scoring. One model worker processes jobs sequentially and batches work within each request |
| Deadlines | 20 seconds for upload and 90 seconds for queueing plus inference |

Inference errors use an `error` object with `code` and `message`. For HTTP 422 on a single upload, the body also includes the filename.

| HTTP status | Code | Meaning |
| --- | --- | --- |
| 400 | `invalid_upload` | Invalid multipart data, field, filename, or file count |
| 401 | `unauthorized` | Bearer credentials missing or invalid |
| 408 | `upload_timeout` | Upload deadline exceeded |
| 413 | `upload_too_large` | Request exceeds 64 MiB |
| 422 | `unreadable_media` | Single file is unsupported, invalid, or exceeds decode limits |
| 503 | `busy` | Worker unavailable or all request slots occupied. Retry after the `Retry-After` delay with backoff |
| 504 | `inference_timeout` | Client deadline exceeded while waiting for inference |

Images are decoded from memory. Videos are written to a randomly named temporary file while they are scored. The server removes the file on normal completion. Mount a writable temporary directory in read-only deployments and size it for uploads. Uploaded contents are never sent to a model provider.

Media decoding runs in-process. When an HTTP request times out or disconnects, the worker stops taking on remaining files as it reaches the next cancellation check. Decoder and inference calls already in progress can finish. For deployments that accept external uploads, run the service with OS or container CPU and memory limits and use a process supervisor. The [distribution guide](DISTRIBUTION.md) includes a non-root container setup.

## Measure a local deployment

Run the loopback benchmark from a source checkout after building the binary. It uses the shipped demo PNG and two one-second clips, then verifies overload and clean shutdown.

```sh
cargo build --release --locked --bin tintype
python3 scripts/bench-api.py target/release/tintype
```

On a Mac18,5 with 12 logical CPUs and 32 GiB RAM, the CPU engine with the Rust video decoder measured a median of 88 ms for one 948 KiB PNG and 617 ms for 16 copies, about 26 photos per second. The 1-second H.264 and HEVC test clips took 1.31 s and 1.36 s respectively. A batch of 14 PNGs and both clips took 3.20 s. These measurements include localhost HTTP transfer, multipart parsing, inference, and JSON serialization. The four excess requests in an eight-way 16-photo burst received HTTP 503 in under 2 ms with `Expect: 100-continue`. The other four finished in 0.62–2.45 s. Video latency will depend on resolution, codec, and frame coverage.
