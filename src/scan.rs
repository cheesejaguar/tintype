use anyhow::{ensure, Context, Result};
use clap::Args;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use tintype::detector::{self, Detector, Device, ModelSpec};
use tintype::{ingest, video};

const CHUNK: usize = 256;

#[derive(Args)]
pub struct DemoOptions {
    /// Photo weights.
    #[arg(long)]
    weights: Option<PathBuf>,
    /// Write the score and fixture label as JSON.
    #[arg(long)]
    json: bool,
    /// Use the more sensitive photo profile.
    #[arg(long)]
    sensitive: bool,
    /// Inference engine (auto, gpu, gpu-f32, or cpu).
    #[arg(long, default_value = "auto")]
    device: String,
}

pub fn demo(options: DemoOptions) -> Result<()> {
    let weights = resolve_weights(options.weights)?;
    let mut spec = ModelSpec::load_for(&weights)?;
    let profile = if options.sensitive {
        Detector::use_sensitive_profile(&mut spec);
        Profile::Sensitive
    } else {
        Profile::Default
    };
    ensure!(
        !spec.views.is_empty(),
        "photo model must declare at least one view"
    );
    let image = tintype::image_io::decode(include_bytes!("../site/assets/demo.png"))?;
    let detector = Detector::load(
        &weights,
        Some(spec.clone()),
        spec.views.len(),
        super::parse_device(&options.device)?,
    )?;
    let results = detector.score_images(&[image]);
    let views = &results[0];
    let evaluated;
    let logits = if let Some(exit) = spec.early_exit.filter(|exit| exit.covers(views.len())) {
        evaluated = exit.looked_at(views, detector.threshold_logit());
        &evaluated
    } else {
        views
    };
    let mut row = Row::empty(Path::new("demo.png"), "image", profile.name());
    row.set_score(
        detector.probability(logits),
        mean(logits),
        detector.threshold_logit(),
    );
    row.views = Some(logits.len());
    ensure!(
        row.score.is_some(),
        "the demo model returned a non-finite score"
    );
    row.fixture_label = Some("ai-generated");
    let mut output = io::stdout().lock();
    if !options.json {
        writeln!(output, "Included sample • AI-generated (known label)")?;
    }
    row.write(&mut output, options.json)
}

#[derive(Args)]
pub struct ScanOptions {
    /// Photo weights. Defaults to the installed models, or models/ in a source checkout.
    #[arg(long)]
    weights: Option<PathBuf>,
    /// Video weights. Defaults to the sibling checkpoint if present.
    #[arg(long)]
    video_weights: Option<PathBuf>,
    /// Frames scored per video (default: the model profile).
    #[arg(long, value_parser = parse_frames)]
    frames: Option<usize>,
    /// Two views per image at the strict threshold, for higher throughput.
    #[arg(long)]
    fast: bool,
    /// Find more AI images, with more false alarms.
    #[arg(long, conflicts_with = "fast")]
    sensitive: bool,
    /// Flag fewer real images, with lower recall.
    #[arg(long, conflicts_with_all = ["fast", "sensitive"])]
    strict: bool,
    /// Score every photo view and every sampled frame.
    #[arg(long)]
    all_views: bool,
    /// Spread samples over the first N seconds of video. Use 0 for the whole clip.
    #[arg(long, default_value_t = 30.0, value_parser = parse_seconds)]
    max_seconds: f64,
    /// Write one JSON object per file.
    #[arg(long)]
    json: bool,
    /// Inference engine (auto, gpu, gpu-f32, or cpu).
    #[arg(long, default_value = "auto")]
    device: String,
}

fn parse_frames(value: &str) -> Result<usize, String> {
    value
        .parse()
        .ok()
        .filter(|n| (1..=256).contains(n))
        .ok_or_else(|| "expected an integer between 1 and 256".into())
}

fn parse_seconds(value: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite() && *n >= 0.0)
        .ok_or_else(|| "expected a finite, non-negative number of seconds".into())
}

pub fn resolve_weights(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    // An empty value counts as unset, as PowerShell and some service managers
    // define variables with empty values rather than removing them.
    if let Some(dir) = std::env::var_os("TINTYPE_MODEL_DIR").filter(|d| !d.is_empty()) {
        let path = PathBuf::from(dir).join("tintype.safetensors");
        ensure!(
            path.is_file(),
            "TINTYPE_MODEL_DIR does not contain {}",
            path.display()
        );
        return Ok(path);
    }
    let executable = std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok());
    if let Some(bin) = executable.as_deref().and_then(Path::parent) {
        for dir in [bin.join("models"), bin.join("../share/tintype/models")] {
            let path = dir.join("tintype.safetensors");
            if path.is_file() {
                return Ok(path);
            }
        }
    }
    let checkout = PathBuf::from("models/tintype.safetensors");
    ensure!(checkout.is_file(), "could not find the Tintype photo weights. Set --weights or TINTYPE_MODEL_DIR, or install a release bundle");
    Ok(checkout)
}

#[derive(Clone, Copy)]
enum Profile {
    Default,
    Fast,
    Strict,
    Sensitive,
}

impl Profile {
    fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Fast => "fast",
            Self::Strict => "strict",
            Self::Sensitive => "sensitive",
        }
    }
}

struct Scan {
    options: ScanOptions,
    profile: Profile,
    device: Device,
    threads: usize,
    photo: Option<Detector>,
    video: Option<Detector>,
    video_frames: usize,
    photo_second: Option<PathBuf>,
    video_second: Option<PathBuf>,
}

impl Scan {
    fn new(
        options: ScanOptions,
        photo_second: Option<PathBuf>,
        video_second: Option<PathBuf>,
    ) -> Result<Self> {
        #[cfg(not(target_os = "macos"))]
        ensure!(
            video_second.is_none(),
            "--second-opinion is available on macOS"
        );
        let profile = if options.fast {
            Profile::Fast
        } else if options.strict {
            Profile::Strict
        } else if options.sensitive {
            Profile::Sensitive
        } else {
            Profile::Default
        };
        let device = super::parse_device(&options.device)?;
        let threads = std::thread::available_parallelism().map_or(4, usize::from);
        Ok(Self {
            options,
            profile,
            device,
            threads,
            photo: None,
            video: None,
            video_frames: 0,
            photo_second,
            video_second,
        })
    }

    fn photos(&mut self, paths: &[PathBuf], warm: bool) -> Result<Vec<Row>> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        if self.photo.is_none() {
            let weights = resolve_weights(self.options.weights.clone())?;
            let mut spec = ModelSpec::load_for(&weights)?;
            match self.profile {
                Profile::Default => {}
                Profile::Fast => Detector::use_fast_profile(&mut spec),
                Profile::Strict => Detector::use_strict_profile(&mut spec),
                Profile::Sensitive => Detector::use_sensitive_profile(&mut spec),
            }
            ensure!(
                !spec.views.is_empty(),
                "photo model must declare at least one view"
            );
            let batch = if warm {
                64
            } else {
                64.min(paths.len().saturating_mul(spec.views.len()))
            };
            self.photo = Some(Detector::load(&weights, Some(spec), batch, self.device)?);
        }
        let det = self.photo.as_ref().unwrap();
        let exit = det.spec.early_exit.as_ref().filter(|e| {
            !self.options.fast && !self.options.all_views && e.covers(det.spec.views.len())
        });
        let threads = self.threads.min(paths.len());
        let (logits, _) = match exit {
            Some(e)
                if !warm
                    && !matches!(self.device, Device::Cpu)
                    && paths.len() <= 64 / det.spec.views.len() =>
            {
                let (all, timing) = det.score_files(paths, threads);
                let seen = all
                    .into_iter()
                    .map(|r| r.map(|l| e.looked_at(&l, det.threshold_logit())))
                    .collect();
                (seen, timing)
            }
            Some(e) => det.score_files_adaptive(paths, threads, e),
            None => det.score_files(paths, threads),
        };
        let mut rows: Vec<_> = paths
            .iter()
            .zip(&logits)
            .map(|(path, result)| {
                let mut row = Row::empty(path, "image", self.profile.name());
                if let Some(values) = result {
                    row.set_score(det.probability(values), mean(values), det.threshold_logit());
                    row.views = Some(values.len());
                } else {
                    row.error = Some(decode_error(tintype::image_io::load(path)));
                }
                row
            })
            .collect();
        if let Some(weights) = &self.photo_second {
            let band = det
                .spec
                .second_opinion
                .context("the photo model has no second-opinion band")?;
            let first: Vec<_> = logits
                .iter()
                .map(|r| r.as_ref().map(|l| det.calibrated_logit(l)))
                .collect();
            self.photo = None;
            for (i, score) in
                detector::photo_second_opinion(&band, weights, paths, &first, self.device, threads)?
                    .0
            {
                if score.is_finite() {
                    rows[i].score = Some(score);
                    rows[i].second_opinion = true;
                }
            }
        }
        Ok(rows)
    }

    fn videos(&mut self, paths: &[PathBuf]) -> Result<Vec<Row>> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        if self.video.is_none() {
            let weights = match &self.options.video_weights {
                Some(path) => path.clone(),
                None => {
                    detector::video_weights_for(&resolve_weights(self.options.weights.clone())?)
                }
            };
            let spec = ModelSpec::load_for(&weights)?;
            let batch = if self.video_second.is_none()
                && !self.options.all_views
                && spec.video.as_ref().is_some_and(|v| v.early_exit.is_some())
            {
                96
            } else {
                64
            };
            let mut det = Detector::load(&weights, Some(spec), batch, self.device)?;
            self.video_frames = det.use_video_profile();
            ensure!(
                !det.spec.views.is_empty(),
                "video model must declare at least one view"
            );
            self.video = Some(det);
        }
        let det = self.video.as_ref().unwrap();
        let frames = self.options.frames.unwrap_or(self.video_frames);
        ensure!(
            (1..=256).contains(&frames),
            "video profile must request between 1 and 256 frames"
        );
        let sampling = video::Sampling {
            frames,
            max_seconds: self.options.max_seconds,
            consecutive: false,
        };
        let exit = det
            .spec
            .video
            .as_ref()
            .and_then(|v| v.early_exit.as_ref())
            .filter(|_| {
                self.video_second.is_none()
                    && !self.options.all_views
                    && frames == self.video_frames
            });
        let threads = self
            .threads
            .min(paths.len())
            .min(if frames > 32 { 1 } else { 4 });
        let (results, _) = det.score_videos_with_coverage(paths, sampling, threads, exit);
        #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
        let mut rows: Vec<_> = paths
            .iter()
            .zip(results)
            .map(|(path, result)| {
                let mut row = Row::empty(path, "video", "video");
                if let Some(result) = result {
                    let logit = video::aggregate(
                        &result
                            .frame_logits
                            .iter()
                            .map(|v| mean(v))
                            .collect::<Vec<_>>(),
                    );
                    row.set_score(det.probability(&[logit]), logit, det.threshold_logit());
                    row.coverage = Some(result.coverage);
                } else {
                    row.error = Some(decode_error(video::frame_at(path, 0.0)));
                }
                row
            })
            .collect();
        #[cfg(target_os = "macos")]
        if let Some(weights) = &self.video_second {
            let band = det
                .spec
                .video
                .as_ref()
                .and_then(|v| v.second_opinion)
                .context("the video model has no second-opinion band")?;
            let pick: Vec<_> = rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    row.coverage.is_some_and(|c| c.complete)
                        && row.logit.is_some_and(|z| band.wanted(z))
                })
                .map(|(i, row)| (i, row.path.clone()))
                .collect();
            if !pick.is_empty() {
                self.video = None;
                let paths: Vec<_> = pick.iter().map(|(_, path)| path.clone()).collect();
                let (second, _) = tintype::eval::waverep_logits(weights, &paths, self.device, 16)?;
                for ((i, _), second) in pick.into_iter().zip(second) {
                    if let Some(z) = second {
                        rows[i].score = Some(band.probability(rows[i].logit.unwrap(), z));
                        rows[i].second_opinion = true;
                    }
                }
            }
        }
        Ok(rows)
    }

    fn run_chunk(
        &mut self,
        inputs: Vec<Input>,
        warm: bool,
        output: &mut impl Write,
        counts: &mut Counts,
    ) -> Result<()> {
        let (mut photos, mut videos) = (Vec::new(), Vec::new());
        for input in &inputs {
            match input {
                Input::Media(path) if ingest::is_video(path) => videos.push(path.clone()),
                Input::Media(path) => photos.push(path.clone()),
                Input::Problem(..) => {}
            }
        }
        if !warm && !matches!(self.profile, Profile::Default) && !videos.is_empty() {
            anyhow::bail!(
                "--fast, --sensitive and --strict apply to photos only; run videos separately"
            );
        }
        let mut photo_rows = self.photos(&photos, warm)?.into_iter();
        let mut video_rows = if matches!(self.profile, Profile::Default) {
            self.videos(&videos)?
        } else {
            Vec::new()
        }
        .into_iter();
        for input in inputs {
            let row = match input {
                Input::Media(path) if ingest::is_video(&path) => {
                    video_rows.next().unwrap_or_else(|| {
                        Row::failed(path, "video", "the requested profile applies to photos only; scan this clip with the default profile".into())
                    })
                }
                Input::Media(_) => photo_rows.next().unwrap(),
                Input::Problem(path, error) => Row::failed(path, "path", error),
            };
            counts.total += 1;
            counts.failed += usize::from(row.score.is_none());
            row.write(output, self.options.json)?;
        }
        output.flush()?;
        Ok(())
    }
}

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

fn decode_error<T>(r: Result<T>) -> String {
    r.err()
        .map_or_else(|| "could not decode".into(), |e| format!("{e:#}"))
}

#[derive(Default)]
struct Counts {
    total: usize,
    failed: usize,
}

impl Counts {
    fn finish(self) -> Result<()> {
        if self.failed > 0 {
            eprintln!(
                "{} of {} inputs could not be scored",
                self.failed, self.total
            );
            std::process::exit(2);
        }
        Ok(())
    }
}

enum Input {
    Media(PathBuf),
    Problem(PathBuf, String),
}

pub fn detect(
    paths: Vec<PathBuf>,
    options: ScanOptions,
    photo_second: Option<PathBuf>,
    video_second: Option<PathBuf>,
) -> Result<()> {
    let mut scan = Scan::new(options, photo_second, video_second)?;
    let mut counts = Counts::default();
    let mut output = BufWriter::new(io::stdout().lock());
    scan.run_chunk(
        paths.into_iter().map(Input::Media).collect(),
        false,
        &mut output,
        &mut counts,
    )?;
    counts.finish()
}

pub fn batch(roots: Vec<PathBuf>, options: ScanOptions) -> Result<()> {
    let mut scan = Scan::new(options, None, None)?;
    let mut counts = Counts::default();
    let mut output = BufWriter::new(io::stdout().lock());
    let mut inputs = Vec::with_capacity(CHUNK);
    for root in roots {
        // Explicit file paths are attempted even with an unusual suffix.
        // Recursive scans match known media extensions and skip symlink entries.
        for entry in walkdir::WalkDir::new(&root)
            .follow_links(false)
            .follow_root_links(false)
            .sort_by_file_name()
        {
            let input = match entry {
                Ok(e)
                    if e.file_type().is_file()
                        && (e.depth() == 0
                            || ingest::is_image(e.path())
                            || ingest::is_video(e.path())) =>
                {
                    Input::Media(e.into_path())
                }
                Ok(e) if e.depth() == 0 && !e.file_type().is_dir() => {
                    Input::Problem(e.into_path(), "expected a regular file or directory".into())
                }
                Ok(_) => continue,
                Err(e) => Input::Problem(e.path().unwrap_or(&root).to_owned(), e.to_string()),
            };
            inputs.push(input);
            if inputs.len() == CHUNK {
                scan.run_chunk(std::mem::take(&mut inputs), true, &mut output, &mut counts)?;
            }
        }
    }
    scan.run_chunk(inputs, true, &mut output, &mut counts)?;
    ensure!(counts.total > 0, "no supported media found");
    counts.finish()
}

/// One file's result. When a second opinion contributes, the logit and
/// threshold still describe the primary model and the score describes the blend.
struct Row {
    path: PathBuf,
    kind: &'static str,
    score: Option<f32>,
    logit: Option<f32>,
    threshold_logit: Option<f32>,
    profile: &'static str,
    second_opinion: bool,
    views: Option<usize>,
    coverage: Option<video::Coverage>,
    fixture_label: Option<&'static str>,
    error: Option<String>,
}

impl Row {
    fn empty(path: &Path, kind: &'static str, profile: &'static str) -> Self {
        Self {
            path: path.to_owned(),
            kind,
            score: None,
            logit: None,
            threshold_logit: None,
            profile,
            second_opinion: false,
            views: None,
            coverage: None,
            fixture_label: None,
            error: None,
        }
    }

    fn failed(path: PathBuf, kind: &'static str, error: String) -> Self {
        let mut row = Self::empty(&path, kind, "");
        row.error = Some(error);
        row
    }

    fn set_score(&mut self, score: f32, logit: f32, threshold: f32) {
        if score.is_finite() && logit.is_finite() && threshold.is_finite() {
            self.score = Some(score);
            self.logit = Some(logit);
            self.threshold_logit = Some(threshold);
        } else {
            self.error = Some("model returned a non-finite score".into());
        }
    }

    fn write(&self, out: &mut impl Write, json: bool) -> Result<()> {
        let path = self.path.to_string_lossy();
        let Some(score) = self.score else {
            let error = self.error.as_deref().unwrap_or("could not decode");
            if json {
                writeln!(
                    out,
                    "{}",
                    serde_json::json!({"path": path, "kind": self.kind, "error": error})
                )?;
            } else {
                writeln!(out, "{:<14} {:>5}  {path}  [{error}]", "unreadable", "-")?;
            }
            return Ok(());
        };
        let (verdict, label) = match self.coverage {
            Some(c) if !c.complete => ("inconclusive", "INCONCLUSIVE"),
            _ if score > 0.5 => ("ai-generated", "AI-GENERATED"),
            _ => ("likely-real", "likely real"),
        };
        if json {
            let mut object = serde_json::json!({
                "path": path, "kind": self.kind, "verdict": verdict, "score": score,
                "logit": self.logit, "threshold_logit": self.threshold_logit,
                "profile": self.profile, "second_opinion": self.second_opinion
            });
            if let Some(label) = self.fixture_label {
                object["fixture_label"] = label.into();
            }
            if let Some(views) = self.views {
                object["views"] = views.into();
            }
            if let Some(c) = self.coverage {
                object["frames"] = c.frames.into();
                object["frames_expected"] = c.frames_expected.into();
                object["duration_s"] = c.duration.into();
                object["analysed_until_s"] = c.until.into();
                object["complete"] = c.complete.into();
            }
            writeln!(out, "{object}")?;
            return Ok(());
        }
        write!(out, "{label:<14} {score:>5.3}  {path}")?;
        if let Some(c) = self.coverage {
            write!(out, "  [")?;
            if let (Some(until), Some(duration)) = (c.until, c.duration) {
                write!(out, "sampled through {until:.1} s of {duration:.1} s, ")?;
            }
            if c.complete {
                write!(out, "{} frames]", c.frames)?;
            } else {
                write!(
                    out,
                    "decoding stopped after {} of {} frames]",
                    c.frames, c.frames_expected
                )?;
            }
        }
        writeln!(out)?;
        Ok(())
    }
}
