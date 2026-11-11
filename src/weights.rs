//! Named f32 tensors loaded from / saved to safetensors.
use anyhow::{anyhow, Result};
use candle_core::{DType, Device, Tensor};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default, Clone)]
pub struct Weights(pub BTreeMap<String, (Vec<f32>, Vec<usize>)>);

impl Weights {
    /// Load every tensor, stripping `prefix` from names when present.
    pub fn load(path: impl AsRef<Path>, prefix: &str) -> Result<Self> {
        let mut out = BTreeMap::new();
        for (name, t) in candle_core::safetensors::load(path.as_ref(), &Device::Cpu)? {
            let shape = t.dims().to_vec();
            let data = t.to_dtype(DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
            out.insert(name.strip_prefix(prefix).unwrap_or(&name).to_string(), (data, shape));
        }
        Ok(Self(out))
    }

    /// Load a PyTorch checkpoint (`.ckpt` / `.pth`). The pickle is parsed
    /// structurally, never executed. Handles both a bare state dict and one
    /// nested under `state_dict`; `prefix` is stripped from names that have it.
    pub fn load_torch(path: impl AsRef<Path>, prefix: &str) -> Result<Self> {
        let tensors = match candle_core::pickle::read_all_with_key(path.as_ref(), Some("state_dict")) {
            Ok(t) if !t.is_empty() => t,
            _ => candle_core::pickle::read_all(path.as_ref())?,
        };
        let mut out = BTreeMap::new();
        for (name, t) in tensors {
            let shape = t.dims().to_vec();
            let data = t.to_dtype(DType::F32)?.flatten_all()?.to_vec1::<f32>()?;
            out.insert(name.strip_prefix(prefix).unwrap_or(&name).to_string(), (data, shape));
        }
        Ok(Self(out))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let m: std::collections::HashMap<String, Tensor> = self
            .0
            .iter()
            .map(|(k, (d, s))| Ok((k.clone(), Tensor::from_slice(d, s.as_slice(), &Device::Cpu)?)))
            .collect::<Result<_>>()?;
        candle_core::safetensors::save(&m, path.as_ref())?;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&(Vec<f32>, Vec<usize>)> {
        self.0.get(name).ok_or_else(|| anyhow!("missing weight `{name}`"))
    }

    pub fn insert(&mut self, name: &str, data: Vec<f32>, shape: Vec<usize>) {
        self.0.insert(name.to_string(), (data, shape));
    }
}

/// Transpose a row-major `[r, c]` matrix.
pub fn transpose2(d: &[f32], r: usize, c: usize) -> Vec<f32> {
    let mut o = vec![0f32; d.len()];
    for i in 0..r {
        for j in 0..c {
            o[j * r + i] = d[i * c + j];
        }
    }
    o
}
