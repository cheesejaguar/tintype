#![cfg(feature = "provenance")]

use serde_json::Value;
use std::{path::Path, process::Command};

const ROOT: &str = "tests/fixtures/provenance/root.pem";
const CAPTURE: &str = "tests/fixtures/provenance/signed-capture.png";
const GENERATED: &str = "tests/fixtures/provenance/signed-generated.png";

fn inspect(path: &Path, extra: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_tintype"))
        .args(["provenance", "--json"])
        .args(extra)
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn separates_declared_creation_from_signer_trust() {
    for (path, source_type) in [
        (CAPTURE, "digitalCapture"),
        (GENERATED, "trainedAlgorithmicMedia"),
    ] {
        let report = inspect(Path::new(path), &["--trust-anchors", ROOT]);
        assert_eq!(report["status"], "trusted");
        assert_eq!(
            report["creation_claims"][0]["digital_source_type"],
            source_type
        );
    }
    for args in [vec![], vec!["--tsa-anchors", ROOT]] {
        let report = inspect(Path::new(GENERATED), &args);
        assert_eq!(report["status"], "valid-untrusted");
    }
}

#[test]
fn reports_tampered_and_missing_credentials() {
    use std::io::Write;

    let mut changed = include_bytes!("fixtures/provenance/signed-generated.png").to_vec();
    changed[19] ^= 1;
    let mut file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    file.write_all(&changed).unwrap();
    let report = inspect(file.path(), &["--trust-anchors", ROOT]);
    assert_eq!(report["status"], "invalid");
    assert!(report.get("creation_claims").is_none());
    let report = inspect(Path::new("site/assets/demo.png"), &[]);
    assert_eq!(report["status"], "absent");
}

#[test]
fn accepts_png_bytes_without_a_png_extension_and_rejects_a_detached_store() {
    use std::io::Write;

    let image = include_bytes!("fixtures/provenance/signed-generated.png");
    let mut renamed = tempfile::NamedTempFile::new().unwrap();
    renamed.write_all(image).unwrap();
    let report = inspect(renamed.path(), &["--trust-anchors", ROOT]);
    assert_eq!(report["status"], "trusted");

    let mut cursor = 8;
    while &image[cursor + 4..cursor + 8] != b"caBX" {
        let size = u32::from_be_bytes(image[cursor..cursor + 4].try_into().unwrap());
        cursor += size as usize + 12;
    }
    let size = u32::from_be_bytes(image[cursor..cursor + 4].try_into().unwrap());
    let mut detached = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
    detached
        .write_all(&image[cursor + 8..cursor + 8 + size as usize])
        .unwrap();
    let report = inspect(detached.path(), &["--trust-anchors", ROOT]);
    assert_eq!(report["status"], "unsupported");
}
