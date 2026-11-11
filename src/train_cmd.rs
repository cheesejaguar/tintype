//! `veracity train`: fine-tune the detector on the Apple GPU.
#![cfg(target_os = "macos")]

use crate::data::{is_val, spawn_loader, AugConfig, Sampler};
use crate::detector::{ModelSpec, ViewSpec};
use crate::image_io::{self, View};
use crate::ingest::{read_manifest, Item};
use crate::metrics::{self, sigmoid};
use crate::trainer::{OptConfig, Trainer};
use crate::vit::VitConfig;
use crate::weights::Weights;
use anyhow::Result;
use rayon::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub struct TrainArgs {
    pub init: PathBuf,
    pub init_prefix: String,
    pub manifests: Vec<PathBuf>,
    pub out: PathBuf,
    pub steps: u64,
    pub batch: usize,
    pub lr: f32,
    pub warmup: u64,
    pub val_pct: u64,
    pub val_max: usize,
    pub eval_every: u64,
    pub threads: usize,
    pub seed: u64,
    pub aug: AugConfig,
    pub opt: OptConfig,
    pub sampler_alpha: f64,
}

struct ValSet {
    labels: Vec<u8>,
    /// Preprocessed views, `[view][item]` flattened CHW.
    views: Vec<Vec<f32>>,
    n: usize,
}

fn build_val(items: &[Item], views: &[View], max: usize, seed: u64) -> ValSet {
    // Class-balanced, deterministic subsample.
    let mut rng = crate::rng::Rng::new(seed ^ 0xA11CE);
    let mut by: [Vec<&Item>; 2] = [vec![], vec![]];
    for it in items {
        by[it.label as usize].push(it);
    }
    let mut chosen: Vec<&Item> = Vec::new();
    for c in by.iter_mut() {
        rng.shuffle(c);
        chosen.extend(c.iter().take(max / 2));
    }
    let vl = (views[0].size() * views[0].size() * 3) as usize;
    let loaded: Vec<(u8, Vec<Vec<f32>>)> = chosen
        .par_iter()
        .filter_map(|it| {
            let img = if crate::ingest::is_video(&it.path) { crate::video::middle_frame(&it.path).ok()? } else { image_io::load(&it.path).ok()? };
            Some((it.label, views.iter().map(|v| { let mut o = vec![0f32; vl]; v.apply(&img).write_chw(&mut o); o }).collect()))
        })
        .collect();
    let n = loaded.len();
    let mut out = ValSet { labels: Vec::with_capacity(n), views: vec![Vec::with_capacity(n * vl); views.len()], n };
    for (l, vs) in loaded {
        out.labels.push(l);
        for (k, v) in vs.into_iter().enumerate() {
            out.views[k].extend_from_slice(&v);
        }
    }
    out
}

fn eval(tr: &Trainer, val: &ValSet) -> (Vec<Vec<f32>>, String) {
    let vl = 3 * tr.cfg.img * tr.cfg.img;
    let mut logits: Vec<Vec<f32>> = Vec::new();
    let mut buf = vec![0f32; tr.batch * vl];
    for v in &val.views {
        let mut l = Vec::with_capacity(val.n);
        for chunk in v.chunks(tr.batch * vl) {
            buf[..chunk.len()].copy_from_slice(chunk);
            l.extend_from_slice(&tr.logits(&buf)[..chunk.len() / vl]);
        }
        logits.push(l);
    }
    let mut parts = Vec::new();
    let mut report = |name: &str, z: &[f32]| {
        let p: Vec<f32> = z.iter().map(|&x| sigmoid(x)).collect();
        let m = metrics::compute(&p, &val.labels, 0.5);
        parts.push(format!("{name}: auc {:.4} acc {:.4} P {:.3} R {:.3}", m.auroc, m.accuracy, m.precision, m.recall));
    };
    let names = ["fit", "native"];
    for (k, l) in logits.iter().enumerate() {
        report(names.get(k).copied().unwrap_or("view"), l);
    }
    if logits.len() > 1 {
        let mean: Vec<f32> = (0..val.n).map(|i| logits.iter().map(|l| l[i]).sum::<f32>() / logits.len() as f32).collect();
        report("fused", &mean);
    }
    (logits, parts.join(" | "))
}

pub fn run(a: TrainArgs) -> Result<()> {
    std::fs::create_dir_all(&a.out)?;
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open(a.out.join("train.log"))?;
    let mut say = |s: String| {
        println!("{s}");
        let _ = writeln!(log, "{s}");
    };

    let mut items = Vec::new();
    for m in &a.manifests {
        items.extend(read_manifest(m)?);
    }
    let (val_items, train_items): (Vec<Item>, Vec<Item>) = items.into_iter().partition(|it| is_val(it, a.val_pct));
    let sampler = Arc::new(Sampler::new(&train_items, a.sampler_alpha));
    say(format!("train {} images ({}), val pool {}", train_items.len(), sampler.summary(), val_items.len()));

    let size = a.aug.size;
    let specs = vec![ViewSpec::Fit { resize: a.aug.fit_resize, crop: size }, ViewSpec::Native { crop: size }];
    let views: Vec<View> = specs.iter().map(|&v| v.into()).collect();
    let t0 = Instant::now();
    let val = build_val(&val_items, &views, a.val_max, a.seed);
    say(format!("val set: {} images preprocessed in {:.1}s", val.n, t0.elapsed().as_secs_f64()));

    let w = Weights::load(&a.init, &a.init_prefix)?;
    let cfg = VitConfig::small(size as usize);
    let mut tr = Trainer::new(&w, cfg, a.batch, a.opt.clone())?;
    say(format!("init eval  {}", eval(&tr, &val).1));

    let rx = spawn_loader(Arc::new(train_items), sampler, a.aug.clone(), a.batch, a.threads, a.seed);
    let spec = ModelSpec {
        img: size as usize,
        views: crate::detector::thorough_views(size, a.aug.fit_resize),
        fast: Some(crate::detector::Profile { views: crate::detector::fast_views(size, a.aug.fit_resize), logit_scale: 1.0, logit_bias: 0.0 }),
        sensitive: None,
        strict: None,
        early_exit: None,
        arch: String::new(),
        logit_scale: 1.0,
        logit_bias: 0.0,
        threshold: 0.5,
        prefix: String::new(),
        video: Some(crate::detector::VideoSpec::standard(size, a.aug.fit_resize)),
    };
    let save = |tr: &Trainer, name: &str| -> Result<()> {
        let p = a.out.join(format!("{name}.safetensors"));
        tr.export().save(&p)?;
        spec.save_for(&p)
    };

    let (mut ema, mut wait_s) = (f32::NAN, 0f64);
    let mut tick = Instant::now();
    for step in 1..=a.steps {
        let lr = if step <= a.warmup {
            a.lr * step as f32 / a.warmup as f32
        } else {
            let p = (step - a.warmup) as f32 / (a.steps - a.warmup).max(1) as f32;
            a.lr * 0.5 * (1.0 + (std::f32::consts::PI * p).cos())
        };
        let tw = Instant::now();
        let b = rx.recv()?;
        wait_s += tw.elapsed().as_secs_f64();
        let loss = tr.train_step(&b.x, &b.y, lr);
        if !loss.is_finite() {
            anyhow::bail!("loss became non-finite at step {step}");
        }
        ema = if ema.is_nan() { loss } else { 0.98 * ema + 0.02 * loss };
        if step % 50 == 0 {
            let dt = tick.elapsed().as_secs_f64();
            say(format!("step {step:>6}/{} loss {ema:.4} lr {lr:.2e} {:.1} img/s (data wait {:.0}%)", a.steps, 50.0 * a.batch as f64 / dt, 100.0 * wait_s / dt));
            tick = Instant::now();
            wait_s = 0.0;
        }
        if step % a.eval_every == 0 || step == a.steps {
            say(format!("eval @{step}  {}", eval(&tr, &val).1));
            save(&tr, &format!("step_{step:06}"))?;
            save(&tr, "last")?;
            tick = Instant::now();
            wait_s = 0.0;
        }
    }
    Ok(())
}
