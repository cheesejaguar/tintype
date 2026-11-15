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

/// Draws items so that both classes are equally likely and, within a class,
/// generators are sampled with weight `count^alpha` (alpha < 1 lifts rare ones).
pub struct Sampler {
    classes: [Vec<(f64, Vec<usize>)>; 2],
}

impl Sampler {
    pub fn new(items: &[Item], alpha: f64) -> Self {
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

    pub fn sample(&self, label: usize, rng: &mut Rng) -> usize {
        let c = &self.classes[label];
        let r = rng.f32() as f64 * c.last().map(|x| x.0).unwrap_or(0.0);
        let g = &c[c.partition_point(|x| x.0 <= r).min(c.len() - 1)].1;
        g[rng.below(g.len())]
    }

    pub fn summary(&self) -> String {
        format!("{} real sources, {} generators", self.classes[0].len(), self.classes[1].len())
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
