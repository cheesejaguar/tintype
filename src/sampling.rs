//! A training recipe's sampling: which manifest and which generator family
//! each item counts under, the explicit weights given for them, the replay
//! pool, and the effective sampling shares that result
//! (`tintype sampling-shares`, `tintype train --print-sampling`).
//!
//! The sampler itself is [`crate::data::Sampler`]. With no weights and no
//! replay manifests a recipe builds [`Sampler::new`], which draws what it
//! always drew. Nothing here needs the GPU, so it runs on every platform.
use crate::data::{is_val, Sampler, SamplingPlan};
use crate::ingest::{read_manifest, Item};
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The sampling flags of `tintype train`, parsed.
#[derive(Clone, Debug)]
pub struct Options {
    /// `(manifest, weight)`: a manifest is named by its path as given, or by
    /// its short name (see [`manifest_names`]).
    pub manifest_weights: Vec<(String, f64)>,
    /// `(family, weight)`: the family of an item is its `generator_family`
    /// column, or else its generator's entry in `family_map`, or else the
    /// generator name.
    pub family_weights: Vec<(String, f64)>,
    /// Generator name to family, for manifests without the column.
    pub family_map: BTreeMap<String, String>,
    /// Manifests whose items form the replay pool.
    pub replay_manifests: Vec<PathBuf>,
    /// The share of draws that come from the replay pool.
    pub replay_share: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self { manifest_weights: Vec::new(), family_weights: Vec::new(), family_map: BTreeMap::new(), replay_manifests: Vec::new(), replay_share: 0.2 }
    }
}

/// Parse `name=weight` (the name may itself contain `=`).
pub fn parse_weight(s: &str) -> Result<(String, f64)> {
    let (name, w) = s.rsplit_once('=').with_context(|| format!("`{s}`: a weight is written name=weight"))?;
    let w: f64 = w.trim().parse().with_context(|| format!("`{s}`: the weight is not a number"))?;
    anyhow::ensure!(!name.is_empty() && w.is_finite() && w >= 0.0, "`{s}`: a weight needs a name and a finite value that is not negative");
    Ok((name.to_string(), w))
}

fn data_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
}

/// Read a weights file: tab-separated `manifest|family  name  weight` rows;
/// lines starting with `#` are comments. Returns the manifest weights and
/// the family weights.
pub fn read_weights_file(path: &Path) -> Result<(Vec<(String, f64)>, Vec<(String, f64)>)> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let (mut manifests, mut families) = (Vec::new(), Vec::new());
    for l in data_lines(&text) {
        let f: Vec<&str> = l.split('\t').collect();
        anyhow::ensure!(f.len() >= 3, "{}: expected `manifest|family<TAB>name<TAB>weight`, got: {l}", path.display());
        let entry = parse_weight(&format!("{}={}", f[1], f[2])).with_context(|| format!("in {}", path.display()))?;
        match f[0] {
            "manifest" => manifests.push(entry),
            "family" => families.push(entry),
            other => bail!("{}: a row starts with `manifest` or `family`, not `{other}`", path.display()),
        }
    }
    Ok((manifests, families))
}

/// Read a family map: tab-separated `generator  family` rows (further
/// columns are ignored, as are `#` comments). `scripts/group-manifest.py
/// --write-map` writes one from the pattern rules in `docs/groups`.
pub fn read_family_map(path: &Path) -> Result<BTreeMap<String, String>> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut map = BTreeMap::new();
    for l in data_lines(&text) {
        let f: Vec<&str> = l.split('\t').collect();
        anyhow::ensure!(f.len() >= 2 && !f[0].is_empty() && !f[1].is_empty(), "{}: expected `generator<TAB>family`, got: {l}", path.display());
        if let Some(old) = map.insert(f[0].to_string(), f[1].to_string()) {
            anyhow::ensure!(old == f[1], "{}: generator {} is mapped to both {old} and {}", path.display(), f[0], f[1]);
        }
    }
    Ok(map)
}

/// A short name for each manifest: the directory of a `manifest.tsv` (the
/// store's name), or else the file name without its extension. Manifests
/// whose short names collide keep their paths as given.
pub fn manifest_names(paths: &[PathBuf]) -> Vec<String> {
    let short = |p: &PathBuf| -> String {
        let named = if p.file_name().is_some_and(|f| f == "manifest.tsv") { p.parent().and_then(|d| d.file_name()) } else { p.file_stem() };
        named.map_or_else(|| p.display().to_string(), |s| s.to_string_lossy().into_owned())
    };
    let names: Vec<String> = paths.iter().map(short).collect();
    paths.iter().zip(&names).map(|(p, n)| if names.iter().filter(|m| *m == n).count() > 1 { p.display().to_string() } else { n.clone() }).collect()
}

/// The items of a recipe with what the sampler needs to know about each.
pub struct Recipe {
    pub items: Vec<Item>,
    /// Per item, the index in `manifests` of the manifest it was read from.
    pub origin: Vec<usize>,
    /// The short name of every manifest, replay manifests last.
    pub manifests: Vec<String>,
    /// Per manifest, whether it feeds the replay pool.
    pub replay: Vec<bool>,
    /// Per item, the index in `families` of its generator family.
    pub family: Vec<usize>,
    pub families: Vec<String>,
    pub plan: SamplingPlan,
}

impl Recipe {
    /// Read the manifests (replay manifests after the others, each in the
    /// order given) and resolve the weights. Fails on a weight that names no
    /// manifest or no family of the recipe, or names two manifests.
    pub fn load(manifests: &[PathBuf], opts: &Options, alpha: f64) -> Result<Self> {
        let paths: Vec<PathBuf> = manifests.iter().chain(&opts.replay_manifests).cloned().collect();
        for (k, p) in paths.iter().enumerate() {
            anyhow::ensure!(!paths[..k].contains(p), "{} is listed twice", p.display());
        }
        let names = manifest_names(&paths);
        let replay: Vec<bool> = (0..paths.len()).map(|k| k >= manifests.len()).collect();
        let (mut items, mut origin) = (Vec::new(), Vec::new());
        for (k, p) in paths.iter().enumerate() {
            let read = read_manifest(p)?;
            origin.extend(std::iter::repeat(k).take(read.len()));
            items.extend(read);
        }

        let mut families: Vec<String> = Vec::new();
        let mut index: BTreeMap<String, usize> = BTreeMap::new();
        let family: Vec<usize> = items
            .iter()
            .map(|it| {
                let name = it.generator_family().or_else(|| opts.family_map.get(&it.generator).map(String::as_str)).unwrap_or(&it.generator);
                *index.entry(name.to_string()).or_insert_with(|| {
                    families.push(name.to_string());
                    families.len() - 1
                })
            })
            .collect();

        let mut manifest_w = vec![1.0; paths.len()];
        for (k, (name, w)) in opts.manifest_weights.iter().enumerate() {
            anyhow::ensure!(!opts.manifest_weights[..k].iter().any(|(n, _)| n == name), "manifest {name} is given two weights");
            let hits: Vec<usize> = (0..paths.len()).filter(|&j| names[j] == *name || paths[j].to_string_lossy() == *name).collect();
            match hits.as_slice() {
                [j] => manifest_w[*j] = *w,
                [] => bail!("no manifest is called {name} (the manifests are: {})", names.join(", ")),
                _ => bail!("{name} names more than one manifest; use its path"),
            }
        }
        let mut family_w = vec![1.0; families.len()];
        for (k, (name, w)) in opts.family_weights.iter().enumerate() {
            anyhow::ensure!(!opts.family_weights[..k].iter().any(|(n, _)| n == name), "family {name} is given two weights");
            match index.get(name) {
                Some(&j) => family_w[j] = *w,
                None => bail!("no item has the generator family {name}; `tintype sampling-shares --top 0` lists the families of a recipe"),
            }
        }
        let weighted = manifest_w.iter().chain(&family_w).any(|&w| w != 1.0);
        let plan = SamplingPlan {
            alpha,
            weights: if weighted { (0..items.len()).map(|i| manifest_w[origin[i]] * family_w[family[i]]).collect() } else { Vec::new() },
            replay: if opts.replay_manifests.is_empty() { Vec::new() } else { origin.iter().map(|&k| replay[k]).collect() },
            replay_share: opts.replay_share,
        };
        anyhow::ensure!((0.0..=1.0).contains(&plan.replay_share), "--replay-share must be between 0 and 1");
        Ok(Self { items, origin, manifests: names, replay, family, families, plan })
    }

    /// Take out the validation items ([`is_val`]): the recipe that is left
    /// is what training samples from. Order is kept on both sides.
    pub fn split_validation(self, val_pct: u64) -> (Self, Vec<Item>) {
        let Recipe { items, origin, manifests, replay, family, families, plan } = self;
        let mut train = Recipe { items: Vec::new(), origin: Vec::new(), manifests, replay, family: Vec::new(), families, plan: SamplingPlan { weights: Vec::new(), replay: Vec::new(), ..plan.clone() } };
        let mut val = Vec::new();
        for (i, it) in items.into_iter().enumerate() {
            if is_val(&it, val_pct) {
                val.push(it);
            } else {
                train.items.push(it);
                train.origin.push(origin[i]);
                train.family.push(family[i]);
                if !plan.weights.is_empty() {
                    train.plan.weights.push(plan.weights[i]);
                }
                if !plan.replay.is_empty() {
                    train.plan.replay.push(plan.replay[i]);
                }
            }
        }
        (train, val)
    }

    /// Whether the recipe has neither weights nor a replay pool.
    pub fn is_plain(&self) -> bool {
        self.plan.weights.is_empty() && self.plan.replay.is_empty()
    }

    /// The sampler of this recipe: [`Sampler::new`] when it is plain.
    pub fn sampler(&self) -> Result<Sampler> {
        if self.is_plain() {
            Ok(Sampler::new(&self.items, self.plan.alpha))
        } else {
            Sampler::with_plan(&self.items, &self.plan)
        }
    }

    /// Per class, the sampling share and item count of every value of a key.
    fn shares_by<'a>(&'a self, p: &[f64], key: impl Fn(usize) -> &'a str) -> [BTreeMap<&'a str, (f64, usize)>; 2] {
        let mut out = [BTreeMap::new(), BTreeMap::new()];
        for (i, it) in self.items.iter().enumerate() {
            let e = out[it.label as usize].entry(key(i)).or_insert((0.0, 0));
            e.0 += p[i];
            e.1 += 1;
        }
        out
    }

    /// The effective sampling shares of the recipe as Markdown tables: per
    /// manifest, per generator family and per generator. A share is of the
    /// draws of that class (each class is half of all draws), computed from
    /// the sampler's weights, not estimated from draws. `top` limits the
    /// family and generator tables to the largest shares of each class; 0
    /// prints every row.
    pub fn shares_report(&self, sampler: &Sampler, top: usize) -> String {
        let p = sampler.probabilities(self.items.len());
        let class_items = [self.items.iter().filter(|it| it.label == 0).count(), self.items.iter().filter(|it| it.label == 1).count()];
        let pct = |x: f64| format!("{:.2}%", 100.0 * x);
        let mut out = format!(
            "Sampling shares of {} items ({} real, {} AI) from {} manifests; generators weigh count^{}{}{}.\nA share is of the draws of its class; each class is half of all draws. \"Lift\" is the share divided by the share of the class's items.\n",
            self.items.len(),
            class_items[0],
            class_items[1],
            self.manifests.len(),
            self.plan.alpha,
            if self.plan.weights.is_empty() { "" } else { ", times the manifest and family weights given" },
            if self.plan.replay.is_empty() { String::new() } else { format!("; replay pool at {} of the draws", pct(self.plan.replay_share as f64)) },
        );

        let by_manifest = self.shares_by(&p, |i| self.manifests[self.origin[i]].as_str());
        out.push_str("\n| Manifest | Pool | Real items | Share of real draws | AI items | Share of AI draws |\n|---|---|---|---|---|---|\n");
        for (k, name) in self.manifests.iter().enumerate() {
            let (r, a) = (by_manifest[0].get(name.as_str()).copied().unwrap_or((0.0, 0)), by_manifest[1].get(name.as_str()).copied().unwrap_or((0.0, 0)));
            out.push_str(&format!("| {name} | {} | {} | {} | {} | {} |\n", if self.replay[k] { "replay" } else { "main" }, r.1, pct(r.0), a.1, pct(a.0)));
        }

        let mut table = |title: &str, shares: [BTreeMap<&str, (f64, usize)>; 2]| {
            for (label, class) in ["real", "AI"].into_iter().enumerate() {
                let mut rows: Vec<(&str, (f64, usize))> = shares[label].iter().map(|(k, v)| (*k, *v)).collect();
                rows.sort_by(|a, b| b.1 .0.total_cmp(&a.1 .0).then(a.0.cmp(b.0)));
                let shown = if top == 0 { rows.len() } else { top.min(rows.len()) };
                out.push_str(&format!("\n{title}, {class} class: {} in all{}\n\n| {title} | Items | Share of {class} draws | Lift |\n|---|---|---|---|\n", rows.len(), if shown < rows.len() { format!(", the {shown} largest shares shown") } else { String::new() }));
                for (name, (share, n)) in &rows[..shown] {
                    out.push_str(&format!("| {name} | {n} | {} | {:.2} |\n", pct(*share), share / (*n as f64 / class_items[label].max(1) as f64)));
                }
                if shown < rows.len() {
                    let rest = &rows[shown..];
                    out.push_str(&format!("| ({} others) | {} | {} | |\n", rest.len(), rest.iter().map(|r| r.1 .1).sum::<usize>(), pct(rest.iter().map(|r| r.1 .0).sum())));
                }
            }
        };
        // Without a family column or map, a family is a generator name and one table says it all.
        let named = (0..self.items.len()).any(|i| self.families[self.family[i]] != self.items[i].generator);
        if named {
            table("Generator family", self.shares_by(&p, |i| self.families[self.family[i]].as_str()));
        }
        table("Generator", self.shares_by(&p, |i| self.items[i].generator.as_str()));
        if !named {
            out.push_str("\nNo manifest has a generator_family column and no --family-map was given: each generator is its own family.\n");
        }
        out
    }

    /// Family weights that move each AI family's share of AI draws to
    /// `share^gamma`, renormalised: 1 changes nothing, 0 gives every family
    /// the same share, 0.5 is half way on a log scale. Real sources are left
    /// alone. Only for a plain recipe: weights multiply, so the shares of a
    /// recipe that already has weights or a replay pool would not land where
    /// this says.
    pub fn rebalance_families(&self, sampler: &Sampler, gamma: f64) -> Result<Vec<(String, f64)>> {
        anyhow::ensure!(self.is_plain(), "family weights are suggested from a recipe without weights and without a replay pool");
        anyhow::ensure!(gamma.is_finite() && gamma >= 0.0, "the exponent must be finite and not negative");
        let p = sampler.probabilities(self.items.len());
        let shares = self.shares_by(&p, |i| self.families[self.family[i]].as_str());
        let raw: Vec<(String, f64, f64)> = shares[1].iter().filter(|(_, v)| v.0 > 0.0).map(|(k, v)| (k.to_string(), v.0, v.0.powf(gamma - 1.0))).collect();
        // Scaled so that the weights average 1 over the draws: the numbers then read as "times as often".
        let mean: f64 = raw.iter().map(|(_, s, w)| s * w).sum();
        Ok(raw.into_iter().map(|(k, _, w)| (k, w / mean)).collect())
    }
}

/// A weights file ([`read_weights_file`]) holding these family weights.
pub fn weights_file_text(header: &str, families: &[(String, f64)]) -> String {
    let mut out: String = header.lines().map(|l| format!("# {l}\n")).collect();
    for (name, w) in families {
        out.push_str(&format!("family\t{name}\t{w:.6}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tintype-sampling-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// Two stores and an older one: `new` has 400 flux and 100 sdxl images
    /// and 300 reals, `social` has 200 reposted AI images with a family
    /// column and 100 reals, `old` has 300 GAN images and 200 reals.
    fn stores(dir: &Path) -> (Vec<PathBuf>, PathBuf) {
        let write = |store: &str, rows: &[(usize, u8, &str, &str)]| {
            let d = dir.join(store);
            std::fs::create_dir_all(&d).unwrap();
            let mut text = String::new();
            for (n, label, generator, family) in rows {
                for i in 0..*n {
                    text.push_str(&format!("{generator}-{i}.png\t{label}\t{generator}\t{store}"));
                    text.push_str(&if family.is_empty() { "\n".to_string() } else { format!("\t\t\t\t{family}\n") });
                }
            }
            std::fs::write(d.join("manifest.tsv"), text).unwrap();
            d.join("manifest.tsv")
        };
        let new = write("new", &[(400, 1, "flux", ""), (100, 1, "sdxl", ""), (300, 0, "real:cam", "")]);
        let social = write("social", &[(200, 1, "reddit-wild", "unknown"), (100, 0, "real:reddit", "")]);
        let old = write("old", &[(300, 1, "stylegan", ""), (200, 0, "real:ffhq", "")]);
        (vec![new, social], old)
    }

    fn share<'a>(r: &'a Recipe, s: &Sampler, label: u8, key: impl Fn(usize) -> &'a str, value: &str) -> f64 {
        let p = s.probabilities(r.items.len());
        (0..r.items.len()).filter(|&i| r.items[i].label == label && key(i) == value).map(|i| p[i]).sum()
    }

    #[test]
    fn a_plain_recipe_is_the_plain_sampler() {
        let dir = scratch("plain");
        let (manifests, _) = stores(&dir);
        let r = Recipe::load(&manifests, &Options::default(), 0.5).unwrap();
        assert!(r.is_plain() && r.plan.weights.is_empty() && r.plan.replay.is_empty());
        assert_eq!(r.manifests, vec!["new", "social"]);
        // A family map alone names families and changes no draw.
        let mapped = Recipe::load(&manifests, &Options { family_map: BTreeMap::from([("flux".to_string(), "latent-diffusion".to_string()), ("sdxl".to_string(), "latent-diffusion".to_string())]), ..Default::default() }, 0.5).unwrap();
        assert!(mapped.is_plain());
        let (a, b) = (r.sampler().unwrap(), mapped.sampler().unwrap());
        let (mut ra, mut rb, mut rc) = (crate::rng::Rng::new(5), crate::rng::Rng::new(5), crate::rng::Rng::new(5));
        let plain = Sampler::new(&r.items, 0.5);
        for k in 0..5000 {
            let x = plain.sample(k % 2, &mut rc);
            assert_eq!((a.sample(k % 2, &mut ra), b.sample(k % 2, &mut rb)), (x, x));
        }
        // count^0.5: flux 20, sdxl 10, reddit-wild sqrt(200).
        let total = 20.0 + 10.0 + 200f64.sqrt();
        assert!((share(&r, &a, 1, |i| r.items[i].generator.as_str(), "reddit-wild") - 200f64.sqrt() / total).abs() < 1e-12);
        // The column wins over the map, the map over the generator name.
        assert!((share(&mapped, &b, 1, |i| mapped.families[mapped.family[i]].as_str(), "latent-diffusion") - 30.0 / total).abs() < 1e-12);
        assert!((share(&mapped, &b, 1, |i| mapped.families[mapped.family[i]].as_str(), "unknown") - 200f64.sqrt() / total).abs() < 1e-12);
        assert!(mapped.families.contains(&"real:cam".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn manifest_and_family_weights_multiply_an_items_mass() {
        let dir = scratch("weights");
        let (manifests, _) = stores(&dir);
        let opts = Options { manifest_weights: vec![("social".into(), 3.0)], family_weights: vec![("sdxl".into(), 2.0), ("real:cam".into(), 0.5)], ..Default::default() };
        let r = Recipe::load(&manifests, &opts, 0.5).unwrap();
        let s = r.sampler().unwrap();
        // AI: flux 20, sdxl 10 x 2, reddit-wild sqrt(200) x 3.
        let total = 20.0 + 20.0 + 3.0 * 200f64.sqrt();
        assert!((share(&r, &s, 1, |i| r.items[i].generator.as_str(), "sdxl") - 20.0 / total).abs() < 1e-12);
        assert!((share(&r, &s, 1, |i| r.manifests[r.origin[i]].as_str(), "social") - 3.0 * 200f64.sqrt() / total).abs() < 1e-12);
        // Real: cam sqrt(300) x 0.5, reddit 10 x 3.
        let real = 0.5 * 300f64.sqrt() + 30.0;
        assert!((share(&r, &s, 0, |i| r.items[i].generator.as_str(), "real:reddit") - 30.0 / real).abs() < 1e-12);
        // The report shows the same numbers.
        let report = r.shares_report(&s, 0);
        assert!(report.contains(&format!("| social | main | 100 | {:.2}% | 200 | {:.2}% |", 100.0 * 30.0 / real, 300.0 * 200f64.sqrt() / total)), "{report}");
        assert!(report.contains("| sdxl | 100 |"), "{report}");
        // A manifest can be named by its path; a name that matches nothing is an error, as is a family no item has.
        let by_path = Options { manifest_weights: vec![(manifests[1].display().to_string(), 3.0)], ..Default::default() };
        assert_eq!(Recipe::load(&manifests, &by_path, 0.5).unwrap().plan.weights.iter().filter(|w| **w == 3.0).count(), 300);
        assert!(Recipe::load(&manifests, &Options { manifest_weights: vec![("socials".into(), 2.0)], ..Default::default() }, 0.5).is_err());
        assert!(Recipe::load(&manifests, &Options { family_weights: vec![("gan".into(), 2.0)], ..Default::default() }, 0.5).is_err());
        assert!(Recipe::load(&manifests, &Options { family_weights: vec![("sdxl".into(), 2.0), ("sdxl".into(), 3.0)], ..Default::default() }, 0.5).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_replay_pool_gets_its_share_and_the_validation_split_keeps_the_plan_aligned() {
        let dir = scratch("replay");
        let (manifests, old) = stores(&dir);
        let opts = Options { replay_manifests: vec![old.clone()], replay_share: 0.25, manifest_weights: vec![("social".into(), 2.0)], ..Default::default() };
        let all = Recipe::load(&manifests, &opts, 0.5).unwrap();
        assert_eq!((all.manifests.clone(), all.replay.clone()), (vec!["new".to_string(), "social".to_string(), "old".to_string()], vec![false, false, true]));
        let n = all.items.len();
        let (train, val) = all.split_validation(10);
        assert_eq!(train.items.len() + val.len(), n);
        assert!(!val.is_empty() && train.items.iter().all(|it| !is_val(it, 10)));
        assert_eq!((train.origin.len(), train.family.len(), train.plan.weights.len(), train.plan.replay.len()), (train.items.len(), train.items.len(), train.items.len(), train.items.len()));
        for i in 0..train.items.len() {
            assert_eq!(train.plan.replay[i], train.items[i].dataset == "old");
            assert_eq!(train.plan.weights[i], if train.items[i].dataset == "social" { 2.0 } else { 1.0 });
        }
        let s = train.sampler().unwrap();
        for label in [0u8, 1] {
            assert!((share(&train, &s, label, |i| train.manifests[train.origin[i]].as_str(), "old") - 0.25).abs() < 1e-6);
        }
        assert!(train.shares_report(&s, 3).contains("| old | replay |"));
        // The same manifest cannot be both.
        assert!(Recipe::load(&manifests, &Options { replay_manifests: vec![manifests[0].clone()], ..Default::default() }, 0.5).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rebalancing_moves_family_shares_to_the_stated_power() {
        let dir = scratch("rebalance");
        let (manifests, old) = stores(&dir);
        let all: Vec<PathBuf> = manifests.iter().cloned().chain([old]).collect();
        let map = BTreeMap::from([("flux".to_string(), "latent-diffusion".to_string()), ("sdxl".to_string(), "latent-diffusion".to_string()), ("stylegan".to_string(), "gan".to_string())]);
        let plain = Recipe::load(&all, &Options { family_map: map.clone(), ..Default::default() }, 0.5).unwrap();
        let before = plain.sampler().unwrap();
        let family = |r: &Recipe, s: &Sampler, f: &str| -> f64 {
            let p = s.probabilities(r.items.len());
            (0..r.items.len()).filter(|&i| r.items[i].label == 1 && r.families[r.family[i]] == f).map(|i| p[i]).sum()
        };
        for gamma in [0.0, 0.5, 1.0] {
            let weights = plain.rebalance_families(&before, gamma).unwrap();
            assert_eq!(weights.iter().map(|w| w.0.as_str()).collect::<Vec<_>>(), vec!["gan", "latent-diffusion", "unknown"]);
            // Written to a file and read back, the weights give the promised shares.
            let file = dir.join(format!("weights-{gamma}.tsv"));
            std::fs::write(&file, weights_file_text("made by a test\nsecond line", &weights)).unwrap();
            let (m, f) = read_weights_file(&file).unwrap();
            assert!(m.is_empty());
            let r = Recipe::load(&all, &Options { family_map: map.clone(), family_weights: f, ..Default::default() }, 0.5).unwrap();
            let after = r.sampler().unwrap();
            let want: Vec<f64> = ["gan", "latent-diffusion", "unknown"].iter().map(|f| family(&plain, &before, f).powf(gamma)).collect();
            for (k, f) in ["gan", "latent-diffusion", "unknown"].iter().enumerate() {
                assert!((family(&r, &after, f) - want[k] / want.iter().sum::<f64>()).abs() < 1e-5, "gamma {gamma}, {f}");
            }
            // Real sources keep their shares.
            let (pb, pa) = (before.probabilities(plain.items.len()), after.probabilities(r.items.len()));
            assert!((0..r.items.len()).filter(|&i| r.items[i].label == 0).all(|i| (pb[i] - pa[i]).abs() < 1e-12));
            // A recipe that already has weights is refused.
            assert!(r.rebalance_families(&after, gamma).is_err() || gamma == 1.0 && r.is_plain());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn weights_names_and_maps_parse() {
        assert_eq!(parse_weight("openfake-reddit=0.5").unwrap(), ("openfake-reddit".to_string(), 0.5));
        assert_eq!(parse_weight("data/a=b/manifest.tsv=2").unwrap(), ("data/a=b/manifest.tsv".to_string(), 2.0));
        assert!(parse_weight("gan").is_err() && parse_weight("gan=-1").is_err() && parse_weight("=1").is_err() && parse_weight("gan=nan").is_err());
        let names = manifest_names(&["data/store/openfake/manifest.tsv".into(), "data/vstore/k400/manifest.tsv".into(), "runs/replay.tsv".into(), "data/a/x/manifest.tsv".into(), "data/b/x/manifest.tsv".into()]);
        assert_eq!(names, vec!["openfake", "k400", "replay", "data/a/x/manifest.tsv", "data/b/x/manifest.tsv"]);
        let dir = scratch("parse");
        std::fs::write(dir.join("map.tsv"), "# generator\tfamily\tbasis\nflux\tlatent-diffusion\tname\n\nIP-Adapter FaceID\tlatent-diffusion\n").unwrap();
        let map = read_family_map(&dir.join("map.tsv")).unwrap();
        assert_eq!(map.get("IP-Adapter FaceID").map(String::as_str), Some("latent-diffusion"));
        assert_eq!(map.len(), 2);
        std::fs::write(dir.join("bad-map.tsv"), "flux\tlatent-diffusion\nflux\tgan\n").unwrap();
        assert!(read_family_map(&dir.join("bad-map.tsv")).is_err());
        std::fs::write(dir.join("w.tsv"), "# comment\nmanifest\topenfake-reddit\t0.5\nfamily\tgan\t2\n").unwrap();
        assert_eq!(read_weights_file(&dir.join("w.tsv")).unwrap(), (vec![("openfake-reddit".to_string(), 0.5)], vec![("gan".to_string(), 2.0)]));
        std::fs::write(dir.join("bad.tsv"), "generator\tgan\t2\n").unwrap();
        assert!(read_weights_file(&dir.join("bad.tsv")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
