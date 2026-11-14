//! Partial edits: images in which only a region was replaced (a pasted or
//! inpainted patch, or everything but the subject). This is a separate
//! evaluation target with a separate output. Nothing here feeds the verdict
//! `detect` gives, and no shipped file is read for writing.
//!
//! Three steps, each a subcommand:
//!
//! - `paste-edits` builds pasted-region composites from images already on
//!   disk, with the controls that separate "found generated pixels" from
//!   "found a seam" and "found a re-save" ([`paste_edits`]).
//! - `score-windows` scores the model's own views and a dense grid of
//!   native-resolution windows, and records where each window lies
//!   ([`score_windows`]).
//! - `partial-report` compares pooling rules at the same authentic
//!   false-positive rate and measures how well the flagged windows match the
//!   masks ([`report`]).
//!
//! A partial-edit store is a manifest with the grouping columns
//! ([`crate::ingest::GROUP_COLUMNS`]: `parent_id` is the authentic original,
//! `transform` names the edit) and a side file `edits.tsv` beside it that
//! points at the masks ([`read_edits`]). In such a store the label column
//! means "contains generated pixels", not "wholly generated": keep these
//! manifests out of training and calibration.
use crate::detector::{Detector, Device, ModelSpec, ViewSpec};
use crate::image_io::{self, Rgb, View};
use crate::ingest::Item;
use crate::metrics;
use anyhow::{bail, ensure, Context, Result};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// A rectangle of the original image, in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    fn text(&self) -> String {
        format!("{},{},{},{}", self.x, self.y, self.w, self.h)
    }
}

/// `n` window origins spread evenly over one axis, edge to edge (one window
/// sits in the middle).
fn axis(full: u32, crop: u32, n: usize) -> Vec<u32> {
    let room = (full - crop) as u64;
    if n <= 1 {
        return vec![(room / 2) as u32];
    }
    let last = n as u64 - 1;
    (0..n as u64).map(|i| ((room * i + last / 2) / last) as u32).collect()
}

/// The dense grid of `crop`-sized native-resolution windows over a
/// `w x h` image: origins no more than `stride` apart along each axis, the
/// first and last window flush with the image border. When that would give
/// more than `max` windows the wider-spaced axis keeps its count and the
/// other is thinned, so the grid still spans the image (with gaps once the
/// spacing passes `crop`; [`spacing`] says). An image smaller than the crop
/// gets one window: the centre square the native view upscales.
pub fn grid_windows(w: u32, h: u32, crop: u32, stride: u32, max: usize) -> Vec<Rect> {
    if w.min(h) < crop {
        let s = w.min(h);
        return vec![Rect { x: (w - s) / 2, y: (h - s) / 2, w: s, h: s }];
    }
    let count = |full: u32| ((full - crop) as usize).div_ceil(stride.max(1) as usize) + 1;
    let (mut nx, mut ny) = (count(w), count(h));
    let gap = |full: u32, n: usize| if n > 1 { (full - crop) as f64 / (n - 1) as f64 } else { f64::INFINITY };
    while nx * ny > max.max(1) {
        // Thin the axis whose windows are closer together.
        if nx > 1 && (ny == 1 || gap(w, nx) <= gap(h, ny)) {
            nx -= 1;
        } else {
            ny -= 1;
        }
    }
    let (xs, ys) = (axis(w, crop, nx), axis(h, crop, ny));
    ys.iter().flat_map(|&y| xs.iter().map(move |&x| Rect { x, y, w: crop, h: crop })).collect()
}

/// The largest distance between neighbouring window origins of a grid, in
/// pixels: above the crop size, parts of the image lie in no window.
pub fn spacing(windows: &[Rect]) -> u32 {
    let along = |mut v: Vec<u32>| {
        v.sort_unstable();
        v.dedup();
        v.windows(2).map(|p| p[1] - p[0]).max().unwrap_or(0)
    };
    along(windows.iter().map(|r| r.x).collect()).max(along(windows.iter().map(|r| r.y).collect()))
}

/// The pixels of one grid window, as the network is shown them: the window
/// itself at native resolution, or the upscaled centre square of an image
/// smaller than the crop (what the native view does).
fn window_pixels(img: &Rgb, r: Rect, crop: u32) -> Rgb {
    if img.w.min(img.h) < crop {
        View::Native { crop }.apply(img)
    } else {
        img.crop(r.x as i64, r.y as i64, crop, crop)
    }
}

/// Name and position of one of the model's own views in a `w x h` image.
/// `rich` are the texture-ranked window origins ([`image_io::rich_windows`]).
/// The whole-image view is given the centre square it shows.
pub fn view_box(v: ViewSpec, w: u32, h: u32, rich: &[(i64, i64)]) -> (String, Rect) {
    let short = w.min(h);
    let centred = |s: u32| Rect { x: (w - s.min(w)) / 2, y: (h - s.min(h)) / 2, w: s.min(w), h: s.min(h) };
    // The same rounding as `Rgb::center_crop`.
    let centre = |crop: u32| Rect { x: ((w as f64 - crop as f64) / 2.0).round() as u32, y: ((h as f64 - crop as f64) / 2.0).round() as u32, w: crop, h: crop };
    match v {
        ViewSpec::Fit { resize, crop } | ViewSpec::FitPillow { resize, crop } => ("fit".into(), centred((short as u64 * crop as u64 / resize.max(1) as u64) as u32)),
        ViewSpec::Native { crop } => ("native".into(), if short < crop { centred(short) } else { centre(crop) }),
        ViewSpec::NativeRich { crop, rank } => {
            let name = format!("rich{rank}");
            match rich.get((rank as usize).min(rich.len().saturating_sub(1))) {
                Some(&(x, y)) if short >= crop => (name, Rect { x: x as u32, y: y as u32, w: crop, h: crop }),
                _ => (name, centred(short)),
            }
        }
        ViewSpec::NativeAt { crop, fx, fy } => {
            let name = format!("at{fx}-{fy}");
            if short < crop {
                return (name, centred(short));
            }
            let pos = |full: u32, f: u32| ((full as i64 * f as i64 / 1000) - crop as i64 / 2).clamp(0, (full - crop) as i64) as u32;
            (name, Rect { x: pos(w, fx), y: pos(h, fy), w: crop, h: crop })
        }
        ViewSpec::SquashPillow { .. } | ViewSpec::CenterPad { .. } => ("view".into(), Rect { x: 0, y: 0, w, h }),
    }
}

/// Mean of the `k` largest values (all of them when there are fewer).
pub fn top_k_mean(values: &[f32], k: usize) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| b.total_cmp(a));
    v.truncate(k.max(1));
    Some(v.iter().sum::<f32>() / v.len() as f32)
}

fn mean(values: &[f32]) -> Option<f32> {
    (!values.is_empty()).then(|| values.iter().sum::<f32>() / values.len() as f32)
}

fn max(values: &[f32]) -> Option<f32> {
    values.iter().copied().max_by(|a, b| a.total_cmp(b))
}

/// How `score-windows` runs.
pub struct WindowOptions {
    /// Largest distance between neighbouring grid windows, in pixels.
    pub stride: u32,
    /// Most grid windows per image.
    pub max_windows: usize,
    pub batch: usize,
    pub threads: usize,
}

/// What a decode worker learns about an image besides its crops.
struct Layout {
    size: (u32, u32),
    views: Vec<(String, Rect)>,
    grid: Vec<Rect>,
}

/// Score every image of `items` twice over: the model's own views (all of
/// them, as `score --all-views` does) and a dense grid of native-resolution
/// windows. Writes one row per image to `out`:
///
/// `path label generator dataset prob logit...` exactly as `score
/// --all-views` writes them, then `size=WxH`, `deployed=prob/views/logit`
/// (what `detect` reports: the early-exit result, when the spec has a
/// schedule), `views=name,x,y,w,h;...` (where each of the model's views
/// lies) and `grid=x,y,w,h,logit;...`.
///
/// The global score is unchanged by any of this: the grid is extra
/// evaluation output.
pub fn score_windows(weights: &Path, items: &[Item], out: &Path, device: Device, o: &WindowOptions) -> Result<()> {
    let mut spec = ModelSpec::load_for(weights)?;
    ensure!(!spec.views.is_empty(), "the model declares no views");
    let exit = spec.early_exit.take();
    let crop = spec.img as u32;
    let own: Vec<ViewSpec> = spec.views.clone();
    let mut det = Detector::load(weights, Some(spec), o.batch.max(1), device)?;
    let threshold = det.threshold_logit();
    // The model's own views are cut by its preprocessor; the network is then
    // fed ready-made crops, which the centre crop of a crop-sized image passes through untouched.
    let pre = det.preprocessor();
    det.spec.views = vec![ViewSpec::Native { crop }];
    let layouts: Vec<std::sync::Mutex<Option<Layout>>> = items.iter().map(|_| std::sync::Mutex::new(None)).collect();
    let (stride, cap) = (o.stride, o.max_windows);
    let (results, timing) = det.score_stream(items.len(), o.threads.max(1), |i| {
        let img = image_io::load(&items[i].path).ok()?;
        let ranks = own.iter().filter_map(|v| if let ViewSpec::NativeRich { rank, .. } = v { Some(*rank as usize + 1) } else { None }).max();
        let rich = match ranks {
            Some(k) if img.w.min(img.h) >= crop => image_io::rich_windows(&img, crop, k),
            _ => Vec::new(),
        };
        let grid = if stride == 0 { Vec::new() } else { grid_windows(img.w, img.h, crop, stride, cap) };
        let mut crops = pre.crops(&img);
        crops.extend(grid.iter().map(|&r| window_pixels(&img, r, crop)));
        *layouts[i].lock().unwrap() = Some(Layout { size: (img.w, img.h), views: own.iter().map(|&v| view_box(v, img.w, img.h, &rich)).collect(), grid });
        Some(crops)
    });
    let mut w = std::io::BufWriter::new(std::fs::File::create(out).with_context(|| format!("creating {}", out.display()))?);
    let (mut bad, mut windows) = (0usize, 0usize);
    for ((it, r), layout) in items.iter().zip(&results).zip(layouts) {
        let (Some(r), Some(layout)) = (r, layout.into_inner().unwrap()) else {
            bad += 1;
            continue;
        };
        let logits: Vec<f32> = r.iter().map(|v| v[0]).collect();
        let (views, grid) = logits.split_at(own.len());
        windows += grid.len();
        let seen = match exit.as_ref().filter(|e| e.covers(views.len())) {
            Some(e) => e.looked_at(views, threshold),
            None => views.to_vec(),
        };
        let line = format!(
            "{}\t{}\t{}\t{}\t{:.6}\t{}\tsize={}x{}\tdeployed={:.6}/{}/{:.5}\tviews={}\tgrid={}",
            it.path.display(),
            it.label,
            it.generator,
            it.dataset,
            det.probability(views),
            views.iter().map(|x| format!("{x:.5}")).collect::<Vec<_>>().join("\t"),
            layout.size.0,
            layout.size.1,
            det.probability(&seen),
            seen.len(),
            mean(&seen).unwrap_or(f32::NAN),
            layout.views.iter().map(|(n, b)| format!("{n},{}", b.text())).collect::<Vec<_>>().join(";"),
            layout.grid.iter().zip(grid).map(|(b, z)| format!("{},{z:.5}", b.text())).collect::<Vec<_>>().join(";"),
        );
        writeln!(w, "{line}")?;
    }
    w.flush()?;
    eprintln!("scored {} images ({bad} unreadable) and {windows} grid windows in {:.1}s ({:.1}s in the model)", items.len() - bad, timing.wall_s, timing.model_s);
    Ok(())
}

/// One row of a `score-windows` file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scored {
    pub path: String,
    pub label: u8,
    pub generator: String,
    pub dataset: String,
    /// Probability from all of the model's views.
    pub prob: f32,
    /// Logits of the model's views, in profile order.
    pub view_logits: Vec<f32>,
    pub size: (u32, u32),
    /// What `detect` reports: `(probability, views looked at, mean logit)`.
    pub deployed: Option<(f32, usize, f32)>,
    pub views: Vec<(String, Rect)>,
    pub grid: Vec<(Rect, f32)>,
}

fn rect_of(f: &[&str]) -> Result<Rect> {
    ensure!(f.len() >= 4, "a window needs x,y,w,h");
    Ok(Rect { x: f[0].parse()?, y: f[1].parse()?, w: f[2].parse()?, h: f[3].parse()? })
}

pub fn parse_scored(line: &str) -> Result<Scored> {
    let f: Vec<&str> = line.split('\t').collect();
    ensure!(f.len() >= 5, "a score row needs path, label, generator, dataset and probability");
    let mut s = Scored { path: f[0].into(), label: f[1].parse()?, generator: f[2].into(), dataset: f[3].into(), prob: f[4].parse()?, ..Default::default() };
    for field in &f[5..] {
        let Some((key, value)) = field.split_once('=') else {
            s.view_logits.push(field.parse().with_context(|| format!("bad logit `{field}`"))?);
            continue;
        };
        match key {
            "size" => {
                let (w, h) = value.split_once('x').context("size is WxH")?;
                s.size = (w.parse()?, h.parse()?);
            }
            "deployed" => {
                let p: Vec<&str> = value.split('/').collect();
                ensure!(p.len() == 3, "deployed is prob/views/logit");
                s.deployed = Some((p[0].parse()?, p[1].parse()?, p[2].parse()?));
            }
            "views" => {
                for v in value.split(';').filter(|v| !v.is_empty()) {
                    let p: Vec<&str> = v.split(',').collect();
                    s.views.push((p[0].to_string(), rect_of(&p[1..])?));
                }
            }
            "grid" => {
                for v in value.split(';').filter(|v| !v.is_empty()) {
                    let p: Vec<&str> = v.split(',').collect();
                    ensure!(p.len() == 5, "a grid window is x,y,w,h,logit");
                    s.grid.push((rect_of(&p)?, p[4].parse()?));
                }
            }
            _ => {}
        }
    }
    Ok(s)
}

pub fn read_window_scores(path: &Path) -> Result<Vec<Scored>> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    text.lines().filter(|l| !l.is_empty()).map(|l| parse_scored(l).with_context(|| format!("in {}", path.display()))).collect()
}

/// One row of `edits.tsv`, the side file of a partial-edit store:
///
/// `path  mask  kind  generated  x  y  w  h  area  donor  donor_generator`
///
/// separated by tabs, lines starting with `#` ignored, paths relative to the
/// file's directory. Only the first four columns are required. `mask` is an
/// 8-bit greyscale image the size of the item (0 = untouched pixel, 255 =
/// replaced, in between = the blend weight), or `-` for an item with no
/// edit. `kind` names the edit (`parent`, `resave`, `self-paste`,
/// `authentic-paste`, `generated-paste`, `inpaint`, ...). `generated` is 1
/// when the replaced pixels came from a generator.
#[derive(Clone, Debug, PartialEq)]
pub struct Edit {
    pub mask: Option<PathBuf>,
    pub kind: String,
    pub generated: bool,
}

/// The key files are joined on: the absolute path, so that a score file, a
/// manifest and an edits file agree however each was addressed.
pub fn path_key(p: &Path) -> String {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf()).display().to_string()
}

/// Read `edits.tsv`, keyed by the item's path ([`path_key`]) as a manifest
/// in the same directory resolves it.
pub fn read_edits(path: &Path) -> Result<BTreeMap<String, Edit>> {
    let root = path.parent().unwrap_or(Path::new("."));
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut out = BTreeMap::new();
    for l in text.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let f: Vec<&str> = l.split('\t').collect();
        ensure!(f.len() >= 4, "bad edits line (need path, mask, kind, generated): {l}");
        let mask = (f[1] != "-" && !f[1].is_empty()).then(|| root.join(f[1]));
        out.insert(path_key(&root.join(f[0])), Edit { mask, kind: f[2].to_string(), generated: f[3] == "1" });
    }
    Ok(out)
}

/// An edit mask as a summed-area table, so the share of any window that was
/// replaced is four lookups.
pub struct Mask {
    w: u32,
    h: u32,
    integ: Vec<u64>,
}

impl Mask {
    /// `alpha` is `w * h` blend weights, 0 to 255.
    pub fn new(w: u32, h: u32, alpha: &[u8]) -> Self {
        let (ww, hh) = (w as usize, h as usize);
        let mut integ = vec![0u64; (ww + 1) * (hh + 1)];
        for y in 0..hh {
            let mut row = 0u64;
            for x in 0..ww {
                row += alpha[y * ww + x] as u64;
                integ[(y + 1) * (ww + 1) + x + 1] = integ[y * (ww + 1) + x + 1] + row;
            }
        }
        Self { w, h, integ }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        let img = image::load_from_memory(&bytes).with_context(|| format!("decoding {}", path.display()))?.into_luma8();
        Ok(Self::new(img.width(), img.height(), img.as_raw()))
    }

    pub fn size(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    /// Mean blend weight inside `r`, 0 to 1: the share of the window that
    /// was replaced. The part of `r` outside the image counts as untouched.
    pub fn coverage(&self, r: Rect) -> f32 {
        if r.w == 0 || r.h == 0 {
            return 0.0;
        }
        let (x0, y0) = (r.x.min(self.w) as usize, r.y.min(self.h) as usize);
        let (x1, y1) = ((r.x + r.w).min(self.w) as usize, (r.y + r.h).min(self.h) as usize);
        let at = |x: usize, y: usize| self.integ[y * (self.w as usize + 1) + x];
        let sum = at(x1, y1) + at(x0, y0) - at(x1, y0) - at(x0, y1);
        (sum as f64 / (255.0 * r.w as f64 * r.h as f64)) as f32
    }

    /// Share of the whole image that was replaced.
    pub fn area(&self) -> f32 {
        self.coverage(Rect { x: 0, y: 0, w: self.w, h: self.h })
    }
}

/// A score an image-level decision can be taken on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    /// What `detect` reports today: the mean of the views early exit looked at.
    Deployed,
    /// Mean over all of the model's views (today's pooling, no early exit).
    ViewsMean,
    ViewsMax,
    ViewsTop(usize),
    GridMean,
    GridMax,
    GridTop(usize),
}

impl Stat {
    pub fn name(self) -> String {
        match self {
            Stat::Deployed => "global (deployed)".into(),
            Stat::ViewsMean => "views mean".into(),
            Stat::ViewsMax => "views max".into(),
            Stat::ViewsTop(k) => format!("views top-{k}"),
            Stat::GridMean => "grid mean".into(),
            Stat::GridMax => "grid max".into(),
            Stat::GridTop(k) => format!("grid top-{k}"),
        }
    }

    /// The statistic for one scored image, in logits.
    pub fn of(self, s: &Scored) -> Option<f32> {
        let grid = || s.grid.iter().map(|g| g.1).collect::<Vec<f32>>();
        match self {
            Stat::Deployed => s.deployed.map(|d| d.2),
            Stat::ViewsMean => mean(&s.view_logits),
            Stat::ViewsMax => max(&s.view_logits),
            Stat::ViewsTop(k) => top_k_mean(&s.view_logits, k),
            Stat::GridMean => mean(&grid()),
            Stat::GridMax => max(&grid()),
            Stat::GridTop(k) => top_k_mean(&grid(), k),
        }
    }
}

/// A scored image with what the store says about it.
pub struct Entry {
    pub scored: Scored,
    /// The manifest's `transform` column: the slice the item is reported in.
    pub transform: String,
    /// The kind of edit, from `edits.tsv` (else the transform).
    pub kind: String,
    /// The authentic original the item was derived from.
    pub parent: String,
    pub generated: bool,
    pub mask: Option<Mask>,
}

/// Join a `score-windows` file with the store's manifest and edits. Items the
/// manifest does not list are dropped; the count of those is returned too.
pub fn join(scored: Vec<Scored>, items: &[Item], edits: &BTreeMap<String, Edit>) -> Result<(Vec<Entry>, usize)> {
    let by_path: BTreeMap<String, &Item> = items.iter().map(|it| (path_key(&it.path), it)).collect();
    let (mut out, mut unknown) = (Vec::new(), 0);
    for s in scored {
        let key = path_key(Path::new(&s.path));
        let Some(it) = by_path.get(&key) else {
            unknown += 1;
            continue;
        };
        let edit = edits.get(&key);
        let transform = it.transform().unwrap_or(if it.label == 0 { "parent" } else { "(none)" }).to_string();
        let mask = match edit.and_then(|e| e.mask.as_ref()) {
            Some(p) => {
                let m = Mask::load(p)?;
                ensure!(m.size() == s.size || s.size == (0, 0), "mask {} is {}x{} but {} is {}x{}", p.display(), m.w, m.h, s.path, s.size.0, s.size.1);
                Some(m)
            }
            None => None,
        };
        out.push(Entry { kind: edit.map_or_else(|| transform.clone(), |e| e.kind.clone()), parent: it.parent_id().unwrap_or(&s.path).to_string(), generated: edit.map_or(it.label == 1, |e| e.generated), transform, mask, scored: s });
    }
    Ok((out, unknown))
}

fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3))
}

/// How the report is computed.
#[derive(Clone, Debug)]
pub struct ReportOptions {
    /// Authentic false-positive rates every pooling rule is operated at.
    pub fprs: Vec<f64>,
    /// `k` of the top-k means (the views use `min(k, 2)`).
    pub top_k: usize,
    /// A window counts as edited when the mask covers more than this share of it.
    pub overlap: f32,
    /// The transform whose items are the authentic images cutoffs are set on.
    pub cutoff_on: String,
    /// Set cutoffs on half of the parents and report on the other half.
    pub disjoint: bool,
}

impl Default for ReportOptions {
    fn default() -> Self {
        Self { fprs: vec![0.01, 0.001], top_k: 3, overlap: 0.1, cutoff_on: "parent".into(), disjoint: false }
    }
}

/// The pooling rules every entry has a value for.
pub fn stats_for(entries: &[Entry], top_k: usize) -> Vec<Stat> {
    [Stat::Deployed, Stat::ViewsMean, Stat::ViewsMax, Stat::ViewsTop(top_k.clamp(1, 2)), Stat::GridMean, Stat::GridMax, Stat::GridTop(top_k.max(1))]
        .into_iter()
        .filter(|st| !entries.is_empty() && entries.iter().all(|e| st.of(&e.scored).is_some()))
        .collect()
}

/// Which half of a parent-disjoint split an entry is in: 0 sets cutoffs, 1 is reported.
fn fold(e: &Entry) -> u64 {
    (fnv(&e.parent) >> 17) & 1
}

/// Flag counts of every slice under every pooling rule, each rule operated
/// at the cutoff that lets `fpr` of the authentic images through.
pub struct FlagTable {
    pub fpr: f64,
    /// Authentic images the cutoffs were set on, and how many of them a cutoff may flag.
    pub authentic: usize,
    pub allowed: usize,
    pub cutoffs: Vec<Option<f32>>,
    /// `(slice, images, images flagged per rule)`.
    pub rows: Vec<(String, usize, Vec<usize>)>,
}

/// The slices of a report in reading order: controls first, then edits, and
/// after the transforms of one kind a pooled row when it has several.
fn slices(entries: &[&Entry]) -> Vec<(String, Vec<usize>)> {
    let rank = |kind: &str| ["parent", "resave", "self-paste", "authentic-paste"].iter().position(|k| *k == kind).unwrap_or(4);
    let mut by: BTreeMap<(usize, String, String), Vec<usize>> = BTreeMap::new();
    for (i, e) in entries.iter().enumerate() {
        by.entry((rank(&e.kind), e.kind.clone(), e.transform.clone())).or_default().push(i);
    }
    let mut out: Vec<(String, Vec<usize>)> = Vec::new();
    let keys: Vec<&(usize, String, String)> = by.keys().collect();
    for (n, key) in keys.iter().enumerate() {
        out.push((key.2.clone(), by[*key].clone()));
        let last_of_kind = keys.get(n + 1).is_none_or(|next| next.1 != key.1);
        let of_kind: Vec<usize> = by.iter().filter(|(k, _)| k.1 == key.1).flat_map(|(_, v)| v.iter().copied()).collect();
        if last_of_kind && by.keys().filter(|k| k.1 == key.1).count() > 1 {
            out.push((format!("all {}", key.1), of_kind));
        }
    }
    out
}

/// The authentic entries cutoffs are set on, and the entries reported.
fn partition<'a>(entries: &'a [Entry], o: &ReportOptions) -> (Vec<&'a Entry>, Vec<&'a Entry>) {
    let authentic = entries.iter().filter(|e| e.transform == o.cutoff_on && !e.generated && (!o.disjoint || fold(e) == 0)).collect();
    let reported = entries.iter().filter(|e| !o.disjoint || fold(e) == 1).collect();
    (authentic, reported)
}

fn cutoff(authentic: &[&Entry], st: Stat, fpr: f64) -> Option<f32> {
    let scores: Vec<f32> = authentic.iter().filter_map(|e| st.of(&e.scored)).collect();
    metrics::empirical_cutoff(&scores, &vec![0u8; scores.len()], fpr)
}

pub fn flag_table(entries: &[Entry], stats: &[Stat], fpr: f64, o: &ReportOptions) -> FlagTable {
    let (authentic, reported) = partition(entries, o);
    let cutoffs: Vec<Option<f32>> = stats.iter().map(|&st| cutoff(&authentic, st, fpr)).collect();
    let rows = slices(&reported)
        .into_iter()
        .map(|(name, idx)| {
            let flagged = stats.iter().zip(&cutoffs).map(|(st, c)| idx.iter().filter(|&&i| c.is_some_and(|c| st.of(&reported[i].scored).is_some_and(|z| z > c))).count()).collect();
            (name, idx.len(), flagged)
        })
        .collect();
    FlagTable { fpr, authentic: authentic.len(), allowed: (fpr * authentic.len() as f64).floor() as usize, cutoffs, rows }
}

/// How well the grid windows above a cutoff match the masks of one slice.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Localisation {
    pub slice: String,
    pub images: usize,
    pub windows: usize,
    /// Windows the mask covers by more than the stated share.
    pub edited: usize,
    pub flagged: usize,
    pub flagged_edited: usize,
    /// Flagged windows the mask touches without reaching the stated share.
    pub flagged_touching: usize,
    /// Images with at least one edited window.
    pub with_edit: usize,
    /// Images in which a flagged window is an edited one.
    pub hit: usize,
    /// Images whose highest-scoring window is an edited one (needs no cutoff).
    pub top_on_edit: usize,
    /// Sum over those images of the share of their windows that are edited:
    /// what `top_on_edit` would be by chance.
    pub chance: f64,
}

impl Localisation {
    /// Share of flagged windows that are edited.
    pub fn precision(&self) -> f64 {
        self.flagged_edited as f64 / self.flagged as f64
    }
    /// Share of edited windows that are flagged.
    pub fn recall(&self) -> f64 {
        self.flagged_edited as f64 / self.edited as f64
    }
}

/// Window-level localisation per slice, flagging grid windows above `cut`.
pub fn localise(entries: &[Entry], cut: f32, o: &ReportOptions) -> Vec<Localisation> {
    let (_, reported) = partition(entries, o);
    slices(&reported)
        .into_iter()
        .map(|(slice, idx)| {
            let mut l = Localisation { slice, images: idx.len(), ..Default::default() };
            for &i in &idx {
                let e = reported[i];
                let cover: Vec<f32> = e.scored.grid.iter().map(|(r, _)| e.mask.as_ref().map_or(0.0, |m| m.coverage(*r))).collect();
                let edited = cover.iter().filter(|&&c| c > o.overlap).count();
                l.windows += cover.len();
                l.edited += edited;
                let mut hit = false;
                for ((_, z), &c) in e.scored.grid.iter().zip(&cover) {
                    if *z > cut {
                        l.flagged += 1;
                        l.flagged_edited += (c > o.overlap) as usize;
                        l.flagged_touching += (c > 0.0 && c <= o.overlap) as usize;
                        hit |= c > o.overlap;
                    }
                }
                l.hit += hit as usize;
                if edited > 0 {
                    l.with_edit += 1;
                    l.chance += edited as f64 / cover.len() as f64;
                    let top = (0..cover.len()).max_by(|&a, &b| e.scored.grid[a].1.total_cmp(&e.scored.grid[b].1)).unwrap();
                    l.top_on_edit += (cover[top] > o.overlap) as usize;
                }
            }
            l
        })
        .collect()
}

fn pct(k: usize, n: usize) -> String {
    if n == 0 {
        "–".into()
    } else {
        format!("{k}/{n} ({:.1}%)", 100.0 * k as f64 / n as f64)
    }
}

/// A rate such as 0.001 as `0.1`.
fn percent(rate: f64) -> String {
    let s = format!("{:.3}", rate * 100.0);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn ratio(v: f64) -> String {
    if v.is_finite() {
        format!("{:.1}%", 100.0 * v)
    } else {
        "–".into()
    }
}

/// The report as markdown: per-slice flag rates under every pooling rule at
/// equal authentic false-positive rates, then localisation from the grid.
pub fn report(entries: &[Entry], o: &ReportOptions) -> String {
    let stats = stats_for(entries, o.top_k);
    let mut out = String::new();
    let mut line = |s: String| {
        out.push_str(&s);
        out.push('\n');
    };
    let (authentic, reported) = partition(entries, o);
    line(format!("Partial edits: {} scored images, {} reported, cutoffs set on {} authentic `{}` images{}.", entries.len(), reported.len(), authentic.len(), o.cutoff_on, if o.disjoint { " of the other half of the parents" } else { " (the same images the `parent` row reports: in-sample)" }));
    line("Every pooling rule is operated at its own cutoff, the one that lets the stated share of those authentic images through, so the columns compare at equal authentic FPR. Cells are images flagged.".into());
    if stats.is_empty() || authentic.is_empty() {
        line(format!("\nNothing to compare: no `{}` images to set cutoffs on, or no pooling rule every image has a value for.", o.cutoff_on));
        return out;
    }
    for &fpr in &o.fprs {
        let t = flag_table(entries, &stats, fpr, o);
        line(format!("\n## Images flagged at {}% authentic FPR\n", percent(fpr)));
        line(format!("Cutoffs may flag {} of {} authentic images.{}", t.allowed, t.authentic, if t.allowed == 0 { format!(" With {} authentic images this rate is not resolved: the cutoff is the highest authentic score, and all that is shown is a rate under 1/{}.", t.authentic, t.authentic) } else { String::new() }));
        line(format!("\n| slice | images | {} |", stats.iter().map(|s| s.name()).collect::<Vec<_>>().join(" | ")));
        line(format!("|---|---:|{}", "---:|".repeat(stats.len())));
        line(format!("| cutoff (logit) | | {} |", t.cutoffs.iter().map(|c| c.map_or("–".into(), |c| format!("{c:.3}"))).collect::<Vec<_>>().join(" | ")));
        for (name, n, flagged) in &t.rows {
            line(format!("| {name} | {n} | {} |", flagged.iter().map(|&k| pct(k, *n)).collect::<Vec<_>>().join(" | ")));
        }
    }
    if stats.contains(&Stat::GridMax) {
        line(format!("\n## Localisation from the grid windows\n\nA window counts as edited when the mask covers more than {:.0}% of it. A window is flagged when its logit is above the `grid max` cutoff, so an authentic image has a flagged window at the stated rate. \"Top window on the edit\" needs no cutoff: the share of edited images whose highest-scoring window is an edited one, beside what picking a window at random would give.", o.overlap * 100.0));
        for &fpr in &o.fprs {
            let Some(cut) = cutoff(&authentic, Stat::GridMax, fpr) else { continue };
            line(format!("\n### Window cutoff {cut:.3} ({}% authentic FPR for `grid max`)\n", percent(fpr)));
            line("| slice | images | windows | edited | flagged | flagged and edited | flagged, touching the edit | precision | recall | images with a flagged edited window | top window on the edit | by chance |".into());
            line("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|".into());
            for l in localise(entries, cut, o) {
                line(format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                    l.slice,
                    l.images,
                    l.windows,
                    l.edited,
                    l.flagged,
                    l.flagged_edited,
                    l.flagged_touching,
                    ratio(l.precision()),
                    ratio(l.recall()),
                    pct(l.hit, l.with_edit),
                    pct(l.top_on_edit, l.with_edit),
                    ratio(l.chance / l.with_edit as f64)
                ));
            }
        }
    }
    out
}

/// The separate per-item output: the global score as it is today, and the
/// local scores beside it. One row per reported image:
///
/// `path transform kind parent_id generated global_prob global_logit
/// views_mean grid_max grid_top_k windows edited_area flagged top_window
/// top_window_coverage`
///
/// `flagged` lists the grid windows above the `grid max` cutoff of the first
/// `--fpr` as `x,y,w,h,logit,coverage`.
pub fn per_item(entries: &[Entry], o: &ReportOptions) -> String {
    let (authentic, reported) = partition(entries, o);
    let cut = o.fprs.first().and_then(|&f| cutoff(&authentic, Stat::GridMax, f));
    let mut out = String::from("# path\ttransform\tkind\tparent_id\tgenerated\tglobal_prob\tglobal_logit\tviews_mean\tgrid_max\tgrid_top_k\twindows\tedited_area\tflagged\ttop_window\ttop_window_coverage\n");
    let num = |v: Option<f32>| v.map_or("-".to_string(), |v| format!("{v:.5}"));
    for e in reported {
        let s = &e.scored;
        let cover = |r: Rect| e.mask.as_ref().map_or(0.0, |m| m.coverage(r));
        let flagged: Vec<String> = s.grid.iter().filter(|(_, z)| cut.is_some_and(|c| *z > c)).map(|(r, z)| format!("{},{z:.5},{:.3}", r.text(), cover(*r))).collect();
        let top = s.grid.iter().max_by(|a, b| a.1.total_cmp(&b.1));
        let (global_prob, global_logit) = s.deployed.map_or((s.prob, Stat::ViewsMean.of(s)), |d| (d.0, Some(d.2)));
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{:.6}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            s.path,
            e.transform,
            e.kind,
            e.parent,
            e.generated as u8,
            global_prob,
            num(global_logit),
            num(Stat::ViewsMean.of(s)),
            num(Stat::GridMax.of(s)),
            num(Stat::GridTop(o.top_k.max(1)).of(s)),
            s.grid.len(),
            e.mask.as_ref().map_or("-".to_string(), |m| format!("{:.4}", m.area())),
            if flagged.is_empty() { "-".to_string() } else { flagged.join(";") },
            top.map_or("-".to_string(), |(r, z)| format!("{},{z:.5}", r.text())),
            top.map_or("-".to_string(), |(r, _)| format!("{:.3}", cover(*r))),
        ));
    }
    out
}

/// How the seam of a pasted region is finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blend {
    /// The region replaces the pixels under it outright.
    Hard,
    /// The region fades in over a band inside its border.
    Feather,
}

impl Blend {
    pub fn name(self) -> &'static str {
        match self {
            Blend::Hard => "hard",
            Blend::Feather => "feather",
        }
    }

    pub fn parse_list(s: &str) -> Result<Vec<Blend>> {
        s.split(',')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(|t| match t {
                "hard" => Ok(Blend::Hard),
                "feather" => Ok(Blend::Feather),
                other => bail!("unknown blend `{other}` (use hard, feather)"),
            })
            .collect()
    }
}

/// Blend weights (0 to 255) of a `w x h` region: all 255 for a hard edge; for
/// a feathered one a linear ramp over a band a tenth of the shorter side
/// wide (4 to 64 pixels) inside the border.
pub fn region_alpha(w: u32, h: u32, blend: Blend) -> Vec<u8> {
    let band = (w.min(h) / 10).clamp(4, 64) as f32;
    let mut a = vec![255u8; (w * h) as usize];
    if blend == Blend::Feather {
        for y in 0..h {
            for x in 0..w {
                let d = x.min(w - 1 - x).min(y).min(h - 1 - y) as f32;
                a[(y * w + x) as usize] = (255.0 * ((d + 0.5) / band).min(1.0)).round() as u8;
            }
        }
    }
    a
}

/// Paste `patch` (the size of `at`) into `base` with the given blend
/// weights, in place.
pub fn paste(base: &mut Rgb, patch: &Rgb, at: Rect, alpha: &[u8]) {
    for y in 0..at.h {
        for x in 0..at.w {
            let a = alpha[(y * at.w + x) as usize] as u32;
            let (d, s) = ((((at.y + y) * base.w + at.x + x) * 3) as usize, ((y * at.w + x) * 3) as usize);
            for c in 0..3 {
                base.data[d + c] = ((patch.data[s + c] as u32 * a + base.data[d + c] as u32 * (255 - a) + 127) / 255) as u8;
            }
        }
    }
}

/// The full-image mask of a region pasted at `at` into a `w x h` image.
pub fn full_mask(w: u32, h: u32, at: Rect, alpha: &[u8]) -> Vec<u8> {
    let mut m = vec![0u8; (w * h) as usize];
    for y in 0..at.h {
        let d = ((at.y + y) * w + at.x) as usize;
        m[d..d + at.w as usize].copy_from_slice(&alpha[(y * at.w) as usize..((y + 1) * at.w) as usize]);
    }
    m
}

/// A region covering about `area` of a `w x h` image, with the image's own
/// aspect ratio varied by up to a third either way, at a random position.
pub fn random_region(w: u32, h: u32, area: f32, rng: &mut crate::rng::Rng) -> Rect {
    let side = area.clamp(0.0001, 0.9).sqrt();
    let stretch = rng.range(0.75, 1.3333);
    let rw = ((w as f32 * side * stretch).round() as u32).clamp(8.min(w), w);
    let rh = ((w as f32 * h as f32 * area / rw as f32).round() as u32).clamp(8.min(h), h);
    Rect { x: rng.below((w - rw + 1) as usize) as u32, y: rng.below((h - rh + 1) as usize) as u32, w: rw, h: rh }
}

/// Where in the same image a self-paste takes its pixels from: a region of
/// the same size that does not overlap `at` when one exists, otherwise the
/// candidate that overlaps it least.
pub fn self_source(w: u32, h: u32, at: Rect, rng: &mut crate::rng::Rng) -> Rect {
    let overlap = |r: &Rect| {
        let ox = (r.x + r.w).min(at.x + at.w).saturating_sub(r.x.max(at.x));
        let oy = (r.y + r.h).min(at.y + at.h).saturating_sub(r.y.max(at.y));
        ox as u64 * oy as u64
    };
    let mut best = Rect { x: w - at.w - at.x, y: h - at.h - at.y, ..at };
    for _ in 0..64 {
        if overlap(&best) == 0 {
            break;
        }
        let r = Rect { x: rng.below((w - at.w + 1) as usize) as u32, y: rng.below((h - at.h + 1) as usize) as u32, ..at };
        if overlap(&r) < overlap(&best) {
            best = r;
        }
    }
    best
}

/// The JPEG quality (1 to 100, libjpeg's scale) whose standard luminance
/// table is closest to the one in `bytes`, or `None` when the file has no
/// 8-bit luminance table. Encoders with their own tables get the nearest
/// standard quality, which is the best a standard encoder can do.
pub fn jpeg_quality(bytes: &[u8]) -> Option<u8> {
    // The standard luminance table sums to this (ITU T.81 Annex K).
    const STANDARD_SUM: f64 = 3688.0;
    let mut i = 2;
    while i + 4 <= bytes.len() && bytes[i] == 0xFF {
        let (marker, len) = (bytes[i + 1], u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize);
        if marker == 0xDA {
            break;
        }
        if marker == 0xDB {
            let mut seg = bytes.get(i + 4..i + 2 + len)?;
            while let Some((&head, rest)) = seg.split_first() {
                let wide = head >> 4 != 0;
                let n = if wide { 128 } else { 64 };
                let table = rest.get(..n)?;
                if head & 15 == 0 && !wide {
                    let scale = 100.0 * table.iter().map(|&v| v as f64).sum::<f64>() / STANDARD_SUM;
                    let q = if scale <= 100.0 { (200.0 - scale) / 2.0 } else { 5000.0 / scale };
                    return Some(q.round().clamp(1.0, 100.0) as u8);
                }
                seg = &rest[n..];
            }
        }
        i += 2 + len;
    }
    None
}

/// The format a composite is saved in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Saved {
    Jpeg(u8),
    Png,
}

impl Saved {
    /// The parent's own format where this encoder can write it: JPEG at the
    /// parent's estimated quality, or PNG. `None` for anything else (lossy
    /// WebP, GIF, TIFF), which cannot be re-saved alike.
    pub fn like(bytes: &[u8]) -> Option<Saved> {
        match bytes {
            [0xFF, 0xD8, ..] => Some(Saved::Jpeg(jpeg_quality(bytes).unwrap_or(90))),
            [0x89, b'P', b'N', b'G', ..] => Some(Saved::Png),
            _ => None,
        }
    }

    pub fn ext(self) -> &'static str {
        match self {
            Saved::Jpeg(_) => "jpg",
            Saved::Png => "png",
        }
    }

    pub fn name(self) -> String {
        match self {
            Saved::Jpeg(q) => format!("jpeg-q{q}"),
            Saved::Png => "png".into(),
        }
    }

    pub fn encode(self, img: &Rgb) -> Result<Vec<u8>> {
        use image::ImageEncoder;
        let mut buf = Vec::new();
        match self {
            Saved::Jpeg(q) => image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, q).encode(&img.data, img.w, img.h, image::ExtendedColorType::Rgb8)?,
            Saved::Png => image::codecs::png::PngEncoder::new(&mut buf).write_image(&img.data, img.w, img.h, image::ExtendedColorType::Rgb8)?,
        }
        Ok(buf)
    }
}

fn write_mask(path: &Path, w: u32, h: u32, mask: &[u8]) -> Result<()> {
    use image::ImageEncoder;
    let mut buf = Vec::new();
    image::codecs::png::PngEncoder::new(&mut buf).write_image(mask, w, h, image::ExtendedColorType::L8)?;
    std::fs::write(path, buf).with_context(|| format!("writing {}", path.display()))
}

/// How `paste-edits` builds a store.
pub struct PasteOptions {
    /// Authentic parents to use.
    pub parents: usize,
    /// Further authentic images of the same kind, stored as `parent` rows
    /// only: more authentic images to set cutoffs on.
    pub extra_parents: usize,
    /// Shares of the image the pasted region covers.
    pub areas: Vec<f32>,
    pub blends: Vec<Blend>,
    pub seed: u64,
    /// Parents must have a shorter side of at least this many pixels and a
    /// longer side of at most `max_side`.
    pub min_side: u32,
    pub max_side: u32,
    /// Dataset name written to the manifest.
    pub dataset: String,
    /// Save everything derived as PNG instead of in the parent's format.
    pub png: bool,
    pub threads: usize,
}

/// What a parent yields: manifest rows and edit rows, in order.
type Rows = (Vec<String>, Vec<String>);

/// A crop of `size` from a random donor that is large enough, with the
/// donor's path and generator. Pixels are taken at native resolution: a
/// resized donor would not carry its generator's traces.
fn donor_patch(donors: &[Item], size: (u32, u32), skip: Option<&Path>, rng: &mut crate::rng::Rng) -> Option<(Rgb, PathBuf, String, Option<String>)> {
    for _ in 0..24 {
        let d = &donors[rng.below(donors.len())];
        if skip == Some(d.path.as_path()) {
            continue;
        }
        let Ok(img) = image_io::load(&d.path) else { continue };
        if img.w < size.0 || img.h < size.1 {
            continue;
        }
        let (x, y) = (rng.below((img.w - size.0 + 1) as usize), rng.below((img.h - size.1 + 1) as usize));
        return Some((img.crop(x as i64, y as i64, size.0, size.1), d.path.clone(), d.generator.clone(), d.generator_family().map(str::to_string)));
    }
    None
}

/// Everything derived from one parent, written under `out`. `None` when the
/// parent does not qualify (unreadable, the wrong size, or a format that
/// cannot be re-saved alike).
fn derive(parent: &Item, index: usize, real: &[Item], fake: &[Item], out: &Path, o: &PasteOptions) -> Result<Option<(Rows, Rows)>> {
    let Ok(bytes) = std::fs::read(&parent.path) else { return Ok(None) };
    let Some(like) = Saved::like(&bytes) else { return Ok(None) };
    let Ok(img) = image_io::decode(&bytes) else { return Ok(None) };
    if img.w.min(img.h) < o.min_side || img.w.max(img.h) > o.max_side {
        return Ok(None);
    }
    let saved = if o.png { Saved::Png } else { like };
    let mut rng = crate::rng::Rng::new(o.seed ^ fnv(&parent.path.display().to_string()));
    let key = format!("{index:05}");
    let parent_id = parent.parent_id().map(str::to_string).unwrap_or_else(|| {
        // Store, shard and file: two stores can share a dataset name and their shard names.
        let tail: Vec<String> = parent.path.components().rev().take(3).map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
        tail.into_iter().rev().collect::<Vec<_>>().join("/")
    });
    std::fs::create_dir_all(out.join("img").join(&key))?;
    std::fs::create_dir_all(out.join("masks").join(&key))?;
    let (mut manifest, mut edits) = (Vec::new(), Vec::new());
    let row = |manifest: &mut Vec<String>, rel: &str, label: u8, generator: &str, family: Option<&str>, transform: &str| {
        let mut it = Item { path: rel.into(), label, generator: generator.to_string(), dataset: o.dataset.clone(), extra: parent.extra.clone() };
        it.set_group(0, &parent_id);
        it.set_group(3, family.unwrap_or(""));
        it.set_group(4, transform);
        manifest.push(it.manifest_line(rel));
    };
    // The parent itself, byte for byte.
    let rel = format!("img/{key}/parent.{}", like.ext());
    std::fs::write(out.join(&rel), &bytes)?;
    row(&mut manifest, &rel, 0, &parent.generator, None, "parent");
    edits.push(format!("{rel}\t-\tparent\t0\t-\t-\t-\t-\t0\t{}\t-", parent.path.display()));
    // What an extra parent contributes: this row alone.
    let alone = (manifest.clone(), edits.clone());
    // The same encode path with nothing pasted.
    let rel = format!("img/{key}/resave.{}", saved.ext());
    std::fs::write(out.join(&rel), saved.encode(&img)?)?;
    row(&mut manifest, &rel, 0, &parent.generator, None, "resave");
    edits.push(format!("{rel}\t-\tresave\t0\t-\t-\t-\t-\t0\t{}\t{}", parent.path.display(), saved.name()));
    for &area in &o.areas {
        // One region per area, shared by every kind of paste and every blend, so the slices differ only in what was pasted.
        let at = random_region(img.w, img.h, area, &mut rng);
        let from = self_source(img.w, img.h, at, &mut rng);
        let generated = donor_patch(fake, (at.w, at.h), None, &mut rng);
        let authentic = donor_patch(real, (at.w, at.h), Some(&parent.path), &mut rng);
        for &blend in &o.blends {
            let alpha = region_alpha(at.w, at.h, blend);
            let mask = full_mask(img.w, img.h, at, &alpha);
            let covered = mask.iter().map(|&v| v as f64).sum::<f64>() / (255.0 * mask.len() as f64);
            let tag = format!("{}-area{area}", blend.name());
            let mask_rel = format!("masks/{key}/{tag}.png");
            write_mask(&out.join(&mask_rel), img.w, img.h, &mask)?;
            let own = img.crop(from.x as i64, from.y as i64, at.w, at.h);
            let kinds: [(&str, Option<(&Rgb, String, String, Option<String>)>); 3] = [
                ("self-paste", Some((&own, format!("{}@{},{}", parent.path.display(), from.x, from.y), parent.generator.clone(), None))),
                ("authentic-paste", authentic.as_ref().map(|(p, path, g, _)| (p, path.display().to_string(), g.clone(), None))),
                ("generated-paste", generated.as_ref().map(|(p, path, g, f)| (p, path.display().to_string(), g.clone(), Some(f.clone().unwrap_or_else(|| g.clone()))))),
            ];
            for (kind, source) in kinds {
                let Some((patch, donor, donor_generator, family)) = source else { continue };
                let mut composite = img.clone();
                paste(&mut composite, patch, at, &alpha);
                let rel = format!("img/{key}/{kind}-{tag}.{}", saved.ext());
                std::fs::write(out.join(&rel), saved.encode(&composite)?)?;
                let is_generated = kind == "generated-paste";
                let generator = if is_generated { format!("partial:{donor_generator}") } else { parent.generator.clone() };
                row(&mut manifest, &rel, is_generated as u8, &generator, family.as_deref(), &format!("{kind}-{tag}"));
                edits.push(format!("{rel}\t{mask_rel}\t{kind}\t{}\t{}\t{}\t{}\t{}\t{covered:.4}\t{donor}\t{donor_generator}", is_generated as u8, at.x, at.y, at.w, at.h));
            }
        }
    }
    Ok(Some(((manifest, edits), alone)))
}

/// Build a pasted-region store under `out` from authentic images (`real`,
/// label 0) and generated ones (`fake`, label 1). Each parent yields, as
/// separately named `transform` slices:
///
/// - `parent`: the original file, byte for byte;
/// - `resave`: the parent through the encoder every composite goes through,
///   nothing pasted;
/// - for each area and blend, with the region in the same place:
///   `self-paste` (pixels from elsewhere in the same image),
///   `authentic-paste` (pixels from another authentic image) and
///   `generated-paste` (pixels from a generated image, at native resolution).
///
/// Only `generated-paste` contains generated pixels and is labelled 1. The
/// others are the controls: a detector that flags them is reacting to the
/// seam or to the re-save. Composites are saved in the parent's format
/// (JPEG at its estimated quality, or PNG). After the parents, up to
/// `extra_parents` further qualifying images are stored as `parent` rows
/// alone. Returns `(parents, items)`.
pub fn paste_edits(real: &[Item], fake: &[Item], out: &Path, o: &PasteOptions) -> Result<(usize, usize)> {
    use rayon::prelude::*;
    ensure!(real.iter().all(|i| i.label == 0) && !real.is_empty(), "--real needs authentic items only (label 0)");
    ensure!(fake.iter().all(|i| i.label == 1) && !fake.is_empty(), "--fake needs generated items only (label 1)");
    ensure!(!o.areas.is_empty() && o.areas.iter().all(|a| *a > 0.0 && *a < 0.9), "--areas are shares of the image between 0 and 0.9");
    ensure!(!o.blends.is_empty(), "--blends needs at least one of hard, feather");
    ensure!(!out.join("manifest.tsv").exists(), "{} already holds a store; choose a new directory", out.display());
    std::fs::create_dir_all(out)?;
    let mut order: Vec<usize> = (0..real.len()).collect();
    crate::rng::Rng::new(o.seed).shuffle(&mut order);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(o.threads.max(1)).build()?;
    let (mut manifest, mut edits, mut parents) = (Vec::new(), vec!["# path\tmask\tkind\tgenerated\tx\ty\tw\th\tarea\tdonor\tdonor_generator".to_string()], 0usize);
    // Candidates are tried in shuffled order, a batch at a time, until enough qualify.
    // A parent's directory is named after its place in that order, whatever the thread count.
    let batch = (o.threads.max(1) * 4).max(8);
    let mut extra = 0usize;
    for (c, chunk) in order.chunks(batch).enumerate() {
        if parents >= o.parents && extra >= o.extra_parents {
            break;
        }
        let done: Vec<Result<Option<(Rows, Rows)>>> = pool.install(|| chunk.par_iter().enumerate().map(|(k, &i)| derive(&real[i], c * batch + k, real, fake, out, o)).collect());
        for (k, r) in done.into_iter().enumerate() {
            let key = format!("{:05}", c * batch + k);
            let Some(((m, e), alone)) = r? else { continue };
            if parents < o.parents {
                manifest.extend(m);
                edits.extend(e);
                parents += 1;
                continue;
            }
            // A candidate past the number asked for: keep its parent row at most, and remove the rest of what it wrote.
            let keep = if extra < o.extra_parents { alone.0.first().and_then(|l| l.split('\t').next()).map(|rel| out.join(rel)) } else { None };
            for dir in ["img", "masks"] {
                for f in walkdir::WalkDir::new(out.join(dir).join(&key)).into_iter().filter_map(|f| f.ok()).filter(|f| f.file_type().is_file()) {
                    if Some(f.path()) != keep.as_deref() {
                        let _ = std::fs::remove_file(f.path());
                    }
                }
            }
            let _ = std::fs::remove_dir(out.join("masks").join(&key));
            if keep.is_some() {
                manifest.extend(alone.0);
                edits.extend(alone.1);
                extra += 1;
            } else {
                let _ = std::fs::remove_dir(out.join("img").join(&key));
            }
        }
    }
    ensure!(parents > 0, "no authentic image qualified (JPEG or PNG, shorter side at least {}, longer side at most {})", o.min_side, o.max_side);
    std::fs::write(out.join("manifest.tsv"), manifest.join("\n") + "\n")?;
    std::fs::write(out.join("edits.tsv"), edits.join("\n") + "\n")?;
    Ok((parents, manifest.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_spans_the_image_and_respects_the_stride_and_the_cap() {
        // 640 x 448 at stride 112: origins 0..416 and 0..224, edge to edge.
        let g = grid_windows(640, 448, 224, 112, 1000);
        assert_eq!(g.len(), 5 * 3);
        assert_eq!((g[0].x, g[0].y), (0, 0));
        assert_eq!((g.last().unwrap().x, g.last().unwrap().y), (640 - 224, 448 - 224));
        assert!(spacing(&g) <= 112);
        // An awkward size still ends flush with the border and never steps further than asked.
        let g = grid_windows(1000, 700, 224, 224, 1000);
        assert!(g.iter().all(|r| r.x + r.w <= 1000 && r.y + r.h <= 700));
        assert!(g.iter().any(|r| r.x + r.w == 1000) && g.iter().any(|r| r.y + r.h == 700));
        assert!(spacing(&g) <= 224);
        // A cap thins the grid but keeps the corners.
        let g = grid_windows(4000, 3000, 224, 112, 40);
        assert!(g.len() <= 40 && g.len() > 20, "{}", g.len());
        assert!(g.contains(&Rect { x: 0, y: 0, w: 224, h: 224 }) && g.contains(&Rect { x: 4000 - 224, y: 3000 - 224, w: 224, h: 224 }));
        assert!(spacing(&g) > 224);
        // Exactly the crop: one window. Smaller: the centre square.
        assert_eq!(grid_windows(224, 224, 224, 112, 9), vec![Rect { x: 0, y: 0, w: 224, h: 224 }]);
        assert_eq!(grid_windows(200, 120, 224, 112, 9), vec![Rect { x: 40, y: 0, w: 120, h: 120 }]);
    }

    #[test]
    fn view_boxes_are_where_the_views_crop() {
        let mut img = Rgb::new(640, 480);
        for (i, v) in img.data.iter_mut().enumerate() {
            *v = (i * 31 % 251) as u8;
        }
        let rich = image_io::rich_windows(&img, 224, 4);
        for spec in [ViewSpec::Native { crop: 224 }, ViewSpec::NativeRich { crop: 224, rank: 2 }, ViewSpec::NativeAt { crop: 224, fx: 800, fy: 200 }] {
            let (_, b) = view_box(spec, img.w, img.h, &rich);
            let view: View = spec.into();
            assert_eq!(view.apply_with(&img, Some(&rich)).data, img.crop(b.x as i64, b.y as i64, 224, 224).data, "{spec:?}");
        }
        // The whole-image view shows the centre square, scaled.
        assert_eq!(view_box(ViewSpec::Fit { resize: 256, crop: 224 }, 640, 480, &rich), ("fit".to_string(), Rect { x: 110, y: 30, w: 420, h: 420 }));
        // A small image: every native view is the upscaled centre square.
        assert_eq!(view_box(ViewSpec::NativeRich { crop: 224, rank: 0 }, 200, 120, &[]).1, Rect { x: 40, y: 0, w: 120, h: 120 });
    }

    #[test]
    fn a_grid_window_of_a_crop_sized_image_passes_through_the_native_view_untouched() {
        // `score_windows` relies on this: ready-made crops are fed through `Native`.
        let mut img = Rgb::new(224, 224);
        for (i, v) in img.data.iter_mut().enumerate() {
            *v = (i * 7 % 253) as u8;
        }
        assert_eq!(View::Native { crop: 224 }.apply(&img).data, img.data);
    }

    #[test]
    fn pooling_rules() {
        assert_eq!(top_k_mean(&[1.0, 5.0, 3.0, -2.0], 2), Some(4.0));
        assert_eq!(top_k_mean(&[1.0, 5.0], 3), Some(3.0));
        assert_eq!(top_k_mean(&[], 3), None);
        let s = Scored { view_logits: vec![0.0, 2.0, 4.0], grid: vec![(Rect { x: 0, y: 0, w: 1, h: 1 }, -1.0), (Rect { x: 0, y: 0, w: 1, h: 1 }, 7.0)], deployed: Some((0.9, 1, 4.0)), ..Default::default() };
        assert_eq!((Stat::Deployed.of(&s), Stat::ViewsMean.of(&s), Stat::ViewsMax.of(&s), Stat::ViewsTop(2).of(&s)), (Some(4.0), Some(2.0), Some(4.0), Some(3.0)));
        assert_eq!((Stat::GridMean.of(&s), Stat::GridMax.of(&s), Stat::GridTop(3).of(&s)), (Some(3.0), Some(7.0), Some(3.0)));
        assert_eq!(Stat::GridMax.of(&Scored::default()), None);
    }

    #[test]
    fn a_score_row_round_trips_and_reads_as_an_all_views_score_row() {
        let line = "/d/a.jpg\t1\tpartial:flux\tpartial-paste\t0.812345\t1.50000\t-0.25000\tsize=640x448\tdeployed=0.900000/2/1.75000\tviews=fit,96,0,392,392;native,208,112,224,224\tgrid=0,0,224,224,-3.10000;416,224,224,224,6.20000";
        let s = parse_scored(line).unwrap();
        assert_eq!((s.label, s.prob, s.view_logits.clone(), s.size), (1, 0.812345, vec![1.5, -0.25], (640, 448)));
        assert_eq!(s.deployed, Some((0.9, 2, 1.75)));
        assert_eq!(s.views[1], ("native".to_string(), Rect { x: 208, y: 112, w: 224, h: 224 }));
        assert_eq!(s.grid, vec![(Rect { x: 0, y: 0, w: 224, h: 224 }, -3.1), (Rect { x: 416, y: 224, w: 224, h: 224 }, 6.2)]);
        // `report` reads the same file: the logits are the views', nothing else parses as one.
        let d = std::env::temp_dir().join(format!("tintype-partial-row-{}", std::process::id()));
        std::fs::write(&d, format!("{line}\n")).unwrap();
        let read = crate::report::read_scores(&d).unwrap();
        assert_eq!(read["/d/a.jpg"].logits, vec![1.5, -0.25]);
        let _ = std::fs::remove_file(&d);
        // A plain `score --all-views` row reads too, with no grid.
        let plain = parse_scored("/d/b.png\t0\treal:x\tset\t0.010000\t-4.00000\t-5.00000").unwrap();
        assert!(plain.grid.is_empty() && plain.deployed.is_none() && plain.view_logits.len() == 2);
    }

    #[test]
    fn mask_coverage_is_the_replaced_share_of_a_window() {
        // A 100 x 80 image with a hard 40 x 20 region at (10, 30).
        let at = Rect { x: 10, y: 30, w: 40, h: 20 };
        let m = Mask::new(100, 80, &full_mask(100, 80, at, &region_alpha(40, 20, Blend::Hard)));
        assert!((m.area() - 0.1).abs() < 1e-6);
        assert_eq!(m.coverage(at), 1.0);
        assert_eq!(m.coverage(Rect { x: 60, y: 0, w: 40, h: 80 }), 0.0);
        // Half of this window lies on the region.
        assert!((m.coverage(Rect { x: 30, y: 30, w: 40, h: 20 }) - 0.5).abs() < 1e-6);
        // A window hanging over the border counts the outside as untouched.
        assert!((m.coverage(Rect { x: 10, y: 30, w: 40, h: 100 }) - 0.2).abs() < 1e-6);
        // A feathered region covers less than its rectangle and fades at the border.
        let alpha = region_alpha(40, 20, Blend::Feather);
        assert!(alpha[0] < 64 && alpha[20 * 40 / 2 + 20] == 255);
        let soft = Mask::new(100, 80, &full_mask(100, 80, at, &alpha));
        assert!(soft.area() < m.area() && soft.area() > 0.05);
    }

    #[test]
    fn pasting_replaces_only_the_region_and_blends_at_a_feathered_seam() {
        let mut base = Rgb::new(64, 48);
        base.data.fill(10);
        let mut patch = Rgb::new(40, 20);
        patch.data.fill(210);
        let at = Rect { x: 8, y: 16, w: 40, h: 20 };
        let mut hard = base.clone();
        paste(&mut hard, &patch, at, &region_alpha(40, 20, Blend::Hard));
        let px = |im: &Rgb, x: u32, y: u32| im.data[((y * im.w + x) * 3) as usize];
        assert_eq!((px(&hard, 8, 16), px(&hard, 47, 35), px(&hard, 7, 16), px(&hard, 48, 35)), (210, 210, 10, 10));
        let mut soft = base.clone();
        paste(&mut soft, &patch, at, &region_alpha(40, 20, Blend::Feather));
        // Outside untouched, the border in between, the middle fully replaced.
        assert_eq!(px(&soft, 7, 16), 10);
        assert!(px(&soft, 8, 16) > 10 && px(&soft, 8, 16) < 210);
        assert_eq!(px(&soft, 28, 26), 210);
    }

    #[test]
    fn regions_have_the_requested_area_and_self_sources_avoid_them() {
        let mut rng = crate::rng::Rng::new(3);
        for area in [0.02f32, 0.05, 0.12, 0.25] {
            for _ in 0..20 {
                let r = random_region(900, 600, area, &mut rng);
                assert!(r.x + r.w <= 900 && r.y + r.h <= 600);
                let got = (r.w * r.h) as f32 / (900.0 * 600.0);
                assert!((got - area).abs() < 0.01 * area.max(0.05), "{area}: {got}");
                let s = self_source(900, 600, r, &mut rng);
                assert!(s.x + s.w <= 900 && s.y + s.h <= 600 && (s.w, s.h) == (r.w, r.h));
                // A small region always leaves room for a source beside it; a large one may not, but the source is never the region itself.
                let apart = s.x + s.w <= r.x || r.x + r.w <= s.x || s.y + s.h <= r.y || r.y + r.h <= s.y;
                assert!(apart || area > 0.05, "{r:?} {s:?}");
                assert_ne!((s.x, s.y), (r.x, r.y), "{r:?}");
            }
        }
    }

    #[test]
    fn jpeg_quality_is_read_back_from_the_tables() {
        let mut img = Rgb::new(64, 64);
        for (i, v) in img.data.iter_mut().enumerate() {
            *v = (i * 13 % 256) as u8;
        }
        for q in [35u8, 50, 75, 90, 97] {
            let bytes = Saved::Jpeg(q).encode(&img).unwrap();
            let got = jpeg_quality(&bytes).unwrap();
            assert!(got.abs_diff(q) <= 1, "quality {q} read back as {got}");
            assert_eq!(Saved::like(&bytes), Some(Saved::Jpeg(got)));
        }
        let png = Saved::Png.encode(&img).unwrap();
        assert_eq!((jpeg_quality(&png), Saved::like(&png)), (None, Some(Saved::Png)));
        assert_eq!(Saved::like(b"RIFF\0\0\0\0WEBPVP8 "), None);
        // PNG is lossless: the pixels come back.
        assert_eq!(image_io::decode(&png).unwrap().data, img.data);
    }

    fn entry(parent: &str, transform: &str, kind: &str, generated: bool, views: f32, grid: &[f32], mask: Option<Mask>) -> Entry {
        // Two windows side by side in a 448 x 224 image.
        let scored = Scored { path: format!("{parent}/{transform}"), size: (448, 224), view_logits: vec![views], grid: grid.iter().enumerate().map(|(i, &z)| (Rect { x: 224 * i as u32, y: 0, w: 224, h: 224 }, z)).collect(), ..Default::default() };
        Entry { scored, transform: transform.into(), kind: kind.into(), parent: parent.into(), generated, mask }
    }

    fn right_half() -> Mask {
        Mask::new(448, 224, &full_mask(448, 224, Rect { x: 224, y: 0, w: 224, h: 224 }, &region_alpha(224, 224, Blend::Hard)))
    }

    #[test]
    fn pooling_rules_are_compared_at_the_same_authentic_rate() {
        // 100 authentic parents: view means 0..99 hundredths, grid maxima twice that.
        let mut entries: Vec<Entry> = (0..100).map(|i| entry(&format!("p{i}"), "parent", "parent", false, i as f32 / 100.0, &[i as f32 / 50.0, 0.0], None)).collect();
        // Ten edits the mean pooling misses and the max finds, ten it finds too.
        for i in 0..20 {
            let views = if i < 10 { 0.5 } else { 3.0 };
            entries.push(entry(&format!("p{i}"), "generated-paste-hard-area0.05", "generated-paste", true, views, &[0.0, 9.0], Some(right_half())));
        }
        // A control with no generated pixels that nothing should flag.
        for i in 0..20 {
            entries.push(entry(&format!("p{i}"), "self-paste-hard-area0.05", "self-paste", false, 0.1, &[0.2, 0.1], Some(right_half())));
        }
        let o = ReportOptions { fprs: vec![0.01], ..Default::default() };
        let stats = stats_for(&entries, 3);
        // No early-exit result in these rows, so that rule is left out.
        assert_eq!(stats, vec![Stat::ViewsMean, Stat::ViewsMax, Stat::ViewsTop(2), Stat::GridMean, Stat::GridMax, Stat::GridTop(3)]);
        let t = flag_table(&entries, &stats, 0.01, &o);
        assert_eq!((t.authentic, t.allowed), (100, 1));
        let col = |st: Stat| stats.iter().position(|s| *s == st).unwrap();
        let row = |name: &str| t.rows.iter().find(|r| r.0 == name).unwrap_or_else(|| panic!("no row {name}")).clone();
        // Every rule flags exactly its allowance of the authentic images ...
        assert!(row("parent").2.iter().all(|&k| k == 1), "{:?}", row("parent"));
        // ... and at that same rate the mean finds half of the edits, the max all of them.
        let edits = row("generated-paste-hard-area0.05");
        assert_eq!((edits.1, edits.2[col(Stat::ViewsMean)], edits.2[col(Stat::GridMax)]), (20, 10, 20));
        assert!(row("self-paste-hard-area0.05").2.iter().all(|&k| k == 0));
        // Controls come before edits.
        let names: Vec<&str> = t.rows.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(names, vec!["parent", "self-paste-hard-area0.05", "generated-paste-hard-area0.05"]);

        // Localisation: the right-hand window is the edited one, and it is the one flagged.
        let cut = t.cutoffs[col(Stat::GridMax)].unwrap();
        let loc = localise(&entries, cut, &o);
        let l = loc.iter().find(|l| l.slice == "generated-paste-hard-area0.05").unwrap();
        assert_eq!((l.images, l.windows, l.edited, l.flagged, l.flagged_edited, l.hit, l.top_on_edit), (20, 40, 20, 20, 20, 20, 20));
        assert_eq!((l.precision(), l.recall(), l.chance / l.with_edit as f64), (1.0, 1.0, 0.5));
        // The control has edited windows too; none is flagged, and its top window is the untouched one.
        let c = loc.iter().find(|l| l.slice == "self-paste-hard-area0.05").unwrap();
        assert_eq!((c.edited, c.flagged, c.top_on_edit), (20, 0, 0));
        // One authentic window is above the cutoff: a false flag.
        let p = loc.iter().find(|l| l.slice == "parent").unwrap();
        assert_eq!((p.flagged, p.edited, p.with_edit), (1, 0, 0));

        let text = report(&entries, &o);
        assert!(text.contains("## Images flagged at 1% authentic FPR") && text.contains("| generated-paste-hard-area0.05 | 20 | 10/20 (50.0%) |"), "{text}");
        assert!(text.contains("## Localisation from the grid windows"), "{text}");
        let rows = per_item(&entries, &o);
        assert_eq!(rows.lines().count(), 1 + entries.len());
        assert!(rows.lines().any(|l| l.starts_with("p0/generated-paste-hard-area0.05\t") && l.contains("\t224,0,224,224,9.00000,1.000\t")), "{rows}");
    }

    #[test]
    fn scores_manifest_and_edits_join_however_their_paths_were_written() {
        let dir = std::env::temp_dir().join(format!("tintype-partial-join-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("masks")).unwrap();
        write_mask(&dir.join("masks/m.png"), 448, 224, &full_mask(448, 224, Rect { x: 224, y: 0, w: 224, h: 224 }, &region_alpha(224, 224, Blend::Hard))).unwrap();
        std::fs::write(dir.join("edits.tsv"), "# path\tmask\tkind\tgenerated\nimg/a.jpg\tmasks/m.png\tgenerated-paste\t1\nimg/p.jpg\t-\tparent\t0\n").unwrap();
        let edits = read_edits(&dir.join("edits.tsv")).unwrap();
        // The manifest resolves to absolute paths; the score file was written from a path with a detour in it.
        let item = |name: &str, label: u8, transform: &str| {
            let mut it = Item { path: dir.join("img").join(name), label, generator: "g".into(), dataset: "d".into(), extra: vec![] };
            it.set_group(0, "p1");
            it.set_group(4, transform);
            it
        };
        let items = [item("a.jpg", 1, "generated-paste-hard-area0.5"), item("p.jpg", 0, "parent")];
        let scored = |name: &str| Scored { path: dir.join("img/./").join(name).display().to_string(), size: (448, 224), ..Default::default() };
        let (entries, unknown) = join(vec![scored("a.jpg"), scored("p.jpg"), scored("elsewhere.jpg")], &items, &edits).unwrap();
        assert_eq!((entries.len(), unknown), (2, 1));
        assert_eq!((entries[0].kind.as_str(), entries[0].generated, entries[0].parent.as_str()), ("generated-paste", true, "p1"));
        assert!((entries[0].mask.as_ref().unwrap().area() - 0.5).abs() < 1e-6);
        assert!(entries[1].mask.is_none() && entries[1].kind == "parent");
        // A mask of the wrong size is an error, not a silent zero.
        let wrong = Scored { size: (100, 100), ..scored("a.jpg") };
        assert!(join(vec![wrong], &items, &edits).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_parent_disjoint_report_sets_cutoffs_on_other_parents() {
        let mut entries: Vec<Entry> = Vec::new();
        for i in 0..200 {
            entries.push(entry(&format!("p{i}"), "parent", "parent", false, i as f32 / 100.0, &[0.0, 0.0], None));
            entries.push(entry(&format!("p{i}"), "generated-paste-hard-area0.05", "generated-paste", true, 5.0, &[0.0, 9.0], Some(right_half())));
        }
        let o = ReportOptions { fprs: vec![0.01], disjoint: true, ..Default::default() };
        let t = flag_table(&entries, &[Stat::ViewsMean], 0.01, &o);
        let reported = t.rows.iter().find(|r| r.0 == "parent").unwrap().1;
        // The two halves are disjoint and together are all parents; derivatives follow their parent.
        assert_eq!(t.authentic + reported, 200);
        assert!(t.authentic > 60 && reported > 60, "{} / {reported}", t.authentic);
        assert_eq!(t.rows.iter().find(|r| r.0.starts_with("generated")).unwrap().1, reported);
        assert!(report(&entries, &o).contains("of the other half of the parents"));
    }

    #[test]
    fn a_pasted_store_holds_every_slice_with_matching_masks() {
        let dir = std::env::temp_dir().join(format!("tintype-paste-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        let picture = |seed: usize, w: u32, h: u32| {
            let mut im = Rgb::new(w, h);
            for (i, v) in im.data.iter_mut().enumerate() {
                *v = ((i * (seed * 2 + 3)) % 256) as u8;
            }
            im
        };
        let (mut real, mut fake) = (Vec::new(), Vec::new());
        for k in 0..4 {
            // Two JPEG parents, two PNG ones, and one too small to qualify.
            let saved = if k % 2 == 0 { Saved::Jpeg(80) } else { Saved::Png };
            let p = dir.join("src").join(format!("real{k}.{}", saved.ext()));
            std::fs::write(&p, saved.encode(&picture(k, 480, 360)).unwrap()).unwrap();
            real.push(Item { path: p, label: 0, generator: "real:cam".into(), dataset: "src".into(), extra: vec![] });
            let p = dir.join("src").join(format!("fake{k}.png"));
            std::fs::write(&p, Saved::Png.encode(&picture(k + 9, 512, 512)).unwrap()).unwrap();
            fake.push(Item { path: p, label: 1, generator: "gen-x".into(), dataset: "src".into(), extra: vec![] });
        }
        let small = dir.join("src").join("small.png");
        std::fs::write(&small, Saved::Png.encode(&picture(7, 100, 100)).unwrap()).unwrap();
        real.push(Item { path: small, label: 0, generator: "real:cam".into(), dataset: "src".into(), extra: vec![] });
        let o = PasteOptions { parents: 3, extra_parents: 0, areas: vec![0.05, 0.25], blends: vec![Blend::Hard, Blend::Feather], seed: 4, min_side: 224, max_side: 4096, dataset: "partial-paste".into(), png: false, threads: 2 };
        let out = dir.join("store");
        let (parents, n) = paste_edits(&real, &fake, &out, &o).unwrap();
        // Per parent: itself, the re-save, and three kinds of paste for each of two areas and two blends.
        assert_eq!((parents, n), (3, 3 * (2 + 3 * 4)));
        let items = crate::ingest::read_manifest(&out.join("manifest.tsv")).unwrap();
        let edits = read_edits(&out.join("edits.tsv")).unwrap();
        assert_eq!((items.len(), edits.len()), (n, n));
        for it in &items {
            let e = &edits[&path_key(&it.path)];
            let t = it.transform().unwrap();
            assert!(it.parent_id().unwrap().contains("/src/real"), "{:?}", it.parent_id());
            assert_eq!(it.dataset, "partial-paste");
            // Only pasted generated pixels make an item "generated".
            assert_eq!((it.label == 1, e.generated, it.generator.starts_with("partial:")), (t.starts_with("generated-paste"), t.starts_with("generated-paste"), t.starts_with("generated-paste")));
            assert!(t.starts_with(e.kind.as_str()));
            let img = image_io::load(&it.path).unwrap();
            match &e.mask {
                Some(m) => {
                    let m = Mask::load(m).unwrap();
                    assert_eq!(m.size(), (img.w, img.h));
                    let want: f32 = t.rsplit_once("area").unwrap().1.parse().unwrap();
                    assert!(m.area() > want * 0.6 && m.area() < want * 1.1, "{t}: {}", m.area());
                }
                None => assert!(t == "parent" || t == "resave"),
            }
        }
        // The parent is the original file byte for byte; the re-save keeps its format.
        let first = items.iter().find(|i| i.transform() == Some("parent")).unwrap();
        let source = edits_source(&out, &first.path);
        assert_eq!(std::fs::read(&first.path).unwrap(), std::fs::read(source).unwrap());
        assert!(items.iter().filter(|i| i.transform() == Some("resave")).all(|i| items.iter().any(|p| p.transform() == Some("parent") && p.parent_id() == i.parent_id() && p.path.extension() == i.path.extension())));
        // Running again with the same seed gives the same store; a second run into it is refused.
        let again = dir.join("again");
        paste_edits(&real, &fake, &again, &o).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("edits.tsv")).unwrap(), std::fs::read_to_string(again.join("edits.tsv")).unwrap());
        assert_eq!(std::fs::read_to_string(out.join("manifest.tsv")).unwrap(), std::fs::read_to_string(again.join("manifest.tsv")).unwrap());
        assert!(paste_edits(&real, &fake, &out, &o).is_err());
        // Extra parents add authentic `parent` rows and nothing else.
        let more = dir.join("more");
        let (parents, n) = paste_edits(&real, &fake, &more, &PasteOptions { parents: 2, extra_parents: 5, ..o }).unwrap();
        assert_eq!((parents, n), (2, 2 * 14 + 2));
        let items = crate::ingest::read_manifest(&more.join("manifest.tsv")).unwrap();
        assert_eq!(items.iter().filter(|i| i.transform() == Some("parent")).count(), 4);
        assert!(items.iter().all(|i| i.path.is_file()));
        assert_eq!(walkdir::WalkDir::new(more.join("img")).into_iter().filter_map(|f| f.ok()).filter(|f| f.file_type().is_file()).count(), n);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The original file a store's `parent` row was copied from (the donor column of `edits.tsv`).
    fn edits_source(store: &Path, item: &Path) -> PathBuf {
        let rel = item.strip_prefix(store).unwrap().to_str().unwrap().to_string();
        let text = std::fs::read_to_string(store.join("edits.tsv")).unwrap();
        let row = text.lines().find(|l| l.starts_with(&format!("{rel}\t"))).unwrap();
        PathBuf::from(row.split('\t').nth(9).unwrap())
    }
}
