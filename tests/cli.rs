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
    // Each image's own time is printed only with the diagnostic counters.
    assert!(!String::from_utf8_lossy(&o.stderr).contains("latency_ms"));
    let o = Command::new(env!("CARGO_BIN_EXE_tintype"))
        .args(["latency", "--weights", weights.to_str().unwrap(), "--manifest", dir.join("manifest.tsv").to_str().unwrap(), "--n", "3", "--device", "cpu"])
        .env("TINTYPE_STATS", "1")
        .current_dir(common::repo())
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr).to_string();
    assert_eq!(err.lines().filter(|l| l.starts_with("latency_ms ")).count(), 3, "{err}");
    // The warm-up images' times too: the counters include them.
    assert_eq!(err.lines().filter(|l| l.starts_with("warmup_ms ")).count(), 5, "{err}");
    assert!(err.lines().any(|l| l.starts_with("stats: ")), "{err}");
    assert_eq!(stdout(&o).split(':').next(), out.split(':').next());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn work_counters_go_to_standard_error_only_when_asked_for() {
    let dir = common::scratch("cli-stats");
    let png = dir.join("fixture.png");
    common::write_fixture(&png);
    let weights = common::photo_weights();
    let args = ["detect", "--device", "cpu", "--all-views", "--json", "--weights", weights.to_str().unwrap(), png.to_str().unwrap()];
    let run = |stats: Option<&str>| {
        let mut c = Command::new(env!("CARGO_BIN_EXE_tintype"));
        c.args(args).current_dir(common::repo()).env_remove("TINTYPE_STATS");
        if let Some(v) = stats {
            c.env("TINTYPE_STATS", v);
        }
        c.output().expect("running the tintype binary")
    };
    let (plain, asked, zero) = (run(None), run(Some("1")), run(Some("0")));
    assert!(plain.status.success() && asked.status.success(), "{}", String::from_utf8_lossy(&asked.stderr));
    // Standard output is the same either way; the counters are one line on standard error.
    assert_eq!(stdout(&plain), stdout(&asked));
    assert!(!String::from_utf8_lossy(&plain.stderr).contains("stats:") && !String::from_utf8_lossy(&zero.stderr).contains("stats:"));
    let err = String::from_utf8_lossy(&asked.stderr).to_string();
    let line = err.lines().find(|l| l.starts_with("stats: ")).unwrap_or_else(|| panic!("no stats line in: {err}"));
    let value = |name: &str| -> u64 { line.split_whitespace().find_map(|f| f.strip_prefix(&format!("{name}="))).unwrap_or_else(|| panic!("no {name} in {line}")).parse().unwrap() };
    // One photo, every view looked at: six logical views, no video frames.
    assert_eq!(value("views_scored") + value("views_reused"), 6, "{line}");
    assert!(value("views_materialised") <= 6 && value("views_materialised") >= 1, "{line}");
    assert_eq!((value("frames_decoded"), value("frames_converted"), value("clips_reopened")), (0, 0, 0), "{line}");
    // The forward pass is divided among its parts: they are all there and
    // add up to no more than the pass (each is rounded to a millisecond).
    let parts = ["patch_ms", "norm_ms", "qkv_ms", "attention_ms", "softmax_ms", "gather_ms", "proj_ms", "fc1_ms", "gelu_ms", "fc2_ms", "head_ms", "residual_ms"];
    let sum: u64 = parts.iter().map(|p| value(p)).sum();
    assert!(value("forward_ms") >= 1 && sum <= value("forward_ms") + parts.len() as u64, "{line}");
    assert!(value("qkv_ms") + value("proj_ms") + value("fc1_ms") + value("fc2_ms") >= 1, "{line}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_photo_below_the_crop_size_is_scored_once_per_distinct_view_with_the_same_output() {
    let dir = common::scratch("cli-reuse");
    let png = dir.join("small.png");
    // 120 x 96: the centre crop and the four ranked windows are one upscaled crop.
    image::imageops::resize(&common::fixture_pixels(), 120, 96, image::imageops::FilterType::Triangle).save(&png).unwrap();
    let weights = common::photo_weights();
    let run = |reuse: Option<&str>, extra: &[&str]| {
        let mut c = Command::new(env!("CARGO_BIN_EXE_tintype"));
        c.args(["detect", "--device", "cpu", "--json", "--weights", weights.to_str().unwrap()]).args(extra).arg(&png).current_dir(common::repo());
        c.env("TINTYPE_STATS", "1").env_remove("TINTYPE_VIEW_REUSE");
        if let Some(v) = reuse {
            c.env("TINTYPE_VIEW_REUSE", v);
        }
        let o = c.output().expect("running the tintype binary");
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let err = String::from_utf8_lossy(&o.stderr).to_string();
        let line = err.lines().find(|l| l.starts_with("stats: ")).unwrap_or_else(|| panic!("no stats line in: {err}")).to_string();
        let value = move |name: &str| -> u64 { line.split_whitespace().find_map(|f| f.strip_prefix(&format!("{name}="))).unwrap().parse().unwrap() };
        (stdout(&o), value)
    };
    // One photo through early exit: every run is one view, so reuse is on by
    // default on every engine and the output is the same bytes as without it.
    let (default_out, default) = run(None, &[]);
    let (off_out, off) = run(Some("off"), &[]);
    assert_eq!(default_out, off_out);
    let looked = json_lines_of(&default_out)[0]["views"].as_u64().unwrap();
    assert_eq!(off("views_scored"), looked);
    assert_eq!(off("views_reused"), 0);
    assert_eq!(default("views_scored") + default("views_reused"), looked);
    assert!(default("views_scored") <= 2 && default("views_materialised") <= 2, "scored {}, materialised {}", default("views_scored"), default("views_materialised"));
    // Every view looked at, reuse forced on: two runs through the network stand for six views.
    let (on_out, on) = run(Some("on"), &["--all-views"]);
    let (all_out, all) = run(Some("off"), &["--all-views"]);
    assert_eq!((on("views_scored"), on("views_reused"), on("views_materialised")), (2, 4, 2));
    assert_eq!((all("views_scored"), all("views_reused"), all("views_materialised")), (6, 0, 6));
    let (a, b) = (&json_lines_of(&on_out)[0], &json_lines_of(&all_out)[0]);
    assert_eq!((a["views"].as_u64(), a["verdict"].as_str()), (Some(6), b["verdict"].as_str()));
    assert!((a["logit"].as_f64().unwrap() - b["logit"].as_f64().unwrap()).abs() < 1e-4, "{on_out} {all_out}");
    let _ = std::fs::remove_dir_all(&dir);
}

fn json_lines_of(out: &str) -> Vec<serde_json::Value> {
    out.lines().map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("not one JSON object per line ({e}): {l}"))).collect()
}

#[test]
fn the_int8_simulation_is_opt_in_takes_what_cpu_takes_and_leaves_cpu_alone() {
    let dir = common::scratch("cli-int8");
    let png = dir.join("fixture.png");
    common::write_fixture(&png);
    let weights = common::photo_weights();
    let clip = common::repo().join("tests/fixtures/video.mp4");
    // The photo, and the clip too when no variant is named.
    let run = |device: &str, variant: Option<&str>| {
        let mut c = Command::new(env!("CARGO_BIN_EXE_tintype"));
        c.args(["detect", "--device", device, "--all-views", "--json", "--weights", weights.to_str().unwrap(), png.to_str().unwrap()]);
        if variant.is_none() {
            c.arg(&clip);
        }
        c.current_dir(common::repo()).env_remove("TINTYPE_INT8");
        if let Some(v) = variant {
            c.env("TINTYPE_INT8", v);
        }
        c.output().expect("running the tintype binary")
    };
    let cpu = run("cpu", None);
    assert!(cpu.status.success(), "{}", String::from_utf8_lossy(&cpu.stderr));
    // The variable is read by the simulation only: `cpu` gives the same bytes whatever it says.
    let plain = json_lines(&cpu);
    assert_eq!(json_lines(&run("cpu", Some("all:tensor:4"))), plain[..1]);
    assert_eq!(json_lines(&run("cpu", Some("no such variant"))), plain[..1]);
    let logit = |row: &serde_json::Value| row["logit"].as_f64().unwrap();
    let mut default = Vec::new();
    for variant in [None, Some("weights"), Some("mlp:tensor"), Some("all:token:7")] {
        let o = run("cpu-int8", variant);
        assert!(o.status.success(), "{variant:?}: {}", String::from_utf8_lossy(&o.stderr));
        let rows = json_lines(&o);
        // The same photo and the same clip, scored: the same fields, views and frames, and logits that moved a little.
        assert_eq!(rows.len(), if variant.is_none() { 2 } else { 1 }, "{variant:?}");
        for (got, want) in rows.iter().zip(&plain) {
            assert_eq!((&got["path"], &got["kind"], &got["profile"], got.get("views"), got.get("frames")), (&want["path"], &want["kind"], &want["profile"], want.get("views"), want.get("frames")), "{variant:?}");
            assert_eq!(got.as_object().unwrap().keys().collect::<Vec<_>>(), want.as_object().unwrap().keys().collect::<Vec<_>>(), "{variant:?}");
            let moved = (logit(got) - logit(want)).abs();
            // Not the same number and not a different answer: on this machine the largest move is about one.
            assert!(moved > 0.0 && moved < 6.0, "{variant:?}: logit {} against {}", logit(got), logit(want));
        }
        if variant.is_none() {
            default = rows;
        }
    }
    // With no variable the simulation is the `all` variant.
    assert_eq!(json_lines(&run("cpu-int8", Some("all:token:8"))), default[..1]);
    let bad = run("cpu-int8", Some("no such variant"));
    assert!(!bad.status.success() && String::from_utf8_lossy(&bad.stderr).contains("unknown INT8 variant"), "{}", String::from_utf8_lossy(&bad.stderr));
    // The device is never picked by itself and is named in the error for an unknown one.
    let unknown = tintype(&["detect", "--device", "int8", png.to_str().unwrap()]);
    assert!(!unknown.status.success() && String::from_utf8_lossy(&unknown.stderr).contains("cpu-int8"));
    let _ = std::fs::remove_dir_all(&dir);
}
