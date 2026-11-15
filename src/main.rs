use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tintype::detector::{Detector, Device, ModelSpec};
use tintype::ingest;

#[cfg(feature = "provenance")]
mod provenance;
mod scan;
use scan::ScanOptions;

#[derive(Parser)]
#[command(name = "tintype", version, about = "AI-generated image and video detection")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan image and video files.
    Detect {
        /// Image or video files.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        options: ScanOptions,
        /// WaveRep weights for an optional video second opinion (macOS only).
        #[arg(long)]
        second_opinion: Option<PathBuf>,
        /// Community Forensics 384 px weights for an optional photo second opinion.
        #[arg(long, conflicts_with_all = ["fast", "sensitive", "strict"])]
        photo_second_opinion: Option<PathBuf>,
    },
    /// Recursively scan directories with bounded memory and stream results.
    Batch {
        /// Files or directory roots to scan, in the supplied order.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        options: ScanOptions,
    },
    /// Score the included labeled sample offline.
    Demo(scan::DemoOptions),
    /// Verify embedded C2PA Content Credentials offline.
    #[cfg(feature = "provenance")]
    Provenance(provenance::ProvenanceArgs),
    /// Start the versioned HTTP API for photos and video.
    Serve(tintype::server::ServeArgs),
    /// Score every image in a manifest: `path label generator dataset prob logits...`.
    Score {
        #[arg(long)]
        weights: PathBuf,
        /// Run a reference detector instead: `commfor`, `swinv2`.
        #[arg(long)]
        reference: Option<String>,
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 64)]
        batch: usize,
        #[arg(long, default_value = "auto")]
        device: String,
        /// Override the model's views, e.g. `fit,native,rich4`.
        #[arg(long)]
        views: Option<String>,
        /// Use the two-view fast profile.
        #[arg(long)]
        fast: bool,
        /// Use the sensitive profile.
        #[arg(long, conflicts_with = "fast")]
        sensitive: bool,
        /// Use the strict profile.
        #[arg(long, conflicts_with_all = ["fast", "sensitive"])]
        strict: bool,
        /// Look at every view of every image (no early exit).
        #[arg(long)]
        all_views: bool,
        /// Community Forensics weights: give the photos inside the model's band a second opinion (see `detect --photo-second-opinion`).
        #[arg(long, conflicts_with_all = ["reference", "views", "fast", "sensitive", "strict"])]
        photo_second_opinion: Option<PathBuf>,
    },
    /// Score every clip in a manifest (frame sampling + aggregation).
    ScoreVideo {
        #[arg(long)]
        weights: PathBuf,
        /// Run a reference detector instead: `waverep`.
        #[arg(long)]
        reference: Option<String>,
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
        /// Frames analysed per clip (default: the model's video profile).
        #[arg(long)]
        frames: Option<usize>,
        /// Override the views, e.g. `fit,rich5`.
        #[arg(long)]
        views: Option<String>,
        /// WaveRep weights: give uncertain clips a second opinion (see `detect --second-opinion`).
        #[arg(long)]
        second_opinion: Option<PathBuf>,
        /// Look at every frame of every clip (no early exit).
        #[arg(long)]
        all_frames: bool,
        #[arg(long, default_value_t = 64)]
        batch: usize,
        #[arg(long, default_value = "auto")]
        device: String,
    },
    /// Compare score files: `--system name=scores.tsv[@threshold]` (repeatable).
    Report {
        #[arg(long = "system", required = true)]
        systems: Vec<String>,
        #[arg(long)]
        per_generator: bool,
        /// Print markdown tables with bootstrap confidence intervals.
        #[arg(long)]
        markdown: bool,
        /// With --markdown: also print a pooled table under this name.
        #[arg(long)]
        pooled: Option<String>,
        /// Name of one of the systems: print the others operated at its false-positive count on each set.
        #[arg(long)]
        at_rate_of: Option<String>,
        /// Print the fixed-budget tables instead: recall at each `--fpr`,
        /// the actual rates at each system's own cutoff, precision at
        /// `--prevalence`, and the same per `--slice-by` slice.
        #[arg(long, conflicts_with_all = ["markdown", "at_rate_of", "per_generator"])]
        operating_points: bool,
        /// With --operating-points: empirical false-positive rates to operate at.
        #[arg(long, value_delimiter = ',', default_value = "0.001,0.01")]
        fpr: Vec<f64>,
        /// With --operating-points: the predeclared share of AI-generated items among inputs.
        #[arg(long, default_value_t = 0.01)]
        prevalence: f64,
        /// With --operating-points: break results down by dataset, generator, source, family or transform.
        #[arg(long, value_delimiter = ',', default_value = "dataset")]
        slice_by: Vec<String>,
        /// With --operating-points: bootstrap whole parent, scene, source or family groups instead of items.
        #[arg(long)]
        cluster_by: Option<String>,
        /// With --operating-points: manifests whose grouping columns (5 and up) describe the scored files.
        #[arg(long)]
        groups: Vec<PathBuf>,
        /// With --operating-points: smallest slice that counts towards the worst slice.
        #[arg(long, default_value_t = 100)]
        min_slice: usize,
        /// With --operating-points: a false-positive budget. After each
        /// slice table, print every authentic stratum's rate at the frozen
        /// cutoff with Clopper–Pearson bounds, judged against the budget
        /// (how a cutoff frozen on other items transfers). With
        /// --cluster-by, clusters are counted instead of items.
        #[arg(long)]
        transfer_budget: Option<f64>,
        /// With --operating-points: account for a review band of this
        /// half-width in calibrated logits around each frozen cutoff
        /// (several, comma-separated, give one row each): coverage, review
        /// rate per class, and recall with reviewed AI items as misses.
        #[arg(long, value_delimiter = ',')]
        review_margin: Vec<f32>,
        /// With --review-margin: also review when this share of a row's
        /// view or frame logits fall on the other side of
        /// --review-threshold-logit.
        #[arg(long, requires = "review_threshold_logit")]
        review_disagreement: Option<f32>,
        /// The raw threshold on the mean logit of the profile that wrote the
        /// score files (`threshold_logit` in `detect --json`).
        #[arg(long, allow_hyphen_values = true)]
        review_threshold_logit: Option<f32>,
        /// With --operating-points: review decisions recorded elsewhere, as
        /// `detect --json` rows or `path<TAB>0|1[<TAB>reason]` rows.
        #[arg(long)]
        review_flags: Option<PathBuf>,
    },
    /// Score a whole evaluation suite and print the comparison tables.
    Evaluate {
        /// benchmarks, or confirm1 to confirm7.
        suite: String,
        #[arg(long, default_value = "models/tintype.safetensors")]
        weights: PathBuf,
        /// Video model weights (default: `<weights>-video.safetensors` beside the photo weights, if present).
        #[arg(long)]
        video_weights: Option<PathBuf>,
        /// Suffix of the score files written for Tintype.
        #[arg(long, default_value = "tintype")]
        tag: String,
        #[arg(long, default_value = "runs/scores")]
        scores: PathBuf,
        /// Also run the three reference detectors (weights from `tintype data baselines`).
        #[arg(long)]
        baselines: bool,
        /// Keep reference score files that already exist.
        #[arg(long)]
        reuse: bool,
        #[arg(long, default_value = "auto")]
        device: String,
    },
    /// Freeze a checkpoint: write its spec and calibrate photo, fast and video profiles.
    Finalize {
        checkpoint: PathBuf,
        #[arg(long, default_value = "models/tintype.safetensors")]
        weights: PathBuf,
        #[arg(long, default_value = "runs/scores")]
        scores: PathBuf,
        /// False-positive target for photos, averaged over real sources.
        #[arg(long, default_value_t = 0.015)]
        photo_fpr: f64,
        /// False-positive target for the strict and fast photo profiles.
        #[arg(long, default_value_t = 0.005)]
        strict_fpr: f64,
        /// False-positive target for the sensitive photo profile.
        #[arg(long, default_value_t = 0.05)]
        sensitive_fpr: f64,
        /// False-positive target for video, averaged over real sources.
        #[arg(long, default_value_t = 0.01)]
        video_fpr: f64,
        /// Training manifests added after phase 3: their validation split joins the calibration data.
        #[arg(long)]
        also_trained_on: Vec<PathBuf>,
        /// Evaluation manifests scored since phase 3: their items join the calibration data.
        #[arg(long)]
        also_scored_on: Vec<PathBuf>,
        #[arg(long, default_value = "auto")]
        device: String,
    },
    /// Freeze a checkpoint as the video model: write its spec and calibrate the video threshold.
    FinalizeVideo {
        checkpoint: PathBuf,
        #[arg(long, default_value = "models/tintype-video.safetensors")]
        weights: PathBuf,
        #[arg(long, default_value = "runs/scores")]
        scores: PathBuf,
        /// False-positive target, averaged over real sources.
        #[arg(long, default_value_t = 0.01)]
        video_fpr: f64,
        /// Second-opinion stage as `lo,hi,threshold` in first-stage logits (see `detect --second-opinion`).
        #[arg(long, allow_hyphen_values = true)]
        second_opinion: Option<String>,
        #[arg(long, default_value = "auto")]
        device: String,
    },
    /// Store the band of the photo second opinion in a checkpoint's spec
    /// (see `detect --photo-second-opinion`), or remove it with `--band none`.
    SetPhotoSecondOpinion {
        #[arg(long, default_value = "models/tintype.safetensors")]
        weights: PathBuf,
        /// `lo,hi,weight,threshold` in calibrated first-stage logits, or `none`.
        #[arg(long, allow_hyphen_values = true)]
        band: String,
    },
    /// Fit the decision calibration of one profile from scored held-out data.
    Calibrate {
        #[arg(long)]
        weights: PathBuf,
        /// Score files produced by `score` / `score-video` with these weights.
        #[arg(long, required = true)]
        scores: Vec<PathBuf>,
        /// False-positive rate on real items that probability 0.5 should correspond to.
        #[arg(long, default_value_t = 0.005)]
        target_fpr: f64,
        /// photo (default), strict, fast, sensitive or video.
        #[arg(long, default_value = "photo")]
        profile: String,
        /// Also store the standard early-exit schedule in the checkpoint's spec.
        #[arg(long)]
        early_exit: bool,
        /// Threshold rule: pooled, strictest or balanced (over real
        /// sources), or bound: the loosest threshold whose one-sided
        /// Clopper–Pearson upper bound on the false-positive rate, over
        /// independent real items, is within the target.
        #[arg(long, default_value = "balanced")]
        rule: String,
        /// Smallest real source that counts for the strictest / balanced rules.
        #[arg(long, default_value_t = 100)]
        min_source: usize,
        /// Confidence of the Clopper–Pearson bound: fitted to by `--rule
        /// bound`, printed for every rule.
        #[arg(long, default_value_t = 0.95)]
        confidence: f64,
        /// Manifests whose grouping columns (5 and up) describe the scored
        /// files: real items that share a `parent_id` then count once
        /// towards the bound.
        #[arg(long)]
        groups: Vec<PathBuf>,
        /// With --groups: the score that stands for a parent with
        /// derivatives, `max` (the highest of its versions) or `original`.
        #[arg(long, default_value = "max")]
        per_parent: String,
        /// Print, and never store, the threshold each source, transform,
        /// dataset, generator or family would get on its own. Diagnostic only.
        #[arg(long)]
        group_diagnostic: Option<String>,
    },
    /// Re-check, and optionally refit, the early-exit schedule of a
    /// checkpoint from score files that hold every view logit (`score
    /// --all-views`) or every frame logit (`score-video --all-frames`):
    /// run it after a threshold, a calibration, the views or their pooling
    /// change. Prints what the schedule does against scoring everything:
    /// verdicts that differ, pooled and per-source rates, units looked at.
    /// Works on any platform; scores nothing.
    RefitExit {
        /// The checkpoint whose spec gives the thresholds and the stored
        /// schedule. With --write it must be a copy under runs/, never
        /// under models/.
        #[arg(long)]
        weights: PathBuf,
        /// The profiles that share the schedule, comma-separated: photo,
        /// strict, sensitive; or video alone.
        #[arg(long, default_value = "photo")]
        profile: String,
        /// Use this raw threshold on the mean logit instead of the spec's
        /// (one profile only).
        #[arg(long, allow_hyphen_values = true)]
        threshold: Option<f32>,
        /// Score files to fit new margins on. Without them the stored
        /// schedule (or the standard one) is only checked.
        #[arg(long, num_args = 1..)]
        fit: Vec<PathBuf>,
        /// Score files to report on: other items than the ones fitted on.
        #[arg(long, required = true, num_args = 1..)]
        report: Vec<PathBuf>,
        /// Pooled precision and recall under early exit must stay within
        /// this of scoring everything.
        #[arg(long, default_value_t = 0.001)]
        tolerance: f64,
        /// The calibration's false-positive target of each profile, in the
        /// order of --profile, to compare each real source's rate under
        /// early exit with (e.g. 0.015,0.005,0.05).
        #[arg(long, value_delimiter = ',')]
        target_fpr: Vec<f64>,
        /// Smallest real source that counts towards the mean over sources.
        #[arg(long, default_value_t = 100)]
        min_source: usize,
        /// Fitted margins are multiples of this many raw logits.
        #[arg(long, default_value_t = 0.25)]
        step: f32,
        /// The widest margin tried.
        #[arg(long, default_value_t = 12.0)]
        max_margin: f32,
        /// Fit the margins under and over the threshold separately.
        #[arg(long)]
        asymmetric: bool,
        /// Store the fitted schedule in the spec beside --weights.
        #[arg(long, requires = "fit")]
        write: bool,
        /// Write the reported items as the first profile decides them
        /// under the schedule (`path label generator dataset prob
        /// logits...`, the logits being those looked at), for `report`.
        #[arg(long)]
        apply_out: Option<PathBuf>,
        /// The spec (.json) the photo second-opinion band was chosen
        /// under: print the band re-expressed in this checkpoint's
        /// calibration so that the same photos are routed, and store it
        /// with --write.
        #[arg(long)]
        rebase_band_from: Option<PathBuf>,
    },
    /// Time the parts of the pipeline separately: decoding, cropping, and the
    /// network alone at several batch sizes.
    Profile {
        #[arg(long)]
        weights: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
        /// Number of images to decode.
        #[arg(long, default_value_t = 1000)]
        n: usize,
        #[arg(long, default_value = "auto")]
        device: String,
        /// Batch sizes to time the network at.
        #[arg(long, value_delimiter = ',', default_value = "16,32,64,128")]
        batches: Vec<usize>,
        /// Skip the decoding and cropping measurements.
        #[arg(long)]
        model_only: bool,
    },
    /// Single-item latency: one image at a time, file to verdict. Runs on
    /// every platform with `--device cpu`.
    Latency {
        #[arg(long)]
        weights: PathBuf,
        /// Time a reference detector instead: `commfor`, `swinv2`.
        #[arg(long)]
        reference: Option<String>,
        #[arg(long)]
        manifest: PathBuf,
        /// Number of images to time (after 5 warm-up images).
        #[arg(long, default_value_t = 200)]
        n: usize,
        #[arg(long, default_value = "auto")]
        device: String,
        #[arg(long)]
        fast: bool,
        /// Look at every view of every image (no early exit).
        #[arg(long)]
        all_views: bool,
        /// Community Forensics weights: also time the photo second opinion
        /// on the images inside the model's band (see `detect
        /// --photo-second-opinion`), and print the share sent on.
        #[arg(long, conflicts_with_all = ["reference", "fast"])]
        photo_second_opinion: Option<PathBuf>,
    },
    /// Prepare an untrained ViT-S/16 checkpoint for detector training.
    InitTraining {
        /// Local timm-layout ViT-S/16 safetensors backbone.
        #[arg(long)]
        backbone: PathBuf,
        /// Expected SHA-256. Defaults to the pinned timm DINO checkpoint.
        #[arg(long, default_value = tintype::training_init::DINO_SHA256)]
        sha256: String,
        /// New safetensors file. A matching JSON sidecar is written beside it.
        #[arg(long)]
        out: PathBuf,
    },
    /// Fine-tune the detector (Apple GPU).
    Train {
        /// Starting weights (safetensors).
        #[arg(long)]
        init: PathBuf,
        /// Prefix stripped from tensor names in `--init`.
        #[arg(long, default_value = "")]
        init_prefix: String,
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 20000)]
        steps: u64,
        #[arg(long, default_value_t = 32)]
        batch: usize,
        #[arg(long, default_value_t = 5e-5)]
        lr: f32,
        #[arg(long, default_value_t = 500)]
        warmup: u64,
        #[arg(long, default_value_t = 3)]
        val_pct: u64,
        #[arg(long, default_value_t = 2000)]
        val_max: usize,
        #[arg(long, default_value_t = 1000)]
        eval_every: u64,
        #[arg(long, default_value_t = 11)]
        threads: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long, default_value_t = 0.8)]
        layer_decay: f64,
        #[arg(long, default_value_t = 0.05)]
        weight_decay: f64,
        #[arg(long, default_value_t = 0.0)]
        smoothing: f64,
        /// Generator sampling weight exponent (count^alpha).
        #[arg(long, default_value_t = 0.5)]
        sampler_alpha: f64,
        #[command(flatten)]
        aug: AugArgs,
        #[command(flatten)]
        sampling: SamplingFlags,
        /// Print the effective sampling shares of this recipe (per manifest,
        /// generator family and generator) and stop: nothing is trained or written.
        #[arg(long)]
        print_sampling: bool,
    },
    /// Print the effective sampling shares of a training recipe: how often
    /// each manifest, generator family and generator is drawn, per class.
    /// Takes the manifests and sampling flags of `train` and runs on every
    /// platform; nothing is trained or written.
    SamplingShares {
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        /// Percent of every manifest held out for validation by path hash, as in `train`.
        #[arg(long, default_value_t = 3)]
        val_pct: u64,
        /// Generator sampling weight exponent (count^alpha).
        #[arg(long, default_value_t = 0.5)]
        sampler_alpha: f64,
        #[command(flatten)]
        sampling: SamplingFlags,
        /// Rows of the family and generator tables, largest shares first (0 = all).
        #[arg(long, default_value_t = 15)]
        top: usize,
        /// With `--write-weights`: family weights that move each AI family's
        /// share of AI draws to `share^GAMMA`, renormalised (0 = equal
        /// shares, 1 = no change). Real sources are left alone.
        #[arg(long, value_name = "GAMMA", requires = "write_weights")]
        rebalance_families: Option<f64>,
        /// Where to write the weights file that `--rebalance-families` suggests.
        #[arg(long, requires = "rebalance_families")]
        write_weights: Option<PathBuf>,
    },
    /// Write the two sides of the path-hash validation split as manifests:
    /// `train.tsv` and `val.tsv` in `--out-dir`, with absolute paths. `train`
    /// holds out `--val-pct` percent of every manifest by a hash of each
    /// path as spelled on its command line, and `finalize` calibrates on
    /// that share; pass the manifests here exactly as there. For tooling
    /// that must know the sides (derivatives of held-out items do not belong
    /// in training); training on these two files would hash the absolute
    /// paths into a different split.
    HashSplit {
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        #[arg(long, default_value_t = 3)]
        val_pct: u64,
        #[arg(long)]
        out_dir: PathBuf,
    },
    /// Time the CPU side of the training loader (decode and augment) on the photos of a manifest.
    AugmentBench {
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        /// Photos to draw from the manifests, the way training draws them.
        #[arg(long, default_value_t = 512)]
        images: usize,
        /// Generator sampling weight exponent (count^alpha), as in `train`.
        #[arg(long, default_value_t = 0.5)]
        sampler_alpha: f64,
        /// Augmented views to time per photo.
        #[arg(long, default_value_t = 4)]
        rounds: usize,
        #[arg(long, default_value_t = 4)]
        threads: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[command(flatten)]
        aug: AugArgs,
    },
    /// Download and prepare a dataset plan (`tintype data list` shows them).
    Data {
        plan: String,
        /// Resolve and print the downloads without fetching anything.
        #[arg(long)]
        dry_run: bool,
        /// Resolve the downloads without fetching anything, and compare the
        /// files already on disk with their pinned SHA-256; fails if any differs.
        #[arg(long, conflicts_with = "dry_run")]
        check: bool,
        /// Parallel connections per large file.
        #[arg(long, default_value_t = 12)]
        connections: usize,
    },
    /// Extract images from parquet shards into an image store with a manifest.
    Ingest {
        /// Parquet shards.
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
        /// Dataset name recorded in the manifest.
        #[arg(long)]
        dataset: String,
        #[arg(long, default_value = "image")]
        image_col: String,
        #[arg(long)]
        label_col: Option<String>,
        #[arg(long, value_delimiter = ',', default_value = "fake,1")]
        fake_values: Vec<String>,
        #[arg(long, value_delimiter = ',', default_value = "real,0")]
        real_values: Vec<String>,
        /// Label for every row when there is no label column (0 or 1).
        #[arg(long)]
        const_label: Option<u8>,
        #[arg(long)]
        gen_col: Option<String>,
        #[arg(long)]
        real_src_col: Option<String>,
        #[arg(long)]
        const_gen: Option<String>,
        #[arg(long)]
        max_per_file: Option<usize>,
    },
    /// Index an extracted folder tree. Rules are `substring=label:generator`.
    IndexFolder {
        root: PathBuf,
        #[arg(long)]
        dataset: String,
        #[arg(long = "rule", required = true)]
        rules: Vec<String>,
        /// Index video files instead of images.
        #[arg(long)]
        video: bool,
    },
    /// Decode a frame from the middle of every clip in the manifests; list the
    /// clips that cannot be read or take longer than five seconds.
    CheckVideos {
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        /// Rewrite each manifest without the listed clips.
        #[arg(long)]
        prune: bool,
        /// Also draw a re-encoded training frame from every clip (the H.264 augmentation).
        #[arg(long)]
        recompress: bool,
        /// Instead: decode this many frames of every clip with both the system decoder and the pure-Rust reader, and report how they differ (macOS)
        #[arg(long)]
        compare_decoders: Option<usize>,
    },
    /// List test files that are byte-identical to a training file.
    Overlap {
        #[arg(long, required = true)]
        train: Vec<PathBuf>,
        #[arg(long, required = true)]
        test: Vec<PathBuf>,
    },
    /// Print the schema and first rows of a parquet shard.
    ParquetInfo {
        path: PathBuf,
        #[arg(long, default_value_t = 3)]
        rows: usize,
    },
    /// Record a prepared evaluation set: the SHA-256, size and label of every
    /// file its manifests list, and the SHA-256 of each manifest. Run from the
    /// directory that holds `data/`.
    FreezeSet {
        /// A whole evaluation suite: benchmarks, or confirm1 to confirm7.
        #[arg(long, required_unless_present = "manifest")]
        suite: Option<String>,
        /// Manifests to record (instead of, or as well as, a suite's).
        #[arg(long)]
        manifest: Vec<PathBuf>,
        /// Where to write it; a name ending in `.gz` is compressed.
        #[arg(long)]
        out: PathBuf,
        /// Paths are stored relative to this directory.
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Check a local evaluation set against a record written by `freeze-set`:
    /// every file present with the same bytes, every manifest unchanged.
    /// Exits with an error listing every difference.
    VerifySet {
        /// Records written by `freeze-set`.
        #[arg(required = true)]
        frozen: Vec<PathBuf>,
        /// The directory the paths in the records are relative to.
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Divide manifests into fit, calibrate and test sets so that related
    /// items stay together: items that share a `parent_id` or `scene_id`
    /// (manifest columns 5 and 6), and with `--together source,family`
    /// whole sources and generator families. Writes `fit.tsv`,
    /// `calibrate.tsv` and `test.tsv` to `--out-dir`. Record each with
    /// `freeze-set` and check them with `verify-split`.
    SplitGroups {
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        #[arg(long)]
        out_dir: PathBuf,
        /// Target shares of fit, calibrate and test.
        #[arg(long, default_value = "0.6,0.2,0.2")]
        fractions: String,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// What must not cross sets: any of parent, scene, source, family.
        #[arg(long, default_value = "parent,scene")]
        together: String,
        /// A source to keep out of fitting and calibration: every item of it
        /// goes to the test set (`source_id`, or else `real:<name>` for real
        /// items and the dataset for generated ones). Repeatable.
        #[arg(long)]
        hold_out_source: Vec<String>,
        /// A generator family to keep out of fitting and calibration
        /// (`generator_family`, or else the generator name). Repeatable.
        #[arg(long)]
        hold_out_family: Vec<String>,
        /// The `--out-dir` of an earlier split: groups that contain an item
        /// it placed go to the same set (for derivatives made afterwards).
        #[arg(long)]
        existing: Option<PathBuf>,
    },
    /// Fail if any group crosses the given sets: a shared path, `parent_id`
    /// or `scene_id` (or, with `--together`, source or family), a held-out
    /// source or family outside the test set, or, between records written by
    /// `freeze-set`, two files with the same bytes.
    VerifySplit {
        /// `name=file`: a manifest or a `freeze-set` record of the set
        /// called `name` (fit, calibrate, test). Repeatable, also per set.
        #[arg(long = "set", required = true)]
        sets: Vec<String>,
        #[arg(long, default_value = "parent,scene")]
        together: String,
        #[arg(long)]
        held_out_source: Vec<String>,
        #[arg(long)]
        held_out_family: Vec<String>,
    },
    /// Offline experiment: write `(1 - alpha) * A + alpha * B` for each
    /// `--alpha` (a sweep from a fine-tuned checkpoint `--a` towards its
    /// initialiser `--b`), or with `--average` a weighted average of sibling
    /// fine-tunes. The two checkpoints must agree on every tensor's name,
    /// shape and type. Each output gets an uncalibrated spec beside it;
    /// nothing is ever written into `models/`.
    Interpolate {
        /// The fine-tuned checkpoint (alpha 0). Only read.
        #[arg(long, requires_all = ["b", "alpha", "out_dir"], conflicts_with = "average")]
        a: Option<PathBuf>,
        /// Prefix removed from the tensor names of `--a`.
        #[arg(long, default_value = "")]
        a_prefix: String,
        /// The checkpoint to move towards (alpha 1), e.g. the initialiser. Only read.
        #[arg(long, requires = "a")]
        b: Option<PathBuf>,
        /// Prefix removed from the tensor names of `--b`, e.g. `vit.` for Community Forensics.
        #[arg(long, default_value = "")]
        b_prefix: String,
        /// Shares of `--b`, each from 0 to 1, e.g. `0,0.25,0.5,0.75,1`.
        #[arg(long, value_delimiter = ',')]
        alpha: Vec<f64>,
        /// Where a sweep writes `alpha-0.25.safetensors`, its spec and its record.
        #[arg(long)]
        out_dir: Option<PathBuf>,
        /// Instead of a sweep: average these sibling fine-tunes of one initialiser.
        #[arg(long, num_args = 2.., requires = "out")]
        average: Vec<PathBuf>,
        /// With --average: one weight per checkpoint, adding up to 1 (default: equal).
        #[arg(long, value_delimiter = ',', requires = "average")]
        weights: Vec<f64>,
        /// With --average: the file to write.
        #[arg(long, requires = "average")]
        out: Option<PathBuf>,
        /// Spec whose views the outputs get (default: the one beside `--a`, or beside the first `--average` checkpoint).
        #[arg(long)]
        spec: Option<PathBuf>,
        /// Replace outputs that already exist (never anything in `models/`).
        #[arg(long)]
        overwrite: bool,
    },
    /// Offline experiment: replay other ways of pooling the view logits of
    /// all-views score files (`score --all-views`, `*-views.tsv`,
    /// `calib_images.tsv`). For each pool and view order, the threshold and
    /// the early-exit margins are refitted on `--fit`, and the result is
    /// reported on each `--test`. Reads only; changes no model.
    ReplayPool {
        /// All-views score files to fit thresholds, margins and weights on (`*` in a file name matches).
        #[arg(long, required = true)]
        fit: Vec<String>,
        /// `name=scores.tsv[,more.tsv]`: all-views score files to report on, kept apart from `--fit` (repeatable; `*` matches).
        #[arg(long)]
        test: Vec<String>,
        /// The spec the files were scored with (e.g. models/tintype.json): its threshold and early exit are the shipped pool the others are compared with. Only read.
        #[arg(long)]
        spec: Option<PathBuf>,
        /// Pools to replay: mean, trimmed, median, max, topK, fitted (per-view weights fitted on `--fit`), weights:w0/w1/...
        #[arg(long, value_delimiter = ',', default_value = "mean,trimmed,median,top2,top3,max,fitted")]
        pool: Vec<String>,
        /// Orders of looking at the views: shipped, greedy (chosen on `--fit`), or indices such as 0,2,1,3,4,5 (repeatable).
        #[arg(long, default_value = "shipped")]
        order: Vec<String>,
        /// False-positive rate on real items the refitted threshold aims at (the default profile's 1.5%).
        #[arg(long, default_value_t = 0.015)]
        target_fpr: f64,
        /// Threshold rule, as for `calibrate`: balanced (over real sources) or pooled.
        #[arg(long, default_value = "balanced")]
        rule: String,
        /// Smallest real source that counts for the balanced rule.
        #[arg(long, default_value_t = 100)]
        min_source: usize,
        /// How far pooled precision and recall on `--fit` may move from all views when margins are refitted.
        #[arg(long, default_value_t = 0.001)]
        tolerance: f64,
        /// Step of the margin grid, in pooled logits.
        #[arg(long, default_value_t = 0.25)]
        margin_step: f32,
        /// Widest margin on the grid.
        #[arg(long, default_value_t = 12.0)]
        margin_max: f32,
        /// Fit the margins below and above the threshold separately.
        #[arg(long)]
        asymmetric_margins: bool,
        /// Also break the early-exit results down by dataset or generator.
        #[arg(long)]
        slice_by: Option<String>,
        /// Write one score file per test set and candidate here, for `report --operating-points`.
        #[arg(long)]
        write_scores: Option<PathBuf>,
        /// Keep fit items that are also test items in the fit, and mark the output as in-sample (to read a cutoff on slices of the set it was fitted on).
        #[arg(long)]
        in_sample: bool,
    },
    /// Write the frame features the temporal-head experiment reads
    /// (docs/runbooks/temporal-head.md): per clip and sampled frame, the
    /// actual presentation time, the position in the stream, the logits, and
    /// the patch tokens of the whole-frame view pooled to a grid. Offline
    /// tooling: nothing in `detect` uses it. Runs on every platform with
    /// `--device cpu`.
    DumpVideoFeatures {
        /// The video checkpoint (a copy under runs/, with its .json beside it).
        #[arg(long)]
        weights: PathBuf,
        #[arg(long, required = true)]
        manifest: Vec<PathBuf>,
        /// Directory to write meta.json, clips.tsv, frames.tsv and features.f32 to.
        #[arg(long)]
        out: PathBuf,
        /// Frames sampled per clip (default: the model's video profile).
        #[arg(long)]
        frames: Option<usize>,
        /// Spread samples over the first N seconds. Use 0 for the whole clip.
        #[arg(long, default_value_t = 30.0)]
        max_seconds: f64,
        /// Cells along one side of the pooled token map; must divide 14 (1, 2, 7 or 14).
        #[arg(long, default_value_t = 7)]
        grid: usize,
        /// Blocks whose tokens to store (default: the last block).
        #[arg(long, value_delimiter = ',')]
        blocks: Vec<usize>,
        /// Sample pairs instead: half as many positions, each followed by the frame N frames later in the stream.
        #[arg(long, default_value_t = 0)]
        pairs: usize,
        /// Control: make every sample the mean of itself and the N-1 frames before it (synthetic motion blur).
        #[arg(long, default_value_t = 1)]
        temporal_blur: usize,
        /// Score the whole-frame view only (one view per frame instead of the profile's).
        #[arg(long)]
        fit_only: bool,
        /// auto, gpu-f32, gpu or cpu. `auto` is the GPU in single precision where there is one: half precision (`gpu`) is what scoring uses, but it moves some patch tokens a long way.
        #[arg(long, default_value = "auto")]
        device: String,
        /// Decode threads.
        #[arg(long, default_value_t = 4)]
        threads: usize,
    },
    /// Fit a shallow clip head to dumped frame features, on the CPU. Heads
    /// start as the frame mean and learn a correction to it.
    FitTemporalHead {
        /// A `dump-video-features` directory of the fit set.
        #[arg(long)]
        features: PathBuf,
        /// covariates, pool-linear, attn-pool, patch (order-invariant), diff or concat (temporal).
        #[arg(long)]
        kind: String,
        /// Where to write `<out>.json` and `<out>.safetensors`.
        #[arg(long)]
        out: PathBuf,
        /// A second dump to report on after every epoch (for watching the fit only).
        #[arg(long)]
        watch: Option<PathBuf>,
        /// The per-frame logit the head corrects: frame (every view) or fit (the whole-frame view alone).
        #[arg(long, default_value = "frame")]
        base: String,
        /// The stored block to read (default: the last one in the dump).
        #[arg(long)]
        block: Option<usize>,
        #[arg(long, default_value_t = 32)]
        hidden: usize,
        /// How `diff` scales a difference by the time between its frames: none, sqrt or rate.
        #[arg(long, default_value = "none")]
        time_norm: String,
        #[arg(long, default_value_t = 20)]
        epochs: usize,
        #[arg(long, default_value_t = 1e-3)]
        lr: f32,
        #[arg(long, default_value_t = 1e-4)]
        weight_decay: f32,
        #[arg(long, default_value_t = 64)]
        batch: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Control: fit with every clip's frames in random order (an order-blind twin of the head).
        #[arg(long)]
        shuffle_frames: bool,
        /// A stretch holds a cut when its largest frame-to-frame difference is above this many levels ...
        #[arg(long, default_value_t = 12.0)]
        cut_above: f32,
        /// ... and above this many times the clip's median difference.
        #[arg(long, default_value_t = 6.0)]
        cut_ratio: f32,
    },
    /// Compare the frame mean, fitted heads and other score files on dumped
    /// frame features: operating points at cutoffs from a calibration dump,
    /// the shuffled-order control, and results by cuts, repeated frames,
    /// motion and cadence.
    EvalTemporalHead {
        /// A `dump-video-features` directory of the clips to report on.
        #[arg(long)]
        features: PathBuf,
        /// The dump of the calibration clips: cutoffs come from it. Without it they come from `--features` and every rate is exploratory.
        #[arg(long)]
        calibrate: Option<PathBuf>,
        /// `name=path` of a head written by `fit-temporal-head`. Repeatable.
        #[arg(long)]
        head: Vec<String>,
        /// `name=test.tsv[,calibrate.tsv]`: `score-video` files of another system on the same clips (more frames, fewer views). Repeatable.
        #[arg(long)]
        scores: Vec<String>,
        /// False-positive rates to operate at.
        #[arg(long, value_delimiter = ',', default_value = "0.01,0.001")]
        fpr: Vec<f64>,
        /// Seed of the shuffled-order control.
        #[arg(long, default_value_t = 1)]
        shuffle_seed: u64,
        /// Skip the shuffled-order control.
        #[arg(long)]
        no_shuffle: bool,
        /// Write one score file per arm here, for `tintype report --operating-points`.
        #[arg(long)]
        out_dir: Option<PathBuf>,
        #[arg(long, default_value_t = 12.0)]
        cut_above: f32,
        #[arg(long, default_value_t = 6.0)]
        cut_ratio: f32,
    },
    /// Check the released model files (weights and specs) against the
    /// SHA-256 values recorded in the source and in docs/DATA.md.
    VerifyModels {
        /// The repository checkout that holds `models/`.
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
}

/// Explicit sampling weights and the replay pool of a training recipe. With
/// none of them given, sampling is what it always was: both classes equally
/// often and, within a class, a generator with weight `count^alpha`.
#[derive(clap::Args, Clone, Debug)]
struct SamplingFlags {
    /// `MANIFEST=WEIGHT`: multiply the sampling mass of every item of a
    /// manifest (named by its path as given, or by its store directory).
    /// Repeatable.
    #[arg(long, value_name = "MANIFEST=WEIGHT")]
    manifest_weight: Vec<String>,
    /// `FAMILY=WEIGHT`: multiply the sampling mass of every item of a
    /// generator family (manifest column 8, or else `--family-map`, or else
    /// the generator name; `real:<source>` for real items). Repeatable.
    #[arg(long, value_name = "FAMILY=WEIGHT")]
    family_weight: Vec<String>,
    /// A file of weights: tab-separated `manifest|family  name  weight` rows.
    /// `--manifest-weight` and `--family-weight` replace its weight for the same name.
    #[arg(long)]
    weights_file: Option<PathBuf>,
    /// Tab-separated `generator  family` rows naming the family of
    /// generators whose manifests have no `generator_family` column.
    #[arg(long)]
    family_map: Option<PathBuf>,
    /// A manifest of the replay pool (older generators, reposted images):
    /// `--replay-share` of the draws come from these instead. Repeatable.
    #[arg(long)]
    replay_manifest: Vec<PathBuf>,
    /// The share of draws taken from the replay pool.
    #[arg(long, default_value_t = 0.2)]
    replay_share: f32,
}

impl SamplingFlags {
    fn options(&self) -> Result<tintype::sampling::Options> {
        use tintype::sampling;
        let (mut manifest_weights, mut family_weights) = match &self.weights_file {
            Some(f) => sampling::read_weights_file(f)?,
            None => (Vec::new(), Vec::new()),
        };
        // A weight given on the command line replaces the file's weight for the same name.
        let add = |list: &mut Vec<(String, f64)>, flags: &[String], from_file: usize| -> Result<()> {
            for w in flags {
                let (name, weight) = sampling::parse_weight(w)?;
                if let Some(k) = list[..from_file].iter().position(|(n, _)| *n == name) {
                    list[k].1 = weight;
                } else {
                    list.push((name, weight));
                }
            }
            Ok(())
        };
        let (m, f) = (manifest_weights.len(), family_weights.len());
        add(&mut manifest_weights, &self.manifest_weight, m)?;
        add(&mut family_weights, &self.family_weight, f)?;
        let family_map = match &self.family_map {
            Some(f) => sampling::read_family_map(f)?,
            None => Default::default(),
        };
        Ok(sampling::Options { manifest_weights, family_weights, family_map, replay_manifests: self.replay_manifest.clone(), replay_share: self.replay_share })
    }
}

fn parse_device(s: &str) -> Result<Device> {
    Ok(match s {
        "auto" => Device::best(),
        "gpu" => Device::Gpu,
        "gpu-f32" => Device::GpuF32,
        "cpu" => Device::Cpu,
        other => anyhow::bail!("unknown device `{other}` (use auto, gpu, gpu-f32 or cpu)"),
    })
}

#[cfg(target_os = "macos")]
fn system(reference: &Option<String>, fast: bool, sensitive: bool, strict: bool) -> Result<tintype::eval::System> {
    use tintype::eval::{Photo, System};
    Ok(match reference.as_deref() {
        None => System::Tintype { profile: if fast { Photo::Fast } else if sensitive { Photo::Sensitive } else if strict { Photo::Strict } else { Photo::Default } },
        Some("commfor") => System::CommunityForensics,
        Some("swinv2") => System::SwinV2,
        Some("waverep") => System::WaveRep,
        Some(other) => anyhow::bail!("unknown reference detector `{other}` (commfor, swinv2, waverep)"),
    })
}

/// The `f` quantile of sorted values (the same nearest-rank rule the
/// latency line has always used for its median and p90).
fn quantile(sorted: &[f64], f: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * f) as usize]
}

/// `lo,hi` on the command line.
fn parse_pair<T: std::str::FromStr>(text: &str) -> Result<(T, T), String> {
    let parse = |v: &str| v.trim().parse::<T>().map_err(|_| format!("`{text}` is not two numbers `lo,hi`"));
    match text.split_once(',') {
        Some((lo, hi)) => Ok((parse(lo)?, parse(hi)?)),
        None => Err(format!("`{text}` is not two numbers `lo,hi`")),
    }
}

/// The training augmentation. With none of these flags given it is the
/// phase-6 recipe, draw for draw.
#[derive(clap::Args)]
struct AugArgs {
    #[arg(long, default_value_t = 0.5)]
    p_native: f32,
    /// Probability of the messenger-style downscale + JPEG augmentation.
    #[arg(long, default_value_t = 0.25)]
    p_messenger: f32,
    /// Long-side cap of a messenger trip in pixels, drawn from `lo,hi`.
    #[arg(long, default_value = "960,1700", value_parser = parse_pair::<f32>)]
    messenger_cap: (f32, f32),
    /// JPEG quality of a messenger trip, drawn from `lo,hi`.
    #[arg(long, default_value = "55,90", value_parser = parse_pair::<f32>)]
    messenger_quality: (f32, f32),
    /// Messenger trips in a row (share cycles), drawn from `lo,hi` inclusive.
    #[arg(long, default_value = "1,1", value_parser = parse_pair::<u32>)]
    messenger_cycles: (u32, u32),
    /// Probability of a processing chain, applied to real and generated items alike. 0 = off.
    #[arg(long, default_value_t = 0.0)]
    p_chain: f32,
    /// Operations per chain, drawn from `lo,hi` inclusive.
    #[arg(long, default_value = "1,3", value_parser = parse_pair::<u32>)]
    chain_ops: (u32, u32),
    /// How far each chain operation may go, 0 (barely visible) to 1 (heaviest).
    #[arg(long, default_value_t = 0.5)]
    chain_severity: f32,
    /// Operations a chain draws from: resize, jpeg, blur, noise, sharpen, repost,
    /// motion-blur, lens-blur, shot-noise, impulse-noise, pixelate, tone.
    #[arg(long, default_value = "resize,jpeg,blur,noise,sharpen,repost")]
    chain_kinds: String,
    /// Share of chains applied to the finished 224 px view instead of the source image.
    #[arg(long, default_value_t = 0.25)]
    chain_post: f32,
    /// A chain on a larger source image first cuts a random window of this many pixels.
    #[arg(long, default_value_t = 4_000_000)]
    chain_max_pixels: u64,
    /// Curriculum `step:p:max_ops:severity,...`: from each step on, these replace
    /// --p-chain, the upper end of --chain-ops and --chain-severity.
    #[arg(long, default_value = "")]
    chain_schedule: String,
}

impl AugArgs {
    fn config(&self) -> Result<tintype::data::AugConfig> {
        use tintype::data::{AugConfig, ChainOp, ChainStage};
        let unit = |name: &str, v: f32| -> Result<f32> {
            anyhow::ensure!((0.0..=1.0).contains(&v), "--{name} must be within 0..1");
            Ok(v)
        };
        let ordered = |name: &str, lo: f64, hi: f64, min: f64| -> Result<()> {
            anyhow::ensure!(lo <= hi && lo >= min, "--{name} takes lo,hi with {min} <= lo <= hi");
            Ok(())
        };
        ordered("messenger-cap", self.messenger_cap.0 as f64, self.messenger_cap.1 as f64, 1.0)?;
        ordered("messenger-quality", self.messenger_quality.0 as f64, self.messenger_quality.1 as f64, 1.0)?;
        anyhow::ensure!(self.messenger_quality.1 <= 100.0, "--messenger-quality is at most 100");
        ordered("messenger-cycles", self.messenger_cycles.0 as f64, self.messenger_cycles.1 as f64, 1.0)?;
        ordered("chain-ops", self.chain_ops.0 as f64, self.chain_ops.1 as f64, 1.0)?;
        anyhow::ensure!(self.chain_max_pixels >= 224 * 224, "--chain-max-pixels must hold at least one 224 px view");
        Ok(AugConfig {
            p_native: self.p_native,
            p_messenger: self.p_messenger,
            messenger_cap: self.messenger_cap,
            messenger_quality: self.messenger_quality,
            messenger_cycles: self.messenger_cycles,
            p_chain: unit("p-chain", self.p_chain)?,
            chain_ops: self.chain_ops,
            chain_severity: unit("chain-severity", self.chain_severity)?,
            chain_kinds: ChainOp::parse_list(&self.chain_kinds)?,
            chain_post: unit("chain-post", self.chain_post)?,
            chain_max_pixels: self.chain_max_pixels,
            chain_schedule: ChainStage::parse_list(&self.chain_schedule)?,
            ..Default::default()
        })
    }
}

fn load_manifests(ms: &[PathBuf]) -> Result<Vec<ingest::Item>> {
    let mut v = Vec::new();
    for m in ms {
        v.extend(ingest::read_manifest(m)?);
    }
    Ok(v)
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Detect { paths, options, second_opinion, photo_second_opinion } => {
            scan::detect(paths, options, photo_second_opinion, second_opinion)
        }
        Cmd::Batch { paths, options } => scan::batch(paths, options),
        Cmd::Demo(options) => scan::demo(options),
        #[cfg(feature = "provenance")]
        Cmd::Provenance(args) => provenance::run(args),
        Cmd::Serve(args) => tintype::server::run(args, scan::resolve_weights),
        Cmd::InitTraining { backbone, sha256, out } => {
            tintype::training_init::initialize(&backbone, &sha256, &out)
        }
        #[cfg(target_os = "macos")]
        Cmd::Score { weights, reference, manifest, out, batch, device, views, fast, sensitive, strict, all_views, photo_second_opinion } => {
            tintype::eval::score_images(&weights, &system(&reference, fast, sensitive, strict)?, &manifest, &out, parse_device(&device)?, batch, views.as_deref(), all_views, photo_second_opinion.as_deref()).map(|_| ())
        }
        #[cfg(target_os = "macos")]
        Cmd::ScoreVideo { weights, reference, manifest, out, frames, views, second_opinion, all_frames, batch, device } => {
            tintype::eval::score_videos(&weights, &system(&reference, false, false, false)?, &manifest, &out, parse_device(&device)?, batch, frames, views.as_deref(), second_opinion.as_deref(), all_frames).map(|_| ())
        }
        Cmd::Report { systems, per_generator, markdown, pooled, at_rate_of, operating_points, fpr, prevalence, slice_by, cluster_by, groups, min_slice, transfer_budget, review_margin, review_disagreement, review_threshold_logit, review_flags } => {
            anyhow::ensure!(operating_points || (transfer_budget.is_none() && review_margin.is_empty() && review_flags.is_none()), "--transfer-budget, --review-margin and --review-flags go with --operating-points");
            let mut loaded = Vec::new();
            for s in &systems {
                let (name, rest) = s.split_once('=').ok_or_else(|| anyhow::anyhow!("use name=scores.tsv[@threshold]"))?;
                let (path, thr) = match rest.rsplit_once('@') {
                    Some((p, t)) => (p, t.parse::<f32>()?),
                    None => (rest, 0.5),
                };
                loaded.push((name.to_string(), tintype::report::read_scores(std::path::Path::new(path))?, thr));
            }
            if operating_points {
                anyhow::ensure!(fpr.iter().all(|f| (0.0..1.0).contains(f)) && (0.0..=1.0).contains(&prevalence), "--fpr and --prevalence are fractions, e.g. 0.001");
                let cluster_by = match cluster_by.as_deref() {
                    None | Some("none") => None,
                    Some(k) => tintype::split::Key::parse_list(k)?.first().copied(),
                };
                let mut by_path = std::collections::BTreeMap::new();
                for it in load_manifests(&groups)? {
                    by_path.insert(it.path.display().to_string(), it);
                }
                anyhow::ensure!(transfer_budget.is_none_or(|b| b > 0.0 && b < 1.0), "--transfer-budget is a fraction, e.g. 0.01");
                anyhow::ensure!(review_margin.iter().all(|m| m.is_finite() && *m >= 0.0) && review_disagreement.is_none_or(|d| d > 0.0 && d <= 1.0), "--review-margin is a non-negative calibrated-logit distance and --review-disagreement a share above 0 and at most 1");
                anyhow::ensure!(review_disagreement.is_none() || !review_margin.is_empty(), "--review-disagreement goes with --review-margin (use --review-margin 0 for disagreement alone)");
                let review = if review_margin.is_empty() && review_flags.is_none() {
                    None
                } else {
                    Some(tintype::report::ReviewAccounting {
                        margins: review_margin,
                        disagreement: review_disagreement.zip(review_threshold_logit),
                        flags: review_flags.as_deref().map(tintype::report::read_review_flags).transpose()?,
                    })
                };
                let opts = tintype::report::OperatingPoints { fprs: fpr, prevalence, slice_by, cluster_by, min_slice, groups: by_path, review, transfer_budget };
                tintype::report::operating_points_markdown(&loaded, &opts)?;
            } else if let Some(baseline) = at_rate_of {
                tintype::report::matched_markdown(&loaded, &baseline, pooled.as_deref())?;
            } else if markdown {
                tintype::report::compare_markdown(&loaded, pooled.as_deref());
            } else {
                tintype::report::compare(&loaded, per_generator);
            }
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Cmd::Evaluate { suite, weights, video_weights, tag, scores, baselines, reuse, device } => {
            let b = tintype::eval::Baselines::default();
            let video_weights = video_weights.unwrap_or_else(|| tintype::detector::video_weights_for(&weights));
            tintype::eval::evaluate(&suite, &weights, &video_weights, &tag, &scores, baselines.then_some(&b), reuse, parse_device(&device)?)
        }
        #[cfg(target_os = "macos")]
        Cmd::Finalize { checkpoint, weights, scores, photo_fpr, strict_fpr, sensitive_fpr, video_fpr, also_trained_on, also_scored_on, device } => {
            tintype::eval::finalize(&checkpoint, &weights, &scores, photo_fpr, strict_fpr, sensitive_fpr, video_fpr, (&also_trained_on, &also_scored_on), parse_device(&device)?)
        }
        #[cfg(target_os = "macos")]
        Cmd::FinalizeVideo { checkpoint, weights, scores, video_fpr, second_opinion, device } => {
            let band = match second_opinion {
                Some(s) => {
                    let v: Vec<f32> = s.split(',').map(|x| x.trim().parse::<f32>()).collect::<Result<_, _>>()?;
                    anyhow::ensure!(v.len() == 3 && v[0] < v[1], "--second-opinion takes lo,hi,threshold with lo < hi");
                    Some(tintype::detector::SecondOpinion { lo: v[0], hi: v[1], threshold: v[2] })
                }
                None => None,
            };
            tintype::eval::finalize_video(&checkpoint, &weights, &scores, video_fpr, band, parse_device(&device)?)
        }
        #[cfg(target_os = "macos")]
        Cmd::Calibrate { weights, scores, target_fpr, profile, early_exit, rule, min_source, confidence, groups, per_parent, group_diagnostic } => {
            if early_exit {
                let mut spec = ModelSpec::load_for(&weights)?;
                spec.early_exit = Some(tintype::detector::EarlyExit::standard());
                spec.save_for(&weights)?;
            }
            use tintype::eval::Rule;
            let rule = match rule.as_str() {
                "pooled" => Rule::Pooled,
                "strictest" => Rule::Strictest(min_source),
                "balanced" => Rule::Balanced(min_source),
                "bound" => Rule::Bound,
                other => anyhow::bail!("unknown rule `{other}` (pooled, strictest, balanced, bound)"),
            };
            let mut by_path = std::collections::BTreeMap::new();
            for it in load_manifests(&groups)? {
                by_path.insert(it.path.display().to_string(), it);
            }
            let opts = tintype::eval::CalibrateOptions { confidence, groups: by_path, per_parent: tintype::calibration::PerParent::parse(&per_parent)?, group_diagnostic, min_group: min_source };
            tintype::eval::calibrate_with(&weights, &scores, target_fpr, &profile, rule, &opts)
        }
        Cmd::RefitExit { weights, profile, threshold, fit, report, tolerance, target_fpr, min_source, step, max_margin, asymmetric, write, apply_out, rebase_band_from } => {
            use tintype::exit_fit;
            if write {
                // Before any work: the released specs are never rewritten.
                exit_fit::refuse_released(&weights)?;
            }
            let mut spec = ModelSpec::load_for(&weights)?;
            let names: Vec<&str> = profile.split(',').map(str::trim).filter(|p| !p.is_empty()).collect();
            anyhow::ensure!(!names.is_empty(), "--profile names at least one profile");
            let video = names.contains(&"video");
            anyhow::ensure!(!video || names.len() == 1, "the video profile has a schedule of its own: check it separately from the photo profiles");
            anyhow::ensure!(threshold.is_none() || names.len() == 1, "--threshold replaces the threshold of one profile");
            anyhow::ensure!(target_fpr.is_empty() || target_fpr.len() == names.len(), "--target-fpr takes one target per profile, in the order of --profile");
            // (name, logit scale, logit bias) of each profile; its raw threshold is -bias / scale.
            let mut profiles: Vec<(&str, f32, f32)> = Vec::new();
            for &name in &names {
                let (scale, bias) = match name {
                    "photo" => (spec.logit_scale, spec.logit_bias),
                    "strict" => spec.strict.as_ref().map(|p| (p.logit_scale, p.logit_bias)).ok_or_else(|| anyhow::anyhow!("{} has no strict profile", weights.display()))?,
                    "sensitive" => spec.sensitive.as_ref().map(|p| (p.logit_scale, p.logit_bias)).ok_or_else(|| anyhow::anyhow!("{} has no sensitive profile", weights.display()))?,
                    "video" => spec.video.as_ref().map(|v| (v.logit_scale, v.logit_bias)).ok_or_else(|| anyhow::anyhow!("{} has no video profile", weights.display()))?,
                    "fast" => anyhow::bail!("the fast profile scores two views and has no early exit"),
                    other => anyhow::bail!("unknown profile `{other}` (use photo, strict, sensitive or video)"),
                };
                // An explicit threshold keeps the profile's slope and moves its zero.
                profiles.push((name, scale, threshold.map_or(bias, |t| -scale * t)));
            }
            let thresholds: Vec<f32> = profiles.iter().map(|&(_, scale, bias)| -bias / scale).collect();
            let (units, unit, stored) = match &spec.video {
                Some(v) if video => (v.frames, "frame", v.early_exit.clone()),
                _ => (spec.views.len(), "view", spec.early_exit.clone()),
            };
            let template = stored.clone().unwrap_or_else(|| if video { tintype::detector::EarlyExit::video_standard() } else { tintype::detector::EarlyExit::standard() });
            anyhow::ensure!(template.covers(units), "the schedule does not cover the {units} {unit}s of the profile");
            let show = |e: &tintype::detector::EarlyExit| e.stages.iter().map(|s| if s.below == s.above { format!("{}: {}", s.views, s.below) } else { format!("{}: -{} / +{}", s.views, s.below, s.above) }).collect::<Vec<_>>().join(", ");
            println!("{}: {} schedule, order {:?}, margins after N {unit}s (raw logits) {}", weights.display(), if stored.is_some() { "stored" } else { "no stored schedule; the standard" }, template.order, show(&template));
            let (rows, skipped) = exit_fit::read_rows(&report, units)?;
            if skipped > 0 {
                println!("{skipped} rows of the report files do not hold {units} logits and are left out");
            }
            let fitted = if fit.is_empty() {
                None
            } else {
                let (fit_rows, fit_skipped) = exit_fit::read_rows(&fit, units)?;
                if fit_skipped > 0 {
                    println!("{fit_skipped} rows of the fit files do not hold {units} logits and are left out");
                }
                let opts = exit_fit::FitOptions { tolerance, step, max_margin, asymmetric };
                let fitted = exit_fit::fit_margins(&fit_rows, &template, &thresholds, &opts)?;
                println!("fitted on {} items, pooled precision and recall within {tolerance} of every {unit} at {}: margins after N {unit}s {}", fit_rows.len(), names.join(", "), show(&fitted));
                for (&(name, ..), &thr) in profiles.iter().zip(&thresholds) {
                    let c = exit_fit::check(&fit_rows, &fitted, thr);
                    println!("  on the fit items, {name} (threshold {thr:.3}): {:.2} {unit}s per item; precision {:.4} against {:.4}, recall {:.4} against {:.4}", c.mean_units, c.exit.precision(), c.all.precision(), c.exit.recall(), c.all.recall());
                }
                if fit.iter().any(|f| report.contains(f)) {
                    println!("NOTE: a file is both fitted on and reported on. The report below is in-sample for it: exploratory, not a check.");
                }
                Some(fitted)
            };
            for (k, (&(name, ..), &thr)) in profiles.iter().zip(&thresholds).enumerate() {
                for (what, schedule) in [(if stored.is_some() { "stored schedule" } else { "standard schedule" }, Some(&template)), ("fitted schedule", fitted.as_ref())] {
                    let Some(schedule) = schedule else { continue };
                    println!("\n## {name}, threshold {thr:.3} (raw logit), {what}\n");
                    for l in exit_fit::check_lines(&exit_fit::check(&rows, schedule, thr), unit, target_fpr.get(k).copied(), min_source) {
                        println!("{l}");
                    }
                }
            }
            let schedule = fitted.as_ref().unwrap_or(&template);
            // The photo second-opinion band is in the default profile's calibrated logits: say what it routes now.
            if let (false, Some(band), Some(&(_, scale, bias))) = (video, spec.second_opinion, profiles.iter().find(|p| p.0 == "photo")) {
                let band = match &rebase_band_from {
                    Some(old_spec) => {
                        let old: ModelSpec = serde_json::from_str(&std::fs::read_to_string(old_spec)?)?;
                        let old_band = old.second_opinion.ok_or_else(|| anyhow::anyhow!("{} has no photo second-opinion band", old_spec.display()))?;
                        let moved = exit_fit::rebase_photo_band(&old_band, (old.logit_scale, old.logit_bias), (scale, bias));
                        println!("\nphoto second-opinion band of {}: lo {}, hi {}, weight {}, threshold {}; the same routing and rule in this calibration: lo {:.4}, hi {:.4}, weight {:.4}, threshold {:.4}", old_spec.display(), old_band.lo, old_band.hi, old_band.weight, old_band.threshold, moved.lo, moved.hi, moved.weight, moved.threshold);
                        if write {
                            spec.second_opinion = Some(moved);
                        }
                        moved
                    }
                    None => band,
                };
                let thr = -bias / scale;
                let mean = |v: &[f32]| v.iter().sum::<f32>() / v.len() as f32;
                let mut routed = [[0usize; 2]; 2]; // [every view, early exit][real, AI]
                for r in &rows {
                    routed[0][r.label as usize] += band.wanted(scale * mean(&r.units) + bias) as usize;
                    routed[1][r.label as usize] += band.wanted(scale * mean(&schedule.looked_at(&r.units, thr)) + bias) as usize;
                }
                let (nr, nf) = (rows.iter().filter(|r| r.label == 0).count().max(1), rows.iter().filter(|r| r.label == 1).count().max(1));
                println!("\nphoto second-opinion band ({}, {}) in calibrated logits is ({:.3}, {:.3}) in raw logits around the threshold {thr:.3}.", band.lo, band.hi, (band.lo - bias) / scale, (band.hi - bias) / scale);
                for (what, k) in [(format!("every {unit}"), routed[0]), ("the early-exit views".to_string(), routed[1])] {
                    println!("  routed from {what}: {} of {} real ({:.2}%), {} of {} AI ({:.2}%)", k[0], nr, 100.0 * k[0] as f64 / nr as f64, k[1], nf, 100.0 * k[1] as f64 / nf as f64);
                }
                println!("  The band and its rule were not refitted: that takes second-stage scores on calibration data.");
            } else if let Some(band) = spec.video.as_ref().and_then(|v| v.second_opinion).filter(|_| video) {
                let routed = rows.iter().filter(|r| band.wanted(r.units.iter().sum::<f32>() / r.units.len() as f32)).count();
                println!("\nvideo second-opinion band ({}, {}) is in raw logits and does not move with the calibration; the threshold {:.3} sits {:.3} above its lower edge. It routes {routed} of {} clips from every frame (early exit is off when the second opinion runs).", band.lo, band.hi, thresholds[0], thresholds[0] - band.lo, rows.len());
            }
            if let Some(out) = &apply_out {
                let (_, scale, bias) = profiles[0];
                let n = exit_fit::apply(&report, out, Some(schedule), units, scale, bias)?;
                println!("\n{}: {n} items as {} decides them under the {} schedule", out.display(), profiles[0].0, if fitted.is_some() { "fitted" } else { "stored" });
            }
            if write {
                let Some(fitted) = fitted else { anyhow::bail!("--write stores a fitted schedule: pass --fit") };
                if video {
                    spec.video.as_mut().expect("checked above").early_exit = Some(fitted);
                } else {
                    spec.early_exit = Some(fitted);
                }
                spec.save_for(&weights)?;
                println!("\nwrote the fitted schedule to {}", ModelSpec::sidecar(&weights).display());
            }
            Ok(())
        }
        Cmd::Latency { weights, reference, manifest, n, device, fast, all_views, photo_second_opinion } => {
            let items = ingest::read_manifest(&manifest)?;
            let mut spec = match reference.as_deref() {
                Some("commfor") => ModelSpec::commfor(384),
                Some("swinv2") => ModelSpec::swinv2(),
                Some(other) => anyhow::bail!("unknown reference detector `{other}`"),
                None => ModelSpec::load_for(&weights)?,
            };
            if fast {
                Detector::use_fast_profile(&mut spec);
            }
            let nv = spec.views.len();
            let exit = spec.early_exit.clone().filter(|_| reference.is_none() && !fast && !all_views);
            // On the CPU early exit runs the views one or two at a time; on the GPU all
            // six go in one batch and the stopping rule is applied afterwards.
            let device = parse_device(&device)?;
            let stepwise = exit.is_some() && matches!(device, Device::Cpu);
            let det = Detector::load(&weights, Some(spec), if stepwise { 2 } else { nv }, device)?;
            let pre = det.preprocessor();
            let mut buf = vec![0u8; nv * 3 * det.spec.img * det.spec.img];
            // The second stage of the photo second opinion: both networks stay loaded, as
            // in a server, and the second one is asked only about the images inside the band.
            let mut second = match &photo_second_opinion {
                Some(weights2) => {
                    let band = det.spec.second_opinion.ok_or_else(|| anyhow::anyhow!("{} has no second-opinion band for photos", weights.display()))?;
                    let det2 = Detector::load(weights2, Some(ModelSpec::commfor(384)), 1, device)?;
                    let buf2 = vec![0u8; 3 * det2.spec.img * det2.spec.img];
                    // One untimed run, so that the first image inside the band does not pay for the warm-up.
                    std::hint::black_box(det2.logits(&buf2));
                    Some((band, det2.preprocessor(), det2, buf2))
                }
                None => None,
            };
            let mut ms = Vec::new();
            let mut asked_ms = Vec::new();
            for (k, it) in items.iter().take(n + 5).enumerate() {
                let t = std::time::Instant::now();
                let Ok(img) = tintype::image_io::load(&it.path) else { continue };
                pre.apply(&img, &mut buf);
                let seen = match &exit {
                    Some(e) if stepwise => det.logits_adaptive(&buf, e),
                    Some(e) => e.looked_at(&det.logits(&buf)[..nv], det.threshold_logit()),
                    None => det.logits(&buf)[..nv].to_vec(),
                };
                let mut p = det.probability(&seen);
                let mut asked = false;
                if let Some((band, pre2, det2, buf2)) = &mut second {
                    let first = det.calibrated_logit(&seen);
                    if band.wanted(first) {
                        pre2.apply(&img, buf2);
                        p = band.probability(first, det2.logits(buf2)[0]);
                        asked = true;
                    }
                }
                std::hint::black_box(p);
                if k >= 5 {
                    ms.push(t.elapsed().as_secs_f64() * 1e3);
                    if asked {
                        asked_ms.push(t.elapsed().as_secs_f64() * 1e3);
                    }
                }
            }
            anyhow::ensure!(!ms.is_empty(), "none of the first {} files of {} could be read", n + 5, manifest.display());
            ms.sort_by(|a, b| a.total_cmp(b));
            println!("single-image latency over {} images: median {:.1} ms, p90 {:.1} ms, p95 {:.1} ms, mean {:.1} ms", ms.len(), quantile(&ms, 0.5), quantile(&ms, 0.9), quantile(&ms, 0.95), ms.iter().sum::<f64>() / ms.len() as f64);
            if second.is_some() {
                asked_ms.sort_by(|a, b| a.total_cmp(b));
                let share = 100.0 * asked_ms.len() as f64 / ms.len() as f64;
                match asked_ms.is_empty() {
                    true => println!("second opinion: 0 of {} images inside the band", ms.len()),
                    false => println!("second opinion: {} of {} images inside the band ({share:.1}%); those took median {:.1} ms, p95 {:.1} ms", asked_ms.len(), ms.len(), quantile(&asked_ms, 0.5), quantile(&asked_ms, 0.95)),
                }
            }
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Cmd::Profile { weights, manifest, n, device, batches, model_only } => {
            use rayon::prelude::*;
            use std::time::Instant;
            let spec = ModelSpec::load_for(&weights)?;
            let device = parse_device(&device)?;
            if !model_only {
                let items = ingest::read_manifest(&manifest)?;
                let paths: Vec<PathBuf> = items.iter().take(n).map(|i| i.path.clone()).collect();
                let t = Instant::now();
                let imgs: Vec<tintype::image_io::Rgb> = paths.par_iter().filter_map(|p| tintype::image_io::load(p).ok()).collect();
                let s = t.elapsed().as_secs_f64();
                let mp: f64 = imgs.iter().map(|i| i.w as f64 * i.h as f64 / 1e6).sum();
                println!("decode: {} images, {:.0} megapixels in {:.2}s: {:.0} img/s, {:.0} MP/s", imgs.len(), mp, s, imgs.len() as f64 / s, mp / s);
                let pre = Detector::load(&weights, Some(spec.clone()), 1, Device::Cpu)?.preprocessor();
                let t = Instant::now();
                let k: usize = imgs.par_iter().map(|im| pre.crops(im).len()).sum();
                let s = t.elapsed().as_secs_f64();
                println!("crops: {} views of {} images in {:.2}s: {:.0} img/s", k, imgs.len(), s, imgs.len() as f64 / s);
            }
            let vl = 3 * spec.img * spec.img;
            for &b in &batches {
                let det = Detector::load(&weights, Some(spec.clone()), b, device)?;
                let mut rng = tintype::rng::Rng::new(7);
                let x: Vec<u8> = (0..b * vl).map(|_| rng.below(256) as u8).collect();
                for _ in 0..3 {
                    std::hint::black_box(det.logits(&x));
                }
                let (t, mut runs) = (Instant::now(), 0usize);
                while t.elapsed().as_secs_f64() < 3.0 {
                    std::hint::black_box(det.logits(&x));
                    runs += 1;
                }
                let s = t.elapsed().as_secs_f64();
                println!("network batch {b}: {:.1} ms per batch, {:.0} views/s", s * 1e3 / runs as f64, (runs * b) as f64 / s);
            }
            Ok(())
        }
        #[cfg(target_os = "macos")]
        Cmd::Train { init, init_prefix, manifest, out, steps, batch, lr, warmup, val_pct, val_max, eval_every, threads, seed, layer_decay, weight_decay, smoothing, sampler_alpha, aug, sampling, print_sampling } => {
            tintype::train_cmd::run(tintype::train_cmd::TrainArgs {
                init,
                init_prefix,
                manifests: manifest,
                out,
                steps,
                batch,
                lr,
                warmup,
                val_pct,
                val_max,
                eval_every,
                threads,
                seed,
                aug: aug.config()?,
                opt: tintype::trainer::OptConfig { layer_decay, weight_decay, smoothing, ..Default::default() },
                sampler_alpha,
                sampling: sampling.options()?,
                print_sampling,
            })
        }
        Cmd::SamplingShares { manifest, val_pct, sampler_alpha, sampling, top, rebalance_families, write_weights } => {
            let (recipe, val_items) = tintype::sampling::Recipe::load(&manifest, &sampling.options()?, sampler_alpha)?.split_validation(val_pct);
            let sampler = recipe.sampler()?;
            println!("train {} images ({}), val pool {}\n", recipe.items.len(), sampler.summary(), val_items.len());
            print!("{}", recipe.shares_report(&sampler, top));
            if let (Some(gamma), Some(out)) = (rebalance_families, write_weights) {
                let weights = recipe.rebalance_families(&sampler, gamma)?;
                let header = format!("Family weights that move each AI family's share of AI draws to share^{gamma}, renormalised.\nWritten by `tintype sampling-shares --rebalance-families {gamma}` from {} manifests, alpha {sampler_alpha}, validation {val_pct}%.", manifest.len());
                std::fs::write(&out, tintype::sampling::weights_file_text(&header, &weights))?;
                println!("\n{}: weights for {} AI families", out.display(), weights.len());
            }
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        Cmd::Score { .. } | Cmd::ScoreVideo { .. } | Cmd::Evaluate { .. } | Cmd::Finalize { .. } | Cmd::FinalizeVideo { .. } | Cmd::Calibrate { .. } | Cmd::Profile { .. } | Cmd::Train { .. } => {
            anyhow::bail!("this command uses the Apple GPU engine and needs macOS on Apple Silicon; `detect --device cpu` works everywhere")
        }
        Cmd::HashSplit { manifest, val_pct, out_dir } => {
            let items = load_manifests(&manifest)?;
            anyhow::ensure!(!items.is_empty(), "the manifests list no items");
            std::fs::create_dir_all(&out_dir)?;
            let (mut train, mut val, mut counts) = (String::new(), String::new(), [0usize; 2]);
            for it in &items {
                let side = tintype::data::is_val(it, val_pct);
                counts[side as usize] += 1;
                let line = it.manifest_line(&std::path::absolute(&it.path)?.display().to_string()) + "\n";
                if side { val.push_str(&line) } else { train.push_str(&line) }
            }
            std::fs::write(out_dir.join("train.tsv"), train)?;
            std::fs::write(out_dir.join("val.tsv"), val)?;
            println!("{} items on the training side, {} held out ({val_pct}% by path hash)", counts[0], counts[1]);
            Ok(())
        }
        Cmd::AugmentBench { manifest, images, sampler_alpha, rounds, threads, seed, aug } => {
            let cfg = aug.config()?;
            let t = tintype::data::bench_augment(&load_manifests(&manifest)?, &cfg, images, rounds, threads, seed, sampler_alpha)?;
            println!("augmentation: {}", cfg.chain_summary().unwrap_or_else(|| "the phase-6 recipe".into()));
            println!("{} photos, {:.2} megapixels on average, {rounds} views each", t.images, t.megapixels);
            println!("decode   {:.2} ms per photo", t.decode_ms);
            println!("augment  {:.2} ms per view (median {:.2}, 95th percentile {:.2})", t.augment_ms, t.augment_p50_ms, t.augment_p95_ms);
            println!("loader   {:.1} views/s per thread; {:.0} views/s with 11 threads if they scale", t.per_thread(), 11.0 * t.per_thread());
            Ok(())
        }
        Cmd::SetPhotoSecondOpinion { weights, band } => {
            let mut spec = ModelSpec::load_for(&weights)?;
            spec.second_opinion = if band == "none" {
                None
            } else {
                let v: Vec<f32> = band.split(',').map(|x| x.trim().parse::<f32>()).collect::<Result<_, _>>()?;
                anyhow::ensure!(v.len() == 4 && v[0] < v[1], "--band takes lo,hi,weight,threshold with lo < hi");
                Some(tintype::detector::PhotoSecondOpinion { lo: v[0], hi: v[1], weight: v[2], threshold: v[3] })
            };
            spec.save_for(&weights)
        }
        Cmd::Data { plan, dry_run, check, connections } => {
            if plan == "list" {
                for (name, what) in tintype::plans::PLANS {
                    println!("{name:<14} {what}");
                }
                return Ok(());
            }
            tintype::plans::run(&plan, dry_run, check, connections)
        }
        Cmd::Ingest { inputs, out, dataset, image_col, label_col, fake_values, real_values, const_label, gen_col, real_src_col, const_gen, max_per_file } => {
            let spec = ingest::ParquetSpec { image_col, label_col, fake_values, real_values, const_label, gen_col, real_src_col, const_gen, max_per_file };
            let (n, skipped) = ingest::ingest_parquet(&inputs, &out, &dataset, &spec)?;
            println!("{dataset}: wrote {n} images, skipped {skipped} rows -> {}", out.display());
            Ok(())
        }
        Cmd::IndexFolder { root, dataset, rules, video } => {
            let rules = rules
                .iter()
                .map(|r| {
                    let (pat, rest) = r.split_once('=').ok_or_else(|| anyhow::anyhow!("rule must be substring=label:generator"))?;
                    let (label, gen) = rest.split_once(':').ok_or_else(|| anyhow::anyhow!("rule must be substring=label:generator"))?;
                    Ok((pat.to_string(), label.parse()?, gen.to_string()))
                })
                .collect::<Result<Vec<_>>>()?;
            let n = ingest::index_folder(&root, &dataset, &rules, video)?;
            println!("{dataset}: indexed {n} files under {}", root.display());
            Ok(())
        }
        Cmd::CheckVideos { manifest, prune, recompress, compare_decoders } => {
            use rayon::prelude::*;
            #[cfg(target_os = "macos")]
            if let Some(limit) = compare_decoders {
                for m in &manifest {
                    let items = ingest::read_manifest(m)?;
                    let rows: Vec<(PathBuf, Result<tintype::avf::Comparison>, Option<String>)> = items
                        .par_iter()
                        .map(|it| {
                            let c = tintype::avf::compare_with_pure_rust(&it.path, limit);
                            // When the comparison fails, say which reader could not open the clip.
                            let rust_only = c.is_err().then(|| tintype::swdec::open(&it.path, None).err().map(|e| e.to_string())).flatten();
                            (it.path.clone(), c, rust_only)
                        })
                        .collect();
                    let (mut clips, mut exact, mut frames, mut same, mut failed) = (0usize, 0usize, 0usize, 0usize, 0usize);
                    for (p, c, rust_error) in &rows {
                        match c {
                            Ok(c) => {
                                clips += 1;
                                frames += c.frames;
                                same += c.identical;
                                let ok = c.identical == c.frames && c.produced.0 == c.produced.1 && c.max_time_difference < 1e-3 && (c.durations.0 - c.durations.1).abs() < 1e-3;
                                exact += ok as usize;
                                if !ok {
                                    println!(
                                        "{}\tframes {} (system {}, rust {})\tidentical {}\ttag mismatches {}\tmax sample difference {}\tmax time difference {:.4}s\tdurations {:.3} / {:.3}\ttags (bt709, full range) {:?}\tdeclared {:?}\tsize {:?}",
                                        p.display(), c.frames, c.produced.0, c.produced.1, c.identical, c.tag_mismatches, c.max_sample_difference, c.max_time_difference, c.durations.0, c.durations.1, c.tags, c.declared, c.size
                                    );
                                }
                            }
                            Err(e) => {
                                failed += 1;
                                println!("{}\tnot compared: {}", p.display(), rust_error.as_ref().map_or_else(|| format!("system decoder: {e}"), |r| format!("pure-Rust reader: {r}")));
                            }
                        }
                    }
                    println!("{}: {clips} clips compared, {exact} identical in every frame, time and duration; {same} of {frames} frames identical; {failed} not compared", m.display());
                }
                return Ok(());
            }
            #[cfg(not(target_os = "macos"))]
            anyhow::ensure!(compare_decoders.is_none(), "--compare-decoders needs the macOS system decoder to compare against");
            for m in &manifest {
                let items = ingest::read_manifest(m)?;
                let bad: Vec<(PathBuf, String)> = items
                    .par_iter()
                    .filter_map(|it| {
                        let t = std::time::Instant::now();
                        let mut r = tintype::video::middle_frame(&it.path);
                        if recompress && r.is_ok() {
                            let seed = it.path.to_string_lossy().bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3));
                            let mut rng = tintype::rng::Rng::new(seed);
                            r = tintype::video::recompressed_only(&it.path, &mut rng, &std::env::temp_dir());
                        }
                        let secs = t.elapsed().as_secs_f64();
                        match r {
                            Err(e) => Some((it.path.clone(), format!("unreadable: {e}"))),
                            Ok(_) if secs > 5.0 => Some((it.path.clone(), format!("slow: {secs:.0}s"))),
                            Ok(_) => None,
                        }
                    })
                    .collect();
                for (p, why) in &bad {
                    println!("{}\t{why}", p.display());
                }
                println!("{}: {} of {} clips listed", m.display(), bad.len(), items.len());
                if prune && !bad.is_empty() {
                    let root = m.parent().unwrap_or(std::path::Path::new(""));
                    let drop: std::collections::HashSet<PathBuf> = bad.into_iter().map(|(p, _)| p).collect();
                    let text = std::fs::read_to_string(m)?;
                    let kept: Vec<&str> = text.lines().filter(|l| l.split('\t').next().is_some_and(|rel| !drop.contains(&root.join(rel)))).collect();
                    std::fs::write(m, kept.join("\n") + "\n")?;
                }
            }
            Ok(())
        }
        Cmd::Overlap { train, test } => {
            let (tr, te) = (load_manifests(&train)?, load_manifests(&test)?);
            let dups = ingest::exact_overlap(&tr, &te);
            for (a, b) in dups.iter().take(20) {
                println!("{} == {}", a.display(), b.display());
            }
            println!("{} of {} test files are byte-identical to one of {} training files", dups.len(), te.len(), tr.len());
            Ok(())
        }
        Cmd::ParquetInfo { path, rows } => ingest::parquet_info(&path, rows),
        Cmd::FreezeSet { suite, manifest, out, root } => {
            use tintype::integrity::Frozen;
            let mut manifests: Vec<PathBuf> = match &suite {
                Some(s) => tintype::plans::suite(s)?.iter().map(|set| root.join(set.manifest)).collect(),
                None => Vec::new(),
            };
            manifests.extend(manifest);
            let frozen = Frozen::build(&manifests, &root)?;
            frozen.write(&out)?;
            let bytes: u64 = frozen.rows.iter().map(|r| r.size).sum();
            println!("{}: {} files ({:.2} GB) from {} manifests", out.display(), frozen.rows.len(), bytes as f64 / 1e9, frozen.manifests.len());
            Ok(())
        }
        Cmd::VerifySet { frozen, root } => {
            let mut failed = 0;
            for f in &frozen {
                let record = tintype::integrity::Frozen::read(f)?;
                let problems = record.verify(&root);
                for p in &problems {
                    println!("{p}");
                }
                println!("{}: {} files, {} manifests: {}", f.display(), record.rows.len(), record.manifests.len(), if problems.is_empty() { "identical".to_string() } else { format!("{} differences", problems.len()) });
                failed += problems.len();
            }
            anyhow::ensure!(failed == 0, "{failed} differences from the frozen evaluation sets");
            Ok(())
        }
        Cmd::SplitGroups { manifest, out_dir, fractions, seed, together, hold_out_source, hold_out_family, existing } => {
            use tintype::split;
            let items = load_manifests(&manifest)?;
            anyhow::ensure!(!items.is_empty(), "the manifests list no items");
            let opts = split::Options { fractions: split::parse_fractions(&fractions)?, seed, together: split::Key::parse_list(&together)?, hold_out_sources: hold_out_source, hold_out_families: hold_out_family };
            // Paths are written absolute (symbolic links kept), so the sets load from anywhere.
            let abs = |p: &std::path::Path| std::path::absolute(p).map(|a| a.display().to_string());
            let mut pinned = std::collections::BTreeMap::new();
            if let Some(dir) = &existing {
                for (s, name) in split::SETS.iter().enumerate() {
                    for it in ingest::read_manifest(&dir.join(format!("{name}.tsv")))? {
                        pinned.insert(abs(&it.path)?, s);
                    }
                }
            }
            let keyed: Vec<ingest::Item> = items.iter().map(|it| Ok(ingest::Item { path: abs(&it.path)?.into(), ..it.clone() })).collect::<Result<_>>()?;
            let sets = split::assign(&keyed, &opts, &pinned)?;
            std::fs::create_dir_all(&out_dir)?;
            for (s, name) in split::SETS.iter().enumerate() {
                let text: String = keyed.iter().zip(&sets).filter(|(_, &x)| x == s).map(|(it, _)| it.manifest_line(&it.path.display().to_string()) + "\n").collect();
                std::fs::write(out_dir.join(format!("{name}.tsv")), text)?;
            }
            for l in split::summary(&keyed, &sets) {
                println!("{l}");
            }
            // The split is only as good as its grouping columns: say how many items had any.
            let grouped = keyed.iter().filter(|it| it.parent_id().is_some() || it.scene_id().is_some()).count();
            println!("{grouped} of {} items carry a parent_id or scene_id; kept together: {}", keyed.len(), opts.together.iter().map(|k| k.name()).collect::<Vec<_>>().join(", "));
            let named: Vec<(String, Vec<ingest::Item>)> = split::SETS.iter().enumerate().map(|(s, name)| (name.to_string(), keyed.iter().zip(&sets).filter(|(_, &x)| x == s).map(|(it, _)| it.clone()).collect())).collect();
            let mut problems = split::crossings(&named, &opts.together);
            problems.extend(split::hold_out_leaks(&named, &opts.hold_out_sources, &opts.hold_out_families));
            anyhow::ensure!(problems.is_empty(), "the split leaks: {}", problems.join("; "));
            Ok(())
        }
        Cmd::VerifySplit { sets, together, held_out_source, held_out_family } => {
            use tintype::split;
            let keys = split::Key::parse_list(&together)?;
            let mut named: Vec<(String, Vec<ingest::Item>)> = Vec::new();
            let mut hashes: Vec<(String, String, String)> = Vec::new();
            for s in &sets {
                let (name, file) = s.split_once('=').ok_or_else(|| anyhow::anyhow!("use --set name=file"))?;
                let file = std::path::Path::new(file);
                // A freeze-set record carries the hashes too; anything else is read as a manifest.
                let items = match tintype::integrity::Frozen::read(file) {
                    Ok(frozen) => frozen
                        .rows
                        .into_iter()
                        .map(|r| {
                            hashes.push((name.to_string(), r.sha256, r.path.clone()));
                            ingest::Item { path: r.path.into(), label: r.label, generator: r.generator, dataset: r.dataset, extra: r.groups }
                        })
                        .collect(),
                    Err(_) => ingest::read_manifest(file)?,
                };
                match named.iter_mut().find(|(n, _)| n == name) {
                    Some((_, v)) => v.extend(items),
                    None => named.push((name.to_string(), items)),
                }
            }
            let mut problems = split::crossings(&named, &keys);
            problems.extend(split::hold_out_leaks(&named, &held_out_source, &held_out_family));
            problems.extend(split::shared_bytes(&hashes));
            for p in &problems {
                println!("{p}");
            }
            let sizes: Vec<String> = named.iter().map(|(n, v)| format!("{n} {}", v.len())).collect();
            println!("{}: {}", sizes.join(", "), if problems.is_empty() { format!("no group crosses sets ({})", keys.iter().map(|k| k.name()).collect::<Vec<_>>().join(", ")) } else { format!("{} leaks", problems.len()) });
            if hashes.is_empty() {
                println!("note: identical bytes across sets are only checked between freeze-set records");
            }
            anyhow::ensure!(problems.is_empty(), "{} groups or files cross the sets", problems.len());
            Ok(())
        }
        Cmd::Interpolate { a, a_prefix, b, b_prefix, alpha, out_dir, average, weights, out, spec, overwrite } => {
            tintype::interpolate::run(tintype::interpolate::Args { a, a_prefix, b, b_prefix, alphas: alpha, out_dir, average, weights, out, spec, overwrite })
        }
        Cmd::ReplayPool { fit, test, spec, pool, order, target_fpr, rule, min_source, tolerance, margin_step, margin_max, asymmetric_margins, slice_by, write_scores, in_sample } => {
            use tintype::pool::{Refit, Rule};
            let rule = match rule.as_str() {
                "pooled" => Rule::Pooled,
                "balanced" => Rule::Balanced(min_source),
                other => anyhow::bail!("unknown rule `{other}` (pooled, balanced)"),
            };
            anyhow::ensure!(margin_step > 0.0 && margin_max >= 0.0, "--margin-step must be positive and --margin-max not negative");
            anyhow::ensure!(slice_by.as_deref().is_none_or(|s| s == "dataset" || s == "generator"), "--slice-by takes dataset or generator");
            let grid: Vec<f32> = (0..=(margin_max / margin_step).round() as usize).map(|i| i as f32 * margin_step).collect();
            let args = tintype::pool::Args { fit, test, spec, pools: pool, orders: order, target_fpr, rule, refit: Refit { tolerance, grid, symmetric: !asymmetric_margins }, slice_by, write_scores, in_sample };
            tintype::pool::run(args, &mut std::io::stdout().lock())
        }
        Cmd::DumpVideoFeatures { weights, manifest, out, frames, max_seconds, grid, blocks, pairs, temporal_blur, fit_only, device, threads } => {
            anyhow::ensure!(max_seconds.is_finite() && max_seconds >= 0.0 && frames.is_none_or(|n| (1..=256).contains(&n)), "--frames is 1 to 256 and --max-seconds is not negative");
            // Tokens in single precision unless half precision is asked for by name.
            let device = match parse_device(&device)? {
                Device::Gpu if device == "auto" => Device::GpuF32,
                d => d,
            };
            let o = tintype::temporal::DumpOptions { frames, max_seconds, grid, blocks, follow: pairs, blur: temporal_blur.max(1), fit_only, device, threads };
            let m = tintype::temporal::dump(&weights, &manifest, &out, &o)?;
            let bytes = m.frame_records * m.record as u64 * 4;
            println!(
                "{}: {} clips ({} unreadable), {} frame records of {} values ({:.1} MB) in {:.1}s: {:.1} ms per clip end to end, {:.1} ms per clip in the network, {:.1} ms per clip decoding and cropping on {} threads",
                out.display(),
                m.clips,
                m.unreadable,
                m.frame_records,
                m.record,
                bytes as f64 / 1e6,
                m.wall_s,
                1000.0 * m.wall_s / m.clips.max(1) as f64,
                1000.0 * m.model_s / m.clips.max(1) as f64,
                1000.0 * m.decode_s / m.clips.max(1) as f64,
                m.threads
            );
            Ok(())
        }
        Cmd::FitTemporalHead { features, kind, out, watch, base, block, hidden, time_norm, epochs, lr, weight_decay, batch, seed, shuffle_frames, cut_above, cut_ratio } => {
            use tintype::temporal_head as th;
            let set = tintype::temporal::FeatureSet::open(&features)?;
            let watch = watch.map(|w| tintype::temporal::FeatureSet::open(&w)).transpose()?;
            let base = match base.as_str() {
                "frame" => th::Base::Frame,
                "fit" => th::Base::Fit,
                other => anyhow::bail!("unknown base `{other}` (use frame or fit)"),
            };
            anyhow::ensure!(base == th::Base::Fit || set.meta.views.len() > 1, "this dump scored the whole-frame view only: use --base fit");
            let block = block.or(set.meta.blocks.last().copied()).unwrap_or(0);
            let spec = th::HeadSpec { format: th::FORMAT, kind: th::Kind::parse(&kind)?, base, block, grid: set.meta.grid, dim: set.meta.dim, hidden, follow: set.meta.follow, time_norm: th::TimeNorm::parse(&time_norm)?, trained_shuffled: false, cut_rule: th::CutRule { above: cut_above, ratio: cut_ratio }, fit: None };
            let t = std::time::Instant::now();
            let head = th::fit(&set, spec, &th::FitOptions { epochs, lr, weight_decay, batch, seed, shuffle_frames }, watch.as_ref(), |line| println!("{line}"))?;
            head.save(&out)?;
            println!("{}: {} head, {} parameters, fitted on {} clips in {:.1}s", out.display(), head.spec.kind.name(), head.parameters(), set.clips.len(), t.elapsed().as_secs_f64());
            Ok(())
        }
        Cmd::EvalTemporalHead { features, calibrate, head, scores, fpr, shuffle_seed, no_shuffle, out_dir, cut_above, cut_ratio } => {
            use tintype::temporal_head as th;
            anyhow::ensure!(!fpr.is_empty() && fpr.iter().all(|f| (0.0..1.0).contains(f)), "--fpr takes fractions, e.g. 0.01,0.001");
            let set = tintype::temporal::FeatureSet::open(&features)?;
            let calibrate = calibrate.map(|c| tintype::temporal::FeatureSet::open(&c)).transpose()?;
            let mut heads = Vec::new();
            for h in &head {
                let (name, path) = h.split_once('=').ok_or_else(|| anyhow::anyhow!("use --head name=path"))?;
                heads.push((name.to_string(), th::Head::load(std::path::Path::new(path))?));
            }
            let mut external = Vec::new();
            for s in &scores {
                let (name, files) = s.split_once('=').ok_or_else(|| anyhow::anyhow!("use --scores name=test.tsv[,calibrate.tsv]"))?;
                let (test, cal) = match files.split_once(',') {
                    Some((t, c)) => (t, Some(PathBuf::from(c))),
                    None => (files, None),
                };
                external.push(th::External { name: name.to_string(), test: test.into(), calibrate: cal });
            }
            let o = th::EvalOptions { fprs: fpr, shuffle_seed: (!no_shuffle).then_some(shuffle_seed), cut_rule: th::CutRule { above: cut_above, ratio: cut_ratio }, out_dir };
            print!("{}", th::evaluate(&set, calibrate.as_ref(), &heads, &external, &o)?);
            Ok(())
        }
        Cmd::VerifyModels { root } => {
            let (lines, ok) = tintype::integrity::check_models(&root);
            for l in lines {
                println!("{l}");
            }
            anyhow::ensure!(ok, "the model files differ from the released ones");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tintype::data::{AugConfig, ChainOp};

    fn train_aug(extra: &[&str]) -> Result<AugConfig> {
        let mut args = vec!["tintype", "train", "--init", "init.safetensors", "--manifest", "m.tsv", "--out", "runs/x"];
        args.extend_from_slice(extra);
        match Cli::try_parse_from(args)?.cmd {
            Cmd::Train { aug, .. } => aug.config(),
            _ => unreachable!(),
        }
    }

    #[test]
    fn train_without_augmentation_flags_builds_the_phase_6_recipe() {
        // data.rs proves this exact configuration byte-identical to the
        // augmentation before chains existed.
        assert_eq!(train_aug(&[]).unwrap(), AugConfig { p_native: 0.5, p_messenger: 0.25, ..Default::default() });
        assert_eq!(train_aug(&["--p-messenger", "0", "--p-native", "0.3"]).unwrap(), AugConfig { p_native: 0.3, p_messenger: 0.0, ..Default::default() });
    }

    #[test]
    fn chain_flags_reach_the_augmentation_and_bad_values_are_refused() {
        let cfg = train_aug(&["--p-chain", "0.5", "--chain-ops", "1,5", "--chain-severity", "1", "--chain-kinds", "blur,tone", "--chain-post", "0", "--chain-max-pixels", "1000000", "--chain-schedule", "0:0:1:0,2000:0.5:3:0.5", "--messenger-cap", "720,2048", "--messenger-quality", "61,92", "--messenger-cycles", "1,6"]).unwrap();
        assert_eq!((cfg.p_chain, cfg.chain_ops, cfg.chain_severity, cfg.chain_post, cfg.chain_max_pixels), (0.5, (1, 5), 1.0, 0.0, 1_000_000));
        assert_eq!(cfg.chain_kinds, vec![ChainOp::Blur, ChainOp::Tone]);
        assert_eq!(cfg.chain_schedule.len(), 2);
        assert_eq!((cfg.messenger_cap, cfg.messenger_quality, cfg.messenger_cycles), ((720.0, 2048.0), (61.0, 92.0), (1, 6)));
        for bad in [
            vec!["--p-chain", "1.5"],
            vec!["--chain-ops", "3,1"],
            vec!["--chain-ops", "0,2"],
            vec!["--chain-ops", "3"],
            vec!["--chain-severity", "-0.1"],
            vec!["--chain-kinds", "webp"],
            vec!["--chain-post", "2"],
            vec!["--chain-max-pixels", "100"],
            vec!["--chain-schedule", "0:0.5"],
            vec!["--messenger-cap", "2048,720"],
            vec!["--messenger-quality", "55,101"],
            vec!["--messenger-cycles", "0,3"],
        ] {
            assert!(train_aug(&bad).is_err(), "{bad:?}");
        }
    }
}
