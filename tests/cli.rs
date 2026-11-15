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

#[test]
fn detect_gives_the_photo_second_opinion_weights_to_the_photo_stage() {
    let dir = common::scratch("cli-detect-second");
    let photo = dir.join("photo.png");
    common::write_fixture(&photo);
    // A copy of the released model whose band takes in every photo: the shipped files are never edited.
    let weights = dir.join("copy.safetensors");
    std::fs::copy(common::photo_weights(), &weights).unwrap();
    std::fs::copy(common::photo_weights().with_extension("json"), weights.with_extension("json")).unwrap();
    let o = tintype(&["set-photo-second-opinion", "--weights", weights.to_str().unwrap(), "--band", "-1000,1000,0.5,0"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let missing = dir.join("no-such-weights.safetensors");
    let base = ["detect", "--device", "cpu", "--json", "--weights", weights.to_str().unwrap(), photo.to_str().unwrap()];
    // The photo is inside the band, so the second network is needed and its missing weights are an error.
    let mut args = base.to_vec();
    args.extend_from_slice(&["--photo-second-opinion", missing.to_str().unwrap()]);
    let o = tintype(&args);
    assert!(!o.status.success(), "a photo inside the band must reach the second stage: {}", stdout(&o));
    // The video second opinion has nothing to do with a photo (and exists on macOS only).
    #[cfg(target_os = "macos")]
    {
        let mut args = base.to_vec();
        args.extend_from_slice(&["--second-opinion", missing.to_str().unwrap()]);
        let o = tintype(&args);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert_eq!(json_lines(&o)[0]["second_opinion"], false);
    }
    let _ = std::fs::remove_dir_all(&dir);
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
fn sampling_shares_prints_the_recipe_and_weights_and_replay_change_it() {
    let dir = common::scratch("cli-sampling");
    let store = |name: &str, rows: &[(usize, u8, &str)]| -> String {
        let d = dir.join(name);
        std::fs::create_dir_all(&d).unwrap();
        let text: String = rows.iter().flat_map(|(n, label, generator)| (0..*n).map(move |i| format!("{generator}-{i}.png\t{label}\t{generator}\t{name}\n"))).collect();
        std::fs::write(d.join("manifest.tsv"), text).unwrap();
        d.join("manifest.tsv").to_str().unwrap().to_string()
    };
    let new = store("new", &[(400, 1, "flux"), (100, 1, "sdxl"), (300, 0, "real:cam")]);
    let social = store("social", &[(900, 1, "reddit-wild"), (100, 0, "real:reddit")]);
    let old = store("old", &[(400, 1, "stylegan"), (100, 0, "real:ffhq")]);
    // No validation split, so the shares are exact: sqrt(400), sqrt(100), sqrt(900) of 60.
    let plain = tintype(&["sampling-shares", "--manifest", &new, "--manifest", &social, "--val-pct", "0"]);
    assert!(plain.status.success(), "{}", String::from_utf8_lossy(&plain.stderr));
    let out = stdout(&plain);
    for want in ["train 1800 images (2 real sources, 3 generators), val pool 0", "| new | main | 300 | 63.40% | 500 | 50.00% |", "| social | main | 100 | 36.60% | 900 | 50.00% |", "| reddit-wild | 900 | 50.00% | 0.78 |", "| sdxl | 100 | 16.67% | 2.33 |", "each generator is its own family"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    // Weights, a family map and a replay pool, through a weights file and the flags.
    std::fs::write(dir.join("map.tsv"), "flux\tlatent-diffusion\nsdxl\tlatent-diffusion\nstylegan\tgan\n").unwrap();
    std::fs::write(dir.join("weights.tsv"), "# halve the reposts\nmanifest\tsocial\t0.5\n").unwrap();
    let d = |name: &str| dir.join(name).to_str().unwrap().to_string();
    let o = tintype(&["sampling-shares", "--manifest", &new, "--manifest", &social, "--val-pct", "0", "--weights-file", &d("weights.tsv"), "--family-weight", "latent-diffusion=2", "--family-map", &d("map.tsv"), "--replay-manifest", &old, "--replay-share", "0.25"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    // Main pool, AI: (20 + 10) x 2 against 30 x 0.5, so 80% and 20% of the 75% that is not replay.
    for want in ["replay pool of 1 real sources, 1 generators at 25% / 25% of real / AI draws", "| old | replay | 100 | 25.00% | 400 | 25.00% |", "| latent-diffusion | 500 | 60.00% |", "| reddit-wild | 900 | 15.00% |", "| gan | 400 | 25.00% |"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    // A suggested weights file moves the family shares to equal, and is read back by the same command.
    let o = tintype(&["sampling-shares", "--manifest", &new, "--manifest", &social, "--val-pct", "0", "--family-map", &d("map.tsv"), "--rebalance-families", "0", "--write-weights", &d("equal.tsv")]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let o = tintype(&["sampling-shares", "--manifest", &new, "--manifest", &social, "--val-pct", "0", "--family-map", &d("map.tsv"), "--weights-file", &d("equal.tsv")]);
    assert!(stdout(&o).contains("| latent-diffusion | 500 | 50.00% |") && stdout(&o).contains("| reddit-wild | 900 | 50.00% |"), "{}", stdout(&o));
    // A flag replaces the file's weight for the same name: back to 1, the shares are the plain ones.
    let o = tintype(&["sampling-shares", "--manifest", &new, "--manifest", &social, "--val-pct", "0", "--weights-file", &d("weights.tsv"), "--manifest-weight", "social=1"]);
    assert!(stdout(&o).contains("| social | main | 100 | 36.60% | 900 | 50.00% |"), "{} {}", stdout(&o), String::from_utf8_lossy(&o.stderr));
    // A weight that names nothing is refused rather than ignored.
    let o = tintype(&["sampling-shares", "--manifest", &new, "--manifest-weight", "socials=2"]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("no manifest is called socials"), "{}", String::from_utf8_lossy(&o.stderr));
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
    assert_eq!(out.lines().count(), 1, "without --photo-second-opinion there is no second line: {out}");
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
    assert!(err.lines().any(|l| l.starts_with("stats: ")), "{err}");
    assert_eq!(stdout(&o).split(':').next(), out.split(':').next());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn latency_with_a_photo_second_opinion_needs_a_band_and_the_default_profile() {
    let dir = common::scratch("cli-latency-second");
    common::write_fixture(&dir.join("0.png"));
    std::fs::write(dir.join("manifest.tsv"), "0.png\t0\treal:fixture\tdemo\n").unwrap();
    let manifest = dir.join("manifest.tsv");
    // A copy of the released model without its band: the shipped files are never edited.
    let weights = dir.join("copy.safetensors");
    std::fs::copy(common::photo_weights(), &weights).unwrap();
    std::fs::copy(common::photo_weights().with_extension("json"), weights.with_extension("json")).unwrap();
    let o = tintype(&["set-photo-second-opinion", "--weights", weights.to_str().unwrap(), "--band", "none"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let second = dir.join("no-such-weights.safetensors");
    let run = |weights: &Path, extra: &[&str]| {
        let mut args = vec!["latency", "--weights", weights.to_str().unwrap(), "--manifest", manifest.to_str().unwrap(), "--n", "1", "--device", "cpu", "--photo-second-opinion", second.to_str().unwrap()];
        args.extend_from_slice(extra);
        tintype(&args)
    };
    let o = run(&weights, &[]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("has no second-opinion band for photos"), "{}", String::from_utf8_lossy(&o.stderr));
    // With the band of the released spec, the second network is needed, and its weights are missing here.
    let o = run(&common::photo_weights(), &[]);
    assert!(!o.status.success() && !String::from_utf8_lossy(&o.stderr).contains("second-opinion band"), "{}", String::from_utf8_lossy(&o.stderr));
    for conflicting in [&["--fast"][..], &["--reference", "commfor"][..]] {
        let o = run(&common::photo_weights(), conflicting);
        assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("cannot be used with"), "{}", String::from_utf8_lossy(&o.stderr));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A JSON row without the review fields.
fn without_review(row: &serde_json::Value) -> serde_json::Value {
    let mut r = row.clone();
    if let Some(o) = r.as_object_mut() {
        o.remove("review");
        o.remove("review_reason");
    }
    r
}

/// Whether two JSON values are the same but for the last bits of their
/// numbers. Two runs of the CPU engine over several files can put the views
/// into batches differently, and with Accelerate a view's logit depends on
/// what it is run with (by about 1e-6), so rows from separate runs are
/// compared this way and not bit for bit.
fn same_but_for_rounding(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value;
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y || matches!((x.as_f64(), y.as_f64()), (Some(x), Some(y)) if (x - y).abs() < 1e-4),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same_but_for_rounding(x, y)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(k, x)| y.get(k).is_some_and(|y| same_but_for_rounding(x, y))),
        _ => a == b,
    }
}

#[test]
fn review_is_off_by_default_and_only_adds_fields_when_on() {
    let dir = common::scratch("cli-review");
    let (a, b, bad) = (dir.join("a.png"), dir.join("b.png"), dir.join("broken.png"));
    common::write_fixture(&a);
    image::imageops::flip_vertical(&common::fixture_pixels()).save(&b).unwrap();
    std::fs::write(&bad, b"not an image").unwrap();
    let video = common::repo().join("tests/fixtures/video.mp4");
    let files: [&Path; 4] = [&a, &video, &bad, &b];

    // Off: no review key in JSON, no review note in text.
    let plain = detect(&files, &["--json", "--frames", "4"]);
    let plain_rows = json_lines(&plain);
    assert_eq!(plain_rows.len(), 4);
    assert!(plain_rows.iter().all(|r| r.get("review").is_none() && r.get("review_reason").is_none()), "{}", stdout(&plain));
    let plain_text = detect(&files, &["--frames", "4"]);
    assert!(!stdout(&plain_text).contains("needs review"), "{}", stdout(&plain_text));

    for margin in [0.000001f64, 0.75, 3.0, 50.0] {
        let m = margin.to_string();
        let on = detect(&files, &["--json", "--frames", "4", "--review-margin", &m]);
        // The exit status does not change: 2 here, for the unreadable file.
        assert_eq!(on.status.code(), plain.status.code());
        let rows = json_lines(&on);
        assert_eq!(rows.len(), 4);
        for (row, before) in rows.iter().zip(&plain_rows) {
            // Everything that was there is there unchanged, the verdict and the score included.
            assert!(same_but_for_rounding(&without_review(row), before), "margin {margin}: {row} against {before}");
            if row.get("error").is_some() {
                // An unreadable file has no verdict to review.
                assert!(row.get("review").is_none(), "{row}");
                continue;
            }
            // The fields are consistent with the score: its logit is 0 at the threshold.
            let score = score_of(row).unwrap();
            let calibrated = (score / (1.0 - score)).ln();
            let review = row["review"].as_bool().unwrap_or_else(|| panic!("no review field in {row}"));
            if (calibrated.abs() - margin).abs() > 1e-3 {
                assert_eq!(review, calibrated.abs() < margin, "margin {margin}: {row}");
            }
            assert_eq!(row.get("review_reason").and_then(|r| r.as_str()), review.then_some("margin"), "{row}");
        }
        // Text: the same lines, with a note on the ones that need review.
        let text = detect(&files, &["--frames", "4", "--review-margin", &m]);
        assert_eq!(text.status.code(), plain_text.status.code());
        let (with_note, before) = (stdout(&text), stdout(&plain_text));
        assert_eq!(with_note.lines().count(), 4);
        for ((line, old), row) in with_note.lines().zip(before.lines()).zip(&rows) {
            let expect = if row.get("review").and_then(|r| r.as_bool()) == Some(true) { format!("{old}  [needs review: margin]") } else { old.to_string() };
            assert_eq!(line, expect);
        }
    }
    // The widest band marks every scored file and the narrowest marks none.
    let all = json_lines(&detect(&files, &["--json", "--frames", "4", "--review-margin", "50"]));
    assert!(all.iter().filter(|r| r.get("error").is_none()).all(|r| r["review"] == true));
    let none = json_lines(&detect(&files, &["--json", "--frames", "4", "--review-margin", "0"]));
    assert!(none.iter().filter(|r| r.get("error").is_none()).all(|r| r["review"] == false && r.get("review_reason").is_none()));

    // Disagreement alone: `view_disagreement` is the only reason it can give.
    let split = json_lines(&detect(&files, &["--json", "--frames", "4", "--all-views", "--review-disagreement", "0.01"]));
    for (row, before) in split.iter().zip(json_lines(&detect(&files, &["--json", "--frames", "4", "--all-views"]))) {
        assert!(same_but_for_rounding(&without_review(row), &before), "{row} against {before}");
        if let Some(review) = row.get("review").and_then(|r| r.as_bool()) {
            assert_eq!(row.get("review_reason").and_then(|r| r.as_str()), review.then_some("view_disagreement"), "{row}");
        }
    }

    // `batch` takes the same flag.
    let o = tintype(&["batch", "--device", "cpu", "--json", "--review-margin", "50", a.to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(json_lines(&o)[0]["review_reason"], "margin");

    // A malformed band is a usage error, before any inference.
    for flags in [["--review-margin", "-1"], ["--review-margin", "NaN"], ["--review-disagreement", "0"], ["--review-disagreement", "1.5"]] {
        let o = detect(&[&a], &[flags[0], flags[1]]);
        assert_eq!(o.status.code(), Some(2), "{flags:?}");
        assert!(o.stdout.is_empty());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_review_band_in_the_spec_applies_to_the_default_profile_only() {
    let dir = common::scratch("cli-review-spec");
    let png = dir.join("fixture.png");
    common::write_fixture(&png);
    // A candidate checkpoint: the released weights (linked, not rewritten) with a band in a spec of its own.
    let weights = dir.join("candidate.safetensors");
    if std::fs::hard_link(common::photo_weights(), &weights).is_err() {
        std::fs::copy(common::photo_weights(), &weights).unwrap();
    }
    let mut spec: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(common::repo().join("models/tintype.json")).unwrap()).unwrap();
    spec["review"] = serde_json::json!({"margin": 50.0});
    std::fs::write(dir.join("candidate.json"), serde_json::to_string_pretty(&spec).unwrap()).unwrap();
    let run = |extra: &[&str]| {
        let mut args = vec!["detect", "--device", "cpu", "--json", "--weights", weights.to_str().unwrap()];
        args.extend_from_slice(extra);
        args.push(png.to_str().unwrap());
        json_lines(&tintype(&args)).remove(0)
    };
    let stored = run(&[]);
    assert_eq!((stored["review"].as_bool(), stored["review_reason"].as_str()), (Some(true), Some("margin")), "{stored}");
    // The released spec gives the same result without the fields.
    assert_eq!(without_review(&stored), json_lines(&detect(&[&png], &["--json"])).remove(0));
    // A flag overrides the stored margin.
    assert_eq!(run(&["--review-margin", "0"])["review"], false);
    // The stored band is in the default profile's calibrated logits: other profiles ignore it ...
    assert!(run(&["--strict"]).get("review").is_none());
    // ... unless a flag gives one.
    assert_eq!(run(&["--strict", "--review-margin", "50"])["review"], true);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn report_accounts_for_review_and_transfer_only_when_asked() {
    let dir = common::scratch("cli-review-report");
    // 200 real items with logits -6.01..1.95 and 200 AI items with logits -3.01..4.95, in steps of 0.04
    // (off the band edges, so rounding decides nothing): prob = sigmoid(logit). Each has two "view"
    // logits, 1 either side of it for real items and 3 either side for AI items.
    let sigmoid = |z: f64| 1.0 / (1.0 + (-z).exp());
    let mut text = String::new();
    for i in 0..200 {
        let z = -4.01 + 0.04 * i as f64;
        text.push_str(&format!("r{i}.jpg\t0\treal:{}\tdemo\t{:.6}\t{:.5}\t{:.5}\n", if i < 150 { "cam" } else { "phone" }, sigmoid(z - 2.0), z - 2.0 + 1.0, z - 2.0 - 1.0));
        text.push_str(&format!("f{i}.png\t1\tgen-a\tdemo\t{:.6}\t{:.5}\t{:.5}\n", sigmoid(z + 1.0), z + 1.0 + 3.0, z + 1.0 - 3.0));
    }
    let scores = dir.join("scores.tsv");
    std::fs::write(&scores, text).unwrap();
    let system = format!("demo={}@0.5", scores.display());
    let base = tintype(&["report", "--system", &system, "--operating-points", "--slice-by", "generator"]);
    assert!(base.status.success(), "{}", String::from_utf8_lossy(&base.stderr));
    assert!(!stdout(&base).contains("review") && !stdout(&base).contains("transfer"), "{}", stdout(&base));

    let o = tintype(&["report", "--system", &system, "--operating-points", "--slice-by", "generator", "--review-margin", "0.5,1", "--transfer-budget", "0.05"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    // Every line of the plain report is still there, in order.
    let mut rest = out.lines();
    for line in stdout(&base).lines() {
        assert!(rest.any(|l| l == line), "the plain report's line is missing or out of order: {line}");
    }
    // 49 real and 124 AI items are above 0 (flagged). A band of half-width 0.5 holds 25 of each class,
    // 12 of them flagged; one of half-width 1 holds 50 of each, 25 of them flagged.
    for want in [
        "demo, review accounting at the frozen cutoff 0.5.",
        "| none | 100.00% | 0 of 200 (0.00%) | 0 of 200 (0.00%) | 0 of 0 | 49 of 200 | 24.50% |",
        "| margin 0.5 | 87.50% | 25 of 200 (12.50%) | 25 of 200 (12.50%) | 50 of 50 | 37 of 200 | 18.50% |",
        "| margin 1 | 75.00% | 50 of 200 (25.00%) | 50 of 200 (25.00%) | 100 of 100 | 24 of 200 | 12.00% |",
        "demo transfer by generator: authentic items flagged at the frozen cutoff 0.5, against a false-positive budget of 5.00%.",
        "| real:cam | 150 | 0 | 0.00% | 0.00% | 1.98% | within |",
        "| real:phone | 50 | 49 | 98.00% |",
        "| exceeds |",
        "1 of 2 authentic generator strata are shown within the budget, 1 exceed it, 0 are not resolved.",
    ] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    // Recall with reviewed AI items as misses: 112 of 200 at margin 0.5, against 124 of 200 without the band.
    let row = out.lines().find(|l| l.starts_with("| margin 0.5 |")).unwrap();
    assert!(row.contains("| 56.00% (") && row.contains("| 62.00% |"), "{row}");

    // Disagreement needs the raw threshold; with it, rows whose two logits straddle it are reviewed too.
    let o = tintype(&["report", "--system", &system, "--operating-points", "--review-margin", "0", "--review-disagreement", "0.5"]);
    assert!(!o.status.success());
    let o = tintype(&["report", "--system", &system, "--operating-points", "--review-margin", "0", "--review-disagreement", "0.5", "--review-threshold-logit", "0"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    // A real row's two logits straddle 0 when its own is within 1 of it (50 rows); an AI row's within 3 (150 rows).
    assert!(stdout(&o).contains("| margin 0, or disagreement of 0.5 or more | 50.00% | 50 of 200 (25.00%) | 150 of 200 (75.00%) | 0 of 200 |"), "{}", stdout(&o));

    // Decisions recorded by `detect --json`, or as plain rows.
    let mut flags = String::from("{\"path\":\"unreadable.jpg\",\"kind\":\"image\",\"error\":\"could not decode\"}\n");
    for i in 0..200 {
        flags.push_str(&format!("{{\"path\":\"r{i}.jpg\",\"review\":{}}}\n", i == 0));
        flags.push_str(&format!("f{i}.png\t{}\tview_disagreement\n", (i >= 190) as u8));
    }
    let recorded = dir.join("flags.jsonl");
    std::fs::write(&recorded, &flags).unwrap();
    let o = tintype(&["report", "--system", &system, "--operating-points", "--review-flags", recorded.to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    // One real and the ten highest AI items reviewed: recall 114 of 200 with them as misses.
    assert!(stdout(&o).contains("| recorded decisions | 97.25% | 1 of 200 (0.50%) | 10 of 200 (5.00%) | 1 of 11 | 49 of 200 | 24.50% |"), "{}", stdout(&o));
    assert!(stdout(&o).lines().find(|l| l.starts_with("| recorded decisions |")).unwrap().contains("| 57.00% ("));
    // A scored item without a recorded decision is an error, not a silent "not reviewed".
    std::fs::write(&recorded, "r0.jpg\t1\n").unwrap();
    let o = tintype(&["report", "--system", &system, "--operating-points", "--review-flags", recorded.to_str().unwrap()]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("399 of 400 scored items have no row"), "{}", String::from_utf8_lossy(&o.stderr));
    // The review flags belong to the fixed-budget tables.
    assert!(!tintype(&["report", "--system", &system, "--review-margin", "1"]).status.success());
    let _ = std::fs::remove_dir_all(&dir);
}

/// An all-views score file of `n` items: reals around -4, AI around +6, a
/// seventh of them hard, six logits each from integer arithmetic.
fn write_view_scores(path: &Path, n: usize, salt: u32) {
    let mut text = String::new();
    for i in 0..n {
        let label = i % 2;
        let hard = i % 7 == 0;
        let noise = |k: u32, scale: f64| {
            let mut h = (i as u32).wrapping_mul(0x9E37_79B1) ^ k.wrapping_mul(0x85EB_CA77) ^ salt.wrapping_mul(0xC2B2_AE35);
            h ^= h >> 15;
            h = h.wrapping_mul(0x2C1B_3C6D);
            h ^= h >> 12;
            ((h % 2001) as f64 / 1000.0 - 1.0) * scale
        };
        let centre = if hard { 1.5 + noise(100, 2.5) } else if label == 1 { 6.0 + noise(100, 3.0) } else { -4.0 + noise(100, 3.0) };
        let views: Vec<String> = (0..6).map(|k| format!("{:.5}", centre + noise(k, if hard { 4.0 } else { 2.0 }))).collect();
        let (generator, dataset) = if label == 0 { (if i % 4 == 0 { "real:cam" } else { "real:phone" }, "demo") } else { ("gen-a", "demo") };
        text.push_str(&format!("{salt}-{i}.jpg\t{label}\t{generator}\t{dataset}\t0.5\t{}\n", views.join("\t")));
    }
    std::fs::write(path, text).unwrap();
}

#[test]
fn refit_exit_checks_fits_and_never_writes_under_models() {
    let dir = common::scratch("cli-refit");
    // The tool reads the spec beside --weights and score files; it needs no weights and scores nothing.
    let spec_text = std::fs::read_to_string(common::repo().join("models/tintype.json")).unwrap();
    std::fs::write(dir.join("candidate.json"), &spec_text).unwrap();
    let weights = dir.join("candidate.safetensors");
    let (fit, report, decided) = (dir.join("fit.tsv"), dir.join("report.tsv"), dir.join("decided.tsv"));
    write_view_scores(&fit, 3000, 1);
    write_view_scores(&report, 2000, 2);
    let w = weights.to_str().unwrap();

    // Check only: the stored schedule against scoring every view, per profile, per source.
    let o = tintype(&["refit-exit", "--weights", w, "--profile", "photo,strict", "--report", report.to_str().unwrap(), "--target-fpr", "0.015,0.005", "--apply-out", decided.to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    for want in ["stored schedule, order [2, 0, 1, 3, 4, 5], margins after N views (raw logits) 1: 6, 2: 3, 3: 2, 4: 1.5, 5: 1", "## photo, threshold 1.692 (raw logit), stored schedule", "## strict, threshold 3.39", "2000 items (1000 AI, 1000 real);", "verdicts that differ from every view:", "| every view |", "| early exit |", "| real:cam | 500 |", "| real:phone | 500 |", "Mean over real sources with at least 100 items:", "the calibration target is 1.50%", "the calibration target is 0.50%", "photo second-opinion band (-7, 0.5) in calibrated logits", "2000 items as photo decides them under the stored schedule"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    // The applied file is a score file `report` reads: one row per item, the logits those looked at.
    let rows = std::fs::read_to_string(&decided).unwrap();
    assert_eq!(rows.lines().count(), 2000);
    assert!(rows.lines().all(|l| (6..=11).contains(&l.split('\t').count())) && rows.lines().any(|l| l.split('\t').count() == 6));
    // Nothing was written without --write.
    assert_eq!(std::fs::read_to_string(dir.join("candidate.json")).unwrap(), spec_text);

    // Fit on one file, report on the other, and store the result in the copy.
    let o = tintype(&["refit-exit", "--weights", w, "--fit", fit.to_str().unwrap(), "--report", report.to_str().unwrap(), "--write"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    for want in ["fitted on 3000 items, pooled precision and recall within 0.001 of every view at photo: margins after N views 1: ", "## photo, threshold 1.692 (raw logit), fitted schedule", "wrote the fitted schedule to "] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    assert!(!out.contains("NOTE: a file is both fitted on and reported on"), "{out}");
    let written: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("candidate.json")).unwrap()).unwrap();
    let before: serde_json::Value = serde_json::from_str(&spec_text).unwrap();
    assert_eq!(written["early_exit"]["order"], before["early_exit"]["order"]);
    assert_ne!(written["early_exit"]["stages"], before["early_exit"]["stages"]);
    // Only the schedule changed.
    let mut same = written.clone();
    same["early_exit"] = before["early_exit"].clone();
    assert_eq!(same, before);
    // Fitting and reporting on the same file says so.
    let o = tintype(&["refit-exit", "--weights", w, "--fit", fit.to_str().unwrap(), "--report", fit.to_str().unwrap()]);
    assert!(stdout(&o).contains("NOTE: a file is both fitted on and reported on"), "{}", stdout(&o));

    // The released spec is refused before anything is read, and stays what it was.
    let o = tintype(&["refit-exit", "--weights", "models/tintype.safetensors", "--fit", fit.to_str().unwrap(), "--report", report.to_str().unwrap(), "--write"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("refusing to write") && o.stdout.is_empty(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(std::fs::read_to_string(common::repo().join("models/tintype.json")).unwrap(), spec_text);
    let (lines, ok) = tintype::integrity::check_models(&common::repo());
    assert!(ok, "{}", lines.join("\n"));
    // A file scored with early exit does not hold every view.
    let o = tintype(&["refit-exit", "--weights", w, "--report", decided.to_str().unwrap(), "--profile", "video"]);
    assert!(!o.status.success());
    let _ = std::fs::remove_dir_all(&dir);
}

/// `calibrate` is one of the commands that only build on macOS. It reads
/// score files and the spec beside `--weights`, and runs no network.
#[cfg(target_os = "macos")]
#[test]
fn calibrate_bound_rule_counts_parents_and_says_how_many_it_needs() {
    let dir = common::scratch("cli-calibrate-bound");
    let spec_text = std::fs::read_to_string(common::repo().join("models/tintype.json")).unwrap();
    let fresh = |name: &str| {
        std::fs::write(dir.join(format!("{name}.json")), &spec_text).unwrap();
        dir.join(format!("{name}.safetensors")).to_str().unwrap().to_string()
    };
    // 400 real parents, each an original and a JPEG copy, and 400 AI items. Real logits rise with the parent.
    let (mut scores, mut manifest) = (String::new(), String::new());
    for p in 0..400 {
        let z = -6.0 + p as f64 * 0.01;
        for (suffix, shift, transform) in [("", 0.0, ""), ("-jpeg", 0.5, "jpeg-q70")] {
            scores.push_str(&format!("{}/r{p}{suffix}.jpg\t0\treal:cam\tdemo\t0.1\t{:.5}\t{:.5}\n", dir.display(), z + shift, z + shift));
            manifest.push_str(&format!("r{p}{suffix}.jpg\t0\treal:cam\tdemo\tp{p}\t\t\t\t{transform}\n"));
        }
        scores.push_str(&format!("{}/f{p}.png\t1\tgen-a\tdemo\t0.9\t{:.5}\t{:.5}\n", dir.display(), 2.0 + p as f64 * 0.01, 2.0 + p as f64 * 0.01));
    }
    let (score_file, manifest_file) = (dir.join("calibrate.tsv"), dir.join("manifest.tsv"));
    std::fs::write(&score_file, scores).unwrap();
    std::fs::write(&manifest_file, manifest).unwrap();
    let s = score_file.to_str().unwrap();

    // The default rule writes what it always wrote, and now also says what bound it achieved.
    let w = fresh("balanced");
    let o = tintype(&["calibrate", "--weights", &w, "--scores", s, "--target-fpr", "0.01"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    assert!(out.contains("photo: 800 real / 400 AI items; logit threshold ") && out.contains("  flagged at the threshold: 8 of 800 real items, taken as independent; one-sided 95% Clopper–Pearson upper bound on the false-positive rate 1.797%"), "{out}");

    // Items: 800 look like enough to show 1% (299 needed), and the bound rule flags 3 of them.
    let w = fresh("items");
    let o = tintype(&["calibrate", "--weights", &w, "--scores", s, "--target-fpr", "0.01", "--rule", "bound"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(stdout(&o).contains("flagged at the threshold: 3 of 800 real items, taken as independent; one-sided 95% Clopper–Pearson upper bound on the false-positive rate 0.966%"), "{}", stdout(&o));
    // Parents: 400 independent units, one score each (the higher, here the JPEG copy's). One flag
    // among 400 would put the bound at 1.18%, so none may be flagged.
    let w = fresh("parents");
    let o = tintype(&["calibrate", "--weights", &w, "--scores", s, "--target-fpr", "0.01", "--rule", "bound", "--groups", manifest_file.to_str().unwrap(), "--group-diagnostic", "transform"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    for want in ["flagged at the threshold: 0 of 400 real parents (800 items, one score per parent: the highest of its versions); one-sided 95% Clopper–Pearson upper bound on the false-positive rate 0.746%", "  real:cam                     n=800   false positives at threshold ", "DIAGNOSTIC ONLY, by transform", "[diagnostic] jpeg-q70", "[diagnostic] none"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    let o = tintype(&["calibrate", "--weights", &fresh("original"), "--scores", s, "--target-fpr", "0.01", "--rule", "bound", "--groups", manifest_file.to_str().unwrap(), "--per-parent", "original"]);
    assert!(stdout(&o).contains("0 of 400 real parents (800 items, one score per parent: its unprocessed original)"), "{}", stdout(&o));
    // The threshold was written to the copy, as the calibrated zero of the photo profile.
    let written: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("parents.json")).unwrap()).unwrap();
    let thr = -written["logit_bias"].as_f64().unwrap() / written["logit_scale"].as_f64().unwrap();
    // Flag strictly above the highest parent score, the top JPEG copy's: -6 + 399 * 0.01 + 0.5.
    assert!((thr - (-6.0 + 3.99 + 0.5)).abs() < 1e-3, "{thr}");

    // 0.1% cannot be shown from 400 parents: the error says how many it takes, and nothing is written.
    let w = fresh("short");
    let o = tintype(&["calibrate", "--weights", &w, "--scores", s, "--target-fpr", "0.001", "--rule", "bound", "--groups", manifest_file.to_str().unwrap()]);
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr).to_string();
    assert!(err.contains("at least 2995 with none flagged") && err.contains("these scores hold 400"), "{err}");
    assert_eq!(std::fs::read_to_string(dir.join("short.json")).unwrap(), spec_text);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn interpolate_end_points_score_like_their_sources_and_nothing_is_written_to_models() {
    let dir = common::scratch("cli-interpolate");
    let png = dir.join("fixture.png");
    common::write_fixture(&png);
    // The two released checkpoints are sibling fine-tunes with one layout.
    let (a, b) = (common::photo_weights(), common::repo().join("models/tintype-video.safetensors"));
    let sweep = dir.join("sweep");
    let o = tintype(&["interpolate", "--a", a.to_str().unwrap(), "--b", b.to_str().unwrap(), "--alpha", "0,0.5,1", "--out-dir", sweep.to_str().unwrap()]);
    assert!(o.status.success(), "{}{}", stdout(&o), String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    assert!(out.contains("agree on all 152 tensors"), "{out}");
    assert!(out.contains(&format!("alpha-0.00.safetensors: alpha 0: every tensor is bit for bit that of {}", a.display())), "{out}");
    assert!(out.contains(&format!("alpha-1.00.safetensors: alpha 1: every tensor is bit for bit that of {}", b.display())), "{out}");

    let score = |weights: &Path| -> (serde_json::Value, String) {
        let o = tintype(&["detect", "--device", "cpu", "--json", "--weights", weights.to_str().unwrap(), png.to_str().unwrap()]);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        (json_lines(&o).remove(0), String::from_utf8_lossy(&o.stderr).to_string())
    };
    let mut logits = Vec::new();
    for (alpha, source) in [("0.00", &a), ("1.00", &b)] {
        // The source run through the sweep's spec: the same preprocessing and the same (identity) calibration.
        let same_path = dir.join(format!("source-{alpha}.safetensors"));
        std::fs::copy(source, &same_path).unwrap();
        std::fs::copy(sweep.join(format!("alpha-{alpha}.json")), same_path.with_extension("json")).unwrap();
        let (end_point, warning) = score(&sweep.join(format!("alpha-{alpha}.safetensors")));
        let (reference, _) = score(&same_path);
        assert_eq!(end_point["logit"], reference["logit"], "alpha {alpha}");
        assert_eq!(end_point["score"], reference["score"], "alpha {alpha}");
        assert_eq!(end_point["views"], 6, "{end_point}");
        // The interpolant says on standard error that it is not calibrated; a finished model does not.
        assert!(warning.contains("is not calibrated (photo, fast, video)"), "{warning}");
        logits.push(end_point["logit"].as_f64().unwrap());
        std::fs::remove_file(&same_path).unwrap();
    }
    let (middle, _) = score(&sweep.join("alpha-0.50.safetensors"));
    assert!(middle["logit"].as_f64().unwrap() != logits[0] && middle["logit"].as_f64().unwrap() != logits[1], "{middle} {logits:?}");
    let o = detect(&[&png], &[]);
    assert!(!String::from_utf8_lossy(&o.stderr).contains("not calibrated"), "{}", String::from_utf8_lossy(&o.stderr));

    // Uncalibrated weights are not served.
    let o = tintype(&["serve", "--device", "cpu", "--bind", "127.0.0.1:0", "--weights", sweep.join("alpha-0.50.safetensors").to_str().unwrap()]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("refusing to serve uncalibrated weights"), "{}", String::from_utf8_lossy(&o.stderr));

    // Nothing goes into models/, even when asked to overwrite; a second sweep does not replace the first.
    let models = common::repo().join("models");
    let before: Vec<_> = std::fs::read_dir(&models).unwrap().map(|e| e.unwrap().file_name()).collect();
    for out_dir in [models.clone(), models.join("sweep")] {
        let o = tintype(&["interpolate", "--a", a.to_str().unwrap(), "--b", b.to_str().unwrap(), "--alpha", "0.5", "--out-dir", out_dir.to_str().unwrap(), "--overwrite"]);
        assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("is a models directory"), "{}", String::from_utf8_lossy(&o.stderr));
    }
    let o = tintype(&["interpolate", "--average", a.to_str().unwrap(), b.to_str().unwrap(), "--out", a.to_str().unwrap(), "--overwrite"]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("is a models directory"), "{}", String::from_utf8_lossy(&o.stderr));
    let after: Vec<_> = std::fs::read_dir(&models).unwrap().map(|e| e.unwrap().file_name()).collect();
    assert_eq!(before, after);
    let o = tintype(&["interpolate", "--a", a.to_str().unwrap(), "--b", b.to_str().unwrap(), "--alpha", "0.5", "--out-dir", sweep.to_str().unwrap()]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("already exists"), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(tintype(&["verify-models"]).status.success());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn replay_pool_refits_each_pool_on_the_fit_file_and_reports_on_the_test_file() {
    let dir = common::scratch("cli-replay-pool");
    // Six view logits per image around a level: real near -4, AI near +4, every seventh image near the threshold.
    let write = |name: &str, salt: u32, n: u32| {
        let mut text = String::new();
        for i in 0..n {
            let label = i % 2;
            let noise = |v: u32| ((i.wrapping_mul(2_654_435_761).wrapping_add(v.wrapping_mul(40_503)).wrapping_add(salt) >> 8) % 1000) as f32 / 1000.0 - 0.5;
            let level = if i % 7 == 0 { 1.5 + 3.0 * noise(9) } else if label == 1 { 5.0 + 3.0 * noise(9) } else { -5.0 + 3.0 * noise(9) };
            let logits: Vec<String> = (0..6).map(|v| format!("{:.5}", level + 3.0 * noise(v))).collect();
            text.push_str(&format!("{name}/{i}.png\t{label}\t{}\tset-{}\t0.5\t{}\n", if label == 1 { "gen".to_string() } else { format!("real:cam{}", (i / 2) % 2) }, i % 3, logits.join("\t")));
        }
        let p = dir.join(format!("{name}.all-views.tsv"));
        std::fs::write(&p, text).unwrap();
        p
    };
    let fit = write("fit", 1, 3000);
    write("test", 77, 2000);
    // A test set may name several files, and a `*` in a file name matches (here test.all-views.tsv).
    std::fs::write(dir.join("empty.tsv"), "").unwrap();
    let test_arg = format!("held-out={},{}", dir.join("test.*.tsv").display(), dir.join("empty.tsv").display());
    let spec = common::repo().join("models/tintype.json");
    let o = tintype(&["replay-pool", "--fit", fit.to_str().unwrap(), "--test", &test_arg, "--spec", spec.to_str().unwrap(), "--pool", "mean,median,top2,max,fitted", "--order", "shipped", "--order", "greedy", "--slice-by", "dataset", "--write-scores", dir.join("scores").to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    for want in [
        "Fit items: 3000 (1500 AI, 1500 real) from 1 file(s). Target false-positive rate 1.50%, averaged over the 2 real sources with at least 100 items.",
        "| shipped (mean, spec as is) | 2,0,1,3,4,5 | 1.692 | 0.664 | 6 / 3 / 2 / 1.5 / 1 |",
        "| mean | 2,0,1,3,4,5 |",
        "| top 2 |",
        "| fitted weights ",
        "(greedy)",
        "`early_exit` for max, order 2,0,1,3,4,5",
        "### Test: held-out (2000 items: 1000 AI, 1000 real)",
        "| shipped (mean, spec as is), all views |",
        "held-out by dataset: median | 2,0,1,3,4,5",
        "| set-0 |",
    ] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    // The shipped row is the reference: it changes no verdict against itself.
    assert!(out.lines().any(|l| l.starts_with("| shipped (mean, spec as is) | 2,0,1,3,4,5 |") && l.ends_with("| +0 −0 AI, +0 −0 real |")), "{out}");
    // The files written are ordinary score files that `report` reads.
    let system = format!("median={}", dir.join("scores/held-out.median.201345.tsv").display());
    let o = tintype(&["report", "--system", &system, "--operating-points"]);
    assert!(o.status.success() && stdout(&o).contains("2000 items scored by all 1 systems: 1000 AI, 1000 real"), "{}{}", stdout(&o), String::from_utf8_lossy(&o.stderr));

    // A file written with early exit (fewer logits per image) cannot be replayed, and the error says what can.
    std::fs::write(dir.join("short.tsv"), "a.png\t1\tgen\tset\t0.9\t4.0\t5.0\n").unwrap();
    let o = tintype(&["replay-pool", "--fit", dir.join("short.tsv").to_str().unwrap(), "--spec", spec.to_str().unwrap()]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("has 2 view logits, not 6"), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(tintype(&["verify-models"]).status.success());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn augment_bench_times_the_loader_with_and_without_chains() {
    let dir = common::scratch("cli-augment-bench");
    let mut text = String::new();
    for i in 0..4 {
        common::write_fixture(&dir.join(format!("{i}.png")));
        text.push_str(&format!("{i}.png\t{}\treal:fixture\tdemo\n", i % 2));
    }
    std::fs::write(dir.join("manifest.tsv"), text).unwrap();
    let manifest = dir.join("manifest.tsv");
    let run = |extra: &[&str]| {
        let mut args = vec!["augment-bench", "--manifest", manifest.to_str().unwrap(), "--images", "4", "--rounds", "2", "--threads", "2"];
        args.extend_from_slice(extra);
        tintype(&args)
    };
    let o = run(&[]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    assert!(out.starts_with("augmentation: the phase-6 recipe\n4 photos, ") && out.contains("\ndecode ") && out.contains("\naugment ") && out.contains("\nloader "), "{out}");
    let o = run(&["--p-chain", "1", "--chain-ops", "1,5", "--chain-severity", "1", "--chain-kinds", "resize,jpeg,blur,noise,sharpen,repost,motion-blur,lens-blur,shot-noise,impulse-noise,pixelate,tone", "--messenger-cycles", "1,6", "--messenger-cap", "720,2048"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = stdout(&o);
    assert!(out.starts_with("augmentation: messenger cap (720.0, 2048.0) px") && out.contains("chains p 1 of 1-5 operations from [resize,jpeg,blur,noise,sharpen,repost,motion-blur"), "{out}");
    // A value that makes no sense is refused before any file is read.
    let o = run(&["--chain-kinds", "webp"]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("unknown chain operation `webp`"), "{}", String::from_utf8_lossy(&o.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn hash_split_writes_the_two_sides_train_holds_apart() {
    let dir = common::scratch("cli-hash-split");
    write_family_manifest(&dir, 300);
    let d = |name: &str| dir.join(name).to_str().unwrap().to_string();
    let o = tintype(&["hash-split", "--manifest", &d("manifest.tsv"), "--out-dir", &d("sides")]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let side = |name: &str| std::fs::read_to_string(dir.join("sides").join(name)).unwrap();
    let (train, val) = (side("train.tsv"), side("val.tsv"));
    assert_eq!(train.lines().count() + val.lines().count(), 600);
    assert_eq!(stdout(&o), format!("{} items on the training side, {} held out (3% by path hash)\n", train.lines().count(), val.lines().count()));
    assert!((3..60).contains(&val.lines().count()), "{}", val.lines().count());
    // Absolute paths, grouping columns kept, and the same sides on a second run.
    assert!(train.lines().chain(val.lines()).all(|l| std::path::Path::new(l.split('\t').next().unwrap()).is_absolute()));
    assert!(train.lines().any(|l| l.ends_with("\tjpeg-q70")) && train.lines().any(|l| l.split('\t').count() == 5));
    let o = tintype(&["hash-split", "--manifest", &d("manifest.tsv"), "--out-dir", &d("again")]);
    assert!(o.status.success());
    assert_eq!(std::fs::read_to_string(dir.join("again/val.tsv")).unwrap(), val);
    // A larger share holds out more; zero holds out nothing.
    let o = tintype(&["hash-split", "--manifest", &d("manifest.tsv"), "--out-dir", &d("none"), "--val-pct", "0"]);
    assert!(stdout(&o).starts_with("600 items on the training side, 0 held out"), "{}", stdout(&o));
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
    let name = |p: &str| Path::new(p).file_name().unwrap().to_str().unwrap().to_string();
    assert_eq!(rows.iter().map(|r| (name(r[0]), r[1], r[5])).collect::<Vec<_>>(), [("a.mp4", "0", "scene-a.mp4"), ("b.mp4", "1", "scene-b.mp4"), ("c.mp4", "1", "scene-c.mp4"), ("d.mp4", "0", "scene-d.mp4")].map(|(n, l, s)| (n.to_string(), l, s)), "{clips}");
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

#[test]
fn coverage_modes_report_every_window_and_the_default_output_has_none_of_it() {
    let video = common::repo().join("tests/fixtures/video.mp4");
    let one = |extra: &[&str]| {
        let o = detect(&[&video], extra);
        assert!(o.status.success(), "{extra:?}: {}", String::from_utf8_lossy(&o.stderr));
        json_lines(&o).remove(0)
    };
    let floats = |v: &serde_json::Value| v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect::<Vec<f64>>();
    // Without the flag nothing is added.
    let plain = one(&["--json"]);
    assert!(plain.get("segments").is_none() && plain.get("coverage_mode").is_none() && plain.get("cuts_s").is_none(), "{plain}");

    // Early, middle and late windows share the profile's sixteen frames, and every one is scored.
    let row = one(&["--json", "--coverage", "segments"]);
    assert_eq!((row["coverage_mode"].as_str(), row["frames"].as_u64(), row["frames_expected"].as_u64(), row["complete"].as_bool()), (Some("segments"), Some(16), Some(16), Some(true)));
    assert!(["ai-generated", "likely-real"].contains(&row["verdict"].as_str().unwrap()));
    let segments = row["segments"].as_array().unwrap();
    assert_eq!(segments.iter().map(|s| s["name"].as_str().unwrap()).collect::<Vec<_>>(), vec!["early", "middle", "late"]);
    assert_eq!(segments.iter().map(|s| s["frames"].as_u64().unwrap()).collect::<Vec<_>>(), vec![6, 5, 5]);
    assert_eq!((segments[0]["start_s"].as_f64(), segments[2]["end_s"].as_f64()), (Some(0.0), Some(1.0)));
    let mut all = Vec::new();
    for s in segments {
        let (times, logits) = (floats(&s["sampled_s"]), floats(&s["frame_logits"]));
        assert_eq!((times.len(), logits.len()), (s["frames"].as_u64().unwrap() as usize, s["frames"].as_u64().unwrap() as usize));
        // A window's frames lie in it (the nearest frame may be a frame interval outside).
        assert!(times.iter().all(|t| *t >= s["start_s"].as_f64().unwrap() - 0.1 && *t <= s["end_s"].as_f64().unwrap() + 0.1), "{s}");
        // Its logit is the mean of its frames, and its score is below a half exactly when that is below the clip threshold.
        let mean = logits.iter().sum::<f64>() / logits.len() as f64;
        assert!((mean - s["logit"].as_f64().unwrap()).abs() < 1e-5, "{s}");
        assert_eq!(s["score"].as_f64().unwrap() > 0.5, mean > row["threshold_logit"].as_f64().unwrap(), "{s}");
        all.extend(logits);
    }
    // The clip's score is still the mean over every scored frame.
    assert!((all.iter().sum::<f64>() / all.len() as f64 - row["logit"].as_f64().unwrap()).abs() < 1e-5);

    // `uniform` is the default sampling with every frame reported: the same frames, so the same clip logit, as scoring all of them.
    let uniform = one(&["--json", "--coverage", "uniform", "--frames", "4", "--max-seconds", "0"]);
    let every = one(&["--json", "--all-views", "--frames", "4", "--max-seconds", "0"]);
    assert_eq!(floats(&uniform["segments"][0]["sampled_s"]), vec![0.0, 0.25, 0.5, 0.75]);
    assert_eq!(uniform["segments"][0]["name"], "span");
    assert!((uniform["logit"].as_f64().unwrap() - every["logit"].as_f64().unwrap()).abs() < 1e-5, "{uniform} vs {every}");
    assert_eq!(uniform["analysed_until_s"], every["analysed_until_s"]);

    // `shots` also says which cuts it found; two windows are numbered.
    let shots = one(&["--json", "--coverage", "shots", "--segments", "2", "--frames", "6"]);
    assert!(shots["cuts_s"].is_array(), "{shots}");
    assert_eq!(shots["segments"].as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap()).collect::<Vec<_>>(), vec!["1", "2"]);

    // The text line keeps its shape and gains the windows.
    let o = detect(&[&video], &["--coverage", "segments"]);
    let text = stdout(&o);
    assert!(text.lines().count() == 1 && text.contains("16 frames]  segments: early ") && text.contains(", late "), "{text}");

    // The window count needs a coverage mode, and the mode must be a known one.
    for flags in [vec!["--segments", "2"], vec!["--coverage", "everything"], vec!["--coverage", "segments", "--segments", "0"]] {
        let mut args = vec!["detect", "test.mp4"];
        args.extend(flags);
        let o = tintype(&args);
        assert_eq!(o.status.code(), Some(2), "{args:?}");
        assert!(o.stdout.is_empty());
    }
}

/// A 480 x 384 picture built from the fixture, different for each `seed`.
fn varied_picture(seed: u32) -> image::RgbImage {
    let base = common::fixture_pixels();
    image::RgbImage::from_fn(480, 384, |x, y| {
        let p = base.get_pixel((x + 37 * seed) % 320, (y + 53 * seed) % 256);
        image::Rgb([p[0], p[(1 + seed as usize) % 3], p[2] ^ (seed as u8 * 16)])
    })
}

#[test]
fn partial_edit_store_is_built_scored_and_reported_apart_from_the_global_score() {
    let dir = common::scratch("cli-partial");
    let d = |name: &str| dir.join(name).to_str().unwrap().to_string();
    let mut manifest = String::new();
    for k in 0..3u32 {
        varied_picture(k).save(dir.join(format!("real{k}.png"))).unwrap();
        varied_picture(k + 7).save(dir.join(format!("fake{k}.png"))).unwrap();
        manifest += &format!("real{k}.png\t0\treal:fixture\tfixtures\nfake{k}.png\t1\tgen-fixture\tfixtures\n");
    }
    std::fs::write(dir.join("manifest.tsv"), manifest).unwrap();
    let o = tintype(&["paste-edits", "--real", &d("manifest.tsv"), "--fake", &d("manifest.tsv"), "--out", &d("store"), "--parents", "3", "--areas", "0.12", "--blends", "hard", "--min-side", "224"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(stdout(&o).contains("15 items from 3 authentic parents"), "{}", stdout(&o));
    let store = std::fs::read_to_string(dir.join("store/manifest.tsv")).unwrap();
    let transforms: std::collections::BTreeSet<&str> = store.lines().map(|l| l.split('\t').nth(8).unwrap()).collect();
    assert_eq!(transforms.into_iter().collect::<Vec<_>>(), vec!["authentic-paste-hard-area0.12", "generated-paste-hard-area0.12", "parent", "resave", "self-paste-hard-area0.12"]);
    // Only the composites with generated pixels are labelled 1, and each row names its parent.
    assert!(store.lines().all(|l| (l.split('\t').nth(1) == Some("1")) == l.ends_with("\tgenerated-paste-hard-area0.12") && l.split('\t').nth(4).is_some_and(|p| p.contains("/real"))), "{store}");

    let weights = common::photo_weights();
    let o = tintype(&["score-windows", "--weights", weights.to_str().unwrap(), "--manifest", &d("store/manifest.tsv"), "--out", &d("windows.tsv"), "--device", "cpu", "--stride", "224", "--threads", "2"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let scored = std::fs::read_to_string(dir.join("windows.tsv")).unwrap();
    assert_eq!(scored.lines().count(), 15);
    // 480 x 384 at a stride of 224: three by two windows, the last flush with the border.
    assert!(scored.lines().all(|l| l.contains("\tsize=480x384\t") && l.split("\tgrid=").nth(1).is_some_and(|g| g.split(';').count() == 6 && g.contains(";256,160,224,224,"))), "{scored}");

    // The global score in that file is the one `detect` gives for the same file.
    let parent = dir.join("store").join(store.lines().find(|l| l.ends_with("\tparent")).unwrap().split('\t').next().unwrap());
    let detected = score_of(&json_lines(&detect(&[&parent], &["--json"]))[0]).unwrap();
    let row = scored.lines().find(|l| l.starts_with(&format!("{}\t", parent.display()))).unwrap();
    let deployed: f64 = row.split("\tdeployed=").nth(1).unwrap().split('/').next().unwrap().parse().unwrap();
    assert!((deployed - detected).abs() < 1e-4, "score-windows says {deployed}, detect says {detected}");

    let o = tintype(&["partial-report", "--scores", &d("windows.tsv"), "--manifest", &d("store/manifest.tsv"), "--fpr", "0.34", "--out", &d("per-item.tsv")]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let report = stdout(&o);
    assert!(report.contains("## Images flagged at 34% authentic FPR") && report.contains("Cutoffs may flag 1 of 3 authentic images."), "{report}");
    assert!(report.contains("| slice | images | global (deployed) | views mean | views max | views top-2 | grid mean | grid max | grid top-3 |"), "{report}");
    // Every rule flags exactly its allowance of the authentic parents: the columns are at the same rate.
    let parents = report.lines().find(|l| l.starts_with("| parent | 3 |")).unwrap();
    assert_eq!(parents.matches("1/3 (33.3%)").count(), 7, "{report}");
    assert!(report.contains("| generated-paste-hard-area0.12 | 3 |") && report.contains("## Localisation from the grid windows"), "{report}");
    // The per-item output keeps the global score and adds the local ones.
    let items = std::fs::read_to_string(dir.join("per-item.tsv")).unwrap();
    assert!(items.starts_with("# path\ttransform\tkind\tparent_id\tgenerated\tglobal_prob\t") && items.lines().count() == 16, "{items}");
    // Cutoffs frozen on one set are applied to another as they are: here the same set, so the same counts.
    let o = tintype(&["partial-report", "--scores", &d("windows.tsv"), "--manifest", &d("store/manifest.tsv"), "--fpr", "0.34", "--freeze-cutoffs", &d("cutoffs.tsv")]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let frozen = std::fs::read_to_string(dir.join("cutoffs.tsv")).unwrap();
    assert!(frozen.contains("# authentic\t3\n") && frozen.contains("34\tgrid max\t") && frozen.contains("34\tglobal (deployed)\t"), "{frozen}");
    let o = tintype(&["partial-report", "--scores", &d("windows.tsv"), "--manifest", &d("store/manifest.tsv"), "--fpr", "0.34", "--cutoffs", &d("cutoffs.tsv")]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let applied = stdout(&o);
    assert!(applied.contains("at cutoffs frozen on 3 authentic images of another set"), "{applied}");
    assert_eq!(applied.lines().find(|l| l.starts_with("| parent | 3 |")), Some(parents), "{applied}");
    let o = tintype(&["partial-report", "--scores", &d("windows.tsv"), "--manifest", &d("store/manifest.tsv"), "--cutoffs", &d("cutoffs.tsv"), "--disjoint-cutoff"]);
    assert_eq!(o.status.code(), Some(2));
    // A second store in the same place is refused, not merged.
    let o = tintype(&["paste-edits", "--real", &d("manifest.tsv"), "--fake", &d("manifest.tsv"), "--out", &d("store"), "--min-side", "224"]);
    assert!(!o.status.success());
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
