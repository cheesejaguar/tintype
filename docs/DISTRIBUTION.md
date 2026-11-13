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
See [training separate checkpoints](TRAINING.md) for the native initializer
and calibration workflow.

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

### Published images

The [container workflow](../.github/workflows/containers.yml) publishes
versioned Linux AMD64 and ARM64 images to GitHub Container Registry. The
code-only image takes your models from a read-only mount. The bundled image
includes the noncommercial Tintype checkpoints. Both v0.1.0 images support anonymous pulls.

```sh
export TINTYPE_API_TOKEN="$(openssl rand -hex 32)"
docker run --rm --init -e TINTYPE_API_TOKEN -p 127.0.0.1:8080:8080 \
  -v "$PWD/my-models:/models:ro" ghcr.io/cheesejaguar/tintype:v0.1.0
docker run --rm --init -e TINTYPE_API_TOKEN -p 127.0.0.1:8080:8080 \
  ghcr.io/cheesejaguar/tintype-bundled:v0.1.0
```

The workflow summary records both index digests for pinning deployments.
To check image provenance, run
`gh attestation verify oci://ghcr.io/cheesejaguar/tintype@sha256:DIGEST --repo cheesejaguar/tintype`
with the digest from that summary.

## Release workflow

1. Merge the version bump in `Cargo.toml` and `Cargo.lock` after native CI
   passes. Push a `vX.Y.Z` tag on the merged commit. The tag must match the
   version reported by `tintype --version`.
2. Wait for the [release workflow](../.github/workflows/release.yml) to pass
   on macOS ARM64, Linux x86-64, and Linux ARM64 and create the draft.
   Check for `tintype-macos-arm64.tar.gz`, `tintype-linux-x86_64.tar.gz`,
   `tintype-linux-arm64.tar.gz`, `tintype-models.tar.gz`, and `SHA256SUMS`.
   Verify the checksums and the build attestations for the tagged ref.
   The workflow checks the packaged model digests against the ones compiled
   into the executable.
3. Review the release notes and publish the draft as a stable release. With
   a maintainer's GitHub CLI session, run
   `gh release edit "$tag" --draft=false --prerelease=false --latest`
   after setting `tag` to the release tag. Verify that `releases/latest`
   resolves to the published version.

The [Pages workflow](../.github/workflows/pages.yml) deploys the static
site and the checked-in installer once a stable release carries all installer
assets. Publishing the draft triggers the deployment. In repository Settings → Pages, set
the source to **GitHub Actions**. After deployment, verify the site and
the exact public install command from a clean directory.

### Publish release containers

After publishing a stable GitHub Release, dispatch the container workflow
from the default branch.

```sh
gh workflow run containers.yml -f tag=v0.1.0 -F publish=false
gh workflow run containers.yml -f tag=v0.1.0 -F publish=true
```

The first dispatch verifies the signed release archives and smoke-tests both
images on both native architectures. The second repeats those checks before
pushing the tested images, verifying their platform indexes, and attesting
the two index digests. It uses the repository's GitHub Actions token with
package write access.

Verify the published manifests and layers with an anonymous registry
client before announcing a version. If anonymous pulls fail for a new
package you intend to distribute publicly, check its visibility in Package
settings. [GitHub package settings](https://docs.github.com/en/packages/learn-github-packages/configuring-a-packages-access-control-and-visibility#configuring-visibility-of-packages-for-your-personal-account).
