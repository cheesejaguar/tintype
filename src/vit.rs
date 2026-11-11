//! Plain Vision Transformer (timm layout) forward pass on candle.
use candle_core::{IndexOp, Result, Tensor, D};
use candle_nn::{conv2d, layer_norm, linear, Conv2d, Conv2dConfig, LayerNorm, Linear, Module, VarBuilder};

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

/// Apply a linear layer to `[B, N, D]` as a single 2-D matmul (one GPU kernel
/// instead of B batched ones).
fn lin2d(l: &Linear, x: &Tensor) -> Result<Tensor> {
    let (b, n, d) = x.dims3()?;
    let y = x.reshape((b * n, d))?.matmul(&l.weight().t()?)?;
    let y = match l.bias() {
        Some(bias) => y.broadcast_add(bias)?,
        None => y,
    };
    y.reshape((b, n, ()))
}

struct Block {
    norm1: LayerNorm,
    qkv: Linear,
    proj: Linear,
    norm2: LayerNorm,
    fc1: Linear,
    fc2: Linear,
    heads: usize,
}

impl Block {
    fn new(vb: VarBuilder, c: &VitConfig) -> Result<Self> {
        Ok(Self {
            norm1: layer_norm(c.dim, 1e-6, vb.pp("norm1"))?,
            qkv: linear(c.dim, c.dim * 3, vb.pp("attn.qkv"))?,
            proj: linear(c.dim, c.dim, vb.pp("attn.proj"))?,
            norm2: layer_norm(c.dim, 1e-6, vb.pp("norm2"))?,
            fc1: linear(c.dim, c.dim * c.mlp_ratio, vb.pp("mlp.fc1"))?,
            fc2: linear(c.dim * c.mlp_ratio, c.dim, vb.pp("mlp.fc2"))?,
            heads: c.heads,
        })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let (b, n, d) = x.dims3()?;
        let hd = d / self.heads;
        let qkv = lin2d(&self.qkv, &self.norm1.forward(x)?)?
            .reshape((b, n, 3, self.heads, hd))?
            .permute((2, 0, 3, 1, 4))?;
        let q = qkv.i(0)?.contiguous()?;
        let k = qkv.i(1)?.contiguous()?;
        let v = qkv.i(2)?.contiguous()?;
        let attn = (q.matmul(&k.t()?)? * (hd as f64).powf(-0.5))?;
        let attn = candle_nn::ops::softmax(&attn, D::Minus1)?;
        let y = attn.matmul(&v)?.transpose(1, 2)?.reshape((b, n, d))?;
        let x = (x + lin2d(&self.proj, &y)?)?;
        let y = lin2d(&self.fc2, &lin2d(&self.fc1, &self.norm2.forward(&x)?)?.gelu_erf()?)?;
        x + y
    }
}

pub struct Vit {
    patch_embed: Conv2d,
    cls: Tensor,
    pos: Tensor,
    blocks: Vec<Block>,
    norm: LayerNorm,
    head: Option<Linear>,
    pub cfg: VitConfig,
}

impl Vit {
    pub fn new(vb: VarBuilder, cfg: VitConfig, head_out: Option<usize>) -> Result<Self> {
        let n = (cfg.img / cfg.patch).pow(2) + 1;
        let pc = Conv2dConfig { stride: cfg.patch, ..Default::default() };
        let blocks = (0..cfg.depth)
            .map(|i| Block::new(vb.pp(format!("blocks.{i}")), &cfg))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            patch_embed: conv2d(3, cfg.dim, cfg.patch, pc, vb.pp("patch_embed.proj"))?,
            cls: vb.get((1, 1, cfg.dim), "cls_token")?,
            pos: vb.get((1, n, cfg.dim), "pos_embed")?,
            blocks,
            norm: layer_norm(cfg.dim, 1e-6, vb.pp("norm"))?,
            head: match head_out {
                Some(o) => Some(linear(cfg.dim, o, vb.pp("head"))?),
                None => None,
            },
            cfg,
        })
    }

    /// Returns final normalized tokens `[B, N+1, D]`.
    pub fn tokens(&self, x: &Tensor) -> Result<Tensor> {
        let b = x.dim(0)?;
        let x = self.patch_embed.forward(x)?.flatten_from(2)?.transpose(1, 2)?;
        let cls = self.cls.broadcast_as((b, 1, self.cfg.dim))?;
        let mut x = Tensor::cat(&[&cls, &x], 1)?.broadcast_add(&self.pos)?;
        for blk in &self.blocks {
            x = blk.forward(&x)?;
        }
        self.norm.forward(&x)
    }

    /// CLS-token head output `[B, out]`.
    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let cls = self.tokens(x)?.i((.., 0))?.contiguous()?;
        match &self.head {
            Some(h) => h.forward(&cls),
            None => Ok(cls),
        }
    }
}
