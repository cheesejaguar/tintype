//! The released model files are the ones whose SHA-256 the source and
//! docs/DATA.md record. Retraining or recalibrating changes them on purpose;
//! then update `MODEL_FILES` in src/integrity.rs and the table in the docs.
mod common;

#[test]
fn model_files_match_their_recorded_sha256() {
    let (lines, ok) = tintype::integrity::check_models(&common::repo());
    assert!(ok, "{}", lines.join("\n"));
}

#[test]
fn the_docs_list_the_same_digests() {
    let docs = std::fs::read_to_string(common::repo().join("docs/DATA.md")).unwrap();
    for (path, _, sha) in tintype::integrity::MODEL_FILES {
        assert!(docs.lines().any(|l| l.contains(path) && l.contains(sha)), "docs/DATA.md has no line with {path} and {sha}");
    }
}
