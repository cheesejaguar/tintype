//! Group-aware splits: assign the items of one or more manifests to a
//! fitting, a calibration and a test set so that related items never end up
//! on both sides (`tintype split-groups`), and check sets for groups that
//! cross (`tintype verify-split`).
//!
//! Items are related when they share a grouping column
//! ([`crate::ingest::GROUP_COLUMNS`]): a derivative and its parent, two
//! renders of one prompt or scene, and, when asked, everything from one
//! source or one generator family. The path-hash split of
//! [`crate::data::is_val`] knows none of this and is left as it is; this
//! module is for the evaluation sets that decide what ships.
use crate::ingest::Item;
use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet};

/// The three sets, in the order their fractions are given.
pub const SETS: [&str; 3] = ["fit", "calibrate", "test"];

/// A grouping column that can keep items together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Parent,
    Scene,
    Source,
    Family,
}

impl Key {
    pub fn name(self) -> &'static str {
        ["parent", "scene", "source", "family"][self as usize]
    }

    /// Parse a comma-separated list such as `parent,scene`.
    pub fn parse_list(s: &str) -> Result<Vec<Key>> {
        let mut out = Vec::new();
        for tok in s.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            let k = match tok {
                "parent" => Key::Parent,
                "scene" => Key::Scene,
                "source" => Key::Source,
                "family" => Key::Family,
                other => bail!("unknown grouping key `{other}` (use parent, scene, source, family)"),
            };
            if !out.contains(&k) {
                out.push(k);
            }
        }
        Ok(out)
    }

    /// The value of this key for an item, if it has one. Parent and scene
    /// come only from their columns. Source and family fall back to what a
    /// four-column manifest already says: see [`source_of`], [`family_of`].
    pub fn of(self, it: &Item) -> Option<String> {
        match self {
            Key::Parent => it.parent_id().map(str::to_string),
            Key::Scene => it.scene_id().map(str::to_string),
            Key::Source => Some(source_of(it).to_string()),
            Key::Family => Some(family_of(it).to_string()),
        }
    }
}

/// Where an item came from: its `source_id` column, or else the real source
/// (`real:<name>`) of a real item and the dataset of a generated one.
pub fn source_of(it: &Item) -> &str {
    it.source_id().unwrap_or(if it.label == 0 { &it.generator } else { &it.dataset })
}

/// The generator family: the `generator_family` column, or else the
/// generator name (`real:<source>` for real items).
pub fn family_of(it: &Item) -> &str {
    it.generator_family().unwrap_or(&it.generator)
}

#[derive(Clone, Debug)]
pub struct Options {
    /// Target shares of fit, calibrate and test; they must sum to 1.
    pub fractions: [f64; 3],
    pub seed: u64,
    /// Keys whose groups must stay in one set.
    pub together: Vec<Key>,
    /// Sources and families that go to the test set whole.
    pub hold_out_sources: Vec<String>,
    pub hold_out_families: Vec<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self { fractions: [0.6, 0.2, 0.2], seed: 1, together: vec![Key::Parent, Key::Scene], hold_out_sources: Vec::new(), hold_out_families: Vec::new() }
    }
}

/// Parse `--fractions fit,calibrate,test`.
pub fn parse_fractions(s: &str) -> Result<[f64; 3]> {
    let v: Vec<f64> = s.split(',').map(|x| x.trim().parse::<f64>()).collect::<Result<_, _>>()?;
    anyhow::ensure!(v.len() == 3 && v.iter().all(|x| (0.0..=1.0).contains(x)) && (v.iter().sum::<f64>() - 1.0).abs() < 1e-6, "--fractions takes fit,calibrate,test shares that sum to 1, e.g. 0.6,0.2,0.2");
    Ok([v[0], v[1], v[2]])
}

struct UnionFind(Vec<usize>);

impl UnionFind {
    fn find(&mut self, mut x: usize) -> usize {
        while self.0[x] != x {
            self.0[x] = self.0[self.0[x]];
            x = self.0[x];
        }
        x
    }
    fn union(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));
        // The smaller index becomes the root, so a component's root is its first item.
        if a != b {
            self.0[a.max(b)] = a.min(b);
        }
    }
}

fn fnv(seed: u64, s: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64 ^ seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for b in s.bytes() {
        h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3);
    }
    // Finish with a mixer: FNV alone leaves the high bits of short keys correlated.
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^ (h >> 33)
}

/// Assign every item to a set (an index into [`SETS`]).
///
/// Items that share a value of any key in `opts.together` form a group, and
/// groups are closed over chains (a derivative of a scene's sibling belongs
/// with the whole scene). A group that contains a held-out source or family
/// goes to the test set. A group that contains an item of `pinned` (path to
/// set, from an earlier split) goes where that item went; one that touches
/// two sets is an error. Every other group is placed by a hash of its
/// smallest path, stratified so that each stratum is divided in about the
/// target shares: by label and generator, or by label alone when whole
/// sources or families are kept together (a stratum must hold many groups
/// to be divisible). A stratum of one group lands in a set with that set's
/// share as its probability.
pub fn assign(items: &[Item], opts: &Options, pinned: &BTreeMap<String, usize>) -> Result<Vec<usize>> {
    anyhow::ensure!((opts.fractions.iter().sum::<f64>() - 1.0).abs() < 1e-6, "the three shares must sum to 1");
    let mut uf = UnionFind((0..items.len()).collect());
    for &key in &opts.together {
        let mut first: BTreeMap<String, usize> = BTreeMap::new();
        for (i, it) in items.iter().enumerate() {
            if let Some(v) = key.of(it) {
                match first.get(&v) {
                    Some(&j) => uf.union(i, j),
                    None => {
                        first.insert(v, i);
                    }
                }
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..items.len() {
        groups.entry(uf.find(i)).or_default().push(i);
    }
    let path = |i: usize| items[i].path.to_string_lossy().into_owned();
    let mut out = vec![usize::MAX; items.len()];
    // Free groups by stratum: (hash, smallest path, members).
    let mut strata: BTreeMap<(u8, String), Vec<(u64, String, Vec<usize>)>> = BTreeMap::new();
    for members in groups.into_values() {
        let held_out = members.iter().any(|&i| opts.hold_out_sources.iter().any(|s| s == source_of(&items[i])) || opts.hold_out_families.iter().any(|f| f == family_of(&items[i])));
        let pins: BTreeSet<usize> = members.iter().filter_map(|&i| pinned.get(&path(i)).copied()).collect();
        let name = members.iter().map(|&i| path(i)).min().unwrap_or_default();
        let set = match (held_out, pins.len()) {
            (_, n) if n > 1 => bail!("the group of {name} was split across {} sets by the earlier split", n),
            (true, 1) if !pins.contains(&2) => bail!("the group of {name} holds a held-out source or family, but the earlier split put it in the {} set", SETS[*pins.first().unwrap()]),
            (true, _) => Some(2),
            (false, 1) => pins.first().copied(),
            (false, _) => None,
        };
        match set {
            Some(s) => members.iter().for_each(|&i| out[i] = s),
            None => {
                let rep = &items[*members.iter().min_by_key(|&&i| path(i)).unwrap()];
                let coarse = opts.together.iter().any(|k| matches!(k, Key::Source | Key::Family));
                strata.entry((rep.label, if coarse { String::new() } else { rep.generator.clone() })).or_default().push((fnv(opts.seed, &name), name, members));
            }
        }
    }
    let bounds = [opts.fractions[0], opts.fractions[0] + opts.fractions[1]];
    for mut list in strata.into_values() {
        list.sort();
        let total: usize = list.iter().map(|g| g.2.len()).sum();
        let mut before = 0usize;
        for (hash, _, members) in list {
            // A point inside this group's stretch of the stratum, chosen by its hash: large
            // strata divide in the target shares, and a lone group is placed at random.
            let u = (hash >> 11) as f64 / (1u64 << 53) as f64;
            let at = (before as f64 + members.len() as f64 * u) / total as f64;
            let set = if at < bounds[0] { 0 } else if at < bounds[1] { 1 } else { 2 };
            before += members.len();
            members.iter().for_each(|&i| out[i] = set);
        }
    }
    debug_assert!(out.iter().all(|&s| s < 3));
    Ok(out)
}

/// Every way the named sets leak into each other: a value of one of `keys`
/// present in more than one set, or the same path in two sets. One line per
/// problem; empty means the sets are disjoint on those keys.
pub fn crossings(sets: &[(String, Vec<Item>)], keys: &[Key]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut report = |what: &str, seen: BTreeMap<String, BTreeSet<&str>>| {
        for (value, names) in seen.into_iter().filter(|(_, n)| n.len() > 1) {
            problems.push(format!("{what} {value} is in {}", names.into_iter().collect::<Vec<_>>().join(" and ")));
        }
    };
    let mut paths: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
    for (name, items) in sets {
        for it in items {
            paths.entry(it.path.to_string_lossy().into_owned()).or_default().insert(name);
        }
    }
    report("path", paths);
    for &key in keys {
        let mut seen: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
        for (name, items) in sets {
            for it in items {
                if let Some(v) = key.of(it) {
                    seen.entry(v).or_default().insert(name);
                }
            }
        }
        report(key.name(), seen);
    }
    problems
}

/// Held-out sources and families found outside the test set: one line each.
pub fn hold_out_leaks(sets: &[(String, Vec<Item>)], sources: &[String], families: &[String]) -> Vec<String> {
    let mut problems = BTreeSet::new();
    for (name, items) in sets.iter().filter(|(n, _)| n != "test") {
        for it in items {
            if sources.iter().any(|s| s == source_of(it)) {
                problems.insert(format!("held-out source {} is in {name}", source_of(it)));
            }
            if families.iter().any(|f| f == family_of(it)) {
                problems.insert(format!("held-out family {} is in {name}", family_of(it)));
            }
        }
    }
    problems.into_iter().collect()
}

/// Files with identical bytes in more than one set, from `(set, sha256, path)`.
pub fn shared_bytes(rows: &[(String, String, String)]) -> Vec<String> {
    let mut seen: BTreeMap<&str, BTreeMap<&str, &str>> = BTreeMap::new();
    for (set, sha, path) in rows {
        seen.entry(sha).or_default().entry(set).or_insert(path);
    }
    seen.into_iter().filter(|(_, sets)| sets.len() > 1).map(|(sha, sets)| format!("identical bytes ({}…) in {}", &sha[..sha.len().min(12)], sets.iter().map(|(s, p)| format!("{s}: {p}")).collect::<Vec<_>>().join(" and "))).collect()
}

/// One line per set: how many items, how they divide by class, and how many
/// distinct sources and families they come from.
pub fn summary(items: &[Item], sets: &[usize]) -> Vec<String> {
    (0..3)
        .map(|s| {
            let mine: Vec<&Item> = items.iter().zip(sets).filter(|(_, &x)| x == s).map(|(it, _)| it).collect();
            let real = mine.iter().filter(|it| it.label == 0).count();
            let sources: BTreeSet<&str> = mine.iter().map(|it| source_of(it)).collect();
            let families: BTreeSet<&str> = mine.iter().filter(|it| it.label == 1).map(|it| family_of(it)).collect();
            format!("{:<10} {:>7} items ({:.1}%): {} real, {} AI; {} sources, {} generator families", SETS[s], mine.len(), 100.0 * mine.len() as f64 / items.len().max(1) as f64, real, mine.len() - real, sources.len(), families.len())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(path: &str, label: u8, gen: &str, groups: &[&str]) -> Item {
        Item { path: path.into(), label, generator: gen.into(), dataset: "d".into(), extra: groups.iter().map(|g| g.to_string()).collect() }
    }

    fn by_set(items: &[Item], sets: &[usize]) -> Vec<(String, Vec<Item>)> {
        (0..3).map(|s| (SETS[s].to_string(), items.iter().zip(sets).filter(|(_, &x)| x == s).map(|(it, _)| it.clone()).collect())).collect()
    }

    /// 600 parents, each with an original and two derivatives; every third
    /// parent shares a scene with the next one.
    fn family_tree() -> Vec<Item> {
        let mut items = Vec::new();
        for p in 0..600 {
            let (label, gen) = if p % 2 == 0 { (0, "real:cam") } else { (1, if p % 4 == 1 { "flux" } else { "sdxl" }) };
            let scene = if p % 3 == 0 { format!("s{p}") } else if p % 3 == 1 { format!("s{}", p - 1) } else { String::new() };
            for t in ["", "jpeg-q70", "resize-960"] {
                items.push(item(&format!("{t}/{p}.png"), label, gen, &[&format!("p{p}"), &scene, "", "", t]));
            }
        }
        items
    }

    #[test]
    fn groups_stay_together_and_shares_are_close() {
        let items = family_tree();
        let sets = assign(&items, &Options::default(), &BTreeMap::new()).unwrap();
        assert!(crossings(&by_set(&items, &sets), &[Key::Parent, Key::Scene]).is_empty());
        for (s, want) in [0.6, 0.2, 0.2].into_iter().enumerate() {
            let share = sets.iter().filter(|&&x| x == s).count() as f64 / items.len() as f64;
            assert!((share - want).abs() < 0.03, "{}: {share}", SETS[s]);
        }
        // Each class is divided too, not just the total.
        for label in [0u8, 1] {
            let n = items.iter().filter(|it| it.label == label).count() as f64;
            let test = items.iter().zip(&sets).filter(|(it, &s)| it.label == label && s == 2).count() as f64;
            assert!((test / n - 0.2).abs() < 0.04, "label {label}: {}", test / n);
        }
        // Deterministic, and a different seed gives a different split.
        assert_eq!(sets, assign(&items, &Options::default(), &BTreeMap::new()).unwrap());
        assert_ne!(sets, assign(&items, &Options { seed: 2, ..Default::default() }, &BTreeMap::new()).unwrap());
    }

    #[test]
    fn a_path_hash_split_of_the_same_items_leaks_and_is_caught() {
        let items = family_tree();
        // Derivatives placed without regard to their parents, as a path hash would.
        let sets: Vec<usize> = (0..items.len()).map(|i| i % 3).collect();
        let problems = crossings(&by_set(&items, &sets), &[Key::Parent, Key::Scene]);
        assert!(problems.iter().any(|p| p.starts_with("parent p0 is in")), "{problems:?}");
        assert!(problems.iter().any(|p| p.starts_with("scene s0 is in")), "{problems:?}");
        // The same file listed in two sets is a problem whatever the keys.
        let twice = vec![("fit".to_string(), vec![items[0].clone()]), ("test".to_string(), vec![items[0].clone()])];
        assert_eq!(crossings(&twice, &[]), vec!["path /0.png is in fit and test".to_string()]);
    }

    #[test]
    fn held_out_sources_and_families_go_to_test_whole() {
        let mut items = family_tree();
        // A scene shared by a held-out generator and another one: both go to test.
        items.push(item("x/a.png", 1, "newgen", &["xa", "shared"]));
        items.push(item("x/b.png", 1, "flux", &["xb", "shared"]));
        items.extend((0..40).map(|i| item(&format!("n/{i}.png"), 1, "newgen", &[])));
        items.extend((0..40).map(|i| item(&format!("r/{i}.jpg"), 0, "real:newcam", &[])));
        let opts = Options { hold_out_sources: vec!["real:newcam".into()], hold_out_families: vec!["newgen".into()], ..Default::default() };
        let sets = assign(&items, &opts, &BTreeMap::new()).unwrap();
        for (it, &s) in items.iter().zip(&sets) {
            if it.generator == "newgen" || it.generator == "real:newcam" || it.scene_id() == Some("shared") {
                assert_eq!(s, 2, "{}", it.path.display());
            }
        }
        let named = by_set(&items, &sets);
        assert!(hold_out_leaks(&named, &opts.hold_out_sources, &opts.hold_out_families).is_empty());
        assert!(crossings(&named, &[Key::Parent, Key::Scene]).is_empty());
        // Moved by hand into fit, the held-out family is reported.
        let mut leaky = named.clone();
        let moved = leaky[2].1.iter().position(|it| it.generator == "newgen").unwrap();
        let it = leaky[2].1.remove(moved);
        leaky[0].1.push(it);
        assert_eq!(hold_out_leaks(&leaky, &opts.hold_out_sources, &opts.hold_out_families), vec!["held-out family newgen is in fit".to_string()]);
    }

    #[test]
    fn whole_sources_can_be_kept_together() {
        let mut items = Vec::new();
        for s in 0..30 {
            items.extend((0..20).map(|i| item(&format!("src{s}/{i}.jpg"), 0, &format!("real:src{s}"), &[])));
        }
        let opts = Options { together: vec![Key::Source], ..Default::default() };
        let sets = assign(&items, &opts, &BTreeMap::new()).unwrap();
        let named = by_set(&items, &sets);
        assert!(crossings(&named, &[Key::Source]).is_empty());
        assert!(named.iter().all(|(_, v)| !v.is_empty()));
        // Without it, a four-column manifest is split item by item and sources cross.
        let loose = assign(&items, &Options::default(), &BTreeMap::new()).unwrap();
        assert!(!crossings(&by_set(&items, &loose), &[Key::Source]).is_empty());
    }

    #[test]
    fn single_item_generators_are_spread_over_the_sets() {
        // 900 generators with one image each: every stratum is one group.
        let items: Vec<Item> = (0..900).map(|i| item(&format!("g{i}.png"), 1, &format!("gen{i}"), &[])).collect();
        let sets = assign(&items, &Options::default(), &BTreeMap::new()).unwrap();
        for (s, want) in [0.6, 0.2, 0.2].into_iter().enumerate() {
            let share = sets.iter().filter(|&&x| x == s).count() as f64 / 900.0;
            assert!((share - want).abs() < 0.05, "{}: {share}", SETS[s]);
        }
    }

    #[test]
    fn later_derivatives_follow_an_earlier_split() {
        let items = family_tree();
        let (originals, derived): (Vec<Item>, Vec<Item>) = items.into_iter().partition(|it| it.transform().is_none());
        let first = assign(&originals, &Options::default(), &BTreeMap::new()).unwrap();
        let pinned: BTreeMap<String, usize> = originals.iter().zip(&first).map(|(it, &s)| (it.path.to_string_lossy().into_owned(), s)).collect();
        // Split again with the derivatives added, under another seed: the originals stay put.
        let all: Vec<Item> = originals.iter().chain(&derived).cloned().collect();
        let second = assign(&all, &Options { seed: 99, ..Default::default() }, &pinned).unwrap();
        assert_eq!(&second[..originals.len()], &first[..]);
        assert!(crossings(&by_set(&all, &second), &[Key::Parent, Key::Scene]).is_empty());
        // An earlier split that separated a parent from its scene sibling cannot be followed.
        let mut bad = pinned.clone();
        bad.insert("/0.png".into(), 0);
        bad.insert("/1.png".into(), 2);
        assert!(assign(&all, &Options::default(), &bad).is_err());
    }

    #[test]
    fn identical_bytes_across_sets_are_listed() {
        let rows = vec![
            ("fit".to_string(), "aa".repeat(32), "a.png".to_string()),
            ("fit".to_string(), "aa".repeat(32), "copy-of-a.png".to_string()),
            ("test".to_string(), "bb".repeat(32), "b.png".to_string()),
            ("calibrate".to_string(), "bb".repeat(32), "also-b.png".to_string()),
        ];
        let p = shared_bytes(&rows);
        assert_eq!(p.len(), 1, "{p:?}");
        assert!(p[0].contains("calibrate: also-b.png") && p[0].contains("test: b.png"), "{p:?}");
    }

    #[test]
    fn keys_and_fractions_parse() {
        assert_eq!(Key::parse_list("parent, scene,parent").unwrap(), vec![Key::Parent, Key::Scene]);
        assert!(Key::parse_list("prompt").is_err());
        assert_eq!(parse_fractions("0.5,0.25,0.25").unwrap(), [0.5, 0.25, 0.25]);
        assert!(parse_fractions("0.5,0.5").is_err() && parse_fractions("0.6,0.3,0.3").is_err());
        let real = item("a.jpg", 0, "real:cam", &[]);
        let fake = item("b.png", 1, "flux", &["", "", "reddit", "diffusion"]);
        assert_eq!((source_of(&real), family_of(&real)), ("real:cam", "real:cam"));
        assert_eq!((source_of(&fake), family_of(&fake)), ("reddit", "diffusion"));
        assert_eq!(source_of(&item("c.png", 1, "flux", &[])), "d");
    }
}
