"""A minimal client for `tintype serve`, shared by the parity and speed scripts here.

`Server(binary, port, cache)` starts `binary serve --bind 127.0.0.1:port` with
the shipped models and its own GPU cache directory, and waits until /healthz
answers 200. `score(paths)` posts them to /v1/batch, at most 16 to a request,
in order, and returns one result per path in that order, plus the seconds
spent in requests. Each upload's filename is its index in the request and its
extension, so names never collide.
"""
import json
import os
import subprocess
import time
import urllib.error
import urllib.request

MODELS = '/Users/aaron/Code/veracity-scratch/wt-pr55/models'
PER_REQUEST = 16


def multipart(files):
    boundary = 'tintype-parity-boundary-7f3a9c'
    out = []
    for name, data in files:
        out.append(f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="{name}"\r\nContent-Type: application/octet-stream\r\n\r\n'.encode())
        out.append(data)
        out.append(b'\r\n')
    out.append(f'--{boundary}--\r\n'.encode())
    return b''.join(out), f'multipart/form-data; boundary={boundary}'


class Server:
    def __init__(self, binary, port, cache, log):
        os.makedirs(cache, exist_ok=True)
        env = {**os.environ, 'TINTYPE_MODEL_DIR': MODELS, 'TINTYPE_GPU_CACHE': cache}
        env.pop('TINTYPE_API_TOKEN', None)
        self.url = f'http://127.0.0.1:{port}'
        self.proc = subprocess.Popen([binary, 'serve', '--bind', f'127.0.0.1:{port}'], env=env, stdout=log, stderr=log)
        deadline = time.monotonic() + 300
        while True:
            if self.proc.poll() is not None:
                raise SystemExit(f'{binary} serve exited with {self.proc.returncode}')
            try:
                with urllib.request.urlopen(self.url + '/healthz', timeout=5) as r:
                    if r.status == 200:
                        break
            except (urllib.error.URLError, ConnectionError, TimeoutError):
                pass
            if time.monotonic() > deadline:
                raise SystemExit(f'{binary} serve did not become ready')
            time.sleep(0.5)

    def score(self, paths):
        results, spent = [], 0.0
        for c in range(0, len(paths), PER_REQUEST):
            part = paths[c:c + PER_REQUEST]
            files = [(f'{k:02d}{os.path.splitext(p)[1].lower()}', open(p, 'rb').read()) for k, p in enumerate(part)]
            body, ctype = multipart(files)
            req = urllib.request.Request(self.url + '/v1/batch', data=body, headers={'Content-Type': ctype}, method='POST')
            t0 = time.monotonic()
            with urllib.request.urlopen(req, timeout=600) as r:
                answer = json.loads(r.read())
            spent += time.monotonic() - t0
            got = answer['results']
            assert len(got) == len(part), (len(got), len(part))
            results.extend(got)
        return results, spent

    def close(self):
        self.proc.terminate()
        try:
            self.proc.wait(timeout=30)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.wait()
