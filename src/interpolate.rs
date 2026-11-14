//! Weight-space interpolation and averaging of compatible checkpoints, as an
//! offline experiment (`tintype interpolate`).
//!
//! Two uses, both of which only read and write safetensors and so run on
//! every platform:
//!
//! - a sweep `(1 - alpha) * A + alpha * B` between a fine-tuned checkpoint
//!   and the checkpoint it was initialised from (WiSE-FT), to ask whether
//!   moving part of the way back recovers what fine-tuning lost;
//! - a weighted average of several *sibling* fine-tunes of one initialiser
//!   (a "model soup"). Averaging is only meaningful between checkpoints that
//!   started from the same weights: two networks trained from different
//!   initialisations do not share a basin, and their average is noise.
//!
//! Nothing written here is a finished model. Every output gets a spec whose
//! calibration is reset to the identity and marked as unfitted
//! ([`ModelSpec::uncalibrated`]), with the early exit, the strict and
//! sensitive profiles and the second-opinion band removed: all of those were
//! fitted to other weights. See `docs/runbooks/interpolation-pooling.md`.
use crate::detector::{ModelSpec, Profile, VideoSpec};
use anyhow::{bail, ensure, Context, Result};
use candle_core::{DType, Device, Tensor};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// One tensor of a checkpoint. `data` is empty unless the tensor is F32,
/// the only type the arithmetic is defined for here.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub dtype: DType,
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
}

/// A checkpoint's tensors by name, with the source's prefix removed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Checkpoint(pub BTreeMap<String, Entry>);

impl Checkpoint {
    /// Read every tensor of a safetensors file with its type and shape.
    /// `prefix` is removed from the names that have it (`vit.` for the
    /// Community Forensics release), as [`crate::weights::Weights::load`]
    /// does; two tensors that end up with one name are an error.
    pub fn read(path: &Path, prefix: &str) -> Result<Self> {
        let mut out = BTreeMap::new();
        for (name, t) in candle_core::safetensors::load(path, &Device::Cpu).with_context(|| format!("reading {}", path.display()))? {
            let dtype = t.dtype();
            let data = if dtype == DType::F32 { t.flatten_all()?.to_vec1::<f32>()? } else { Vec::new() };
            let short = name.strip_prefix(prefix).unwrap_or(&name).to_string();
            ensure!(out.insert(short.clone(), Entry { dtype, shape: t.dims().to_vec(), data }).is_none(), "{}: two tensors are called `{short}` once the prefix `{prefix}` is removed", path.display());
        }
        ensure!(!out.is_empty(), "{} holds no tensors", path.display());
        Ok(Self(out))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let tensors: HashMap<String, Tensor> = self.0.iter().map(|(k, e)| Ok((k.clone(), Tensor::from_slice(&e.data, e.shape.as_slice(), &Device::Cpu)?))).collect::<Result<_>>()?;
        candle_core::safetensors::save(&tensors, path)?;
        Ok(())
    }

    /// Whether both hold the same tensors with the same bits (so `-0.0` and
    /// `0.0`, or two NaNs with different payloads, count as different).
    pub fn same_bits(&self, other: &Checkpoint) -> bool {
        self.0.len() == other.0.len()
            && self.0.iter().zip(&other.0).all(|((ka, a), (kb, b))| ka == kb && a.dtype == b.dtype && a.shape == b.shape && a.data.len() == b.data.len() && a.data.iter().zip(&b.data).all(|(x, y)| x.to_bits() == y.to_bits()))
    }
}

/// Every way `other` differs from `reference` in layout: a tensor only one
/// of them has, a different shape, a different type, or a type other than
/// F32. Empty means the two can be combined. One line per tensor, in name
/// order.
pub fn layout_differences(reference: &Checkpoint, other: &Checkpoint) -> Vec<String> {
    let mut out = Vec::new();
    for (name, a) in &reference.0 {
        match other.0.get(name) {
            None => out.push(format!("`{name}` is missing from the second checkpoint")),
            Some(b) if a.shape != b.shape => out.push(format!("`{name}` has shape {:?} in the first checkpoint and {:?} in the second", a.shape, b.shape)),
            Some(b) if a.dtype != b.dtype => out.push(format!("`{name}` is {:?} in the first checkpoint and {:?} in the second", a.dtype, b.dtype)),
            Some(_) if a.dtype != DType::F32 => out.push(format!("`{name}` is {:?}; only F32 tensors can be combined", a.dtype)),
            Some(_) => {}
        }
    }
    for name in other.0.keys().filter(|n| !reference.0.contains_key(*n)) {
        out.push(format!("`{name}` is missing from the first checkpoint"));
    }
    out
}

/// `sum_i weights[i] * sources[i]`, tensor by tensor.
///
/// The sum is taken in double precision and rounded once to F32. A source
/// whose weight is exactly 0 is left out of the sum, and a single remaining
/// source with weight exactly 1 is copied: so the end points of a sweep
/// reproduce their source bit for bit (including negative zeros, which
/// `1 * x + 0 * y` would not keep).
///
/// The weights must be non-negative and add up to 1: anything else rescales
/// the network instead of averaging it. Sources whose layouts differ are an
/// error that lists every differing tensor.
pub fn combine(sources: &[&Checkpoint], weights: &[f64]) -> Result<Checkpoint> {
    ensure!(!sources.is_empty() && sources.len() == weights.len(), "need one weight per checkpoint ({} checkpoints, {} weights)", sources.len(), weights.len());
    ensure!(weights.iter().all(|w| w.is_finite() && *w >= 0.0), "weights must be non-negative numbers");
    let total: f64 = weights.iter().sum();
    ensure!((total - 1.0).abs() <= 1e-6, "weights must add up to 1 (they add up to {total})");
    for (i, other) in sources.iter().enumerate().skip(1) {
        let diffs = layout_differences(sources[0], other);
        if !diffs.is_empty() {
            bail!("checkpoint {} cannot be combined with checkpoint 1: {} differences in tensor name, shape or type\n  {}", i + 1, diffs.len(), diffs.join("\n  "));
        }
    }
    let diffs = layout_differences(sources[0], sources[0]);
    ensure!(diffs.is_empty(), "the checkpoint cannot be combined: {}", diffs.join("; "));
    let used: Vec<(&Checkpoint, f64)> = sources.iter().copied().zip(weights.iter().copied()).filter(|(_, w)| *w != 0.0).collect();
    if let [(only, w)] = used[..] {
        if w == 1.0 {
            return Ok(only.clone());
        }
    }
    let mut out = BTreeMap::new();
    for (name, first) in &sources[0].0 {
        let mut acc = vec![0f64; first.data.len()];
        for (src, w) in &used {
            for (a, &x) in acc.iter_mut().zip(&src.0[name].data) {
                *a += w * x as f64;
            }
        }
        out.insert(name.clone(), Entry { dtype: DType::F32, shape: first.shape.clone(), data: acc.into_iter().map(|x| x as f32).collect() });
    }
    Ok(Checkpoint(out))
}

/// The spec written beside combined weights: the views of `source` (so that
/// every candidate of a sweep is preprocessed the same way), with every
/// fitted number removed. The calibration of the photo, fast and video
/// profiles is the identity and each is listed in
/// [`ModelSpec::uncalibrated`]; the strict and sensitive profiles (which are
/// nothing but calibrations), the early exit (margins fitted to other
/// weights) and the second-opinion bands (defined on calibrated logits) are
/// dropped. The tensor names are written without a prefix.
pub fn unfitted_spec(source: &ModelSpec) -> ModelSpec {
    let mut unfitted = vec!["photo".to_string()];
    let fast = source.fast.as_ref().map(|f| Profile { views: f.views.clone(), logit_scale: 1.0, logit_bias: 0.0 });
    let video = source.video.as_ref().map(|v| VideoSpec { frames: v.frames, views: v.views.clone(), logit_scale: 1.0, logit_bias: 0.0, second_opinion: None, early_exit: None });
    unfitted.extend(fast.as_ref().map(|_| "fast".to_string()));
    unfitted.extend(video.as_ref().map(|_| "video".to_string()));
    ModelSpec {
        img: source.img,
        views: source.views.clone(),
        logit_scale: 1.0,
        logit_bias: 0.0,
        threshold: 0.5,
        prefix: String::new(),
        video,
        fast,
        sensitive: None,
        strict: None,
        early_exit: None,
        second_opinion: None,
        arch: source.arch.clone(),
        uncalibrated: Some(unfitted),
    }
}

/// The directory `path` would be written into, with symbolic links
/// resolved as far as the directory exists yet.
fn resolved_dir(path: &Path) -> Result<PathBuf> {
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut existing = std::path::absolute(dir)?;
    let mut rest = Vec::new();
    while !existing.exists() {
        match (existing.file_name().map(|n| n.to_os_string()), existing.parent().map(Path::to_path_buf)) {
            (Some(name), Some(parent)) => {
                rest.push(name);
                existing = parent;
            }
            _ => break,
        }
    }
    let mut out = existing.canonicalize().unwrap_or(existing);
    out.extend(rest.into_iter().rev());
    Ok(out)
}

/// Refuse an output that could replace a released model or an input: any
/// file in a directory called `models`, or anywhere below a `models`
/// directory that holds `tintype.safetensors` (the released weights and
/// specs live in `models/`, and are verified against digests compiled into
/// the binary); a path that is one of `inputs`; and, unless `overwrite`, any
/// path that already exists. Symbolic links in the directory are resolved
/// first, so a link into `models/` is refused too.
pub fn check_output(path: &Path, inputs: &[PathBuf], overwrite: bool) -> Result<()> {
    ensure!(path.extension().is_some_and(|e| e == "safetensors"), "{}: the output must end in .safetensors", path.display());
    let dir = resolved_dir(path)?;
    let released = dir.ancestors().find(|d| d.file_name().is_some_and(|n| n == "models") && (*d == dir || d.join("tintype.safetensors").exists()));
    if let Some(models) = released {
        bail!("refusing to write {}: {} is a models directory, where the released weights live. Write under runs/ instead", path.display(), models.display());
    }
    let name = path.file_name().context("the output has no file name")?;
    let target = dir.join(name);
    for input in inputs {
        let same = input.canonicalize().is_ok_and(|i| i == target || target.canonicalize().is_ok_and(|t| t == i));
        ensure!(!same, "refusing to write {}: it is one of the inputs", path.display());
    }
    for p in [path.to_path_buf(), ModelSpec::sidecar(path), record_path(path)] {
        // `symlink_metadata` also sees a dangling link, which `exists` does not.
        ensure!(overwrite || p.symlink_metadata().is_err(), "{} already exists (pass --overwrite to replace files outside models/)", p.display());
    }
    Ok(())
}

/// Where the record of how `weights` was made is written.
pub fn record_path(weights: &Path) -> PathBuf {
    weights.with_extension("interpolation.json")
}

/// One input of a combination, as it is recorded beside the output.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Source {
    pub path: String,
    pub sha256: String,
    pub prefix: String,
    pub weight: f64,
}

#[derive(Clone, Debug, serde::Serialize)]
struct Record<'a> {
    made_by: String,
    sources: &'a [Source],
    tensors: usize,
    spec_copied_from: String,
    calibration: &'static str,
}

/// Write `combined` to `out` with its unfitted spec and a record of its
/// sources, then read the weights back and check that the file holds exactly
/// the bits that were computed. The three files are written under temporary
/// names first and renamed once all are complete.
pub fn write_output(out: &Path, combined: &Checkpoint, spec: &ModelSpec, sources: &[Source], spec_from: &Path) -> Result<()> {
    let dir = out.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;
    let record = Record {
        made_by: format!("tintype interpolate {}", env!("CARGO_PKG_VERSION")),
        sources,
        tensors: combined.0.len(),
        spec_copied_from: spec_from.display().to_string(),
        calibration: "unfitted: identity calibration, no early exit, no strict or sensitive profile, no second-opinion band. Fit with `tintype calibrate` on separate calibration data before reading any verdict.",
    };
    let finals = [out.to_path_buf(), ModelSpec::sidecar(out), record_path(out)];
    let temporary = |p: &Path| p.with_file_name(format!(".{}.tmp-{}", p.file_name().unwrap_or_default().to_string_lossy(), std::process::id()));
    let write_all = || -> Result<()> {
        combined.save(&temporary(&finals[0]))?;
        ensure!(Checkpoint::read(&temporary(&finals[0]), "")?.same_bits(combined), "{}: the file read back differs from what was computed", out.display());
        std::fs::write(temporary(&finals[1]), serde_json::to_string_pretty(spec)?)?;
        std::fs::write(temporary(&finals[2]), serde_json::to_string_pretty(&record)? + "\n")?;
        // The weights go last: a spec or record without weights is harmless, weights without a spec are not.
        for p in finals.iter().rev() {
            std::fs::rename(temporary(p), p).with_context(|| format!("writing {}", p.display()))?;
        }
        Ok(())
    };
    let result = write_all();
    if result.is_err() {
        for p in &finals {
            let _ = std::fs::remove_file(temporary(p));
        }
    }
    result
}

/// File name of one step of a sweep: `alpha-0.10.safetensors`. Two decimals
/// when that is exact, four otherwise.
pub fn alpha_name(alpha: f64) -> String {
    if ((alpha * 100.0).round() - alpha * 100.0).abs() < 1e-9 {
        format!("alpha-{alpha:.2}.safetensors")
    } else {
        format!("alpha-{alpha:.4}.safetensors")
    }
}

/// What `tintype interpolate` was asked to do.
pub struct Args {
    /// Sweep: the fine-tuned checkpoint (`alpha` 0).
    pub a: Option<PathBuf>,
    pub a_prefix: String,
    /// Sweep: the checkpoint to move towards (`alpha` 1).
    pub b: Option<PathBuf>,
    pub b_prefix: String,
    pub alphas: Vec<f64>,
    pub out_dir: Option<PathBuf>,
    /// Averaging: sibling fine-tunes.
    pub average: Vec<PathBuf>,
    /// Averaging: one weight per checkpoint (default: equal).
    pub weights: Vec<f64>,
    pub out: Option<PathBuf>,
    /// The spec whose views the outputs get (default: the sidecar of `--a`,
    /// or of the first `--average` checkpoint).
    pub spec: Option<PathBuf>,
    pub overwrite: bool,
}

fn source(path: &Path, prefix: &str, weight: f64) -> Result<Source> {
    Ok(Source { path: path.display().to_string(), sha256: crate::integrity::sha256_file(path)?, prefix: prefix.to_string(), weight })
}

pub fn run(args: Args) -> Result<()> {
    if !args.average.is_empty() {
        ensure!(args.a.is_none() && args.b.is_none() && args.alphas.is_empty() && args.out_dir.is_none(), "use either --a/--b/--alpha/--out-dir (a sweep) or --average/--out (an average), not both");
        ensure!(args.average.len() >= 2, "--average takes at least two checkpoints");
        let out = args.out.context("--average needs --out, the file to write")?;
        let n = args.average.len();
        let weights = if args.weights.is_empty() { vec![1.0 / n as f64; n] } else { args.weights.clone() };
        ensure!(weights.len() == n, "--weights needs one weight per checkpoint ({n} checkpoints, {} weights)", weights.len());
        check_output(&out, &args.average, args.overwrite)?;
        let spec_from = args.spec.clone().unwrap_or_else(|| ModelSpec::sidecar(&args.average[0]));
        let spec = unfitted_spec(&serde_json::from_str(&std::fs::read_to_string(&spec_from).with_context(|| format!("reading the spec {} (pass --spec to name another)", spec_from.display()))?)?);
        let loaded: Vec<Checkpoint> = args.average.iter().map(|p| Checkpoint::read(p, "")).collect::<Result<_>>()?;
        let combined = combine(&loaded.iter().collect::<Vec<_>>(), &weights)?;
        let sources: Vec<Source> = args.average.iter().zip(&weights).map(|(p, &w)| source(p, "", w)).collect::<Result<_>>()?;
        write_output(&out, &combined, &spec, &sources, &spec_from)?;
        println!("{}: weighted average of {n} checkpoints ({} tensors, weights {})", out.display(), combined.0.len(), weights.iter().map(|w| format!("{w}")).collect::<Vec<_>>().join(", "));
        println!("Averaging is only meaningful between sibling fine-tunes of one initialiser.");
        println!("The spec beside it is uncalibrated: fit it with `tintype calibrate --weights {}` before reading any verdict.", out.display());
        return Ok(());
    }
    let (Some(a), Some(b), Some(out_dir)) = (args.a, args.b, args.out_dir) else { bail!("a sweep needs --a, --b, --alpha and --out-dir; an average needs --average and --out") };
    ensure!(args.out.is_none() && args.weights.is_empty(), "--out and --weights go with --average; a sweep writes alpha-*.safetensors into --out-dir");
    ensure!(!args.alphas.is_empty(), "--alpha needs at least one value, e.g. 0,0.25,0.5,0.75,1");
    ensure!(args.alphas.iter().all(|x| x.is_finite() && (0.0..=1.0).contains(x)), "every --alpha must lie between 0 and 1");
    let outs: Vec<PathBuf> = args.alphas.iter().map(|&x| out_dir.join(alpha_name(x))).collect();
    for (i, o) in outs.iter().enumerate() {
        ensure!(!outs[..i].contains(o), "--alpha lists {} twice", args.alphas[i]);
        check_output(o, &[a.clone(), b.clone()], args.overwrite)?;
    }
    let spec_from = args.spec.clone().unwrap_or_else(|| ModelSpec::sidecar(&a));
    let spec = unfitted_spec(&serde_json::from_str(&std::fs::read_to_string(&spec_from).with_context(|| format!("reading the spec {} (pass --spec to name another)", spec_from.display()))?)?);
    let (ca, cb) = (Checkpoint::read(&a, &args.a_prefix)?, Checkpoint::read(&b, &args.b_prefix)?);
    let diffs = layout_differences(&ca, &cb);
    if !diffs.is_empty() {
        bail!("{} and {} cannot be interpolated: {} differences in tensor name, shape or type\n  {}", a.display(), b.display(), diffs.len(), diffs.join("\n  "));
    }
    let values: usize = ca.0.values().map(|e| e.data.len()).sum();
    println!("{} and {} agree on all {} tensors ({values} F32 values) in name, shape and type", a.display(), b.display(), ca.0.len());
    let (sa, sb) = (source(&a, &args.a_prefix, 0.0)?, source(&b, &args.b_prefix, 0.0)?);
    for (&alpha, out) in args.alphas.iter().zip(&outs) {
        let combined = combine(&[&ca, &cb], &[1.0 - alpha, alpha])?;
        let sources = [Source { weight: 1.0 - alpha, ..sa.clone() }, Source { weight: alpha, ..sb.clone() }];
        write_output(out, &combined, &spec, &sources, &spec_from)?;
        // What was written has been read back and compared with `combined` already.
        let note = if combined.same_bits(&ca) {
            format!(": every tensor is bit for bit that of {}", a.display())
        } else if combined.same_bits(&cb) {
            format!(": every tensor is bit for bit that of {}", b.display())
        } else {
            String::new()
        };
        println!("{}: alpha {alpha}{note}", out.display());
    }
    println!("Views copied from {}; every spec written is uncalibrated (identity calibration, no early exit).", spec_from.display());
    println!("Fit each candidate with `tintype calibrate --weights <file>` on separate calibration data before reading any verdict.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint(entries: &[(&str, &[usize], &[f32])]) -> Checkpoint {
        Checkpoint(entries.iter().map(|(n, s, d)| (n.to_string(), Entry { dtype: DType::F32, shape: s.to_vec(), data: d.to_vec() })).collect())
    }

    fn save_as(dir: &Path, name: &str, prefix: &str, c: &Checkpoint) -> PathBuf {
        let p = dir.join(name);
        Checkpoint(c.0.iter().map(|(k, e)| (format!("{prefix}{k}"), e.clone())).collect()).save(&p).unwrap();
        p
    }

    fn spec_json() -> String {
        serde_json::to_string(&ModelSpec { early_exit: Some(crate::detector::EarlyExit::standard()), logit_scale: 0.7, logit_bias: -1.2, ..unfitted_spec(&ModelSpec::commfor(224)) }).unwrap()
    }

    #[test]
    fn interpolation_is_the_weighted_sum_and_the_end_points_are_exact() {
        // Negative zero and a value that is not exact in binary: both must survive the end points untouched.
        let a = checkpoint(&[("head.bias", &[2], &[-0.0, 0.1]), ("head.weight", &[1, 2], &[1.0, -3.0])]);
        let b = checkpoint(&[("head.bias", &[2], &[4.0, 0.3]), ("head.weight", &[1, 2], &[5.0, 1.0])]);
        assert!(combine(&[&a, &b], &[1.0, 0.0]).unwrap().same_bits(&a));
        assert!(combine(&[&a, &b], &[0.0, 1.0]).unwrap().same_bits(&b));
        // 1 * (-0.0) + 0 * 4.0 is +0.0 in IEEE arithmetic: the end point is a copy, not that sum.
        assert_eq!(combine(&[&a, &b], &[1.0, 0.0]).unwrap().0["head.bias"].data[0].to_bits(), (-0.0f32).to_bits());
        let mid = combine(&[&a, &b], &[0.75, 0.25]).unwrap();
        assert_eq!(mid.0["head.weight"].data, vec![2.0, -2.0]);
        assert_eq!(mid.0["head.bias"].data, vec![1.0, (0.75 * 0.1f32 as f64 + 0.25 * 0.3f32 as f64) as f32]);
        assert_eq!(mid.0["head.weight"].shape, vec![1, 2]);
        assert!(!mid.same_bits(&a) && !mid.same_bits(&b));
    }

    #[test]
    fn averaging_takes_any_number_of_siblings_and_weights_that_add_up_to_one() {
        let c = |v: f32| checkpoint(&[("w", &[3], &[v, 2.0 * v, -v])]);
        let (x, y, z) = (c(1.0), c(2.0), c(6.0));
        assert_eq!(combine(&[&x, &y, &z], &[1.0 / 3.0; 3]).unwrap().0["w"].data, vec![3.0, 6.0, -3.0]);
        assert_eq!(combine(&[&x, &y, &z], &[0.5, 0.5, 0.0]).unwrap().0["w"].data, vec![1.5, 3.0, -1.5]);
        assert!(combine(&[&x, &y], &[0.5, 0.6]).unwrap_err().to_string().contains("add up to 1"));
        assert!(combine(&[&x, &y], &[1.5, -0.5]).unwrap_err().to_string().contains("non-negative"));
        assert!(combine(&[&x, &y], &[1.0]).unwrap_err().to_string().contains("one weight per checkpoint"));
    }

    #[test]
    fn checkpoints_that_differ_in_name_shape_or_type_are_refused_with_every_difference_listed() {
        let a = checkpoint(&[("blocks.0.w", &[2, 2], &[0.0; 4]), ("cls", &[1, 2], &[0.0; 2]), ("head.bias", &[1], &[0.0]), ("only_a", &[1], &[0.0])]);
        let mut b = checkpoint(&[("blocks.0.w", &[4], &[0.0; 4]), ("cls", &[1, 2], &[0.0; 2]), ("head.bias", &[1], &[0.0]), ("only_b", &[1], &[0.0])]);
        b.0.get_mut("head.bias").unwrap().dtype = DType::F64;
        let diffs = layout_differences(&a, &b);
        assert_eq!(diffs.len(), 4, "{diffs:?}");
        assert!(diffs[0].contains("`blocks.0.w` has shape [2, 2] in the first checkpoint and [4] in the second"), "{diffs:?}");
        assert!(diffs[1].contains("`head.bias` is F32 in the first checkpoint and F64 in the second"), "{diffs:?}");
        assert!(diffs[2].contains("`only_a` is missing from the second"), "{diffs:?}");
        assert!(diffs[3].contains("`only_b` is missing from the first"), "{diffs:?}");
        let e = combine(&[&a, &b], &[0.5, 0.5]).unwrap_err().to_string();
        assert!(e.contains("4 differences") && e.contains("only_b") && e.contains("blocks.0.w"), "{e}");
        assert!(layout_differences(&a, &a).is_empty());
        // Two checkpoints that agree with each other but are not F32 cannot be combined either.
        let mut half = a.clone();
        half.0.get_mut("cls").unwrap().dtype = DType::F16;
        assert!(layout_differences(&half, &half)[0].contains("only F32"));
    }

    #[test]
    fn a_sweep_strips_the_prefix_writes_exact_end_points_and_marks_every_spec_unfitted() {
        let dir = tempfile::tempdir().unwrap();
        let a = checkpoint(&[("head.bias", &[1], &[-0.0]), ("head.weight", &[1, 2], &[1.0, 2.0])]);
        let b = checkpoint(&[("head.bias", &[1], &[1.0]), ("head.weight", &[1, 2], &[3.0, 6.0])]);
        let (pa, pb) = (save_as(dir.path(), "a.safetensors", "", &a), save_as(dir.path(), "b.safetensors", "vit.", &b));
        std::fs::write(dir.path().join("a.json"), spec_json()).unwrap();
        let out_dir = dir.path().join("sweep");
        let args = |alphas: Vec<f64>, overwrite: bool| Args { a: Some(pa.clone()), a_prefix: String::new(), b: Some(pb.clone()), b_prefix: "vit.".into(), alphas, out_dir: Some(out_dir.clone()), average: vec![], weights: vec![], out: None, spec: None, overwrite };
        run(args(vec![0.0, 0.5, 1.0], false)).unwrap();
        assert!(Checkpoint::read(&out_dir.join("alpha-0.00.safetensors"), "").unwrap().same_bits(&a));
        assert!(Checkpoint::read(&out_dir.join("alpha-1.00.safetensors"), "").unwrap().same_bits(&b));
        assert_eq!(Checkpoint::read(&out_dir.join("alpha-0.50.safetensors"), "").unwrap().0["head.weight"].data, vec![2.0, 4.0]);
        // The spec keeps the source's views and nothing that was fitted to it.
        let spec = ModelSpec::load_for(&out_dir.join("alpha-0.50.safetensors")).unwrap();
        assert_eq!(spec.views, ModelSpec::commfor(224).views);
        assert_eq!((spec.logit_scale, spec.logit_bias, spec.prefix.as_str()), (1.0, 0.0, ""));
        assert!(spec.early_exit.is_none() && spec.strict.is_none() && spec.sensitive.is_none() && spec.second_opinion.is_none());
        assert_eq!(spec.uncalibrated, Some(vec!["photo".to_string()]));
        assert!(spec.uncalibrated_warning(Path::new("x.safetensors")).unwrap().contains("not calibrated (photo)"));
        // The record names both sources with their digests and weights.
        let record: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(out_dir.join("alpha-0.50.interpolation.json")).unwrap()).unwrap();
        assert_eq!(record["sources"][1]["prefix"], "vit.");
        assert_eq!(record["sources"][1]["weight"], 0.5);
        assert_eq!(record["sources"][0]["sha256"].as_str().unwrap(), crate::integrity::sha256_file(&pa).unwrap());
        // A second run does not replace anything unless asked to.
        assert!(run(args(vec![0.5], false)).unwrap_err().to_string().contains("already exists"));
        run(args(vec![0.5], true)).unwrap();
        // Without the prefix the names do not match, and nothing is written.
        let e = run(Args { b_prefix: String::new(), out_dir: Some(dir.path().join("other")), ..args(vec![0.5], false) }).unwrap_err().to_string();
        assert!(e.contains("4 differences") && e.contains("`vit.head.bias` is missing from the first"), "{e}");
        assert!(!dir.path().join("other").exists());
        assert!(run(args(vec![1.5], true)).unwrap_err().to_string().contains("between 0 and 1"));
        assert!(run(args(vec![0.25, 0.25], true)).unwrap_err().to_string().contains("twice"));
    }

    #[test]
    fn nothing_is_written_into_a_models_directory_or_over_an_input() {
        let dir = tempfile::tempdir().unwrap();
        let models = dir.path().join("models");
        std::fs::create_dir_all(&models).unwrap();
        let a = checkpoint(&[("w", &[1], &[1.0])]);
        let (pa, pb) = (save_as(&models, "tintype.safetensors", "", &a), save_as(dir.path(), "b.safetensors", "", &a));
        std::fs::write(models.join("tintype.json"), spec_json()).unwrap();
        let before = std::fs::read(&pa).unwrap();
        let sweep = |out_dir: PathBuf| Args { a: Some(pa.clone()), a_prefix: String::new(), b: Some(pb.clone()), b_prefix: String::new(), alphas: vec![0.0, 1.0], out_dir: Some(out_dir), average: vec![], weights: vec![], out: None, spec: None, overwrite: true };
        // Into models/ itself, into a directory below it, and through a symbolic link to it.
        for out_dir in [models.clone(), models.join("sweep")] {
            let e = run(sweep(out_dir)).unwrap_err().to_string();
            assert!(e.contains("is a models directory"), "{e}");
        }
        #[cfg(unix)]
        {
            let link = dir.path().join("runs");
            std::os::unix::fs::symlink(&models, &link).unwrap();
            assert!(run(sweep(link)).unwrap_err().to_string().contains("is a models directory"));
        }
        let average = |out: PathBuf| Args { a: None, a_prefix: String::new(), b: None, b_prefix: String::new(), alphas: vec![], out_dir: None, average: vec![pa.clone(), pb.clone()], weights: vec![], out: Some(out), spec: None, overwrite: true };
        assert!(run(average(models.join("tintype.safetensors"))).unwrap_err().to_string().contains("is a models directory"));
        assert!(run(average(pb.clone())).unwrap_err().to_string().contains("one of the inputs"));
        assert!(run(average(dir.path().join("avg.bin"))).unwrap_err().to_string().contains(".safetensors"));
        assert_eq!(std::fs::read(&pa).unwrap(), before);
        assert_eq!(std::fs::read_dir(&models).unwrap().count(), 2);
        // The same average somewhere harmless is fine.
        let out = dir.path().join("out/avg.safetensors");
        run(average(out.clone())).unwrap();
        assert!(Checkpoint::read(&out, "").unwrap().same_bits(&a));
        assert_eq!(ModelSpec::load_for(&out).unwrap().uncalibrated, Some(vec!["photo".to_string()]));
    }

    #[test]
    fn sweep_file_names_keep_enough_digits() {
        assert_eq!(alpha_name(0.1), "alpha-0.10.safetensors");
        assert_eq!(alpha_name(1.0), "alpha-1.00.safetensors");
        assert_eq!(alpha_name(0.125), "alpha-0.1250.safetensors");
    }
}
