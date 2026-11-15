//! End-to-end fine-tuning on the Apple GPU: forward, autodiff and the AdamW
//! update all live in one MPSGraph, so a training step is a single Metal run.
#![cfg(target_os = "macos")]

use crate::mps::{Graph, Op, Precision, T};
use crate::mps_vit::{build_head, build_vit, build_vit_cls, Frozen, ParamSource};
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

/// A frozen network whose softened probability is part of what the
/// trained one is pulled towards (see [`crate::distill::soft_target`]).
pub struct Teacher<'a> {
    pub weights: &'a Weights,
    pub cfg: VitConfig,
    /// Share of the target that is the teacher's, from 0 to 1.
    pub weight: f64,
    /// The teacher's logit is divided by this before the sigmoid.
    pub temperature: f64,
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
        Self::with_teacher(w, cfg, batch, opt, None)
    }

    /// As [`Trainer::new`], with a teacher that sees the same batch in the
    /// same graph and is never updated.
    pub fn with_teacher(w: &Weights, cfg: VitConfig, batch: usize, opt: OptConfig, teacher: Option<Teacher>) -> Result<Self> {
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
        // With a teacher: (1 - weight) * target + weight * sigmoid(teacher logit / temperature).
        let target = match &teacher {
            Some(t) => {
                anyhow::ensure!((0.0..=1.0).contains(&t.weight) && t.temperature > 0.0, "the teacher's weight is between 0 and 1 and its temperature above 0");
                let trunk = build_vit_cls(&g, t.weights, &t.cfg, &x, batch)?;
                let z = build_head(&g, t.weights, &mut Frozen, &trunk.cls)?;
                let soft = g.div(&g.scalar(1.0), &g.add(&g.scalar(1.0), &g.exp(&g.scale(&z, -1.0 / t.temperature))));
                g.add(&g.scale(&target, 1.0 - t.weight), &g.scale(&soft, t.weight))
            }
            None => target,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distill::{bce_with_logits, soft_target, take_blocks};

    /// Needs the Apple GPU, so it is run by hand:
    /// `cargo test --release --locked --lib trainer -- --ignored`.
    #[test]
    #[ignore = "uses the Apple GPU"]
    fn the_loss_with_a_teacher_is_the_cross_entropy_against_the_soft_target() {
        let (w, cfg) = crate::cpu_vit::small_network();
        // The student: the teacher's first block alone.
        let (student_w, student_cfg) = (take_blocks(&w, &[0]).unwrap(), VitConfig { depth: 1, ..cfg });
        let batch = 4;
        let mut rng = crate::rng::Rng::new(2);
        let x: Vec<f32> = (0..batch * 3 * cfg.img * cfg.img).map(|_| rng.range(-2.0, 2.0)).collect();
        let y = [1.0f32, 0.0, 1.0, 0.0];
        let opt = OptConfig::default();
        let teacher_logits = Trainer::new(&w, cfg, batch, opt.clone()).unwrap().logits(&x);
        let mut plain = Trainer::new(&student_w, student_cfg, batch, opt.clone()).unwrap();
        let student_logits = plain.logits(&x);
        assert!(teacher_logits.iter().zip(&student_logits).any(|(a, b)| (a - b).abs() > 1e-3), "the student should not be the teacher");
        let mean = |target: &dyn Fn(usize) -> f32| (0..batch).map(|i| bce_with_logits(student_logits[i], target(i))).sum::<f32>() / batch as f32;
        // Without a teacher: the labels.
        let loss = plain.train_step(&x, &y, 0.0);
        assert!((loss - mean(&|i| y[i])).abs() < 1e-4, "{loss} against {}", mean(&|i| y[i]));
        for (weight, temperature) in [(0.0f32, 1.0f32), (0.5, 1.0), (1.0, 2.0), (0.3, 4.0)] {
            let teacher = Teacher { weights: &w, cfg, weight: weight as f64, temperature: temperature as f64 };
            let mut tr = Trainer::with_teacher(&student_w, student_cfg, batch, opt.clone(), Some(teacher)).unwrap();
            // The student's own logits are untouched by the teacher being in the graph.
            for (a, b) in tr.logits(&x).iter().zip(&student_logits) {
                assert!((a - b).abs() < 1e-5);
            }
            let want = mean(&|i| soft_target(y[i], teacher_logits[i], weight, temperature));
            let loss = tr.train_step(&x, &y, 0.0);
            assert!((loss - want).abs() < 1e-4, "weight {weight}, temperature {temperature}: {loss} against {want}");
            // Only the student is exported, and a step moves it towards the target.
            assert_eq!(tr.export().0.len(), student_w.0.len());
            let before = tr.train_step(&x, &y, 1e-3);
            for _ in 0..20 {
                tr.train_step(&x, &y, 1e-3);
            }
            let after = tr.train_step(&x, &y, 0.0);
            assert!(after < before, "weight {weight}: the loss went from {before} to {after}");
        }
    }
}
