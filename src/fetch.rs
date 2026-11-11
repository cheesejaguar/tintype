//! Dataset download and unpacking: Hugging Face and plain-URL downloads with
//! parallel ranged requests, and selective extraction from zip / tar archives.
use anyhow::{anyhow, bail, Context, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct RemoteFile {
    pub path: String,
    pub size: u64,
}

const UA: &str = "veracity-fetch";

/// `*`, `?` and `[a-z0-9]` classes; everything else is literal.
pub fn glob_match(pat: &str, s: &str) -> bool {
    fn go(p: &[u8], s: &[u8]) -> bool {
        match p.first() {
            None => s.is_empty(),
            Some(b'*') => (0..=s.len()).any(|i| go(&p[1..], &s[i..])),
            Some(b'?') => !s.is_empty() && go(&p[1..], &s[1..]),
            Some(b'[') => {
                let Some(end) = p.iter().position(|&c| c == b']') else { return false };
                let Some(&c) = s.first() else { return false };
                let class = &p[1..end];
                let mut ok = false;
                let mut i = 0;
                while i < class.len() {
                    if i + 2 < class.len() && class[i + 1] == b'-' {
                        ok |= class[i] <= c && c <= class[i + 2];
                        i += 3;
                    } else {
                        ok |= class[i] == c;
                        i += 1;
                    }
                }
                ok && go(&p[end + 1..], &s[1..])
            }
            Some(&c) => s.first() == Some(&c) && go(&p[1..], &s[1..]),
        }
    }
    go(pat.as_bytes(), s.as_bytes())
}

/// Every file of a Hugging Face dataset repository (paginated tree API).
pub fn hf_list(repo: &str) -> Result<Vec<RemoteFile>> {
    let mut url = format!("https://huggingface.co/api/datasets/{repo}/tree/main?recursive=true&limit=1000");
    let mut out = Vec::new();
    loop {
        let mut resp = ureq::get(&url).header("User-Agent", UA).call().with_context(|| format!("listing {repo}"))?;
        let next = resp
            .headers()
            .get("link")
            .and_then(|v| v.to_str().ok())
            .and_then(|l| l.split(',').find(|p| p.contains("rel=\"next\"")).and_then(|p| Some(p.split('<').nth(1)?.split('>').next()?.to_string())));
        let body = resp.body_mut().with_config().limit(256 << 20).read_to_string()?;
        let v: serde_json::Value = serde_json::from_str(&body)?;
        for e in v.as_array().ok_or_else(|| anyhow!("unexpected listing for {repo}"))? {
            if e["type"] == "file" {
                let size = e["lfs"]["size"].as_u64().or_else(|| e["size"].as_u64()).unwrap_or(0);
                out.push(RemoteFile { path: e["path"].as_str().unwrap_or_default().to_string(), size });
            }
        }
        match next {
            Some(n) => url = n,
            None => break,
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn encode_path(p: &str) -> String {
    p.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub fn hf_url(repo: &str, path: &str) -> String {
    format!("https://huggingface.co/datasets/{repo}/resolve/main/{}", encode_path(path))
}

pub fn modelscope_url(repo: &str, path: &str) -> String {
    format!("https://modelscope.cn/datasets/{repo}/resolve/master/{}", encode_path(path))
}

fn remote_size(url: &str) -> Result<u64> {
    let resp = ureq::get(url).header("User-Agent", UA).header("Range", "bytes=0-0").call()?;
    resp.headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.rsplit('/').next()?.parse().ok())
        .ok_or_else(|| anyhow!("server gave no size for {url}"))
}

fn fetch_range(url: &str, file: &File, start: u64, end: u64) -> Result<()> {
    use std::os::unix::fs::FileExt;
    let mut last = None;
    for attempt in 0..6 {
        let mut pos = start;
        let r = (|| -> Result<()> {
            let resp = ureq::get(url).header("User-Agent", UA).header("Range", format!("bytes={start}-{end}")).call()?;
            let mut reader = resp.into_body().into_reader();
            let mut buf = vec![0u8; 1 << 20];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                file.write_all_at(&buf[..n], pos)?;
                pos += n as u64;
            }
            if pos != end + 1 {
                bail!("short read: {} of {} bytes", pos - start, end + 1 - start);
            }
            Ok(())
        })();
        match r {
            Ok(()) => return Ok(()),
            Err(e) => {
                last = Some(e);
                std::thread::sleep(std::time::Duration::from_secs(2 << attempt.min(4)));
            }
        }
    }
    Err(last.unwrap())
}

/// Download `url` to `dest` using `conns` parallel ranged requests. A file
/// that already has the right size is left alone.
pub fn download(url: &str, dest: &Path, size: Option<u64>, conns: usize) -> Result<()> {
    let size = match size {
        Some(s) if s > 0 => s,
        _ => remote_size(url)?,
    };
    if dest.metadata().map(|m| m.len() == size).unwrap_or(false) {
        return Ok(());
    }
    if let Some(p) = dest.parent() {
        std::fs::create_dir_all(p)?;
    }
    let tmp = dest.with_extension("part");
    let file = File::create(&tmp)?;
    file.set_len(size)?;
    let parts = conns.clamp(1, 16).min((size / (8 << 20)).max(1) as usize);
    let chunk = size.div_ceil(parts as u64);
    let errs: Vec<anyhow::Error> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..parts as u64)
            .map(|i| {
                let (file, url) = (&file, url);
                s.spawn(move || fetch_range(url, file, i * chunk, ((i + 1) * chunk).min(size) - 1))
            })
            .collect();
        hs.into_iter().filter_map(|h| h.join().ok().and_then(|r| r.err())).collect()
    });
    if let Some(e) = errs.into_iter().next() {
        return Err(e.context(format!("downloading {url}")));
    }
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

/// Download every file of `repo` matching one of `includes` into `out`,
/// keeping repository paths. Returns the local paths.
pub fn hf_download(repo: &str, includes: &[&str], out: &Path, conns: usize) -> Result<Vec<PathBuf>> {
    let files: Vec<RemoteFile> = hf_list(repo)?.into_iter().filter(|f| includes.iter().any(|p| glob_match(p, &f.path))).collect();
    if files.is_empty() {
        bail!("nothing in {repo} matches {includes:?}");
    }
    hf_download_files(repo, &files, out, conns)
}

pub fn hf_download_files(repo: &str, files: &[RemoteFile], out: &Path, conns: usize) -> Result<Vec<PathBuf>> {
    use rayon::prelude::*;
    // Many small files: several at once, one connection each. Few large files: one at a time, many connections.
    let small = files.iter().all(|f| f.size < (64 << 20));
    let one = |f: &RemoteFile, c: usize| -> Result<PathBuf> {
        let dest = out.join(&f.path);
        download(&hf_url(repo, &f.path), &dest, Some(f.size), c)?;
        Ok(dest)
    };
    if small {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(conns.clamp(1, 16)).build()?;
        pool.install(|| files.par_iter().map(|f| one(f, 1)).collect())
    } else {
        files.iter().map(|f| one(f, conns)).collect()
    }
}

/// A remote file read through HTTP range requests, so that a zip archive can
/// be listed and selected members extracted without downloading all of it.
/// The tail (where a zip keeps its directory) is fetched once and kept.
pub struct RemoteFileReader {
    url: String,
    size: u64,
    pos: u64,
    tail_start: u64,
    tail: Vec<u8>,
    window_start: u64,
    window: Vec<u8>,
}

impl RemoteFileReader {
    const TAIL: u64 = 8 << 20;
    const AHEAD: u64 = 1 << 20;

    pub fn open(url: &str) -> Result<Self> {
        let size = remote_size(url)?;
        let tail_start = size.saturating_sub(Self::TAIL);
        let tail = Self::range(url, tail_start, size - 1)?;
        Ok(Self { url: url.to_string(), size, pos: 0, tail_start, tail, window_start: 0, window: Vec::new() })
    }

    fn range(url: &str, start: u64, end: u64) -> Result<Vec<u8>> {
        let mut last = None;
        for attempt in 0..10u64 {
            let r = (|| -> Result<Vec<u8>> {
                let resp = ureq::get(url).header("User-Agent", UA).header("Range", format!("bytes={start}-{end}")).call()?;
                let mut v = Vec::with_capacity((end + 1 - start) as usize);
                resp.into_body().into_reader().read_to_end(&mut v)?;
                if v.len() as u64 != end + 1 - start {
                    bail!("short read: {} of {} bytes", v.len(), end + 1 - start);
                }
                Ok(v)
            })();
            match r {
                Ok(v) => return Ok(v),
                Err(e) => {
                    // A rate limit (HTTP 429) needs a real pause; anything else a short one.
                    let limited = e.to_string().contains("429");
                    last = Some(e);
                    std::thread::sleep(if limited { std::time::Duration::from_secs(40 * (attempt + 1)) } else { std::time::Duration::from_millis(500 << attempt) });
                }
            }
        }
        Err(last.unwrap())
    }
}

impl Read for RemoteFileReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.size || out.is_empty() {
            return Ok(0);
        }
        let (src, base) = if self.pos >= self.tail_start {
            (&self.tail, self.tail_start)
        } else {
            if self.pos < self.window_start || self.pos >= self.window_start + self.window.len() as u64 {
                let end = (self.pos + (out.len() as u64).max(Self::AHEAD)).min(self.tail_start) - 1;
                self.window = Self::range(&self.url, self.pos, end).map_err(std::io::Error::other)?;
                self.window_start = self.pos;
            }
            (&self.window, self.window_start)
        };
        let at = (self.pos - base) as usize;
        let n = out.len().min(src.len() - at);
        out[..n].copy_from_slice(&src[at..at + n]);
        self.pos += n as u64;
        Ok(n)
    }
}

impl std::io::Seek for RemoteFileReader {
    fn seek(&mut self, to: std::io::SeekFrom) -> std::io::Result<u64> {
        let p = match to {
            std::io::SeekFrom::Start(p) => p as i128,
            std::io::SeekFrom::End(d) => self.size as i128 + d as i128,
            std::io::SeekFrom::Current(d) => self.pos as i128 + d as i128,
        };
        if p < 0 {
            return Err(std::io::Error::other("seek before the start"));
        }
        self.pos = p as u64;
        Ok(self.pos)
    }
}

/// Member names of a remote zip, read from its directory alone.
pub fn list_remote_zip(url: &str) -> Result<Vec<String>> {
    Ok(zip::ZipArchive::new(RemoteFileReader::open(url)?)?.file_names().filter_map(|s| s.ok().map(|c| c.to_string())).collect())
}

/// Extract the selected members of a remote zip into `out` without
/// downloading the rest of it; returns their names.
pub fn unpack_remote_zip(url: &str, out: &Path, sel: &Select, conns: usize) -> Result<Vec<String>> {
    use rayon::prelude::*;
    std::fs::create_dir_all(out)?;
    let names: Vec<String> = zip::ZipArchive::new(RemoteFileReader::open(url)?)?.file_names().filter_map(|s| s.ok().map(|c| c.to_string())).collect();
    let picked = choose(names, sel);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(conns.clamp(1, 16)).build()?;
    // Each worker reads through its own connection and its own copy of the directory.
    pool.install(|| {
        picked.par_chunks(picked.len().div_ceil(conns.clamp(1, 16)).max(1)).try_for_each(|part| -> Result<()> {
            let mut zip = zip::ZipArchive::new(RemoteFileReader::open(url)?)?;
            for n in part {
                let Some(dest) = target(out, n, sel.flatten) else { continue };
                if let Some(p) = dest.parent() {
                    std::fs::create_dir_all(p)?;
                }
                let mut f = zip.by_name(n)?;
                std::io::copy(&mut f, &mut File::create(&dest)?)?;
            }
            Ok(())
        })
    })?;
    Ok(picked)
}

/// Which archive members to extract.
#[derive(Clone, Debug, Default)]
pub struct Select {
    /// Keep members whose path contains this (empty = any).
    pub contains: String,
    /// Keep members with one of these lowercase extensions (empty = any).
    pub exts: Vec<&'static str>,
    /// Skip members whose full path is in this list.
    pub exclude: Vec<String>,
    /// When not empty, keep only members whose full path is in this list.
    pub only: Vec<String>,
    /// Take this many, evenly spaced over the sorted member list (0 = all).
    pub evenly: usize,
    /// Start the evenly spaced selection half a step in, so that it shares no
    /// member with a selection of the same size that started at zero.
    pub offset_half: bool,
    /// Take the first this many of the sorted member list (0 = all).
    pub first: usize,
    /// Drop directories and write every member directly into the output folder.
    pub flatten: bool,
}

pub const VIDEO_EXTS: [&str; 5] = ["mp4", "mov", "avi", "mkv", "webm"];
pub const IMAGE_EXTS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

/// Evenly spaced `n` of `v` (the selection rule used for every sampled set).
pub fn evenly<T: Clone>(v: &[T], n: usize, offset_half: bool) -> Vec<T> {
    if n == 0 || v.len() <= n {
        return v.to_vec();
    }
    let k = v.len() / n;
    let start = if offset_half { k / 2 } else { 0 };
    (0..n).map(|i| v[start + i * k].clone()).collect()
}

fn choose(mut names: Vec<String>, sel: &Select) -> Vec<String> {
    names.retain(|n| {
        let lower = n.to_ascii_lowercase();
        !n.contains("__MACOSX")
            && !n.ends_with('/')
            && n.contains(&sel.contains)
            && (sel.exts.is_empty() || sel.exts.iter().any(|e| lower.ends_with(&format!(".{e}"))))
            && !sel.exclude.contains(n)
            && (sel.only.is_empty() || sel.only.contains(n))
    });
    names.sort();
    if sel.evenly > 0 {
        names = evenly(&names, sel.evenly, sel.offset_half);
    }
    if sel.first > 0 {
        names.truncate(sel.first);
    }
    names
}

fn target(out: &Path, name: &str, flatten: bool) -> Option<PathBuf> {
    // Refuse absolute paths and parent traversal from untrusted archives.
    let rel = Path::new(name);
    if rel.is_absolute() || rel.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return None;
    }
    Some(if flatten { out.join(rel.file_name()?) } else { out.join(rel) })
}

/// Extract selected members of a `.zip`, `.tar`, `.tar.gz` or `.tgz`.
/// Returns the member names written.
pub fn unpack(archive: &Path, out: &Path, sel: &Select) -> Result<Vec<String>> {
    std::fs::create_dir_all(out)?;
    let name = archive.to_string_lossy().to_ascii_lowercase();
    if name.ends_with(".zip") {
        let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
        let names: Vec<String> = zip.file_names().filter_map(|s| s.ok().map(|c| c.to_string())).collect();
        let picked = choose(names, sel);
        for n in &picked {
            let Some(dest) = target(out, n, sel.flatten) else { continue };
            if let Some(p) = dest.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut f = zip.by_name(n)?;
            std::io::copy(&mut f, &mut File::create(&dest)?)?;
        }
        Ok(picked)
    } else {
        let open = || -> Result<Box<dyn Read>> {
            let f = File::open(archive)?;
            Ok(if name.ends_with(".gz") || name.ends_with(".tgz") { Box::new(flate2::read::GzDecoder::new(f)) } else { Box::new(f) })
        };
        // Tar has no index: one pass to list, one to extract (skipped when taking everything).
        let wanted: Option<std::collections::HashSet<String>> = if sel.evenly > 0 || sel.first > 0 {
            let mut names = Vec::new();
            for e in tar::Archive::new(open()?).entries()? {
                let e = e?;
                if e.header().entry_type().is_file() {
                    names.push(e.path()?.to_string_lossy().to_string());
                }
            }
            Some(choose(names, sel).into_iter().collect())
        } else {
            None
        };
        let mut written = Vec::new();
        for e in tar::Archive::new(open()?).entries()? {
            let mut e = e?;
            if !e.header().entry_type().is_file() {
                continue;
            }
            let n = e.path()?.to_string_lossy().to_string();
            let keep = match &wanted {
                Some(w) => w.contains(&n),
                None => !choose(vec![n.clone()], sel).is_empty(),
            };
            if !keep {
                continue;
            }
            let Some(dest) = target(out, &n, sel.flatten) else { continue };
            if let Some(p) = dest.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut f = File::create(&dest)?;
            std::io::copy(&mut e, &mut f)?;
            f.flush()?;
            written.push(n);
        }
        Ok(written)
    }
}

/// Member names of an archive that pass `sel`'s filters (no extraction).
pub fn list_archive(archive: &Path, sel: &Select) -> Result<Vec<String>> {
    let name = archive.to_string_lossy().to_ascii_lowercase();
    let names: Vec<String> = if name.ends_with(".zip") {
        let zip = zip::ZipArchive::new(File::open(archive)?)?;
        zip.file_names().filter_map(|s| s.ok().map(|c| c.to_string())).collect()
    } else {
        let f = File::open(archive)?;
        let r: Box<dyn Read> = if name.ends_with(".gz") || name.ends_with(".tgz") { Box::new(flate2::read::GzDecoder::new(f)) } else { Box::new(f) };
        let mut v = Vec::new();
        for e in tar::Archive::new(r).entries()? {
            let e = e?;
            if e.header().entry_type().is_file() {
                v.push(e.path()?.to_string_lossy().to_string());
            }
        }
        v
    };
    Ok(choose(names, sel))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_step_selection_shares_nothing_with_the_first() {
        let names: Vec<String> = (0..1037).map(|i| format!("clip_{i:05}.mp4")).collect();
        let first = choose(names.clone(), &Select { evenly: 100, ..Default::default() });
        let second = choose(names, &Select { evenly: 100, offset_half: true, ..Default::default() });
        assert_eq!((first.len(), second.len()), (100, 100));
        assert!(first.iter().all(|n| !second.contains(n)));
    }

    #[test]
    fn globs() {
        assert!(glob_match("data/train-0000[0-4]-of-00031.parquet", "data/train-00003-of-00031.parquet"));
        assert!(!glob_match("data/train-0000[0-4]-of-00031.parquet", "data/train-00007-of-00031.parquet"));
        assert!(glob_match("videos/*", "videos/a/b.mp4"));
        assert!(glob_match("*.csv", "metadata.csv"));
        assert!(!glob_match("*.csv", "metadata.csv.bak"));
        assert!(glob_match("core/test-0000?-of-*.parquet", "core/test-00001-of-00013.parquet"));
    }

    #[test]
    fn even_sampling_and_selection() {
        let v: Vec<u32> = (0..100).collect();
        assert_eq!(evenly(&v, 4, false), vec![0, 25, 50, 75]);
        assert_eq!(evenly(&v, 4, true), vec![12, 37, 62, 87]);
        assert_eq!(evenly(&v, 0, false).len(), 100);
        let names = vec!["a/x.mp4".to_string(), "__MACOSX/a/x.mp4".into(), "a/y.txt".into(), "b/z.MP4".into(), "a/".into()];
        let sel = Select { exts: VIDEO_EXTS.to_vec(), ..Default::default() };
        assert_eq!(choose(names.clone(), &sel), vec!["a/x.mp4", "b/z.MP4"]);
        let sel = Select { contains: "b/".into(), ..Default::default() };
        assert_eq!(choose(names, &sel), vec!["b/z.MP4"]);
    }

    #[test]
    fn archive_paths_cannot_escape() {
        let out = Path::new("/tmp/out");
        assert!(target(out, "../evil", false).is_none());
        assert!(target(out, "/etc/passwd", false).is_none());
        assert_eq!(target(out, "a/b.png", true).unwrap(), out.join("b.png"));
        assert_eq!(encode_path("videos/fake/a b%2Fc.mp4"), "videos/fake/a%20b%252Fc.mp4");
    }
}
