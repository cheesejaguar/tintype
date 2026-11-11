//! End-to-end fine-tuning on the Apple GPU: forward, autodiff and the AdamW
//! update all live in one MPSGraph, so a training step is a single Metal run.
#![cfg(target_os = "macos")]

use crate::mps::{Graph, Op, Precision, T};
use crate::mps_vit::{build_head, build_vit, ParamSource};
use crate::vit::VitConfig;
use crate::weights::{transpose2, Weights};
use anyhow::Result;

struct Entry {
    name: String,
    var: T,
    shape: Vec<isize>,
    linear: bool,
}

#[derive(Default)]
struct Trainable(Vec<Entry>);

impl ParamSource for Trainable {
    fn param(&mut self, g: &Graph, name: &str, data: &[f32], shape: &[isize], linear: bool) -> T {
        let var = g.variable(data, shape);
        self.0.push(Entry { name: name.to_string(), var: var.clone(), shape: shape.to_vec(), linear });
        var
    }
}

#[derive(Clone, Debug)]
pub struct OptConfig {
    pub beta1: f64,
    pub beta2: f64,
    pub eps: f64,
    pub weight_decay: f64,
    /// Layer-wise learning-rate decay (1.0 disables it).
    pub layer_decay: f64,
    /// Global gradient-norm clip (0 disables it).
    pub clip: f64,
    /// Binary label smoothing.
    pub smoothing: f64,
}

impl Default for OptConfig {
    fn default() -> Self {
        Self { beta1: 0.9, beta2: 0.999, eps: 1e-8, weight_decay: 0.05, layer_decay: 0.8, clip: 1.0, smoothing: 0.0 }
    }
}

pub struct Trainer {
    g: Graph,
    params: Vec<Entry>,
    x: T,
    y: T,
    lr_t: T,
    lr: T,
    logits: T,
    loss: T,
    updates: Vec<Op>,
    pub batch: usize,
    pub cfg: VitConfig,
    opt: OptConfig,
    pub step: u64,
}

/// Depth index used for layer-wise lr decay: embeddings 0, block i -> i+1,
/// final norm and head -> depth+1.
fn layer_id(name: &str, depth: usize) -> usize {
    if let Some(rest) = name.strip_prefix("blocks.") {
        rest.split('.').next().and_then(|s| s.parse::<usize>().ok()).map(|i| i + 1).unwrap_or(depth + 1)
    } else if name.starts_with("patch_embed") || name == "cls_token" || name == "pos_embed" {
        0
    } else {
        depth + 1
    }
}

fn no_decay(name: &str) -> bool {
    name.ends_with(".bias") || name.contains("norm") || name == "cls_token" || name == "pos_embed"
}

impl Trainer {
    pub fn new(w: &Weights, cfg: VitConfig, batch: usize, opt: OptConfig) -> Result<Self> {
        let g = Graph::new(Precision::F32)?;
        let (b, s) = (batch as isize, cfg.img as isize);
        let x = g.placeholder(&[b, 3, s, s]);
        let y = g.placeholder(&[b, 1]);
        let lr_t = g.placeholder(&[1]);
        let lr = g.placeholder(&[1]);

        let mut tr = Trainable::default();
        let out = build_vit(&g, w, &mut tr, &cfg, &x, batch)?;
        let logits = build_head(&g, w, &mut tr, &out.cls)?;

        // Binary cross-entropy with logits, numerically stable form.
        let target = if opt.smoothing > 0.0 {
            g.add(&g.scale(&y, 1.0 - opt.smoothing), &g.scalar(opt.smoothing * 0.5))
        } else {
            y.clone()
        };
        let zero = g.scalar(0.0);
        let per = g.add(
            &g.sub(&g.max(&logits, &zero), &g.mul(&logits, &target)),
            &g.ln(&g.add(&g.scalar(1.0), &g.exp(&g.scale(&g.abs(&logits), -1.0)))),
        );
        let loss = g.mean(&per, &[0, 1]);

        let vars: Vec<T> = tr.0.iter().map(|e| e.var.clone()).collect();
        let mut grads = g.gradients(&loss, &vars);

        if opt.clip > 0.0 {
            let mut total = g.scalar(0.0);
            for gr in &grads {
                total = g.add(&total, &g.sum(&g.square(&g.reshape(gr, &[-1])), &[0]));
            }
            let norm = g.add(&g.sqrt(&total), &g.scalar(1e-6));
            let one = g.scalar(1.0);
            let ratio = g.div(&g.scalar(opt.clip), &norm);
            // min(1, clip / norm) == 1 - max(0, 1 - ratio)
            let k = g.sub(&one, &g.max(&g.scalar(0.0), &g.sub(&one, &ratio)));
            grads = grads.iter().map(|gr| g.mul(gr, &k)).collect();
        }

        let mut updates = Vec::with_capacity(tr.0.len() * 3);
        for (e, gr) in tr.0.iter().zip(&grads) {
            let n: usize = e.shape.iter().product::<isize>() as usize;
            let zeros = vec![0f32; n];
            let m = g.variable(&zeros, &e.shape);
            let v = g.variable(&zeros, &e.shape);
            let m2 = g.add(&g.scale(&m, opt.beta1), &g.scale(gr, 1.0 - opt.beta1));
            let v2 = g.add(&g.scale(&v, opt.beta2), &g.scale(&g.square(gr), 1.0 - opt.beta2));
            let scale = opt.layer_decay.powi((cfg.depth + 1 - layer_id(&e.name, cfg.depth)) as i32);
            let step = g.div(&m2, &g.add(&g.sqrt(&v2), &g.scalar(opt.eps)));
            let mut p2 = g.sub(&e.var, &g.mul(&g.scale(&step, scale), &lr_t));
            if opt.weight_decay > 0.0 && !no_decay(&e.name) {
                p2 = g.sub(&p2, &g.mul(&g.scale(&e.var, scale * opt.weight_decay), &lr));
            }
            updates.push(g.assign(&m, &m2));
            updates.push(g.assign(&v, &v2));
            updates.push(g.assign(&e.var, &p2));
        }

        Ok(Self { g, params: tr.0, x, y, lr_t, lr, logits, loss, updates, batch, cfg, opt, step: 0 })
    }

    fn xshape(&self) -> [isize; 4] {
        [self.batch as isize, 3, self.cfg.img as isize, self.cfg.img as isize]
    }

    /// One optimisation step; returns the batch loss (before the update).
    pub fn train_step(&mut self, x: &[f32], y: &[f32], lr: f32) -> f32 {
        self.step += 1;
        let t = self.step as f64;
        let corr = (1.0 - self.opt.beta2.powf(t)).sqrt() / (1.0 - self.opt.beta1.powf(t));
        let (lr_t, lr_w) = ([lr * corr as f32], [lr]);
        let out = self.g.run(
            &[
                (&self.x, x, &self.xshape()),
                (&self.y, y, &[self.batch as isize, 1]),
                (&self.lr_t, &lr_t, &[1]),
                (&self.lr, &lr_w, &[1]),
            ],
            &[self.loss.clone()],
            &self.updates,
        );
        out[0][0]
    }

    /// Forward only (no update): logits for one full batch.
    pub fn logits(&self, x: &[f32]) -> Vec<f32> {
        self.g.run(&[(&self.x, x, &self.xshape())], &[self.logits.clone()], &[]).remove(0)
    }

    /// Read the current parameters back in checkpoint (timm) layout.
    pub fn export(&self) -> Weights {
        let reads: Vec<T> = self.params.iter().map(|e| self.g.read(&e.var)).collect();
        let vals = self.g.run(&[], &reads, &[]);
        let mut w = Weights::default();
        for (e, v) in self.params.iter().zip(vals) {
            if e.linear {
                let (i, o) = (e.shape[0] as usize, e.shape[1] as usize);
                let data = transpose2(&v, i, o);
                let shape = if e.name == "patch_embed.proj.weight" {
                    vec![o, 3, self.cfg.patch, self.cfg.patch]
                } else {
                    vec![o, i]
                };
                w.insert(&e.name, data, shape);
            } else {
                w.insert(&e.name, v, e.shape.iter().map(|&d| d as usize).collect());
            }
        }
        w
    }
}
