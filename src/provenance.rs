use anyhow::{ensure, Context as _, Result};
use c2pa::{
    settings::{Settings, TrustAnchor, TrustListKind},
    validation_results::ValidationState,
    Context, Error, Reader,
};
use clap::Args;
use serde::Serialize;
use std::{
    fs::File,
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
};

const MAX_MEDIA_BYTES: u64 = 512 * 1024 * 1024;
const MAX_TRUST_BYTES: u64 = 1024 * 1024;

#[derive(Args)]
pub struct ProvenanceArgs {
    /// Original media file containing the embedded credential.
    path: PathBuf,
    /// PEM trust anchors for C2PA claim signing.
    #[arg(long)]
    trust_anchors: Option<PathBuf>,
    /// PEM trust anchors for C2PA timestamps.
    #[arg(long)]
    tsa_anchors: Option<PathBuf>,
    /// Write one JSON object.
    #[arg(long)]
    json: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Status {
    Trusted,
    ValidUntrusted,
    Invalid,
    Absent,
    RemoteOnly,
    Unsupported,
}

impl Status {
    fn message(self) -> &'static str {
        match self {
            Self::Trusted => "Trusted under the supplied C2PA trust policy",
            Self::ValidUntrusted => "Valid credential with no trusted signer",
            Self::Invalid => "Credential validation failed",
            Self::Absent => "No embedded C2PA credential found",
            Self::RemoteOnly => "Credential referenced remotely and unchecked",
            Self::Unsupported => "Credential check unsupported for this file format",
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct CreationClaim {
    action: &'static str,
    digital_source_type: &'static str,
}

#[derive(Serialize)]
struct Report {
    status: Status,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    creation_claims: Vec<CreationClaim>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    validation_codes: Vec<String>,
}

impl Report {
    fn new(status: Status) -> Self {
        Self {
            status,
            creation_claims: Vec::new(),
            validation_codes: Vec::new(),
        }
    }

    fn from_reader(reader: Reader) -> Self {
        // Avoid interpreting the SDK's fallback state when validation results
        // are absent. A parsed manifest alone carries no validation assurance.
        let Some(results) = reader.validation_results() else {
            return Self::new(Status::Invalid);
        };
        let mut report = Self::new(match results.validation_state() {
            ValidationState::Trusted => Status::Trusted,
            ValidationState::Valid => Status::ValidUntrusted,
            ValidationState::Invalid => Status::Invalid,
        });
        report.validation_codes = reader
            .validation_status()
            .unwrap_or_default()
            .iter()
            .map(|status| status.code().to_owned())
            .collect();
        report.validation_codes.sort_unstable();
        report.validation_codes.dedup();
        if report.status != Status::Invalid {
            if let Some(manifest) = reader.active_manifest() {
                for assertion in manifest.assertions() {
                    if !assertion.created()
                        || !matches!(assertion.label(), "c2pa.actions" | "c2pa.actions.v2")
                    {
                        continue;
                    }
                    let Ok(value) = assertion.value() else {
                        continue;
                    };
                    if let Some(actions) =
                        value.get("actions").and_then(|actions| actions.as_array())
                    {
                        for action in actions {
                            if let Some(claim) = creation_claim(action) {
                                if !report.creation_claims.contains(&claim) {
                                    report.creation_claims.push(claim);
                                }
                            }
                        }
                    }
                }
            }
        }
        report
    }
}

fn creation_claim(action: &serde_json::Value) -> Option<CreationClaim> {
    if action.get("action")?.as_str()? != "c2pa.created" {
        return None;
    }
    const IPTC: &str = "http://cv.iptc.org/newscodes/digitalsourcetype/";
    let kind = action
        .get("digitalSourceType")?
        .as_str()?
        .strip_prefix(IPTC)?;
    let digital_source_type = match kind {
        "trainedAlgorithmicMedia" => "trainedAlgorithmicMedia",
        "compositeWithTrainedAlgorithmicMedia" => "compositeWithTrainedAlgorithmicMedia",
        "digitalCapture" => "digitalCapture",
        _ => return None,
    };
    Some(CreationClaim {
        action: "c2pa.created",
        digital_source_type,
    })
}

fn trust_anchor(path: PathBuf, kind: TrustListKind) -> Result<TrustAnchor> {
    let file = File::open(&path).with_context(|| format!("opening {}", path.display()))?;
    let mut pem = String::new();
    file.take(MAX_TRUST_BYTES + 1).read_to_string(&mut pem)?;
    ensure!(
        pem.len() as u64 <= MAX_TRUST_BYTES && pem.contains("-----BEGIN CERTIFICATE-----"),
        "{} must contain a PEM certificate bundle of at most 1 MiB",
        path.display()
    );
    Ok(TrustAnchor {
        trust_anchors: pem,
        trust_kind: kind,
        ..TrustAnchor::default()
    })
}

fn media_format(path: &Path, file: &mut File) -> io::Result<Option<&'static str>> {
    // Filter before calling the SDK. Its default registry also reads manifest
    // stores, archives, documents and audio, which this command does not review.
    let mut header = Vec::with_capacity(16);
    file.take(16).read_to_end(&mut header)?;
    let format = match image::guess_format(&header) {
        Ok(image::ImageFormat::Jpeg) => Some("image/jpeg"),
        Ok(image::ImageFormat::Png) => Some("image/png"),
        Ok(image::ImageFormat::WebP) => Some("image/webp"),
        format
            if matches!(header.get(..4), Some(b"II+\0" | b"MM\0+"))
                || matches!(format, Ok(image::ImageFormat::Tiff)) =>
        {
            match path.extension().and_then(|ext| ext.to_str()) {
                Some(ext) if ext.eq_ignore_ascii_case("dng") => Some("dng"),
                Some(ext) if ext.eq_ignore_ascii_case("arw") => Some("arw"),
                Some(ext) if ext.eq_ignore_ascii_case("nef") => Some("nef"),
                _ => Some("image/tiff"),
            }
        }
        Ok(image::ImageFormat::Gif) => Some("image/gif"),
        Ok(image::ImageFormat::Avif) => Some("image/avif"),
        _ if header.starts_with(b"\0\0\0\x0cJXL \r\n\x87\n") => Some("image/jxl"),
        _ if header.starts_with(b"RIFF") && header.get(8..12) == Some(b"AVI ") => Some("video/avi"),
        _ if header.get(4..8) == Some(b"ftyp") => {
            // The SDK uses the same BMFF parser for these media types.
            // Reject the known audio-only brands before consulting the suffix.
            match header.get(8..12) {
                Some(b"M4A " | b"M4B " | b"F4A " | b"F4B ") => None,
                Some(b"avif" | b"avis") => Some("image/avif"),
                Some(b"heic" | b"heix" | b"hevc" | b"hevx") => Some("image/heic"),
                Some(b"mif1" | b"msf1") => Some("image/heif"),
                Some(b"qt  ") => Some("video/quicktime"),
                _ => match path.extension().and_then(|ext| ext.to_str()) {
                    Some(ext) if ext.eq_ignore_ascii_case("mp4") => Some("video/mp4"),
                    Some(ext) if ext.eq_ignore_ascii_case("mov") => Some("video/quicktime"),
                    Some(ext) if ext.eq_ignore_ascii_case("m4v") => Some("video/x-m4v"),
                    _ => None,
                },
            }
        }
        _ => None,
    };
    Ok(format)
}

fn inspect(path: &Path, anchors: Vec<TrustAnchor>) -> Result<Report> {
    ensure!(path.metadata()?.is_file(), "provide a regular media file");
    let mut file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let metadata = file.metadata()?;
    ensure!(metadata.is_file(), "provide a regular media file");
    ensure!(
        metadata.len() <= MAX_MEDIA_BYTES,
        "credential check supports files up to 512 MiB"
    );
    let Some(format) = media_format(path, &mut file)? else {
        return Ok(Report::new(Status::Unsupported));
    };
    file.rewind()?;

    let mut settings = Settings::default();
    settings.verify.verify_after_reading = true;
    settings.verify.ocsp_fetch = false;
    settings.verify.ocsp_fetch_should_override = false;
    settings.verify.remote_manifest_fetch = false;
    settings.core.allowed_network_hosts = Some(Vec::new());
    settings.core.allow_redirects = false;
    settings.core.decode_identity_assertions = false;
    settings.core.max_decompressed_manifest_size_in_mb = 8;
    settings.set_value("trust.anchors", serde_json::to_value(anchors)?)?;
    let ctx = Context::new().with_settings(settings)?;

    let report = match Reader::from_context(ctx).with_stream(format, file) {
        Ok(reader) if reader.is_embedded() => Report::from_reader(reader),
        Ok(_) | Err(Error::RemoteManifestUrl(_)) => Report::new(Status::RemoteOnly),
        Err(Error::JumbfNotFound) => Report::new(Status::Absent),
        Err(Error::UnsupportedType) => Report::new(Status::Unsupported),
        Err(Error::IoError(error)) => return Err(error).context("reading media"),
        Err(_) => Report::new(Status::Invalid),
    };
    Ok(report)
}

pub fn run(args: ProvenanceArgs) -> Result<()> {
    let mut anchors = Vec::new();
    if let Some(path) = args.trust_anchors {
        anchors.push(trust_anchor(path, TrustListKind::Manifest)?);
    }
    if let Some(path) = args.tsa_anchors {
        anchors.push(trust_anchor(path, TrustListKind::TSA)?);
    }
    let report = inspect(&args.path, anchors)?;
    let mut stdout = io::stdout().lock();
    if args.json {
        serde_json::to_writer(&mut stdout, &report)?;
        writeln!(stdout)?;
    } else {
        writeln!(stdout, "C2PA · {}", report.status.message())?;
        for claim in report.creation_claims {
            let wording = match claim.digital_source_type {
                "trainedAlgorithmicMedia" => "AI-generated creation",
                "compositeWithTrainedAlgorithmicMedia" => "AI-assisted composite",
                "digitalCapture" => "Digital capture",
                _ => unreachable!("creation claims have a fixed source vocabulary"),
            };
            writeln!(stdout, "Credential declares: {wording}")?;
        }
    }
    Ok(())
}
