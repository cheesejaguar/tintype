# Train a separate Tintype checkpoint

The released Tintype weights and their sidecars carry CC BY-NC-SA 4.0.
For a different deployment, start from an independently authorized
backbone and train with media you have permission to use for that purpose.
Keep the initializer, training, calibration and evaluation manifests
separate from those for the bundled checkpoints.

## Prepare an initializer

`tintype init-training` runs on macOS and Linux. It accepts a local F32
timm-layout ViT-S/16 backbone at 224 px, verifies its SHA-256, validates
all tensor names and shapes, and writes a checkpoint and sidecar. It
preserves the backbone and gives Tintype's single-logit binary classifier
a zero-initialized head. Its initial score is 0.5 on every readable image.

The example uses the [timm DINO ViT-S/16 checkpoint](https://huggingface.co/timm/vit_small_patch16_224.dino/tree/d1fd955d9d1437db8b109171a4c52d6cd301a5ad).
Its model card labels the weights Apache-2.0 and identifies ImageNet-1k
pretraining. Retain the upstream notices if you distribute a derivative.
Using this initializer carries forward that pretraining history.

```sh
curl --fail --location --output dino-vit-small.safetensors \
  https://huggingface.co/timm/vit_small_patch16_224.dino/resolve/d1fd955d9d1437db8b109171a4c52d6cd301a5ad/model.safetensors

tintype init-training --backbone dino-vit-small.safetensors --out runs/init.safetensors
```

For your own compatible backbone, pass its independently recorded digest
with `--sha256`. A classification head in the input is replaced. The
output checkpoint and sidecar must use new paths.

## Fine-tune and calibrate

`tintype train` runs on the Apple GPU. A manifest is a tab-separated file
with `path`, `label`, `generator`, and `dataset` columns. Paths are relative
to the manifest. Use label `0` for authentic media and `1` for generated
media. Supply rights and source records alongside the manifests. A pilot
can start with item-verified CC0 photos and locally generated images from
explicitly authorized model versions. Record the source file hashes,
rights records, generator revisions, seeds and parent image or clip
identifiers. Preserve each group's source and derivatives in a single
split.

```sh
tintype train --init runs/init.safetensors --manifest my-photo-train.tsv --out runs/photo
tintype score --weights runs/photo/last.safetensors --manifest my-photo-calibration.tsv \
  --out runs/photo-calibration.tsv --all-views
tintype calibrate --weights runs/photo/last.safetensors \
  --scores runs/photo-calibration.tsv --profile photo --target-fpr 0.01
```

Training draws both classes equally often and, within a class, a generator
with weight `count^0.5`. To see what that means for your manifests, and to
change it, without training:

```sh
tintype sampling-shares --manifest my-photo-train.tsv --manifest more.tsv
tintype train ... --manifest-weight more=0.5 --family-weight gan=2 --print-sampling
```

`--manifest-weight` and `--family-weight` multiply the sampling mass of a
manifest's or a generator family's items (the family is manifest column 8,
or else `--family-map`, or else the generator name). `--replay-manifest`
with `--replay-share` takes a fixed share of the draws from older data
during a refresh. With none of these flags the sampler draws exactly what
it drew before they existed. The
[source and family runbook](runbooks/source-family-matching.md) has the
tools that audit a training set's files, group manifests by family and
source, and build a replay manifest.

The training command exports a checkpoint and sidecar. Keep an independent
test set away from training and threshold selection. The repository's
`finalize` commands are recipes for the bundled training mix and use its
dataset lists. Use `score`, `score-video`, and `calibrate` directly with your
own files for a separate model. Set video calibration from a separately
trained video checkpoint and a held-out video manifest.

To serve separately trained checkpoints, mount them into the code-only
container or pass both paths explicitly to the binary.

```sh
tintype serve --weights runs/photo/last.safetensors \
  --video-weights runs/video/last.safetensors
```

The [bundled checkpoint license](../models/LICENSE.md) and [data records](DATA.md)
document the original training sources and their noncommercial restrictions.
