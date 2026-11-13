#!/usr/bin/env python3
"""Measure a local Tintype API with the shipped demo image and test clips."""

import argparse
import concurrent.futures
import http.client
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import tempfile
import threading
import time


ROOT = Path(__file__).resolve().parents[1]
BOUNDARY = "tintype-api-benchmark"
TOKEN = "tintype-local-benchmark"


def multipart(files):
    fields = []
    for name, data in files:
        fields.extend((
            f'--{BOUNDARY}\r\nContent-Disposition: form-data; name="file"; filename="{name}"\r\n\r\n'.encode(),
            data,
            b"\r\n",
        ))
    return b"".join(fields) + f"--{BOUNDARY}--\r\n".encode()


def request(port, method, path, body=None, *, auth=True, expect=False, headers=None):
    request_headers = dict(headers or {})
    if auth:
        request_headers["Authorization"] = "Bearer " + TOKEN
    if body is not None:
        request_headers.setdefault("Content-Type", "multipart/form-data; boundary=" + BOUNDARY)
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=105)
    try:
        if expect:
            connection.putrequest(method, path)
            connection.putheader("Expect", "100-continue")
            connection.putheader("Content-Length", str(len(body)))
            for key, value in request_headers.items():
                connection.putheader(key, value)
            connection.endheaders()
            # HTTPResponse normally hides 100. Read it using the same buffered
            # stream that will read the final response.
            response = http.client.HTTPResponse(connection.sock, method=method)
            _, status, _ = response._read_status()
            if status != 100:
                response.close()
                return status, None
            for line in response.fp:
                if line == b"\r\n":
                    break
            else:
                raise RuntimeError("truncated 100 Continue response")
            connection.send(body)
            response.begin()
        else:
            connection.request(method, path, body, request_headers)
            response = connection.getresponse()
        raw = response.read()
        return response.status, json.loads(raw) if raw else None
    finally:
        connection.close()


def benchmark(port, repeats):
    photo = ("demo.png", (ROOT / "site/assets/demo.png").read_bytes())
    h264 = ("h264.mp4", (ROOT / "tests/fixtures/video.mp4").read_bytes())
    hevc = ("hevc.mp4", (ROOT / "tests/fixtures/hevc.mp4").read_bytes())
    invalid = multipart([("invalid.png", b"invalid")])
    assert request(port, "POST", "/v1/detect", invalid, auth=False)[0] == 401
    assert request(port, "POST", "/v1/detect", invalid)[0] == 422
    status, result = request(
        port, "POST", "/v1/batch", b"", headers={"Content-Length": str(64 * 1024 * 1024 + 1)}
    )
    assert status == 413 and result["error"]["code"] == "upload_too_large", (status, result)

    cases = [
        ("photo", [photo]),
        ("photos16", [photo] * 16),
        ("h264", [h264]),
        ("hevc", [hevc]),
        ("mixed16", [photo] * 14 + [h264, hevc]),
    ]
    for name, files in cases:
        path = "/v1/detect" if len(files) == 1 else "/v1/batch"
        body = multipart(files)
        timings = []
        for _ in range(repeats):
            start = time.perf_counter()
            status, result = request(port, "POST", path, body)
            timings.append(time.perf_counter() - start)
            assert status == 200, (status, result)
            rows = result["results"] if len(files) > 1 else [result]
            assert [row["name"] for row in rows] == [name for name, _ in files], result
            assert all(isinstance(row.get("score"), (int, float)) for row in rows), result
        median = statistics.median(timings)
        print(
            f"{name}: n={repeats}, {len(body)} bytes, median={median * 1000:.1f} ms, "
            f"max={max(timings) * 1000:.1f} ms, {len(files) / median:.1f} files/s",
            flush=True,
        )

    body = multipart([photo] * 16)
    gate = threading.Barrier(8)

    def send(_):
        gate.wait(timeout=10)
        start = time.perf_counter()
        status, _ = request(port, "POST", "/v1/batch", body, expect=True)
        return status, round((time.perf_counter() - start) * 1000)

    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        outcomes = list(pool.map(send, range(8)))
    print(f"eight concurrent 16-photo batches (HTTP status, ms): {outcomes}", flush=True)
    statuses = {status for status, _ in outcomes}
    assert statuses == {200, 503}, outcomes


def wait_ready(server, log):
    end = time.monotonic() + 30
    while time.monotonic() < end:
        with open(log, encoding="utf-8", errors="replace") as startup:
            match = re.search(r"Tintype HTTP listening on 127\.0\.0\.1:(\d+)", startup.read())
        if match:
            port = int(match[1])
            status, result = request(port, "GET", "/healthz", auth=False)
            assert status == 200 and result["status"] == "ok", (status, result)
            return port
        if server.poll() is not None:
            break
        time.sleep(0.025)
    raise RuntimeError(f"Tintype failed to become ready:\n{Path(log).read_text(errors='replace')}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", nargs="?", type=Path, default=ROOT / "target/release/tintype")
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("--device", choices=("auto", "cpu", "gpu", "gpu-f32"), default="cpu")
    args = parser.parse_args()
    if args.repeats < 1:
        parser.error("--repeats must be positive")
    env = dict(os.environ, TINTYPE_API_TOKEN=TOKEN, TINTYPE_DECODER="rust")
    env.pop("TINTYPE_MODEL_DIR", None)
    command = [args.binary.resolve(), "serve", "--bind", "127.0.0.1:0", "--device", args.device,
               "--weights", ROOT / "models/tintype.safetensors"]
    with tempfile.TemporaryDirectory(prefix="tintype-api-bench-") as directory:
        logfile = Path(directory) / "server.log"
        start = time.perf_counter()
        with logfile.open("w") as log, subprocess.Popen(command, cwd=ROOT, env=env, stderr=log) as server:
            try:
                port = wait_ready(server, logfile)
                print(f"startup to readiness: {(time.perf_counter() - start) * 1000:.1f} ms")
                benchmark(port, args.repeats)
            finally:
                server.terminate()
                try:
                    status = server.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait()
                    raise RuntimeError("Tintype did not exit within 15 seconds of SIGTERM")
            assert status == 0, logfile.read_text(errors="replace")


if __name__ == "__main__":
    main()
