//! A cache of compiled GPU networks on disk, so that the GPU engine starts
//! without reading the weights, building the graph or compiling it.
//!
//! Each entry is a directory of MPSGraph packages, one per batch size the
//! engine runs, plus a `ready` file naming those sizes, written last. An
//! entry is made under a temporary name and renamed into place, so a reader
//! sees a whole entry or none. Its name is a hash of everything the compiled
//! network depends on: this build of tintype, the macOS build, the GPU, the
//! weights file (path, size and modification time), the architecture, the
//! precision and the largest batch. Anything that changes gives a new entry,
//! and the least recently used entries are removed once the cache holds
//! more than [`KEEP`] packages (each is the weights in the working
//! precision and the compiled program, about 41 MB for the photo network).
//!
//! `TINTYPE_GPU_CACHE=0` turns the cache off; any other value is the
//! directory to use instead of `~/Library/Caches/tintype/gpu`. When the
//! cache cannot be read or written the engine is built as before.
#![cfg(target_os = "macos")]

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use sha2::{Digest, Sha256};

use crate::mps::{Executable, Precision};

/// Packages kept, over all entries; the most recently used entry is always
/// kept whole. A many-photo run of one network is an entry of three packages
/// (batches of 16, 32 and 64) and a single photo one of one. Eight holds
/// two networks' many-photo entries (the photo network and the second
/// opinion's, say, which would otherwise evict each other on every run) and
/// two single-photo ones, in about 330 MB.
pub const KEEP: usize = 8;

/// Entries in the middle of being written are left alone for this long.
const STALE_SECS: u64 = 3600;

/// The cache directory, or `None` when the cache is turned off.
pub fn root() -> Option<PathBuf> {
    match std::env::var_os("TINTYPE_GPU_CACHE") {
        Some(v) if v == "0" => None,
        Some(v) if !v.is_empty() => Some(PathBuf::from(v)),
        _ => Some(PathBuf::from(std::env::var_os("HOME").filter(|h| !h.is_empty())?).join("Library/Caches/tintype/gpu")),
    }
}

/// One cache entry: where it lives and the feed shape of each package.
pub struct Entry {
    dir: PathBuf,
    root: PathBuf,
    view: [isize; 3],
    prec: Precision,
}

/// What a compiled network depends on besides the weights file.
pub struct Build<'a> {
    pub arch: &'a str,
    pub img: usize,
    pub prefix: &'a str,
    pub prec: Precision,
    pub batch: usize,
    pub gpu: &'a str,
}

fn identity(path: &Path) -> Option<String> {
    let path = path.canonicalize().ok()?;
    let meta = std::fs::metadata(&path).ok()?;
    let mtime = meta.modified().ok()?.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_nanos();
    Some(format!("{}\n{}\n{mtime}", path.display(), meta.len()))
}

fn os_build() -> String {
    objc2_foundation::NSProcessInfo::processInfo().operatingSystemVersionString().to_string()
}

impl Entry {
    /// The entry for `weights` compiled as `build`, or `None` when there is
    /// no cache or the files involved cannot be identified.
    pub fn new(weights: &Path, build: &Build) -> Option<Self> {
        let root = root()?;
        let mut h = Sha256::new();
        for part in [
            "tintype gpu cache 1".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
            identity(&std::env::current_exe().ok()?)?,
            os_build(),
            build.gpu.to_string(),
            identity(weights)?,
            build.arch.to_string(),
            build.img.to_string(),
            build.prefix.to_string(),
            format!("{:?}", build.prec),
            build.batch.to_string(),
        ] {
            h.update(part.as_bytes());
            h.update([0u8]);
        }
        let name: String = h.finalize().iter().take(16).map(|b| format!("{b:02x}")).collect();
        let s = build.img as isize;
        Some(Self { dir: root.join(name), root, view: [3, s, s], prec: build.prec })
    }

    fn package(dir: &Path, batch: usize) -> PathBuf {
        dir.join(format!("b{batch}.mpsgraphpackage"))
    }

    /// The executables of a complete entry, smallest batch first.
    pub fn load(&self) -> Option<Vec<Executable>> {
        let ready = self.dir.join("ready");
        let sizes: Vec<usize> = std::fs::read_to_string(&ready).ok()?.split_whitespace().map(|s| s.parse().ok()).collect::<Option<_>>()?;
        if sizes.is_empty() || sizes.iter().any(|&b| !Self::package(&self.dir, b).is_dir()) {
            return None;
        }
        // Mark it used, for eviction.
        let _ = std::fs::File::options().append(true).open(&ready).and_then(|f| f.set_modified(SystemTime::now()));
        Some(
            sizes
                .into_iter()
                .map(|b| {
                    let shape = [b as isize, self.view[0], self.view[1], self.view[2]];
                    Executable::load(&Self::package(&self.dir, b), &shape, self.prec, 1)
                })
                .collect(),
        )
    }

    /// Write `exes` as this entry, then drop the least recently used
    /// entries beyond [`KEEP`]. Failing to write is not an error: the cache
    /// is only ever a shortcut.
    pub fn store(&self, exes: &[Executable]) {
        let tmp = self.root.join(format!(".tmp-{}-{}", std::process::id(), self.dir.file_name().unwrap().to_string_lossy()));
        let write = || -> std::io::Result<()> {
            std::fs::create_dir_all(&tmp)?;
            for e in exes {
                let p = Self::package(&tmp, e.batch());
                e.save(&p);
                if !p.is_dir() {
                    return Err(std::io::Error::other("package not written"));
                }
            }
            let sizes: Vec<String> = exes.iter().map(|e| e.batch().to_string()).collect();
            std::fs::write(tmp.join("ready"), sizes.join(" ") + "\n")?;
            std::fs::rename(&tmp, &self.dir)
        };
        if write().is_err() {
            let _ = std::fs::remove_dir_all(&tmp);
        }
        evict(&self.root, KEEP);
    }
}

/// Remove the least recently used entries in `root` once the entries
/// before them hold `keep` packages, and temporary entries left behind more
/// than an hour ago.
fn evict(root: &Path, keep: usize) {
    {
        let Ok(dir) = std::fs::read_dir(root) else { return };
        let now = SystemTime::now();
        let mut entries: Vec<(SystemTime, usize, PathBuf)> = Vec::new();
        for e in dir.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(".tmp-") {
                let old = e.metadata().and_then(|m| m.modified()).ok().and_then(|t| now.duration_since(t).ok()).is_some_and(|d| d.as_secs() > STALE_SECS);
                if old {
                    let _ = std::fs::remove_dir_all(&path);
                }
                continue;
            }
            if name.len() != 32 || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
                continue;
            }
            let used = std::fs::metadata(path.join("ready")).and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
            let packages = std::fs::read_to_string(path.join("ready")).map_or(1, |r| r.split_whitespace().count().max(1));
            entries.push((used, packages, path));
        }
        entries.sort_by_key(|e| std::cmp::Reverse(e.0));
        let mut held = 0;
        for (k, (_, packages, path)) in entries.into_iter().enumerate() {
            held += packages;
            if k > 0 && held > keep {
                let _ = std::fs::remove_dir_all(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eviction_keeps_the_most_recently_used_entries_within_the_package_budget() {
        let root = std::env::temp_dir().join(format!("tintype-gpu-cache-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let day = std::time::Duration::from_secs(86_400);
        let now = SystemTime::now();
        let name = |k: usize| format!("{k:032x}");
        // Newest first: three packages, one, two, one, one.
        for (k, sizes) in ["16 32 64", "6", "16 32", "12", "6"].iter().enumerate() {
            let dir = root.join(name(k));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("ready"), format!("{sizes}\n")).unwrap();
            std::fs::File::options().append(true).open(dir.join("ready")).unwrap().set_modified(now - day * k as u32).unwrap();
        }
        // Not entries: an unfinished one from a moment ago, and something else.
        std::fs::create_dir_all(root.join(".tmp-1-x")).unwrap();
        std::fs::create_dir_all(root.join("notes")).unwrap();
        evict(&root, 5);
        let mut left: Vec<String> = std::fs::read_dir(&root).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
        left.sort();
        // Four packages in the first two; the third would make six.
        assert_eq!(left, [".tmp-1-x".to_string(), name(0), name(1), "notes".to_string()]);
        // The newest entry stays even when it alone is over the budget.
        evict(&root, 2);
        let left = std::fs::read_dir(&root).unwrap().flatten().filter(|e| e.file_name().len() == 32).count();
        assert_eq!(left, 1);
        let _ = std::fs::remove_dir_all(&root);
    }
}
