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
    /// SHA-256 of the contents, where the host publishes one (Hugging Face
    /// does for every file stored through LFS, which is nearly every data file).
    pub sha256: Option<String>,
}

const UA: &str = "tintype-fetch";

/// The commit every Hugging Face dataset repository is read at. Listing and
/// downloads both go through it, so a plan sees the same files whatever is
/// pushed upstream later. Each was the head of `main` on 2026-11-11, and each
/// head predates the first download (2026-11-10); see docs/DATA.md.
pub const HF_REVISIONS: [(&str, &str); 28] = [
    ("Rajarshi-Roy-research/Defactify_Image_Dataset", "1c3b714c9691eb6dd11c2c1e92eab1ac29584fa0"),
    ("bitmind/nano-banana", "9ea8da32a5be03f4946e6cb10c2d2f8e90f0a0a4"),
    ("ComplexDataLab/OpenFake", "00b2f0ae3f5f7684c4b00f794f4c500076a9355d"),
    ("Thermostatic/frontier-synthetic-images-2026", "cb1df71639329a7e7b21a75bb6e838f3e38434cf"),
    ("OwensLab/CommunityForensics-Small", "6c539a534c07917307c381f5af4053c6091b5278"),
    ("kyoungjunpark/VidGuard-R1-sampled-video-10k", "ade041d82d6130d39de7844a4b9d4076cae2cb81"),
    ("l8cv/GenBuster-200K-mini", "58af00f022d8272f42de3be912e9a905bf6087d2"),
    ("AIGVDBench/AIGVDBench", "bf48acaa4920990af3dd2a511ea88e63354df305"),
    ("kiyoonkim/kinetics-400-targz", "12c9876b9ee78ee75a1bb7229f54b6e5fb1256c8"),
    ("zengxianyu/open-sora-pexels-subset", "c595493c834ec2c47ceee5d18ecca002aa646540"),
    ("saberzl/So-Fake-Set", "c041015e5480233431a1857c89753db332c3a4ab"),
    ("saberzl/So-Fake-OOD", "b4b34d16fda2020c2923a21d431d1ab3acbdbaff"),
    ("marco-willi/synthbuster-plus", "dbfb72f1ee96e953ee5cff80c58832fe89e1d2b5"),
    ("HorizonTEL/AIGIBench", "e44ec40efe5117a5ccdaa6ff0e89ed934d03d310"),
    ("wasitaigeneratedcom/frontier-image-video-bench-2026-10", "e6a8b83d15fab1da958848368fcd07e943afafd0"),
    ("OmerXYZ/comgenvid", "17f9ab6c548d9a30f5b22c09e17daee74536b653"),
    ("Rapidata/text-2-video-human-preferences-moonvalley-marey", "3905a1774a6e63f4255f34a683a20e61342e478a"),
    ("Rapidata/text-2-video-human-preferences-veo3.1", "9b5ffd5a1ce186b746990e554dba2a03ee74434d"),
    ("Sunain-Labs/sunain-gameplay", "5143e0151144cb4eaba4f58b2bcc594408983292"),
    ("trojblue/sakugabooru2025", "3b6de544855c3722dca0bc0d9983fc94fed6ccc7"),
    ("hdong51/Human-Animal-Cartoon", "799cccce4d335a6ff44654ee7d18220802edefb4"),
    ("TheKernel01/Tiny-GenImage", "89c4fe9efd0ebc7ce5c7641ef57d578ccd639c69"),
    ("34data/StyleGAN3_1024x1024_images", "c72281e0fb940599eb77b0fcb4e92fbdfde74f36"),
    ("javi22/this-person-does-not-exist-10k", "c8d08f1b0b8d7416c960aebcc42c0f0048855915"),
    ("34data/ip-adapter-faceid", "10cfd1e50456cfb44f4acfbc3c05ccba3aba1d16"),
    ("34data/sd35-ip-adapter-faceid", "3fcbf6ff99cda94e67139daa5f002efd7507645c"),
    ("mattymchen/celeba-hq", "1bc5dfbeb766d03abeddf5a48b9871479a4395bb"),
    ("sywang/CNNDetection", "f287cf3c3e92ca2e5451ce200bab8b6f5e0175b2"),
];

/// The commit every ModelScope dataset repository is read at (the head of
/// `master` on 2026-11-11; both heads date from 2024 and 2025).
pub const MODELSCOPE_REVISIONS: [(&str, &str); 2] = [("cccnju/Gen-Video", "4c1cb7ec0f196e4359d764a71f2d36b158b6e877"), ("cccnju/GenVideo-100K", "b31a22ff523263c6915a480966cbab1c87eca345")];

fn revision(table: &[(&str, &'static str)], repo: &str) -> Result<&'static str> {
    table.iter().find(|(r, _)| *r == repo).map(|(_, c)| *c).ok_or_else(|| anyhow!("{repo} has no pinned revision; add its commit to the revision table in src/fetch.rs"))
}

/// The pinned commit of a Hugging Face dataset repository.
pub fn hf_revision(repo: &str) -> Result<&'static str> {
    revision(&HF_REVISIONS, repo)
}

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

/// Every file of a Hugging Face dataset repository at its pinned revision
/// (paginated tree API).
pub fn hf_list(repo: &str) -> Result<Vec<RemoteFile>> {
    let mut url = format!("https://huggingface.co/api/datasets/{repo}/tree/{}?recursive=true&limit=1000", hf_revision(repo)?);
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
                // An LFS object's id is the SHA-256 of its contents. (The git `oid` of a
                // file kept in git itself is a SHA-1 of a git blob, not of the file.)
                let sha256 = e["lfs"]["oid"].as_str().filter(|h| h.len() == 64).map(str::to_string);
                out.push(RemoteFile { path: e["path"].as_str().unwrap_or_default().to_string(), size, sha256 });
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

/// Download URL of a file of a Hugging Face dataset, at the pinned revision.
pub fn hf_url(repo: &str, path: &str) -> Result<String> {
    Ok(format!("https://huggingface.co/datasets/{repo}/resolve/{}/{}", hf_revision(repo)?, encode_path(path)))
}

/// Download URL of a file of a ModelScope dataset, at the pinned revision.
pub fn modelscope_url(repo: &str, path: &str) -> Result<String> {
    Ok(format!("https://modelscope.cn/datasets/{repo}/resolve/{}/{}", revision(&MODELSCOPE_REVISIONS, repo)?, encode_path(path)))
}

fn remote_size(url: &str) -> Result<u64> {
    let resp = ureq::get(url).header("User-Agent", UA).header("Range", "bytes=0-0").call()?;
    resp.headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.rsplit('/').next()?.parse().ok())
        .ok_or_else(|| anyhow!("server gave no size for {url}"))
}

/// Write all of `buf` at byte `offset` without moving a shared cursor.
#[cfg(unix)]
fn write_all_at(file: &File, buf: &[u8], offset: u64) -> std::io::Result<()> {
    std::os::unix::fs::FileExt::write_all_at(file, buf, offset)
}

/// Write all of `buf` at byte `offset`. Each call passes its own offset, so
/// concurrent range writers never depend on the file cursor.
#[cfg(windows)]
fn write_all_at(file: &File, mut buf: &[u8], mut offset: u64) -> std::io::Result<()> {
    use std::os::windows::fs::FileExt;
    while !buf.is_empty() {
        match file.seek_write(buf, offset) {
            Ok(0) => return Err(std::io::ErrorKind::WriteZero.into()),
            Ok(n) => {
                buf = &buf[n..];
                offset += n as u64;
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn fetch_range(url: &str, file: &File, start: u64, end: u64) -> Result<()> {
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
                write_all_at(file, &buf[..n], pos)?;
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

/// What is already on disk at a download's destination.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cached {
    /// Absent, or not the expected size.
    No,
    /// The expected size and SHA-256.
    Verified,
    /// The expected size; no SHA-256 is recorded to check it against.
    SizeOnly,
    /// The expected size but another SHA-256 (this one).
    Corrupt(String),
}

/// Compare the file at `dest` with what a download should produce.
pub fn cached(dest: &Path, size: u64, sha256: Option<&str>) -> Result<Cached> {
    if !dest.metadata().is_ok_and(|m| m.len() == size) {
        return Ok(Cached::No);
    }
    Ok(match sha256 {
        None => Cached::SizeOnly,
        Some(want) => match crate::integrity::sha256_file(dest)? {
            got if got.eq_ignore_ascii_case(want) => Cached::Verified,
            got => Cached::Corrupt(got),
        },
    })
}

/// Download `url` to `dest` using `conns` parallel ranged requests. A file
/// already there is kept when it has the right size and, where `sha256` is
/// given, the right SHA-256; a download that does not match `sha256` is
/// discarded and reported as an error.
pub fn download(url: &str, dest: &Path, size: Option<u64>, sha256: Option<&str>, conns: usize) -> Result<()> {
    let size = match size {
        Some(s) if s > 0 => s,
        _ => remote_size(url)?,
    };
    match cached(dest, size, sha256)? {
        Cached::Verified | Cached::SizeOnly => return Ok(()),
        Cached::Corrupt(got) => eprintln!("{}: has SHA-256 {got}, expected {}; downloading again", dest.display(), sha256.unwrap_or_default()),
        Cached::No => {}
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
    drop(file);
    if let Some(want) = sha256 {
        let got = crate::integrity::sha256_file(&tmp)?;
        if !got.eq_ignore_ascii_case(want) {
            let _ = std::fs::remove_file(&tmp);
            bail!("{url}: downloaded {size} bytes with SHA-256 {got}, expected {want}; discarded");
        }
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
        download(&hf_url(repo, &f.path)?, &dest, Some(f.size), f.sha256.as_deref(), c)?;
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

/// The members of an archive that `sel` picks, in sorted order.
pub fn members(names: &[String], sel: &Select) -> Vec<String> {
    choose(names.to_vec(), sel)
}

/// What is left of `names` (sorted) once the evenly spaced samples in
/// `taken` – (size, started half a step in) – are removed: the files a later
/// sample can draw on without sharing any with the earlier ones.
pub fn unsampled(names: &[String], taken: &[(usize, bool)]) -> Vec<String> {
    let mut used = std::collections::HashSet::new();
    for &(n, half) in taken {
        used.extend(evenly(names, n, half));
    }
    names.iter().filter(|n| !used.contains(*n)).cloned().collect()
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
    // Refuse absolute paths, roots, drive prefixes and parent traversal from
    // untrusted archives. On Windows `/etc/passwd` and `C:x` are not absolute,
    // yet joining them would escape `out`, so only plain names pass.
    use std::path::Component;
    let rel = Path::new(name);
    if rel.components().any(|c| !matches!(c, Component::Normal(_) | Component::CurDir)) {
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
    fn a_later_sample_shares_nothing_with_the_earlier_ones() {
        let names: Vec<String> = (0..1037).map(|i| format!("img_{i:05}.png")).collect();
        let (first, second) = (evenly(&names, 400, false), evenly(&names, 400, true));
        let rest = unsampled(&names, &[(400, false), (400, true)]);
        assert_eq!(rest.len(), 1037 - 800);
        let third = evenly(&rest, 400, false);
        assert_eq!(third.len(), 237);
        assert!(third.iter().all(|n| !first.contains(n) && !second.contains(n)));
        // An archive the earlier sample took whole leaves nothing.
        assert!(unsampled(&names, &[(1500, false)]).is_empty());
    }

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
    fn the_cache_is_trusted_only_with_the_right_hash() {
        let p = std::env::temp_dir().join(format!("tintype-cached-{}", std::process::id()));
        std::fs::write(&p, b"abc").unwrap();
        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert_eq!(cached(&p, 3, Some(abc)).unwrap(), Cached::Verified);
        assert_eq!(cached(&p, 3, Some(&abc.to_uppercase())).unwrap(), Cached::Verified);
        assert_eq!(cached(&p, 3, None).unwrap(), Cached::SizeOnly);
        assert_eq!(cached(&p, 4, Some(abc)).unwrap(), Cached::No);
        // Same size, other bytes: no longer taken for the download.
        std::fs::write(&p, b"abd").unwrap();
        assert!(matches!(cached(&p, 3, Some(abc)).unwrap(), Cached::Corrupt(_)));
        let _ = std::fs::remove_file(&p);
        assert_eq!(cached(&p, 3, Some(abc)).unwrap(), Cached::No);
    }

    /// Downloads a 5.8 KB file; run with `cargo test --release -- --ignored`.
    #[test]
    #[ignore = "network"]
    fn downloads_are_checked_against_their_sha256() {
        let url = "https://huggingface.co/haywoodsloan/ai-image-detector-deploy/resolve/4ae1822603cb1ec980c217cfde1ef1b8e0477a16/training_args.bin";
        let sha = "96bf2e99106e3c60cff8fa494aeff2efb2392fbb9580502f7fa416fcaadd7456";
        let dest = std::env::temp_dir().join(format!("tintype-download-{}", std::process::id()));
        let _ = std::fs::remove_file(&dest);
        download(url, &dest, None, Some(sha), 1).unwrap();
        assert_eq!(crate::integrity::sha256_file(&dest).unwrap(), sha);
        // A cached copy with other bytes is replaced.
        let mut bytes = std::fs::read(&dest).unwrap();
        bytes[0] ^= 1;
        std::fs::write(&dest, &bytes).unwrap();
        download(url, &dest, None, Some(sha), 1).unwrap();
        assert_eq!(crate::integrity::sha256_file(&dest).unwrap(), sha);
        // A download that does not match is refused and leaves nothing behind.
        let _ = std::fs::remove_file(&dest);
        let wrong = "0".repeat(64);
        assert!(download(url, &dest, None, Some(&wrong), 1).is_err());
        assert!(!dest.exists() && !dest.with_extension("part").exists());
    }

    #[test]
    fn archive_paths_cannot_escape() {
        let out = Path::new("/tmp/out");
        assert!(target(out, "../evil", false).is_none());
        assert!(target(out, "/etc/passwd", false).is_none());
        assert!(target(out, "a/../../evil", false).is_none());
        if cfg!(windows) {
            assert!(target(out, "C:evil", false).is_none());
            assert!(target(out, r"..\evil", false).is_none());
            assert!(target(out, r"\\server\share\evil", false).is_none());
        }
        assert_eq!(target(out, "a/b.png", true).unwrap(), out.join("b.png"));
        assert_eq!(encode_path("videos/fake/a b%2Fc.mp4"), "videos/fake/a%20b%252Fc.mp4");
    }
}
