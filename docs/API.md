# Self-hosted HTTP API

`tintype serve` loads the photo and video models once, then processes uploads on the host. It uses the same default photo and video profiles as `tintype detect`. For a media library already on the host, `tintype batch --json ./media` streams results without HTTP or uploads.

## Start the server

```sh
tintype serve --bind 127.0.0.1:8080
curl --fail-with-body http://127.0.0.1:8080/healthz
```

The address defaults to `127.0.0.1:8080`. When the model worker is ready, `GET /healthz` returns JSON with `status` set to `"ok"` and `version` set to the running Tintype version. `HEAD /healthz` returns the same status and headers with an empty body. Health requests need no credentials. `GET /health` is the same check under a path that Cloud Run passes on: it answers `/healthz` itself, because it reserves paths ending in `z`.

`--device auto` uses Apple Metal when available and the CPU elsewhere. Use `--device cpu` to select the CPU explicitly. `--turbo` scores photos in [turbo mode](#turbo-mode) unless a request asks otherwise. `--weights` and `--video-weights` select model files. The server refuses to start with experimental weights whose spec lists an `uncalibrated` profile (written by `tintype interpolate`): their scores are not at any false-positive target, and a response has nowhere to say so. `tintype detect` scores such weights with a warning on standard error. The [distribution guide](DISTRIBUTION.md) covers containers, model discovery, and the licences for bundled and separately supplied weights.

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

Every scored response (`/v1/detect`, `/v1/batch` and the demo) carries a [`Server-Timing`](https://www.w3.org/TR/server-timing/) header with where the server's time went, in milliseconds: reading the upload, waiting for the worker, decoding, preparing views, the network, and the total from queueing to the result. Decoding, views and the network are each added up over the threads that did them, so on several threads they can exceed the total. It holds durations only.

```http
Server-Timing: upload;dur=4.2, queue;dur=0.1, decode;dur=119.7, views;dur=32.1, network;dur=100.0, total;dur=258.3
```

For video, `profile` is `video`. The response reports `frames`, `frames_expected`, `duration_s`, `analysed_until_s`, and `complete`. `analysed_until_s` is the latest sampling target represented in the score. The decoder can use the nearest frame to each target. Both timestamps may be `null` if the information is unavailable. `complete` means the model obtained the evidence its sampling and early-exit profile required. A decodable clip with insufficient coverage receives the `inconclusive` verdict. The server samples at most the first 30 seconds of each clip.

## Turbo mode

Turbo mode scores photos with the 8-bit CPU engine (`cpu-int8`), using the
same model, cutoffs and profile. On Linux and Windows hosts with an 8-bit
dot-product instruction it gives about 1.5 to 2 times the photo throughput
of the standard CPU engine:

- ARM64;
- x86-64 with AVX-VNNI or AVX-512 VNNI.

On the held-out split its accuracy matched the standard engine
([results](runbooks/cpu-profile-int8.md#step-6-integer-kernels-and-students-on-the-frozen-split)).

Ask for it per request with the `mode` query parameter, on `/v1/detect` or
`/v1/batch`:

```sh
curl --fail-with-body -F 'file=@photo.jpg' 'http://127.0.0.1:8080/v1/detect?mode=turbo'
```

Start the server with `--turbo` to make turbo the default. A request can
then ask for `?mode=standard`.

```sh
tintype serve --turbo
```

| `mode` | Photos scored with |
| --- | --- |
| absent | The server's default: the standard engine, or turbo under `serve --turbo` |
| `turbo` | The 8-bit CPU engine |
| `standard` | The engine chosen by `--device` |

**Response.** A photo scored in turbo mode carries `"mode": "turbo"`. No
other field changes meaning, and photos scored by the standard engine have
no `mode` field.

**Videos** always use the standard engine and carry no `mode` field.

**Loading.** The 8-bit engine loads on the first turbo request, or at
startup with `--turbo`.

**Slower hosts.** On x86-64 without VNNI, and on Apple silicon with the
GPU, turbo is slower than the standard engine. The server says so on
standard error when the engine loads.

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
- **Second opinion.** For photos it is on by default in `detect`, `batch` and `serve` when `commfor-384.safetensors` is installed beside the photo weights, and off with `--no-photo-second-opinion` (a flag of each command; `serve` loads the second network once at start). Photo rows then carry `second_opinion: true` and the blended `score`; `logit`, `threshold_logit` and `views` still describe the first stage. Turbo requests and the other profiles use the first stage alone. The video second opinion is `detect` only. The margin is measured on the blended score. Disagreement is among the first stage's views, against the first stage's threshold.
- **Edit check.** `detect` and `batch` take `--edit-check <weights>`, an opt-in second network for locally edited photos (inpainting, partly regenerated regions). It is not bundled and runs on the Apple GPU engine only; `--device cpu` and `--turbo` refuse it. Every readable photo is run through it, and photo rows gain `edit_logit`, `edit_cutoff` and `edit_flagged`. When `edit_flagged` is true the `verdict` is `ai-generated` whatever `score` says; `score`, `logit` and the review fields are unchanged, so a row can have `verdict: "ai-generated"` with `score` below 0.5. `serve` does not offer it. It finds more edited photos and flags more authentic ones; see [the evidence](DATA.md#edit-check).
- **Profiles.** A band stored in a spec is in the default profile's calibrated logits, so `--strict`, `--sensitive` and `--fast` ignore it unless a flag gives one.
- **Errors.** A file that could not be scored has no verdict and no review fields.

A band is a policy fitted on calibration data once the rest of the system is fixed, and an AI-generated file sent to review counts as a miss for automatic detection. `tintype report --operating-points --review-margin …` reports coverage, review rates and recall on that basis; see the [calibration and abstention runbook](runbooks/calibration-abstention.md).

### Video coverage modes (CLI)

`tintype detect --coverage uniform|segments|shots` is an opt-in evaluation mode of the command line. The HTTP API does not offer it, and without the flag no field below appears. The mode scores every planned frame, with no early exit, and adds to a video's JSON row:

| Field | Meaning |
|---|---|
| `coverage_mode` | `uniform`: the default sampling positions over the first `--max-seconds`, as one window. `segments`: the same number of frames divided between `--segments` equal windows of the whole clip (three by default). `shots`: as `segments`, with a window boundary moved onto a hard cut that lies within a quarter of a window of it |
| `segments` | One object per window, in clip order: `name` (`early`, `middle`, `late` for three windows, `1`…`N` otherwise, `span` for `uniform`), `start_s`, `end_s`, `frames`, `frames_expected`, `logit` (mean frame logit of the window), `score` (that logit through the clip calibration), `sampled_s` (time of each scored frame, seconds from the first frame) and `frame_logits` (each frame's mean view logit, in the same order). `score` and `logit` are `null` for a window with no decoded frame |
| `cuts_s` | `shots` only: the hard cuts found, seconds from the first frame |

`verdict`, `score`, `logit` and `threshold_logit` keep their meaning: the clip's score is the mean over all scored frames through the video calibration. `frames` and `frames_expected` count the planned frames, `analysed_until_s` is the latest planned time that was scored, and `complete` is true only when every planned frame was scored, so a clip that ends early is `inconclusive` as before. Window scores use the clip calibration on a few frames each; they are for evaluation ([runbook](runbooks/partial-edits.md)) and are not calibrated as decisions. `segments` and `shots` span the whole clip whatever `--max-seconds` says, `shots` reads the clip twice, and `--coverage` cannot be combined with `--second-opinion`.

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
| Images | JPEG, PNG, WebP, GIF, BMP, TIFF. Up to 50 million pixels, 16,384 pixels per side, and a 32:1 aspect ratio. Images are scored at full resolution, never downscaled |
| Videos | Progressive MP4/MOV/M4V with one H.264 or HEVC video track in 4:2:0. Up to 2.2 million pixels per frame, 4,096 pixels per side, and a 32:1 aspect ratio. Container index up to 4 MiB, with up to 8,192 samples of up to 8 MiB each |
| Concurrent inference requests | Four across uploading, waiting, and scoring. One model worker processes jobs sequentially and batches work within each request |
| Deadlines | 20 seconds for upload and 90 seconds for queueing plus inference |

Inference errors use an `error` object with `code` and `message`. For HTTP 422 on a single upload, the body also includes the filename.

| HTTP status | Code | Meaning |
| --- | --- | --- |
| 400 | `invalid_upload` | Invalid multipart data, field, filename, or file count |
| 400 | `invalid_mode` | The `mode` query parameter is neither `standard` nor `turbo` |
| 401 | `unauthorized` | Bearer credentials missing or invalid |
| 408 | `upload_timeout` | Upload deadline exceeded |
| 413 | `upload_too_large` | Request exceeds 64 MiB |
| 422 | `unreadable_media` | Single file is unsupported, invalid, or exceeds decode limits |
| 503 | `busy` | Worker unavailable or all request slots occupied. Retry after the `Retry-After` delay with backoff |
| 504 | `inference_timeout` | Client deadline exceeded while waiting for inference |

Images are decoded from memory. Videos are written to a randomly named temporary file while they are scored. The server removes the file on normal completion. Mount a writable temporary directory in read-only deployments and size it for uploads. Uploaded contents are never sent to a model provider.

Media decoding runs in-process. When an HTTP request times out or disconnects, the worker stops taking on remaining files as it reaches the next cancellation check. Decoder and inference calls already in progress can finish. For deployments that accept external uploads, run the service with OS or container CPU and memory limits and use a process supervisor. The [distribution guide](DISTRIBUTION.md) includes a non-root container setup.

## Public demo mode

`tintype serve --demo` runs a public demo: anyone may call it without a token, and limits that cannot be turned off protect the host. It is how the [website demo](https://cheesejaguar.github.io/tintype/#try) runs; the [hosting guide](HOSTING.md) covers its deployment on Cloud Run.

```sh
tintype serve --demo --demo-origin https://example.org --bind 0.0.0.0:8080 --device cpu
```

- `--demo-origin` is required: the one origin whose pages may call the demo. Responses carry `Access-Control-Allow-Origin` for it alone, and a request that names any other `Origin` gets 403 `origin_not_allowed` before its body is read. Scripts that send no `Origin` header may call it.
- `--client-address` says where the caller's address comes from: `peer` (the TCP connection, the default) or `last-forwarded-for` (the last `X-Forwarded-For` entry). Use `last-forwarded-for` only behind a front end that appends the real address, such as Cloud Run's. Entries before the last one are the caller's own and are ignored, so they cannot be used to forge a fresh allowance.
- The demo refuses to start with `TINTYPE_API_TOKEN` set: run the token-protected API as a separate service.
- It does not load the video model, has only `GET /healthz`, `GET /health` and `POST /v1/detect`, and ignores `mode`.

### What the demo accepts

| Limit | Value |
| --- | --- |
| Files | One per request. JPEG, PNG or WebP, judged by the file's bytes, not its name. No video |
| Size | 10 MB per photo (10,000,000 bytes), and at most 50 megapixels and 16,384 pixels a side, as for the API |
| Per network, short window | 5 photos per 10 minutes. The window starts with the first photo in it |
| Per network, per day | 20 photos per day. Counts start again at midnight UTC |
| Everyone, per day | 10,000 photos per day across all visitors, the hosted demo's budget. Counts start again at midnight UTC. The website does not state this number; it explains the cap when it is reached |

A network is one IPv4 address, or one IPv6 /64. A photo counts once it is known to be a JPEG, PNG or WebP within the size cap, whether or not it can be decoded. Requests refused before that (too large, too many pixels, another type, more than one file, wrong origin, server busy) do not count. The limits are counted in memory by each server process: they start again when the process restarts, so the instance cap of the deployment is the hard ceiling on cost. The response's `name` is always `photo.jpg`, `photo.png` or `photo.webp`; the uploaded filename is not used.

### Where the caller stands

Every response to `POST /v1/detect` carries the IETF [RateLimit header fields](https://datatracker.ietf.org/doc/draft-ietf-httpapi-ratelimit-headers/), one item per limit, and the older single-limit fields for the limit closest to running out. `t` and `RateLimit-Reset` are seconds until that count starts again.

```http
RateLimit-Policy: "ip-short";q=5;w=600, "ip-daily";q=20;w=86400, "global-daily";q=10000;w=86400
RateLimit: "ip-short";r=4;t=600, "ip-daily";r=19;t=43200, "global-daily";r=19999;t=43200
RateLimit-Limit: 5
RateLimit-Remaining: 4
RateLimit-Reset: 600
Cache-Control: no-store
Access-Control-Expose-Headers: RateLimit, RateLimit-Policy, RateLimit-Limit, RateLimit-Remaining, RateLimit-Reset, Retry-After, Server-Timing
```

A request over a limit gets HTTP 429 with `Retry-After`, before its upload is read. `limit` is `ip_short`, `ip_daily` or `global_daily`; when more than one is spent, it names the one with the longest wait.

```json
{"error":{"code":"rate_limited","limit":"ip_short","message":"You have used the 5 photos allowed per 10 minutes from your network. Try again in 7 minutes, at 12:10 UTC.","retry_after_s":420,"reset_at":"2026-11-16T12:10:00Z"}}
```

| HTTP status | Code | Meaning |
| --- | --- | --- |
| 400 | `invalid_upload` | Not one multipart field named `file`, or more than one photo |
| 403 | `origin_not_allowed` | The request's `Origin` is not `--demo-origin` |
| 413 | `photo_too_large` | The photo exceeds 10 MB |
| 415 | `unsupported_media` | Not a JPEG, PNG or WebP photo |
| 422 | `photo_too_many_pixels` | More than 50 megapixels or 16,384 pixels a side. Not counted |
| 429 | `rate_limited` | A limit is spent; see `limit` and `Retry-After` |

Other errors are as for the API. Photos are decoded from memory and never written to disk. The server logs no addresses, filenames or contents: its only demo log line is an hourly count of photos scored and requests refused. `GET /healthz` adds `demo.scored` and `demo.refused`, counted since the process started.

## Measure a local deployment

Run the loopback benchmark from a source checkout after building the binary. It uses the shipped demo PNG and two one-second clips, then verifies overload and clean shutdown.

```sh
cargo build --release --locked --bin tintype
python3 scripts/bench-api.py target/release/tintype
```

On a Mac18,5 with 12 logical CPUs and 32 GiB RAM, the CPU engine with the Rust video decoder measured a median of 88 ms for one 948 KiB PNG and 617 ms for 16 copies, about 26 photos per second. The 1-second H.264 and HEVC test clips took 1.31 s and 1.36 s respectively. A batch of 14 PNGs and both clips took 3.20 s. These measurements include localhost HTTP transfer, multipart parsing, inference, and JSON serialization. The four excess requests in an eight-way 16-photo burst received HTTP 503 in under 2 ms with `Expect: 100-continue`. The other four finished in 0.62–2.45 s. Video latency will depend on resolution, codec, and frame coverage.
