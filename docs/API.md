# Self-hosted HTTP API

`tintype serve` loads the photo and video models once, then processes uploads on the host. It uses the same default photo and video profiles as `tintype detect`. For a media library already on the host, `tintype batch --json ./media` streams results without HTTP or uploads.

## Start the server

```sh
tintype serve --bind 127.0.0.1:8080
curl --fail-with-body http://127.0.0.1:8080/healthz
```

The address defaults to `127.0.0.1:8080`. `GET /healthz` reports `{"status":"ok","version":"0.1.0"}` when the model worker is ready. `HEAD /healthz` returns the same status and headers with an empty body. Health requests need no credentials.

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

For video, `profile` is `video`. The response reports `frames`, `frames_expected`, `duration_s`, `analysed_until_s`, and `complete`. `analysed_until_s` is the latest sampling target represented in the score. The decoder can use the nearest frame to each target. Both timestamps may be `null` if the information is unavailable. `complete` means the model obtained the evidence its sampling and early-exit profile required. A decodable clip with insufficient coverage receives the `inconclusive` verdict. The server samples at most the first 30 seconds of each clip.

## Detect a batch

Repeat the field `file` up to 16 times. The `results` array follows upload order, including duplicate filenames and per-file errors. A structurally valid batch returns HTTP 200 even if individual files fail. Use the array index to associate results with inputs, and inspect each row before using a score.

```sh
curl --fail-with-body \
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

Media decoding runs in-process. A response deadline bounds the client's wait, and work already executing can continue. For deployments that accept external uploads, run the service with OS or container CPU and memory limits and use a process supervisor. The [distribution guide](DISTRIBUTION.md) includes a non-root container setup.
