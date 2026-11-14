//! The built `tintype` binary, run the way a user runs it. Every detection
//! here uses the CPU engine, so the same tests run on Linux and on macOS.
mod common;

use std::path::Path;
use std::process::{Command, Output};

fn tintype(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tintype")).args(args).current_dir(common::repo()).output().expect("running the tintype binary")
}

fn detect(files: &[&Path], extra: &[&str]) -> Output {
    let weights = common::photo_weights();
    let mut args = vec!["detect", "--device", "cpu", "--weights", weights.to_str().unwrap()];
    args.extend_from_slice(extra);
    args.extend(files.iter().map(|p| p.to_str().unwrap()));
    tintype(&args)
}

fn stdout(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}

fn json_lines(o: &Output) -> Vec<serde_json::Value> {
    stdout(o).lines().map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("not one JSON object per line ({e}): {l}"))).collect()
}

/// The score of a readable file's JSON row: `score` since the review fixes,
/// `ai_probability` before them.
fn score_of(row: &serde_json::Value) -> Option<f64> {
    row.get("score").or_else(|| row.get("ai_probability")).and_then(|v| v.as_f64())
}

#[test]
fn detect_gives_one_verdict_line_per_image() {
    let dir = common::scratch("cli-text");
    let png = dir.join("fixture.png");
    common::write_fixture(&png);
    let o = detect(&[&png], &[]);
    assert!(o.status.success(), "status {:?}, stderr: {}", o.status, String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1, "{out}");
    assert!(lines[0].starts_with("AI-GENERATED") || lines[0].starts_with("likely real"), "{out}");
    assert!(lines[0].ends_with(png.to_str().unwrap()) || lines[0].contains(&format!("{}  ", png.display())), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn detect_json_has_one_object_per_file_in_order() {
    let dir = common::scratch("cli-json");
    let (a, b) = (dir.join("a.png"), dir.join("b.png"));
    common::write_fixture(&a);
    // A different picture: the fixture turned upside down.
    image::imageops::flip_vertical(&common::fixture_pixels()).save(&b).unwrap();
    let o = detect(&[&a, &b], &["--json"]);
    assert!(o.status.success(), "status {:?}, stderr: {}", o.status, String::from_utf8_lossy(&o.stderr));
    let rows = json_lines(&o);
    assert_eq!(rows.len(), 2, "{}", stdout(&o));
    for (row, path) in rows.iter().zip([&a, &b]) {
        assert_eq!(row["path"].as_str(), path.to_str(), "{row}");
        assert_eq!(row["kind"], "image", "{row}");
        assert!(row.get("error").is_none(), "{row}");
        let score = score_of(row).unwrap_or_else(|| panic!("no score in {row}"));
        assert!((0.0..=1.0).contains(&score), "{row}");
        let verdict = row["verdict"].as_str().unwrap_or_else(|| panic!("no verdict in {row}"));
        assert_eq!(verdict, if score > 0.5 { "ai-generated" } else { "likely-real" }, "{row}");
        if row.get("score").is_some() {
            // The fuller record of the review fixes: the evidence and the threshold it was held to.
            assert_eq!(row["profile"], "default", "{row}");
            assert!(row["logit"].is_number() && row["threshold_logit"].is_number(), "{row}");
            assert!(row["second_opinion"].is_boolean(), "{row}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn detect_reports_an_unreadable_file_by_itself() {
    let dir = common::scratch("cli-unreadable");
    let (good, bad) = (dir.join("good.png"), dir.join("broken.png"));
    common::write_fixture(&good);
    std::fs::write(&bad, b"\x89PNG\r\n\x1a\n this is not an image").unwrap();

    let o = detect(&[&good, &bad], &["--json"]);
    let rows = json_lines(&o);
    assert_eq!(rows.len(), 2, "{}", stdout(&o));
    // The readable file still gets its verdict; the other gets an error of its own.
    assert!(score_of(&rows[0]).is_some() && rows[0].get("error").is_none(), "{}", rows[0]);
    assert_eq!(rows[1]["path"].as_str(), bad.to_str());
    assert!(rows[1]["error"].is_string(), "{}", rows[1]);
    assert!(rows[1].get("verdict").is_none() && score_of(&rows[1]).is_none(), "{}", rows[1]);
    // Since the review fixes, a run in which any file could not be read exits
    // with status 2 (they also renamed `ai_probability` to `score`, which is
    // how the two builds are told apart here). Before them it exited 0.
    let expected = if rows[0].get("score").is_some() { Some(2) } else { Some(0) };
    assert_eq!(o.status.code(), expected, "stderr: {}", String::from_utf8_lossy(&o.stderr));

    // The same in text form.
    let o = detect(&[&bad], &[]);
    assert!(stdout(&o).starts_with("unreadable"), "{}", stdout(&o));
    assert_eq!(o.status.code(), expected);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_models_accepts_the_released_files() {
    let o = tintype(&["verify-models"]);
    let out = stdout(&o);
    assert!(o.status.success(), "{out}{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(out.lines().filter(|l| l.starts_with("ok ")).count(), 4, "{out}");
}

#[test]
fn freeze_set_then_verify_set_catches_a_changed_file() {
    let dir = common::scratch("cli-freeze");
    std::fs::create_dir_all(dir.join("data/demo")).unwrap();
    common::write_fixture(&dir.join("data/demo/a.png"));
    std::fs::write(dir.join("data/demo/b.png"), b"placeholder bytes").unwrap();
    std::fs::write(dir.join("data/demo/manifest.tsv"), "a.png\t1\tgen-x\tdemo\nb.png\t0\treal:demo\tdemo\n").unwrap();
    let root = dir.to_str().unwrap();
    let frozen = dir.join("demo.frozen.tsv.gz");
    let o = tintype(&["freeze-set", "--root", root, "--manifest", dir.join("data/demo/manifest.tsv").to_str().unwrap(), "--out", frozen.to_str().unwrap()]);
    assert!(o.status.success(), "{}{}", stdout(&o), String::from_utf8_lossy(&o.stderr));

    let o = tintype(&["verify-set", "--root", root, frozen.to_str().unwrap()]);
    assert!(o.status.success(), "{}{}", stdout(&o), String::from_utf8_lossy(&o.stderr));
    assert!(stdout(&o).contains("2 files, 1 manifests: identical"), "{}", stdout(&o));

    std::fs::write(dir.join("data/demo/b.png"), b"PLACEHOLDER BYTES").unwrap();
    let o = tintype(&["verify-set", "--root", root, frozen.to_str().unwrap()]);
    assert!(!o.status.success());
    assert!(stdout(&o).contains("changed   data/demo/b.png"), "{}", stdout(&o));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn batch_keeps_root_and_file_order_and_continues_after_errors() {
    let dir = common::scratch("cli-batch");
    let subdir = dir.join("album");
    std::fs::create_dir(&subdir).unwrap();
    let good = subdir.join("a.PNG");
    let bad = subdir.join("b.png");
    let video = subdir.join("c.mp4");
    let missing = dir.join("missing.png");
    common::write_fixture(&good);
    std::fs::write(&bad, b"broken").unwrap();
    std::fs::write(&video, b"broken video").unwrap();
    std::fs::write(subdir.join("notes.txt"), b"ignore").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&good, subdir.join("symlink.png")).unwrap();
    let o = tintype(&[
        "batch", "--device", "cpu", "--strict", "--json",
        missing.to_str().unwrap(), subdir.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(2));
    let rows = json_lines(&o);
    assert_eq!(rows.len(), 4);
    for (row, path) in rows.iter().zip([&missing, &good, &bad, &video]) {
        assert_eq!(row["path"].as_str(), path.to_str());
    }
    assert_eq!(rows[1]["profile"], "strict");
    assert!(rows[1]["score"].is_number());
    assert!(rows[0]["error"].is_string() && rows[2]["error"].is_string() && rows[3]["error"].is_string());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn detect_rejects_invalid_sampling_parameters_before_inference() {
    for flags in [["--frames", "0"], ["--frames", "257"], ["--max-seconds", "NaN"]] {
        let o = tintype(&["detect", "test.mp4", flags[0], flags[1]]);
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.is_empty());
    }
}

#[test]
fn mixed_detection_preserves_input_order_and_describes_video_coverage() {
    let dir = common::scratch("cli-video");
    let photo = dir.join("still.png");
    common::write_fixture(&photo);
    let video = common::repo().join("tests/fixtures/video.mp4");
    let o = detect(&[&video, &photo], &["--json", "--frames", "4", "--max-seconds", "0"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let rows = json_lines(&o);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["path"].as_str(), video.to_str());
    assert_eq!(rows[0]["kind"], "video");
    assert_eq!(rows[0]["frames"], 4);
    assert_eq!(rows[0]["frames_expected"], 4);
    assert_eq!(rows[0]["complete"], true);
    assert_eq!(rows[0]["analysed_until_s"], 0.75);
    assert_eq!(rows[1]["path"].as_str(), photo.to_str());
    assert_eq!(rows[1]["kind"], "image");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn an_empty_model_dir_variable_counts_as_unset() {
    // PowerShell can leave a variable defined but empty; the lookup must then
    // fall through to the checkout's models/ directory instead of failing.
    let o = Command::new(env!("CARGO_BIN_EXE_tintype"))
        .args(["demo", "--device", "cpu", "--json"])
        .env("TINTYPE_MODEL_DIR", "")
        .current_dir(common::repo())
        .output()
        .expect("running the tintype binary");
    assert!(o.status.success(), "status {:?}, stderr: {}", o.status, String::from_utf8_lossy(&o.stderr));
    assert_eq!(json_lines(&o).len(), 1);
}

/// A manifest of `n` parents, each an original plus a JPEG derivative that
/// carries the parent's id, with the files it lists.
fn write_family_manifest(dir: &Path, n: usize) {
    let mut text = String::new();
    for p in 0..n {
        let (label, generator) = if p % 2 == 0 { (0, "real:cam") } else { (1, "gen-x") };
        for transform in ["", "jpeg-q70"] {
            let name = format!("{p}{}.bin", if transform.is_empty() { "" } else { "-jpeg" });
            std::fs::write(dir.join(&name), format!("{p} {transform}")).unwrap();
            text.push_str(&format!("{name}\t{label}\t{generator}\tdemo\tp{p}\t\t\t\t{transform}\n"));
        }
    }
    std::fs::write(dir.join("manifest.tsv"), text).unwrap();
}

#[test]
fn split_groups_keeps_parents_together_and_verify_split_catches_a_leak() {
    let dir = common::scratch("cli-split");
    write_family_manifest(&dir, 200);
    let d = |name: &str| dir.join(name).to_str().unwrap().to_string();
    let o = tintype(&["split-groups", "--manifest", &d("manifest.tsv"), "--out-dir", &d("split")]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(stdout(&o).contains("400 of 400 items carry a parent_id or scene_id"), "{}", stdout(&o));
    let rows = |set: &str| std::fs::read_to_string(dir.join("split").join(format!("{set}.tsv"))).unwrap();
    let parents = |set: &str| rows(set).lines().map(|l| l.split('\t').nth(4).unwrap().to_string()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(rows("fit").lines().count() + rows("calibrate").lines().count() + rows("test").lines().count(), 400);
    assert!(parents("fit").is_disjoint(&parents("test")) && parents("fit").is_disjoint(&parents("calibrate")) && parents("calibrate").is_disjoint(&parents("test")));
    assert!(!parents("calibrate").is_empty() && !parents("test").is_empty());
    // The grouping columns survive, and a set with them is recorded as format 2.
    assert!(rows("test").lines().any(|l| l.ends_with("\tjpeg-q70")));
    let mut sets = Vec::new();
    for set in ["fit", "calibrate", "test"] {
        let o = tintype(&["freeze-set", "--manifest", &d(&format!("split/{set}.tsv")), "--out", &d(&format!("{set}.frozen.tsv")), "--root", dir.to_str().unwrap()]);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert!(std::fs::read_to_string(dir.join(format!("{set}.frozen.tsv"))).unwrap().starts_with("# tintype frozen evaluation set, format 2\n"));
        sets.push(format!("{set}={}", d(&format!("{set}.frozen.tsv"))));
    }
    let args = |sets: &[String]| -> Vec<String> { std::iter::once("verify-split".to_string()).chain(sets.iter().flat_map(|s| ["--set".to_string(), s.clone()])).collect() };
    let run = |sets: &[String]| { let a = args(sets); tintype(&a.iter().map(String::as_str).collect::<Vec<_>>()) };
    let o = run(&sets);
    assert!(o.status.success() && stdout(&o).contains("no group crosses sets"), "{} {}", stdout(&o), String::from_utf8_lossy(&o.stderr));

    // Move one derivative from the test set into fitting: its parent now crosses.
    let test_rows = rows("test");
    let moved = test_rows.lines().find(|l| l.ends_with("\tjpeg-q70")).unwrap();
    std::fs::write(dir.join("leaky-fit.tsv"), format!("{}{moved}\n", rows("fit"))).unwrap();
    let o = run(&[format!("fit={}", d("leaky-fit.tsv")), format!("test={}", d("split/test.tsv"))]);
    assert!(!o.status.success(), "{}", stdout(&o));
    assert!(stdout(&o).lines().any(|l| l.starts_with("parent p") && l.ends_with("is in fit and test")), "{}", stdout(&o));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn report_operating_points_gives_the_fixed_budget_tables() {
    let dir = common::scratch("cli-points");
    let mut text = String::new();
    for i in 0..400 {
        text.push_str(&format!("r{i}.jpg\t0\treal:cam\tdemo\t{:.6}\n", i as f32 / 500.0));
        text.push_str(&format!("f{i}.png\t1\t{}\tdemo\t{:.6}\n", if i % 2 == 0 { "gen-a" } else { "gen-b" }, 0.3 + i as f32 / 600.0));
    }
    let scores = dir.join("scores.tsv");
    std::fs::write(&scores, text).unwrap();
    let system = format!("demo={}@0.75", scores.display());
    let o = tintype(&["report", "--system", &system, "--operating-points", "--slice-by", "generator", "--prevalence", "0.05"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    for want in ["| 0.1% empirical FPR |", "| 1% empirical FPR |", "| frozen cutoff | 0.750000 |", "Precision is at 5.00% prevalence", "| gen-a | 200 | 0 |", "| real:cam | 0 | 400 |", "Worst generator slice"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    // The plain tables are what they were.
    let o = tintype(&["report", "--system", &system, "--markdown"]);
    assert!(stdout(&o).contains("| System | Precision | Recall | F1 | False-positive rate | AUROC | Recall at 1% FPR |"), "{}", stdout(&o));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn latency_runs_on_the_cpu_engine_and_reports_p95() {
    let dir = common::scratch("cli-latency");
    let mut text = String::new();
    for i in 0..8 {
        common::write_fixture(&dir.join(format!("{i}.png")));
        text.push_str(&format!("{i}.png\t0\treal:fixture\tdemo\n"));
    }
    std::fs::write(dir.join("manifest.tsv"), text).unwrap();
    let weights = common::photo_weights();
    let o = tintype(&["latency", "--weights", weights.to_str().unwrap(), "--manifest", dir.join("manifest.tsv").to_str().unwrap(), "--n", "3", "--device", "cpu"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    assert!(out.starts_with("single-image latency over 3 images: median ") && out.contains(" ms, p90 ") && out.contains(" ms, p95 ") && out.contains(" ms, mean "), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn feature_dump_head_fit_and_comparison_run_end_to_end_on_the_cpu_engine() {
    let dir = common::scratch("cli-temporal");
    // Four clips: the two fixtures, each once as real and once as AI (a pipeline check, not a detector).
    let fixtures = common::repo().join("tests/fixtures");
    let mut manifest = String::new();
    for (name, label) in [("a.mp4", 0), ("b.mp4", 1), ("c.mp4", 1), ("d.mp4", 0)] {
        std::fs::copy(fixtures.join(if label == 0 { "video.mp4" } else { "hevc.mp4" }), dir.join(name)).unwrap();
        manifest.push_str(&format!("{name}\t{label}\t{}\tfixture\t-\tscene-{name}\n", if label == 1 { "gen" } else { "real:fixture" }));
    }
    std::fs::write(dir.join("manifest.tsv"), manifest).unwrap();
    let weights = common::photo_weights();
    let (dump, pairs) = (dir.join("dump"), dir.join("pairs"));
    let path = |p: &Path| p.to_str().unwrap().to_string();
    let run = |args: &[&str]| {
        let o = tintype(args);
        assert!(o.status.success(), "{args:?}: {}", String::from_utf8_lossy(&o.stderr));
        stdout(&o)
    };
    // Four frames, the whole-frame view only, tokens pooled to 2 x 2.
    let common_args = ["--weights", weights.to_str().unwrap(), "--manifest", &path(&dir.join("manifest.tsv")), "--frames", "4", "--max-seconds", "0", "--grid", "2", "--fit-only", "--device", "cpu", "--threads", "2"];
    let out = run(&[&["dump-video-features", "--out", &path(&dump)], &common_args[..]].concat());
    assert!(out.contains("4 clips (0 unreadable), 16 frame records of 1920 values"), "{out}");
    assert_eq!(std::fs::metadata(dump.join("features.f32")).unwrap().len(), 16 * 1920 * 4);
    let meta: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dump.join("meta.json")).unwrap()).unwrap();
    assert_eq!((meta["frames"].as_u64(), meta["grid"].as_u64(), meta["blocks"][0].as_u64(), meta["views"].as_array().map(|v| v.len())), (Some(4), Some(2), Some(12), Some(1)), "{meta}");
    // Rows in manifest order, with the grouping columns, real timestamps and stream positions.
    let clips = std::fs::read_to_string(dump.join("clips.tsv")).unwrap();
    let rows: Vec<Vec<&str>> = clips.lines().skip(1).map(|l| l.split('\t').collect()).collect();
    assert_eq!(rows.iter().map(|r| (r[0].rsplit('/').next().unwrap(), r[1], r[5])).collect::<Vec<_>>(), vec![("a.mp4", "0", "scene-a.mp4"), ("b.mp4", "1", "scene-b.mp4"), ("c.mp4", "1", "scene-c.mp4"), ("d.mp4", "0", "scene-d.mp4")], "{clips}");
    let frames = std::fs::read_to_string(dump.join("frames.tsv")).unwrap();
    let first: Vec<Vec<&str>> = frames.lines().skip(1).take(4).map(|l| l.split('\t').collect()).collect();
    // The one-second, twelve-frame clip sampled at 0, 0.25, 0.5 and 0.75 s: frames 0, 3, 6 and 9.
    assert_eq!(first.iter().map(|f| (f[1], f[2], f[3], f[7])).collect::<Vec<_>>(), vec![("0", "0", "0.000000", "0"), ("1", "3", "0.250000", "3"), ("2", "6", "0.500000", "3"), ("3", "9", "0.750000", "3")], "{frames}");
    // With one view per frame the frame logit is the whole-frame view's.
    assert!(first.iter().all(|f| f[4] == f[5] && f[4] == f[6]), "{frames}");

    // Pairs: two positions, each with the next frame of the stream.
    run(&[&["dump-video-features", "--out", &path(&pairs), "--pairs", "1"], &common_args[..]].concat());
    let frames = std::fs::read_to_string(pairs.join("frames.tsv")).unwrap();
    assert_eq!(frames.lines().skip(1).take(4).map(|l| l.split('\t').nth(2).unwrap()).collect::<Vec<_>>(), vec!["0", "1", "6", "7"], "{frames}");

    // Fit a temporal head and an order-invariant one, then compare them with the frame mean.
    for kind in ["diff", "pool-linear", "covariates"] {
        let out = run(&["fit-temporal-head", "--features", &path(&dump), "--kind", kind, "--out", &path(&dir.join(kind)), "--base", "fit", "--epochs", "2", "--hidden", "4", "--watch", &path(&dump)]);
        assert!(out.contains("epoch   2: loss ") && out.contains("watch set: AUROC") && out.contains(&format!("{kind} head, ")), "{out}");
        assert!(dir.join(format!("{kind}.json")).exists() && dir.join(format!("{kind}.safetensors")).exists());
    }
    // A head that corrects the all-view logit cannot be fitted to a whole-frame-only dump, and pairs need a paired dump.
    assert!(!tintype(&["fit-temporal-head", "--features", &path(&dump), "--kind", "diff", "--out", &path(&dir.join("x"))]).status.success());
    let o = tintype(&["eval-temporal-head", "--features", &path(&pairs), "--head", &format!("diff={}", path(&dir.join("diff")))]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("--pairs"), "{}", String::from_utf8_lossy(&o.stderr));

    let scores = dir.join("scores");
    let out = run(&["eval-temporal-head", "--features", &path(&dump), "--calibrate", &path(&dump), "--head", &format!("diff={}", path(&dir.join("diff"))), "--head", &format!("pool={}", path(&dir.join("pool-linear"))), "--head", &format!("cov={}", path(&dir.join("covariates"))), "--out-dir", &path(&scores)]);
    assert!(out.starts_with("4 clips scored by all 7 arms: 2 AI, 2 real"), "{out}");
    for needle in ["| fit-mean | 4 frames x 1 view |", "| diff |", "| diff+shuffled |", "| pool |", "| cov+shuffled |", "### Operating at a 1% false-positive cutoff", "### Operating at a 0.1% false-positive cutoff", "### Shuffled-order control", "By frame rate", "tintype report --operating-points"] {
        assert!(out.contains(needle), "missing `{needle}` in:\n{out}");
    }
    // The score files go straight into the fixed-budget report.
    let o = tintype(&["report", "--operating-points", "--system", &format!("mean={}", path(&scores.join("fit-mean.tsv"))), "--system", &format!("diff={}", path(&scores.join("diff.tsv"))), "--min-slice", "1"]);
    assert!(o.status.success() && stdout(&o).contains("4 items scored by all 2 systems: 2 AI, 2 real"), "{}{}", stdout(&o), String::from_utf8_lossy(&o.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}
