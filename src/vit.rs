//! Vision Transformer shapes (timm layout). The forward passes live in
//! `cpu_vit` (CPU) and `mps_vit` (Apple GPU).

#[derive(Clone, Copy, Debug)]
pub struct VitConfig {
    pub img: usize,
    pub patch: usize,
    pub dim: usize,
    pub depth: usize,
    pub heads: usize,
    pub mlp_ratio: usize,
    /// Register tokens inserted after the class token (DINOv2 "reg" models).
    pub reg_tokens: usize,
    /// Per-channel residual scaling (`ls1.gamma`, `ls2.gamma`).
    pub layer_scale: bool,
    /// Position embedding covers the patch tokens only and is added before
    /// the class / register tokens are prepended (timm `no_embed_class`).
    pub pos_on_patches_only: bool,
}

impl VitConfig {
    pub fn small(img: usize) -> Self {
        Self { img, patch: 16, dim: 384, depth: 12, heads: 6, mlp_ratio: 4, reg_tokens: 0, layer_scale: false, pos_on_patches_only: false }
    }

    /// timm `vit_base_patch14_reg4_dinov2` (the WaveRep backbone).
    pub fn dinov2_base_reg4(img: usize) -> Self {
        Self { img, patch: 14, dim: 768, depth: 12, heads: 12, mlp_ratio: 4, reg_tokens: 4, layer_scale: true, pos_on_patches_only: true }
    }

    pub fn by_name(arch: &str, img: usize) -> Option<Self> {
        match arch {
            "" | "vit_small_patch16" => Some(Self::small(img)),
            "vit_base_patch14_reg4_dinov2" => Some(Self::dinov2_base_reg4(img)),
            _ => None,
        }
    }

    /// Tokens before the patch tokens.
    pub fn prefix_tokens(&self) -> usize {
        1 + self.reg_tokens
    }
}
