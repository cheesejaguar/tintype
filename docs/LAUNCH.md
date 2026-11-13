# Tintype launch

## Release page and demo

Use [Tintype’s GitHub Pages site](https://cheesejaguar.github.io/tintype/) as the single destination for install, the sample, and the benchmarks. The site gives visitors the exact command to copy, a visible link to inspect the installer, and a link to the release assets. The same executable offers three entry points for a photo or video, a folder, and a self-hosted service.

```sh
curl -fsSL https://cheesejaguar.github.io/tintype/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
tintype demo --device cpu
tintype demo --device cpu --sensitive
```

The site’s image is the sample bundled in the executable. It was generated with OpenAI ImageGen for Tintype and resized to 880 × 660 without cropping. We dedicate this original fixture to the public domain to the extent possible under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). The SHA-256 of the published bytes is `fd9ae6321d27b602d6f486074d6b93c29a7a37352db29e949f377bcc590d20d9`. The generation prompt was:

> Generate a single original realistic editorial photograph, horizontal 4:3 composition, no text, no logos, no typography, no overlay, no photo border, no image split. A small isolated white concrete waystation with one striking dark orange-red door and a chimney, set against a windswept Icelandic black volcanic landscape. Low dark green moss and an asphalt track fading toward the left distance. A cool slate-gray clouded sky, very subtle analog film grain, plausible gray daylight, weathered concrete and volcanic rock. A sober architectural photograph with the waystation just to the right of center, prominently readable at a 450px crop, a wide open background and no visible people. Photoreal, grounded, rich material detail. Natural muted color, no film artifacts beyond the subtle grain. This is explicitly a synthetic demo fixture for an AI image detector and needs no words.

With the shipped photo checkpoint on CPU, the default profile returns `likely-real` at a score of 0.298 and the sensitive profile returns `ai-generated` at 0.913. Both output rows report `fixture_label: "ai-generated"` with `--json`. Use this example to explain what the profiles do. Sensitive mode can recover missed detections and can also flag more real images. The score summarizes model evidence at a profile’s operating point.

For agent users, install the CLI first and add the [Tintype Agent Skill](../skills/tintype/SKILL.md).

```sh
npx skills add cheesejaguar/tintype --skill tintype
```

Try “Use Tintype to screen `./incoming` for signs of AI generation and summarize what should be reviewed.” The skill guides the agent through local scoring and explains what the output can support.

## Rollout references

- REA puts a short setup command beside concrete showcases and links to the guide for each workflow. Tintype uses the same progression from one command to a local run, then to the CLI and API docs. [REA repository](https://github.com/morluto/rea)
- The Opus 5.5 edition of `/brag` puts a specific artifact first, gives it a one-line install, and links to examples on GitHub Pages. Tintype’s shareable artifact is the exact sample and its two measured verdicts, with runnable commands alongside. [Brag repository](https://github.com/latent-spaces/brag) · [Brag launch site](https://latent-spaces.github.io/brag/)
- Impeccable gives people a copyable install command and shows the work the tool performs. Tintype’s landing page keeps the one-line install visible and shows the scoring result on the downloadable file. [Impeccable site](https://impeccable.style/#install)

## Share copy

**GitHub Release**

> Tintype screens photos and videos for AI generation on your machine or server. The Rust CLI handles individual files and folders, and the self-hosted HTTP API accepts single files and batches. Start with `tintype demo` to reproduce the example from the launch page.
>
> At a matched 1% false-positive rate on a preregistered 23,052-image photo batch, Tintype reached 88.5% recall. Community Forensics reached 63.1% and SwinV2 reached 33.8%. With its shipped settings, Tintype found 89.7% of the generated photos and flagged 130 of 10,747 real photos. The benchmark has the per-source breakdown and the video comparison.
>
> The code is MIT or Apache-2.0. The bundled checkpoints are CC BY-NC-SA 4.0. Commercial deployments require appropriately licensed checkpoints.

**Show HN title**

> Show HN: Tintype, a Rust detector for AI-generated photos and video

**Show HN body**

> I built Tintype to screen photos and videos locally and to run the same models in bulk on a server. There’s a one-line install for Apple silicon macOS and glibc Linux, and a downloadable sample you can score without uploading your media.
>
> One example from the demo surprised me. The default profile misses the known generated image. Sensitive mode flags the same bytes, with a higher false-positive cost. The landing page gives both commands and the full benchmark for checking the tradeoff by source.
>
> Tintype’s code is MIT or Apache-2.0 and the shipped weights are noncommercial. I’d like to see reproducible reports on where it struggles, especially on particular generators, edits, and encodes.

**Short post**

> Tintype screens AI photos and video locally. On 23,052 blind test images, it reached 88.5% recall at 1% false positives. Community Forensics reached 63.1%. Copy the install command and run the demo to reproduce a miss and see how the sensitive profile responds.
>
> https://cheesejaguar.github.io/tintype/

## Release order

1. Build and test the release artifacts on their target platforms. Publish the verified release and confirm the Pages deployment is serving that release’s installer. [Distribution](DISTRIBUTION.md)
2. Copy the exact installer command from the deployed page and run it on a fresh machine. Reproduce the two demo results and check the download, quickstart, API, and benchmark links.
3. Use the Pages URL for the GitHub Release and any launch posts. The page includes a [1200 × 630 share image](../site/assets/share.jpg). When recording a clip, show the install command, the two demo runs, and the benchmark table. Close on the URL so viewers can repeat the result.
4. Ask for misses with the Tintype version, profile, source category, and expected label. Share media only with permission. Link confirmed reports from the benchmark or release notes.

## Benchmark copy

The default photo result and reference rows are measured at their respective shipped thresholds. The same-rate photo and video comparisons set each threshold retrospectively to a 1% false-positive rate on its blind batch. Use the shipped results for what a new installation does and the matched-rate results for model comparisons at a shared false-positive budget. [Benchmark](BENCHMARK.md) · [Blind photo results](results/confirm7.md) · [Blind video results](results/confirm4.md)

On the same 1% budget, the JPEG-only slice of batch 7 So-Fake-OOD social-media images reaches 77.0% recall for Tintype, 46.7% for SwinV2, and 28.8% for Community Forensics. The post-hoc slice contains 2,281 files and checks how the lead changes when format is held fixed. [Same-format analysis](EVIDENCE.md#social-media-jpegs)

The end-to-end photo speed range of 530–563 images per second comes from two runs on an Apple M6 GPU on every eleventh image in batch 7, for a total of 2,096 images. Other hardware, workloads, and profile choices can change throughput. [Raw timing log](results/confirm7_second_opinion.log)
