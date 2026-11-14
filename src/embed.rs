//! Token embeddings of the ViT, for experiments that need more than its
//! logit (the temporal-head harness in [`crate::temporal`]).
//!
//! This is a separate engine from [`crate::detector::Detector`] on purpose:
//! scoring only ever needs the class token of the last block and both
//! scoring engines skip the rest of that block, so reading patch tokens out
//! of them would slow every scan. An [`Embedder`] runs every block in full
//! and hands back the tokens of the blocks asked for, next to the logits.
use crate::cpu_vit::{CpuVit, Tapped};
use crate::detector::{Device, ModelSpec};
#[cfg(target_os = "macos")]
use crate::mps::{Graph, Precision, T};
use crate::vit::VitConfig;
use crate::weights::Weights;
use anyhow::Result;
use std::path::Path;

enum Engine {
    /// `outs` is the logits followed by one tensor per block asked for.
    #[cfg(target_os = "macos")]
    Mps { g: Graph, inp: T, outs: Vec<T> },
    Cpu { vit: CpuVit },
}

pub struct Embedder {
    engine: Engine,
    cfg: VitConfig,
    /// Views per run on the GPU engine (its graph has a fixed batch); the
    /// CPU engine takes what it is given.
    pub batch: usize,
    blocks: Vec<usize>,
}

impl Embedder {
    /// `blocks` are the block counts after which tokens are read out
    /// (`depth()` is the last block, the one the head reads).
    pub fn new(w: &Weights, spec: &ModelSpec, batch: usize, device: Device, blocks: &[usize]) -> Result<Self> {
        let cfg = VitConfig::by_name(&spec.arch, spec.img).ok_or_else(|| anyhow::anyhow!("token embeddings need a ViT, not `{}`", spec.arch))?;
        anyhow::ensure!(blocks.iter().all(|&k| k <= cfg.depth), "block {:?} asked for, but the network has {} blocks", blocks.iter().max(), cfg.depth);
        let batch = batch.max(1);
        let engine = match device {
            #[cfg(target_os = "macos")]
            Device::Gpu | Device::GpuF32 => {
                let g = Graph::new(if device == Device::Gpu { Precision::F16 } else { Precision::F32 })?;
                let s = spec.img as isize;
                let inp = g.placeholder(&[batch as isize, 3, s, s]);
                let (v, taps) = crate::mps_vit::build_vit_tokens(&g, w, &cfg, &g.cast_in(&inp), batch, blocks)?;
                let mut outs = vec![g.cast_out(&crate::mps_vit::build_head(&g, w, &mut crate::mps_vit::Frozen, &v.cls)?)];
                outs.extend(taps.iter().map(|t| g.cast_out(t)));
                Engine::Mps { g, inp, outs }
            }
            #[cfg(not(target_os = "macos"))]
            Device::Gpu | Device::GpuF32 => anyhow::bail!("the GPU engine is only available on macOS; use the CPU engine"),
            Device::Cpu => Engine::Cpu { vit: CpuVit::new(w, cfg)? },
        };
        Ok(Self { engine, cfg, batch, blocks: blocks.to_vec() })
    }

    pub fn load(weights: &Path, spec: &ModelSpec, batch: usize, device: Device, blocks: &[usize]) -> Result<Self> {
        Self::new(&Weights::load(weights, &spec.prefix)?, spec, batch, device, blocks)
    }

    /// Blocks in the network: the largest block that can be asked for.
    pub fn depth(&self) -> usize {
        self.cfg.depth
    }

    pub fn dim(&self) -> usize {
        self.cfg.dim
    }

    /// Patch tokens along one side of a view.
    pub fn side(&self) -> usize {
        self.cfg.img / self.cfg.patch
    }

    /// Tokens per view in [`Tapped::tokens`]: the class token and the patches.
    pub fn tokens_per_view(&self) -> usize {
        self.side() * self.side() + 1
    }

    pub fn blocks(&self) -> &[usize] {
        &self.blocks
    }

    /// Logits and tokens for any number of views, given as interleaved 8-bit
    /// RGB one after another (what [`crate::detector::Preprocessor::apply`]
    /// writes).
    pub fn embed(&self, x: &[u8]) -> Result<Tapped> {
        let vl = 3 * self.cfg.img * self.cfg.img;
        anyhow::ensure!(x.len() % vl == 0, "views are {vl} bytes each");
        match &self.engine {
            Engine::Cpu { vit } => {
                let mut out = Tapped { logits: Vec::new(), tokens: vec![Vec::new(); self.blocks.len()] };
                for chunk in x.chunks(self.batch * vl) {
                    let one = vit.forward_tokens(chunk, &self.blocks)?;
                    out.logits.extend(one.logits);
                    out.tokens.iter_mut().zip(one.tokens).for_each(|(all, t)| all.extend(t));
                }
                Ok(out)
            }
            #[cfg(target_os = "macos")]
            Engine::Mps { g, inp, outs } => {
                let (n, s) = (self.cfg.img * self.cfg.img, self.cfg.img as isize);
                let per_view = self.tokens_per_view() * self.cfg.dim;
                let (k, o) = crate::image_io::norm_consts();
                let mut out = Tapped { logits: Vec::new(), tokens: vec![Vec::new(); self.blocks.len()] };
                // The graph has a fixed batch: the last one is padded with zeros.
                let mut fed = vec![0f32; self.batch * 3 * n];
                for chunk in x.chunks(self.batch * vl) {
                    let views = chunk.len() / vl;
                    fed.fill(0.0);
                    for (v, plane) in chunk.chunks_exact(vl).zip(fed.chunks_exact_mut(3 * n)) {
                        for (i, px) in v.chunks_exact(3).enumerate() {
                            for c in 0..3 {
                                plane[c * n + i] = px[c] as f32 * k[c] - o[c];
                            }
                        }
                    }
                    let mut res = g.run(&[(inp, &fed, &[self.batch as isize, 3, s, s])], outs, &[]).into_iter();
                    let logits = res.next().unwrap_or_default();
                    out.logits.extend_from_slice(&logits[..views * logits.len() / self.batch]);
                    for (all, t) in out.tokens.iter_mut().zip(res) {
                        all.extend_from_slice(&t[..views * per_view]);
                    }
                }
                Ok(out)
            }
        }
    }
}

/// Average patch tokens over square cells: one view's `[side * side, dim]`
/// patch tokens (row by row) become `[grid * grid, dim]`, each the mean of
/// the `(side / grid)^2` patches it covers. `grid` must divide `side`.
pub fn pool_grid(patches: &[f32], side: usize, dim: usize, grid: usize) -> Vec<f32> {
    assert!(grid > 0 && side % grid == 0 && patches.len() == side * side * dim, "a {side}x{side} token map does not pool to {grid}x{grid}");
    let cell = side / grid;
    let mut out = vec![0f32; grid * grid * dim];
    for py in 0..side {
        for px in 0..side {
            let o = &mut out[((py / cell) * grid + px / cell) * dim..][..dim];
            for (a, v) in o.iter_mut().zip(&patches[(py * side + px) * dim..][..dim]) {
                *a += v;
            }
        }
    }
    let inv = 1.0 / (cell * cell) as f32;
    out.iter_mut().for_each(|v| *v *= inv);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pooling_averages_the_patches_of_each_cell() {
        // A 4x4 map of 2-dimensional tokens: (row, column).
        let patches: Vec<f32> = (0..16).flat_map(|i| [(i / 4) as f32, (i % 4) as f32]).collect();
        // Unpooled: the same tokens.
        assert_eq!(pool_grid(&patches, 4, 2, 4), patches);
        // 2x2: the mean row and column of each quadrant.
        assert_eq!(pool_grid(&patches, 4, 2, 2), vec![0.5, 0.5, 0.5, 2.5, 2.5, 0.5, 2.5, 2.5]);
        // One cell: the mean of everything.
        assert_eq!(pool_grid(&patches, 4, 2, 1), vec![1.5, 1.5]);
    }
}
