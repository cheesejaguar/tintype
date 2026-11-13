//! SHA-256 digests that tie results to exact bytes: of downloaded files, of
//! the released model files, and of every file of a prepared evaluation set
//! (a "frozen" manifest, written by `tintype freeze-set` and checked by
//! `tintype verify-set`).
use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};

/// Lowercase hexadecimal.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// SHA-256 of a file's contents, read in 1 MiB pieces.
pub fn sha256_file(path: &Path) -> Result<String> {
    let mut f = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf).with_context(|| format!("reading {}", path.display()))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex(&h.finalize()))
}

/// The released model files: (path in the repository, size in bytes,
/// SHA-256). `docs/DATA.md` lists the same values; `tintype verify-models`
/// and the `models` integration test check the files against them. A
/// retrained or recalibrated model changes these on purpose.
pub const MODEL_FILES: [(&str, u64, &str); 4] = [
    ("models/tintype.safetensors", 86_678_036, "9dcb21dc9800591ffd758a3c7ca6aa65fa5a9f09a223fecc952f34294ae9aeba"),
    ("models/tintype.json", 3_450, "bfe1366bcfec28d15fb9116cb96c721de52de91d6fa4d98b6da4ec7fe6e62709"),
    ("models/tintype-video.safetensors", 86_678_036, "77010f7cd733b23a73f269e275dd9c974f266b32e3c38e0e4d4a12e1723c4881"),
    ("models/tintype-video.json", 2_161, "673e9040d941125691bfb80cbcf130e8efe2fdbf26a08f0b947450c4abca4e9c"),
];

/// Check every released model file under `root` (the repository) against
/// [`MODEL_FILES`]; returns one line per file and whether all matched.
pub fn check_models(root: &Path) -> (Vec<String>, bool) {
    let mut ok = true;
    let lines = MODEL_FILES
        .iter()
        .map(|&(rel, size, want)| {
            let p = root.join(rel);
            match sha256_file(&p) {
                Ok(got) if got == want => format!("ok        {want}  {rel}"),
                Ok(got) => {
                    ok = false;
                    let len = p.metadata().map(|m| m.len()).unwrap_or(0);
                    format!("MISMATCH  {got}  {rel} ({len} bytes; expected {want}, {size} bytes)")
                }
                Err(e) => {
                    ok = false;
                    format!("MISSING   {rel}: {e:#}")
                }
            }
        })
        .collect();
    (lines, ok)
}

/// One file of a frozen evaluation set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrozenRow {
    pub sha256: String,
    pub size: u64,
    pub label: u8,
    pub generator: String,
    pub dataset: String,
    /// As the manifest that listed it resolves it, relative to the directory
    /// the set was frozen from (normally the one holding `data/`).
    pub path: String,
}

/// A prepared evaluation set pinned to its bytes: every manifest it was made
/// from (with that manifest's own SHA-256) and every file those list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frozen {
    pub manifests: Vec<(String, String)>,
    pub rows: Vec<FrozenRow>,
}

const HEADER: &str = "# tintype frozen evaluation set, format 1";
const COLUMNS: &str = "# sha256\tsize\tlabel\tgenerator\tdataset\tpath";

/// A path as stored in a frozen manifest: relative to `root`, with forward slashes.
fn portable(p: &Path, root: &Path) -> Result<String> {
    let (abs, root) = (std::path::absolute(p)?, std::path::absolute(root)?);
    let rel = abs.strip_prefix(&root).with_context(|| format!("{} is outside {}; freeze from the directory that holds the data", p.display(), root.display()))?;
    let parts: Vec<String> = rel.components().filter(|c| !matches!(c, std::path::Component::CurDir)).map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    Ok(parts.join("/"))
}

fn reader(path: &Path) -> Result<Box<dyn BufRead>> {
    let f = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    Ok(if path.extension().is_some_and(|e| e == "gz") { Box::new(std::io::BufReader::new(flate2::read::GzDecoder::new(f))) } else { Box::new(std::io::BufReader::new(f)) })
}

impl Frozen {
    /// Hash every file the manifests list (in parallel). Paths are stored
    /// relative to `root`, normally the directory that holds `data/`.
    pub fn build(manifests: &[PathBuf], root: &Path) -> Result<Self> {
        let mut out = Frozen::default();
        let mut items = Vec::new();
        for m in manifests {
            let list = crate::ingest::read_manifest(m)?;
            out.manifests.push((portable(m, root)?, sha256_file(m)?));
            items.extend(list);
        }
        out.rows = items
            .par_iter()
            .map(|it| {
                let size = it.path.metadata().with_context(|| format!("{} (listed in a manifest)", it.path.display()))?.len();
                Ok(FrozenRow { sha256: sha256_file(&it.path)?, size, label: it.label, generator: it.generator.clone(), dataset: it.dataset.clone(), path: portable(&it.path, root)? })
            })
            .collect::<Result<_>>()?;
        Ok(out)
    }

    /// Write as text, gzip-compressed when the name ends in `.gz`.
    pub fn write(&self, path: &Path) -> Result<()> {
        let mut text = format!("{HEADER}\n");
        for (m, h) in &self.manifests {
            text.push_str(&format!("# manifest\t{m}\t{h}\n"));
        }
        text.push_str(COLUMNS);
        text.push('\n');
        for r in &self.rows {
            for field in [&r.generator, &r.dataset, &r.path] {
                if field.contains(['\t', '\n']) {
                    bail!("a tab or newline in {field:?} cannot be written to a frozen manifest");
                }
            }
            text.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\n", r.sha256, r.size, r.label, r.generator, r.dataset, r.path));
        }
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        if path.extension().is_some_and(|e| e == "gz") {
            let mut z = flate2::write::GzEncoder::new(std::fs::File::create(path)?, flate2::Compression::best());
            z.write_all(text.as_bytes())?;
            z.finish()?;
        } else {
            std::fs::write(path, text)?;
        }
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Self> {
        let mut out = Frozen::default();
        let mut lines = reader(path)?.lines();
        match lines.next() {
            Some(Ok(l)) if l == HEADER => {}
            _ => bail!("{} is not a frozen evaluation set (no `{HEADER}` line)", path.display()),
        }
        for (i, l) in lines.enumerate() {
            let l = l?;
            if let Some(rest) = l.strip_prefix("# manifest\t") {
                let (m, h) = rest.split_once('\t').with_context(|| format!("{}:{}: bad manifest line", path.display(), i + 2))?;
                out.manifests.push((m.to_string(), h.to_string()));
                continue;
            }
            if l.starts_with('#') || l.is_empty() {
                continue;
            }
            let c: Vec<&str> = l.split('\t').collect();
            let ok = c.len() == 6 && c[0].len() == 64 && c[0].bytes().all(|b| b.is_ascii_hexdigit());
            if !ok {
                bail!("{}:{}: expected sha256, size, label, generator, dataset and path", path.display(), i + 2);
            }
            out.rows.push(FrozenRow {
                sha256: c[0].to_string(),
                size: c[1].parse().with_context(|| format!("{}:{}: size", path.display(), i + 2))?,
                label: c[2].parse().with_context(|| format!("{}:{}: label", path.display(), i + 2))?,
                generator: c[3].to_string(),
                dataset: c[4].to_string(),
                path: c[5].to_string(),
            });
        }
        Ok(out)
    }

    /// Compare the files under `root` with this record. Returns one line per
    /// problem: a missing or changed file, a manifest that changed, or a
    /// manifest row that differs from the frozen one. Empty means identical.
    pub fn verify(&self, root: &Path) -> Vec<String> {
        let mut problems: Vec<String> = self
            .rows
            .par_iter()
            .filter_map(|r| {
                let p = root.join(&r.path);
                match p.metadata() {
                    Err(_) => Some(format!("missing   {}", r.path)),
                    Ok(m) if m.len() != r.size => Some(format!("changed   {} ({} bytes, frozen with {})", r.path, m.len(), r.size)),
                    Ok(_) => match sha256_file(&p) {
                        Ok(h) if h == r.sha256 => None,
                        Ok(h) => Some(format!("changed   {} (sha256 {h}, frozen with {})", r.path, r.sha256)),
                        Err(e) => Some(format!("unreadable {}: {e:#}", r.path)),
                    },
                }
            })
            .collect();
        // The manifests carry the labels: any change to one is a problem, and
        // the rows say which.
        let frozen: BTreeMap<&str, &FrozenRow> = self.rows.iter().map(|r| (r.path.as_str(), r)).collect();
        let mut listed: BTreeMap<String, crate::ingest::Item> = BTreeMap::new();
        let mut changed = false;
        for (m, want) in &self.manifests {
            let p = root.join(m);
            match sha256_file(&p) {
                Ok(h) if &h == want => {}
                Ok(_) => {
                    changed = true;
                    problems.push(format!("manifest  {m} differs from the frozen one"));
                }
                Err(_) => {
                    problems.push(format!("missing   manifest {m}"));
                    continue;
                }
            }
            if let Ok(items) = crate::ingest::read_manifest(&p) {
                for it in items {
                    let rel = portable(&it.path, root).unwrap_or_else(|_| it.path.to_string_lossy().into_owned());
                    listed.insert(rel, it);
                }
            }
        }
        if changed {
            for (path, it) in &listed {
                match frozen.get(path.as_str()) {
                    None => problems.push(format!("added     {path} (in a manifest, not in the frozen set)")),
                    Some(r) if (r.label, r.generator.as_str(), r.dataset.as_str()) != (it.label, it.generator.as_str(), it.dataset.as_str()) => {
                        problems.push(format!("relabelled {path}: {} {} {} -> {} {} {}", r.label, r.generator, r.dataset, it.label, it.generator, it.dataset))
                    }
                    Some(_) => {}
                }
            }
            for path in frozen.keys().filter(|p| !listed.contains_key(**p)) {
                problems.push(format!("dropped   {path} (in the frozen set, no longer in its manifest)"));
            }
        }
        problems.sort();
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_digests() {
        assert_eq!(sha256_bytes(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(sha256_bytes(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tintype-integrity-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("set")).unwrap();
        d
    }

    #[test]
    fn a_frozen_set_catches_every_kind_of_change() {
        let d = scratch("frozen");
        std::fs::write(d.join("set/a.png"), b"first image").unwrap();
        std::fs::write(d.join("set/b.jpg"), b"second image").unwrap();
        std::fs::write(d.join("set/manifest.tsv"), "a.png\t1\tgen-x\tdemo\nb.jpg\t0\treal:demo\tdemo\n").unwrap();
        let f = Frozen::build(&[d.join("set/manifest.tsv")], &d).unwrap();
        assert_eq!(f.manifests[0].0, "set/manifest.tsv");
        assert_eq!((f.rows.len(), f.rows[0].path.as_str()), (2, "set/a.png"));
        assert_eq!(f.rows[0].sha256, sha256_bytes(b"first image"));

        for name in ["frozen.tsv", "frozen.tsv.gz"] {
            f.write(&d.join(name)).unwrap();
            assert_eq!(Frozen::read(&d.join(name)).unwrap(), f);
        }
        assert!(f.verify(&d).is_empty());

        // Same size, different bytes.
        std::fs::write(d.join("set/a.png"), b"FIRST image").unwrap();
        let p = f.verify(&d);
        assert!(p.len() == 1 && p[0].starts_with("changed") && p[0].contains("set/a.png"), "{p:?}");
        std::fs::write(d.join("set/a.png"), b"first image").unwrap();

        // A missing file.
        std::fs::remove_file(d.join("set/b.jpg")).unwrap();
        assert!(f.verify(&d).iter().any(|l| l.starts_with("missing") && l.contains("set/b.jpg")));
        std::fs::write(d.join("set/b.jpg"), b"second image").unwrap();

        // A relabelled row and an added one.
        std::fs::write(d.join("set/c.png"), b"third").unwrap();
        std::fs::write(d.join("set/manifest.tsv"), "a.png\t0\tgen-x\tdemo\nb.jpg\t0\treal:demo\tdemo\nc.png\t1\tgen-y\tdemo\n").unwrap();
        let p = f.verify(&d);
        assert!(p.iter().any(|l| l.starts_with("manifest")), "{p:?}");
        assert!(p.iter().any(|l| l.starts_with("relabelled set/a.png")), "{p:?}");
        assert!(p.iter().any(|l| l.starts_with("added") && l.contains("set/c.png")), "{p:?}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn rejects_files_that_are_not_frozen_sets() {
        let d = scratch("reject");
        std::fs::write(d.join("x.tsv"), "a.png\t1\tg\td\n").unwrap();
        assert!(Frozen::read(&d.join("x.tsv")).is_err());
        std::fs::write(d.join("y.tsv"), format!("{HEADER}\nnot-a-hash\t1\t1\tg\td\ta.png\n")).unwrap();
        assert!(Frozen::read(&d.join("y.tsv")).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }
}
