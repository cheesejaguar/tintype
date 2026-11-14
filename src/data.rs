//! Training data: balanced sampling over generators and the augmentation
//! pipeline, run on background threads that feed the GPU.
use crate::image_io::{self, Filter, Rgb};
use crate::ingest::Item;
use crate::rng::Rng;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AugConfig {
    pub size: u32,
    /// Shorter-side target of the "fit" view (inference uses exactly this).
    pub fit_resize: u32,
    /// Probability of training on a native-resolution crop instead of a fit view.
    pub p_native: f32,
    /// Probability of JPEG-compressing before / after the geometric step.
    pub p_jpeg_pre: f32,
    pub p_jpeg_post: f32,
    pub jpeg_quality: (f32, f32),
    /// Probability of a non-bilinear resampling filter.
    pub p_alt_filter: f32,
    /// Probability of jittering the fit scale / rescaling a native crop.
    pub p_scale_jitter: f32,
    pub p_flip: f32,
    /// For video items: probability of H.264 re-encoding before frame grab.
    pub p_video_recompress: f32,
    /// Probability of a messenger-style round trip: downscale to a typical
    /// upload size, then JPEG at the qualities chat and social apps use.
    pub p_messenger: f32,
}

impl Default for AugConfig {
    fn default() -> Self {
        Self {
            size: 224,
            fit_resize: 256,
            p_native: 0.5,
            p_jpeg_pre: 0.25,
            p_jpeg_post: 0.25,
            jpeg_quality: (40.0, 98.0),
            p_alt_filter: 0.3,
            p_scale_jitter: 0.3,
            p_flip: 0.5,
            p_video_recompress: 0.5,
            p_messenger: 0.0,
        }
    }
}

fn random_crop(img: &Rgb, size: u32, rng: &mut Rng) -> Rgb {
    let x = rng.below((img.w.saturating_sub(size) + 1) as usize) as i64;
    let y = rng.below((img.h.saturating_sub(size) + 1) as usize) as i64;
    img.crop(x, y, size, size)
}

/// One randomly augmented training view.
pub fn augment(mut img: Rgb, cfg: &AugConfig, rng: &mut Rng) -> Rgb {
    let q = |rng: &mut Rng| rng.range(cfg.jpeg_quality.0, cfg.jpeg_quality.1).round() as u8;
    if rng.chance(cfg.p_messenger) {
        // What WhatsApp / Instagram / re-sharing do: cap the long side, then
        // re-encode at moderate JPEG quality.
        let cap = rng.range(960.0, 1700.0);
        let long = img.w.max(img.h) as f32;
        if long > cap {
            let f = cap / long;
            let filter = if rng.chance(0.5) { Filter::Bicubic } else { Filter::Bilinear };
            img = img.resize(((img.w as f32 * f) as u32).max(1), ((img.h as f32 * f) as u32).max(1), filter);
        }
        if let Ok(j) = img.jpeg_roundtrip(rng.range(55.0, 90.0).round() as u8) {
            img = j;
        }
    } else if rng.chance(cfg.p_jpeg_pre) && (img.w as u64 * img.h as u64) <= 6_000_000 {
        // JPEG at full resolution models "saved as JPEG, then resized by us".
        // Skipped for very large images to bound the cost.
        if let Ok(j) = img.jpeg_roundtrip(q(rng)) {
            img = j;
        }
    }
    let filter = if rng.chance(cfg.p_alt_filter) { Filter::ALL[rng.below(Filter::ALL.len())] } else { Filter::Bilinear };
    let mut out = if rng.chance(cfg.p_native) {
        // Native view: original pixels, optionally mildly rescaled.
        if rng.chance(cfg.p_scale_jitter) {
            let f = rng.range(0.5, 1.25);
            let (w, h) = (((img.w as f32 * f) as u32).max(cfg.size), ((img.h as f32 * f) as u32).max(cfg.size));
            img = img.resize(w, h, filter);
        }
        if img.w.min(img.h) < cfg.size {
            img = img.resize_short(cfg.size, filter);
        }
        random_crop(&img, cfg.size, rng)
    } else {
        let short = if rng.chance(cfg.p_scale_jitter) { rng.range(cfg.size as f32, cfg.fit_resize as f32 * 1.25) as u32 } else { cfg.fit_resize };
        random_crop(&img.resize_short(short, filter), cfg.size, rng)
    };
    if rng.chance(cfg.p_jpeg_post) {
        if let Ok(j) = out.jpeg_roundtrip(q(rng)) {
            out = j;
        }
    }
    if rng.chance(cfg.p_flip) {
        out.flip_h();
    }
    out
}

/// Explicit sampling weights and a replay pool for [`Sampler::with_plan`].
/// The default plan (no weights, no replay pool) is the sampler as it has
/// always been: [`Sampler::new`].
#[derive(Clone, Debug, Default)]
pub struct SamplingPlan {
    /// Generator sampling weight exponent (`count^alpha`).
    pub alpha: f64,
    /// A multiplier on each item's sampling mass, in item order (its
    /// manifest's weight times its family's). Empty means 1 for every item.
    pub weights: Vec<f64>,
    /// Whether each item belongs to the replay pool, in item order. Empty
    /// means there is no replay pool.
    pub replay: Vec<bool>,
    /// The share of draws taken from the replay pool when both pools hold
    /// the class that was drawn.
    pub replay_share: f32,
}

/// One pool of items: per class, the cumulative weight of each generator
/// and its items.
struct Pool {
    classes: [Vec<(f64, Vec<usize>)>; 2],
    /// For a generator whose items are not equally likely, their cumulative
    /// weights (same indices as `classes`). `None` draws an item uniformly.
    within: [Vec<Option<Vec<f64>>>; 2],
}

impl Pool {
    /// Group the items `member` accepts by class and generator. A generator
    /// weighs `count^alpha` times the mean weight of its items, and inside
    /// it an item is drawn in proportion to its weight, so an item's mass is
    /// `count^(alpha-1) * weight`: with every weight 1 this is the plain
    /// `count^alpha` sampler, computed by the same expressions.
    fn build(items: &[Item], alpha: f64, member: impl Fn(usize) -> bool, weight: impl Fn(usize) -> f64) -> Self {
        let mut groups: [BTreeMap<&str, Vec<usize>>; 2] = [BTreeMap::new(), BTreeMap::new()];
        for (i, it) in items.iter().enumerate() {
            if member(i) {
                groups[it.label as usize].entry(it.generator.as_str()).or_default().push(i);
            }
        }
        let build = |g: &BTreeMap<&str, Vec<usize>>| {
            let mut acc = 0.0;
            let (mut classes, mut within) = (Vec::new(), Vec::new());
            for v in g.values() {
                let w: Vec<f64> = v.iter().map(|&i| weight(i)).collect();
                if w.iter().all(|&x| x == 1.0) {
                    acc += (v.len() as f64).powf(alpha);
                    within.push(None);
                } else {
                    let total: f64 = w.iter().sum();
                    if total <= 0.0 {
                        // Every item weighted to zero: the generator is never drawn.
                        continue;
                    }
                    acc += (v.len() as f64).powf(alpha) * (total / v.len() as f64);
                    let mut run = 0.0;
                    within.push(if w.iter().all(|&x| x == w[0]) { None } else { Some(w.iter().map(|&x| { run += x; run }).collect()) });
                }
                classes.push((acc, v.clone()));
            }
            (classes, within)
        };
        let (real, ai) = (build(&groups[0]), build(&groups[1]));
        Self { classes: [real.0, ai.0], within: [real.1, ai.1] }
    }

    fn sample(&self, label: usize, rng: &mut Rng) -> usize {
        let c = &self.classes[label];
        let r = rng.f32() as f64 * c.last().map(|x| x.0).unwrap_or(0.0);
        let k = c.partition_point(|x| x.0 <= r).min(c.len() - 1);
        let g = &c[k].1;
        match &self.within[label][k] {
            None => g[rng.below(g.len())],
            Some(cum) => {
                let r = rng.f32() as f64 * cum[cum.len() - 1];
                g[cum.partition_point(|&x| x <= r).min(g.len() - 1)]
            }
        }
    }

    /// Add `share` times each item's probability of being drawn, given its class.
    fn add_probabilities(&self, share: [f64; 2], out: &mut [f64]) {
        for label in 0..2 {
            let c = &self.classes[label];
            let total = c.last().map_or(0.0, |x| x.0);
            let mut before = 0.0;
            for (k, (acc, g)) in c.iter().enumerate() {
                let p = share[label] * (acc - before) / total;
                before = *acc;
                match &self.within[label][k] {
                    None => g.iter().for_each(|&i| out[i] += p / g.len() as f64),
                    Some(cum) => {
                        let mut prev = 0.0;
                        for (&i, &x) in g.iter().zip(cum) {
                            out[i] += p * (x - prev) / cum[cum.len() - 1];
                            prev = x;
                        }
                    }
                }
            }
        }
    }
}

/// Draws items so that both classes are equally likely and, within a class,
/// generators are sampled with weight `count^alpha` (alpha < 1 lifts rare ones).
///
/// [`Sampler::with_plan`] adds explicit per-item weights (a manifest's and a
/// generator family's) and a replay pool that a fixed share of the draws
/// comes from. Without them it draws exactly what [`Sampler::new`] draws.
pub struct Sampler {
    main: Pool,
    /// The replay pool and, per class, the probability that a draw comes from it.
    replay: Option<(Pool, [f32; 2])>,
}

impl Sampler {
    pub fn new(items: &[Item], alpha: f64) -> Self {
        Self { main: Pool::build(items, alpha, |_| true, |_| 1.0), replay: None }
    }

    /// A sampler with explicit weights and, if the plan marks any item as
    /// replay, a replay pool. Fails if a weight is negative or not finite,
    /// or if the weights leave a class that has items with nothing to draw.
    pub fn with_plan(items: &[Item], plan: &SamplingPlan) -> anyhow::Result<Self> {
        anyhow::ensure!(plan.weights.is_empty() || plan.weights.len() == items.len(), "the plan has {} weights for {} items", plan.weights.len(), items.len());
        anyhow::ensure!(plan.replay.is_empty() || plan.replay.len() == items.len(), "the plan marks {} items as replay or not, for {} items", plan.replay.len(), items.len());
        anyhow::ensure!(plan.weights.iter().all(|w| w.is_finite() && *w >= 0.0), "sampling weights must be finite and not negative");
        anyhow::ensure!((0.0..=1.0).contains(&plan.replay_share), "the replay share must be between 0 and 1");
        let weight = |i: usize| plan.weights.get(i).copied().unwrap_or(1.0);
        let in_replay = |i: usize| plan.replay.get(i).copied().unwrap_or(false);
        let main = Pool::build(items, plan.alpha, |i| !in_replay(i), weight);
        let replay = plan.replay.iter().any(|&r| r).then(|| Pool::build(items, plan.alpha, in_replay, weight));
        let mut from_replay = [0f32; 2];
        for label in 0..2 {
            let (m, r) = (!main.classes[label].is_empty(), replay.as_ref().is_some_and(|p| !p.classes[label].is_empty()));
            anyhow::ensure!(m || r || !items.iter().any(|it| it.label as usize == label), "the sampling weights leave no {} item to draw", if label == 0 { "real" } else { "AI" });
            // A class that only one pool holds is drawn from that pool.
            from_replay[label] = if m && r { plan.replay_share } else if r { 1.0 } else { 0.0 };
        }
        Ok(Self { main, replay: replay.map(|p| (p, from_replay)) })
    }

    pub fn sample(&self, label: usize, rng: &mut Rng) -> usize {
        match &self.replay {
            // One more draw decides the pool; without a replay pool the draws are what they always were.
            Some((pool, share)) if rng.chance(share[label]) => pool.sample(label, rng),
            _ => self.main.sample(label, rng),
        }
    }

    /// The probability that a draw of an item's class is that item, for every
    /// item (`n` of them). Each class sums to 1.
    pub fn probabilities(&self, n: usize) -> Vec<f64> {
        let mut out = vec![0.0; n];
        let share = self.replay.as_ref().map_or([0.0; 2], |(_, s)| [s[0] as f64, s[1] as f64]);
        self.main.add_probabilities([1.0 - share[0], 1.0 - share[1]], &mut out);
        if let Some((pool, _)) = &self.replay {
            pool.add_probabilities(share, &mut out);
        }
        out
    }

    pub fn summary(&self) -> String {
        let mut s = format!("{} real sources, {} generators", self.main.classes[0].len(), self.main.classes[1].len());
        if let Some((pool, share)) = &self.replay {
            s.push_str(&format!("; replay pool of {} real sources, {} generators at {:.0}% / {:.0}% of real / AI draws", pool.classes[0].len(), pool.classes[1].len(), 100.0 * share[0], 100.0 * share[1]));
        }
        s
    }
}

pub struct Batch {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
}

/// Spawn `threads` workers producing augmented batches forever.
pub fn spawn_loader(items: Arc<Vec<Item>>, sampler: Arc<Sampler>, cfg: AugConfig, batch: usize, threads: usize, seed: u64) -> crossbeam_channel::Receiver<Batch> {
    let (stx, srx) = crossbeam_channel::bounded::<(Vec<f32>, f32)>(batch * 4);
    for t in 0..threads {
        let (items, sampler, cfg, stx) = (items.clone(), sampler.clone(), cfg.clone(), stx.clone());
        std::thread::spawn(move || {
            let mut rng = Rng::new(seed.wrapping_mul(0x9E37_79B9).wrapping_add(t as u64 + 1));
            let n = (cfg.size * cfg.size * 3) as usize;
            let tmp = std::env::temp_dir().join("tintype-loader");
            let _ = std::fs::create_dir_all(&tmp);
            loop {
                let label = rng.below(2);
                let it = &items[sampler.sample(label, &mut rng)];
                let img = if crate::ingest::is_video(&it.path) {
                    crate::video::random_frame(&it.path, &mut rng, cfg.p_video_recompress, &tmp)
                } else {
                    image_io::load(&it.path)
                };
                let Ok(img) = img else { continue };
                let mut v = vec![0f32; n];
                augment(img, &cfg, &mut rng).write_chw(&mut v);
                if stx.send((v, label as f32)).is_err() {
                    return;
                }
            }
        });
    }
    let (btx, brx) = crossbeam_channel::bounded::<Batch>(4);
    std::thread::spawn(move || {
        let n = (cfg.size * cfg.size * 3) as usize;
        loop {
            let mut b = Batch { x: vec![0f32; batch * n], y: Vec::with_capacity(batch) };
            for k in 0..batch {
                let Ok((v, l)) = srx.recv() else { return };
                b.x[k * n..(k + 1) * n].copy_from_slice(&v);
                b.y.push(l);
            }
            if btx.send(b).is_err() {
                return;
            }
        }
    });
    brx
}

/// Deterministic train/validation split by path hash (`val_pct` percent held out).
pub fn is_val(item: &Item, val_pct: u64) -> bool {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in item.path.to_string_lossy().bytes() {
        h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3);
    }
    (h >> 20) % 100 < val_pct
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn item(label: u8, gen: &str, i: usize) -> Item {
        Item { path: PathBuf::from(format!("{gen}/{i}.png")), label, generator: gen.into(), dataset: "t".into(), ..Default::default() }
    }

    #[test]
    fn sampler_respects_class_and_lifts_rare_generators() {
        let mut items = Vec::new();
        items.extend((0..900).map(|i| item(1, "big", i)));
        items.extend((0..100).map(|i| item(1, "small", i)));
        items.extend((0..50).map(|i| item(0, "real:a", i)));
        let s = Sampler::new(&items, 0.5);
        let mut rng = Rng::new(3);
        let mut small = 0;
        for _ in 0..4000 {
            let it = &items[s.sample(1, &mut rng)];
            assert_eq!(it.label, 1);
            small += (it.generator == "small") as usize;
        }
        // sqrt weighting: 10 / (30 + 10) = 25% instead of the raw 10%
        assert!((800..1200).contains(&small), "small sampled {small} times");
        assert_eq!(items[s.sample(0, &mut rng)].label, 0);
    }

    /// The sampler as it was before weights and replay pools, copied
    /// verbatim: the default sampler must draw what this draws.
    struct SamplerReference {
        classes: [Vec<(f64, Vec<usize>)>; 2],
    }

    impl SamplerReference {
        fn new(items: &[Item], alpha: f64) -> Self {
            let mut groups: [BTreeMap<&str, Vec<usize>>; 2] = [BTreeMap::new(), BTreeMap::new()];
            for (i, it) in items.iter().enumerate() {
                groups[it.label as usize].entry(it.generator.as_str()).or_default().push(i);
            }
            let build = |g: &BTreeMap<&str, Vec<usize>>| {
                let mut acc = 0.0;
                g.values()
                    .map(|v| {
                        acc += (v.len() as f64).powf(alpha);
                        (acc, v.clone())
                    })
                    .collect::<Vec<_>>()
            };
            Self { classes: [build(&groups[0]), build(&groups[1])] }
        }

        fn sample(&self, label: usize, rng: &mut Rng) -> usize {
            let c = &self.classes[label];
            let r = rng.f32() as f64 * c.last().map(|x| x.0).unwrap_or(0.0);
            let g = &c[c.partition_point(|x| x.0 <= r).min(c.len() - 1)].1;
            g[rng.below(g.len())]
        }

        fn summary(&self) -> String {
            format!("{} real sources, {} generators", self.classes[0].len(), self.classes[1].len())
        }
    }

    /// A long-tailed mix shaped like the training stores: a few large
    /// generators, hundreds of small ones, sixteen real sources.
    fn long_tail() -> Vec<Item> {
        let mut rng = Rng::new(77);
        let mut items = Vec::new();
        for g in 0..400 {
            let n = if g < 5 { 3000 + 500 * g } else { 1 + rng.below(60) };
            items.extend((0..n).map(|i| item(1, &format!("gen-{g}"), i)));
        }
        for s in 0..16 {
            let n = 200 + rng.below(4000);
            items.extend((0..n).map(|i| item(0, &format!("real:src-{s}"), i)));
        }
        // Interleave, so that a generator's items are not one run of indices.
        rng.shuffle(&mut items);
        items
    }

    #[test]
    fn the_default_sampler_draws_exactly_what_it_always_drew() {
        let items = long_tail();
        for (alpha, seed) in [(0.5, 1u64), (0.5, 20_261_113), (0.0, 3), (1.0, 4), (0.75, 5)] {
            let reference = SamplerReference::new(&items, alpha);
            let plain = Sampler::new(&items, alpha);
            // An empty plan, and one that spells out weight 1 for every item, are the same sampler.
            let empty = Sampler::with_plan(&items, &SamplingPlan { alpha, ..Default::default() }).unwrap();
            let ones = Sampler::with_plan(&items, &SamplingPlan { alpha, weights: vec![1.0; items.len()], replay: vec![false; items.len()], replay_share: 0.2 }).unwrap();
            assert_eq!(plain.summary(), reference.summary());
            let mut rngs = [Rng::new(seed), Rng::new(seed), Rng::new(seed), Rng::new(seed)];
            for k in 0..200_000 {
                // The loader draws the class first, from the same generator.
                let label = rngs[0].below(2);
                for r in &mut rngs[1..] {
                    assert_eq!(r.below(2), label);
                }
                let want = reference.sample(label, &mut rngs[0]);
                assert_eq!(plain.sample(label, &mut rngs[1]), want, "alpha {alpha} seed {seed} draw {k}");
                assert_eq!(empty.sample(label, &mut rngs[2]), want, "empty plan, alpha {alpha} seed {seed} draw {k}");
                assert_eq!(ones.sample(label, &mut rngs[3]), want, "all-ones plan, alpha {alpha} seed {seed} draw {k}");
            }
            // The generators are left in the same state: nothing drew more or less.
            let next = rngs[0].u64();
            assert!(rngs[1..].iter_mut().all(|r| r.u64() == next));
        }
    }

    #[test]
    fn the_stated_probabilities_are_the_frequencies_drawn() {
        let mut items = Vec::new();
        items.extend((0..900).map(|i| item(1, "big", i)));
        items.extend((0..100).map(|i| item(1, "small", i)));
        items.extend((0..64).map(|i| item(1, "old", i)));
        items.extend((0..50).map(|i| item(0, "real:a", i)));
        items.extend((0..50).map(|i| item(0, "real:old", i)));
        // Half of `big` counts double, `small` counts three times, `old` and `real:old` are the replay pool.
        let weights: Vec<f64> = items.iter().enumerate().map(|(i, it)| if it.generator == "big" && i % 2 == 0 { 2.0 } else if it.generator == "small" { 3.0 } else { 1.0 }).collect();
        let replay: Vec<bool> = items.iter().map(|it| it.generator.ends_with("old")).collect();
        let s = Sampler::with_plan(&items, &SamplingPlan { alpha: 0.5, weights, replay, replay_share: 0.2 }).unwrap();
        let p = s.probabilities(items.len());
        for label in [0u8, 1] {
            let total: f64 = items.iter().zip(&p).filter(|(it, _)| it.label == label).map(|(_, x)| x).sum();
            assert!((total - 1.0).abs() < 1e-9, "class {label} sums to {total}");
        }
        let of = |gen: &str| -> f64 { items.iter().zip(&p).filter(|(it, _)| it.generator == gen).map(|(_, x)| x).sum() };
        // Main pool: big 30 x 1.5, small 10 x 3; the replay pool is 20% of both classes.
        assert!((of("big") - 0.8 * 45.0 / 75.0).abs() < 1e-6 && (of("small") - 0.8 * 30.0 / 75.0).abs() < 1e-6, "{} {}", of("big"), of("small"));
        assert!((of("old") - 0.2).abs() < 1e-6 && (of("real:old") - 0.2).abs() < 1e-6 && (of("real:a") - 0.8).abs() < 1e-6);
        // Inside `big`, the doubled half is drawn twice as often.
        assert!((p[0] / p[1] - 2.0).abs() < 1e-9);
        let mut rng = Rng::new(11);
        let mut seen = vec![0usize; items.len()];
        let draws = 400_000;
        for k in 0..draws {
            let i = s.sample(k % 2, &mut rng);
            assert_eq!(items[i].label as usize, k % 2);
            seen[i] += 1;
        }
        for gen in ["big", "small", "old", "real:a", "real:old"] {
            let freq = items.iter().zip(&seen).filter(|(it, _)| it.generator == gen).map(|(_, n)| *n).sum::<usize>() as f64 / (draws / 2) as f64;
            assert!((freq - of(gen)).abs() < 0.01, "{gen}: drawn {freq}, stated {}", of(gen));
        }
        let (doubled, single): (usize, usize) = (0..900).fold((0, 0), |a, i| if i % 2 == 0 { (a.0 + seen[i], a.1) } else { (a.0, a.1 + seen[i]) });
        assert!((doubled as f64 / single as f64 - 2.0).abs() < 0.05, "{doubled} / {single}");
        assert_eq!(s.summary(), "1 real sources, 2 generators; replay pool of 1 real sources, 1 generators at 20% / 20% of real / AI draws");
    }

    #[test]
    fn zero_weights_remove_items_and_an_emptied_class_is_an_error() {
        let mut items = Vec::new();
        items.extend((0..40).map(|i| item(1, "keep", i)));
        items.extend((0..40).map(|i| item(1, "drop", i)));
        items.extend((0..40).map(|i| item(1, "half", i)));
        items.extend((0..10).map(|i| item(0, "real:a", i)));
        let weights: Vec<f64> = items.iter().enumerate().map(|(i, it)| if it.generator == "drop" || (it.generator == "half" && i % 2 == 0) { 0.0 } else { 1.0 }).collect();
        let s = Sampler::with_plan(&items, &SamplingPlan { alpha: 0.5, weights: weights.clone(), ..Default::default() }).unwrap();
        let mut rng = Rng::new(2);
        for _ in 0..20_000 {
            let i = s.sample(1, &mut rng);
            assert!(weights[i] > 0.0, "drew {} (item {i}), which has weight 0", items[i].generator);
        }
        assert_eq!(s.summary(), "1 real sources, 2 generators");
        // A class with items and nothing left to draw is refused, as are bad weights and a plan of the wrong length.
        let no_real: Vec<f64> = items.iter().map(|it| it.label as f64).collect();
        assert!(Sampler::with_plan(&items, &SamplingPlan { alpha: 0.5, weights: no_real, ..Default::default() }).is_err());
        assert!(Sampler::with_plan(&items, &SamplingPlan { alpha: 0.5, weights: vec![-1.0; items.len()], ..Default::default() }).is_err());
        assert!(Sampler::with_plan(&items, &SamplingPlan { alpha: 0.5, weights: vec![1.0; 3], ..Default::default() }).is_err());
        assert!(Sampler::with_plan(&items, &SamplingPlan { alpha: 0.5, replay: vec![false; items.len()], replay_share: 1.5, ..Default::default() }).is_err());
        // A class that only the replay pool holds is drawn from it every time.
        let replay: Vec<bool> = items.iter().map(|it| it.label == 0 || it.generator == "keep").collect();
        let s = Sampler::with_plan(&items, &SamplingPlan { alpha: 0.5, replay, replay_share: 0.1, ..Default::default() }).unwrap();
        assert!((0..2000).all(|_| items[s.sample(0, &mut rng)].label == 0));
        let p = s.probabilities(items.len());
        assert!((items.iter().zip(&p).filter(|(it, _)| it.generator == "keep").map(|(_, x)| x).sum::<f64>() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn augment_always_returns_training_size() {
        let cfg = AugConfig::default();
        let mut rng = Rng::new(9);
        for (w, h) in [(1024, 768), (300, 200), (64, 48), (224, 224)] {
            for _ in 0..20 {
                let mut im = Rgb::new(w, h);
                im.data.iter_mut().enumerate().for_each(|(i, p)| *p = (i * 7 % 251) as u8);
                let o = augment(im, &cfg, &mut rng);
                assert_eq!((o.w, o.h), (224, 224));
            }
        }
    }

    #[test]
    fn val_split_is_deterministic_and_roughly_sized() {
        let n = (0..10_000).filter(|&i| is_val(&item(0, "g", i), 3)).count();
        assert!((200..400).contains(&n), "{n}");
        assert_eq!(is_val(&item(0, "g", 5), 3), is_val(&item(0, "g", 5), 3));
    }
}
