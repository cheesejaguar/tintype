# Distribution

## Prebuilt CLI

```sh
curl -fsSL https://cheesejaguar.github.io/tintype/install.sh | sh
tintype demo
tintype detect --json photo.jpg
tintype batch --json ./media > detections.jsonl
```

The installer selects Apple silicon macOS or glibc Linux (x86-64 or ARM64).
The release builds on macOS 15 and Ubuntu 22.04 runners.
It downloads the versioned executable and model archives, checks both
against the release's SHA-256 manifest, verifies the model files against
the digests compiled into the executable, and links the new binary into
`~/.local/bin`. It runs without administrator privileges. For build
provenance, verify a downloaded archive with
`gh attestation verify tintype-macos-arm64.tar.gz --repo cheesejaguar/tintype`.
Download and inspect [the installer](https://cheesejaguar.github.io/tintype/install.sh)
to run it as a local script.

Set `TINTYPE_VERSION` to install a specific version and
`TINTYPE_INSTALL_DIR` to choose a different absolute executable directory.
The following installs the executable alone.

```sh
curl -fsSL https://cheesejaguar.github.io/tintype/install.sh | sh -s -- --no-models
```

The installer retains previous versions under `~/.local/bin/.tintype/`.
Remove the `tintype` symlink and that directory to uninstall, substituting
your installation path when customized.

Models are located from `--weights`, then `TINTYPE_MODEL_DIR`, then beside
the resolved executable, then `../share/tintype/models` from that
executable. Source checkouts can also use the current directory's
`models/tintype.safetensors`. Put each checkpoint's matching JSON spec in
the same directory. Use `--video-weights` to select a different video
checkpoint. The [code and model licences](../models/LICENSE.md) cover
different parts of the distribution. The bundled checkpoints are
noncommercial. Commercial deployments require weights licensed for that use.

## Containers

Build the bundled image for noncommercial use or build the code-only image
and mount your own checkpoints.

```sh
docker build --target bundled -t tintype:bundled .
docker build --target code-only -t tintype:code .
export TINTYPE_API_TOKEN="$(openssl rand -hex 32)"
docker run --rm --init -e TINTYPE_API_TOKEN -p 127.0.0.1:8080:8080 tintype:bundled
docker run --rm --init -e TINTYPE_API_TOKEN -p 127.0.0.1:8080:8080 \
  -v "$PWD/my-models:/models:ro" tintype:code
```

The container runs as UID and GID 65532. Mounted weights and their sidecars
must be readable by that user. See the [HTTP API](API.md) for upload limits,
health checks, and server options.

## Release workflow

The tag must match the version reported by `tintype --version`. The
[release workflow](../.github/workflows/release.yml) tests the macOS ARM64,
Linux x86-64, and Linux ARM64 binaries on their native runners. It then
packages the binaries and a separate shared model archive into a draft
GitHub Release. Publishing the draft makes that version available to the
one-line installer. The `releases/latest` redirect selects the stable version.

The [Pages workflow](../.github/workflows/pages.yml) deploys the static
site and the checked-in installer once a stable release carries all installer
assets. Publishing the draft triggers the first deployment. In repository Settings → Pages, set
the source to **GitHub Actions**. After deployment, verify the site and
the exact public install command from a clean directory.
