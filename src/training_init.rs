//! Build a native training checkpoint from a verified timm ViT-S/16.
use crate::detector::{thorough_views, ModelSpec};
use crate::integrity::sha256_bytes;
use crate::vit::VitConfig;
use anyhow::{ensure, Context, Result};
use candle_core::{safetensors, DType, Device, Tensor};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use tempfile::NamedTempFile;

pub const DINO_SHA256: &str = "8d4795b8e4e327c6d841e15a2e3e6d88c45346256a43f8cdae8c26d92c806f7a";

const MAX_BACKBONE_BYTES: u64 = 96 * 1024 * 1024;

pub fn initialize(backbone: &Path, expected: &str, output: &Path) -> Result<()> {
    ensure!(
        output.extension().is_some_and(|ext| ext == "safetensors"),
        "--out must end in .safetensors"
    );
    ensure!(
        expected.len() == 64 && expected.bytes().all(|c| c.is_ascii_hexdigit()),
        "--sha256 must be a 64-digit hexadecimal SHA-256"
    );
    let sidecar = ModelSpec::sidecar(output);
    for path in [output, &sidecar] {
        ensure!(
            !path.try_exists()? && !path.is_symlink(),
            "{} already exists",
            path.display()
        );
    }
    let mut data = Vec::new();
    std::fs::File::open(backbone)
        .with_context(|| format!("opening {}", backbone.display()))?
        .take(MAX_BACKBONE_BYTES + 1)
        .read_to_end(&mut data)?;
    ensure!(
        data.len() as u64 <= MAX_BACKBONE_BYTES,
        "backbone exceeds 96 MiB"
    );
    let actual = sha256_bytes(&data);
    ensure!(
        actual.eq_ignore_ascii_case(expected),
        "backbone SHA-256 mismatch: expected {expected}, got {actual}"
    );
    let cfg = VitConfig::small(224);
    let mut tensors = safetensors::load_buffer(&data, &Device::Cpu)
        .context("loading the verified backbone safetensors")?;
    drop(data);
    validate_backbone(&tensors, cfg)?;

    // Tintype represents the two classes with a single logit. A zero head
    // starts at score 0.5 and learns through binary cross-entropy.
    tensors.insert(
        "head.weight".into(),
        Tensor::zeros((1, cfg.dim), DType::F32, &Device::Cpu)?,
    );
    tensors.insert(
        "head.bias".into(),
        Tensor::zeros(1, DType::F32, &Device::Cpu)?,
    );
    let spec = ModelSpec {
        img: cfg.img,
        views: thorough_views(cfg.img as u32, 256),
        logit_scale: 1.0,
        logit_bias: 0.0,
        threshold: 0.5,
        prefix: String::new(),
        video: None,
        fast: None,
        sensitive: None,
        strict: None,
        early_exit: None,
        second_opinion: None,
        review: None,
        arch: "vit_small_patch16".into(),
    };

    let directory = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(directory)?;
    let weights_file = NamedTempFile::new_in(directory)?;
    safetensors::save(&tensors, weights_file.path())?;
    let mut spec_file = NamedTempFile::new_in(directory)?;
    serde_json::to_writer_pretty(&mut spec_file, &spec)?;
    writeln!(spec_file)?;
    // Commit weights last. A second invocation cannot overwrite either file.
    spec_file
        .persist_noclobber(&sidecar)
        .with_context(|| format!("writing {}", sidecar.display()))?;
    if let Err(error) = weights_file.persist_noclobber(output) {
        std::fs::remove_file(&sidecar)
            .with_context(|| format!("rolling back {}", sidecar.display()))?;
        return Err(error).with_context(|| format!("writing {}", output.display()));
    }

    println!("Verified backbone SHA-256 {actual}");
    println!(
        "Wrote untrained checkpoint {} and {}",
        output.display(),
        sidecar.display()
    );
    Ok(())
}

fn validate_backbone(tensors: &HashMap<String, Tensor>, cfg: VitConfig) -> Result<()> {
    let dim = cfg.dim;
    let inner = dim * cfg.mlp_ratio;
    let patches = (cfg.img / cfg.patch).pow(2);
    let require = |name: &str, shape: &[usize]| -> Result<()> {
        let tensor = tensors
            .get(name)
            .with_context(|| format!("missing backbone tensor {name}"))?;
        ensure!(
            tensor.dims() == shape && tensor.dtype() == DType::F32,
            "backbone tensor {name} must be F32 with shape {shape:?}"
        );
        Ok(())
    };
    require("cls_token", &[1, 1, dim])?;
    require("pos_embed", &[1, patches + 1, dim])?;
    require("patch_embed.proj.weight", &[dim, 3, cfg.patch, cfg.patch])?;
    require("patch_embed.proj.bias", &[dim])?;
    require("norm.weight", &[dim])?;
    require("norm.bias", &[dim])?;

    for block in 0..cfg.depth {
        for (name, shape) in [
            ("attn.qkv.weight", &[3 * dim, dim][..]),
            ("attn.qkv.bias", &[3 * dim]),
            ("attn.proj.weight", &[dim, dim]),
            ("attn.proj.bias", &[dim]),
            ("mlp.fc1.weight", &[inner, dim]),
            ("mlp.fc1.bias", &[inner]),
            ("mlp.fc2.weight", &[dim, inner]),
            ("mlp.fc2.bias", &[dim]),
            ("norm1.weight", &[dim]),
            ("norm1.bias", &[dim]),
            ("norm2.weight", &[dim]),
            ("norm2.bias", &[dim]),
        ] {
            require(&format!("blocks.{block}.{name}"), shape)?;
        }
    }
    ensure!(
        tensors
            .keys()
            .filter(|name| name.as_str() != "head.weight" && name.as_str() != "head.bias")
            .count()
            == 6 + 12 * cfg.depth,
        "backbone has extra tensors outside the native ViT-S/16 layout"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unverified_or_wrong_layout_input_does_not_write_outputs() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("input.safetensors");
        let output = dir.path().join("output.safetensors");
        let mut tensors = HashMap::new();
        tensors.insert(
            "cls_token",
            Tensor::zeros((1, 1, 384), DType::F32, &Device::Cpu).unwrap(),
        );
        safetensors::save(&tensors, &source).unwrap();

        let error = initialize(&source, DINO_SHA256, &output).unwrap_err();
        assert!(error.to_string().contains("SHA-256 mismatch"));
        let digest = crate::integrity::sha256_file(&source).unwrap();
        let error = initialize(&source, &digest, &output).unwrap_err();
        assert!(error
            .to_string()
            .contains("missing backbone tensor pos_embed"));
        assert!(!output.exists());
        assert!(!ModelSpec::sidecar(&output).exists());
    }
}
