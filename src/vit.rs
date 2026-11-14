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

    /// ViT-S/16 with `depth` blocks instead of twelve: a shallower student.
    /// `None` outside one to twelve.
    pub fn small_with_depth(img: usize, depth: usize) -> Option<Self> {
        (1..=12).contains(&depth).then(|| Self { depth, ..Self::small(img) })
    }

    /// The name a spec gives a network: empty for the ViT-S/16 Tintype
    /// ships, `vit_small_patch16_d8` for the same with eight blocks.
    pub fn by_name(arch: &str, img: usize) -> Option<Self> {
        match arch {
            "" | "vit_small_patch16" => Some(Self::small(img)),
            "vit_base_patch14_reg4_dinov2" => Some(Self::dinov2_base_reg4(img)),
            _ => arch.strip_prefix("vit_small_patch16_d").and_then(|d| d.parse().ok()).and_then(|d| Self::small_with_depth(img, d)),
        }
    }

    /// The spec name of a ViT-S/16 with this many blocks (see [`VitConfig::by_name`]).
    pub fn small_name(depth: usize) -> String {
        if depth == 12 {
            String::new()
        } else {
            format!("vit_small_patch16_d{depth}")
        }
    }

    /// Tokens before the patch tokens.
    pub fn prefix_tokens(&self) -> usize {
        1 + self.reg_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shallower_networks_are_named_by_their_depth() {
        assert_eq!(VitConfig::by_name("", 224).unwrap().depth, 12);
        assert_eq!(VitConfig::by_name("vit_small_patch16", 224).unwrap().depth, 12);
        for depth in 1..=12 {
            let cfg = VitConfig::by_name(&VitConfig::small_name(depth), 224).unwrap();
            assert_eq!((cfg.depth, cfg.dim, cfg.heads, cfg.patch, cfg.img), (depth, 384, 6, 16, 224));
        }
        assert_eq!(VitConfig::small_name(12), "");
        assert_eq!(VitConfig::small_name(8), "vit_small_patch16_d8");
        for bad in ["vit_small_patch16_d0", "vit_small_patch16_d13", "vit_small_patch16_d", "vit_small_patch16_dx", "vit_small_patch16_d8 ", "vit_tiny"] {
            assert!(VitConfig::by_name(bad, 224).is_none(), "{bad}");
        }
    }
}
