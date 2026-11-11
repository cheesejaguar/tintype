//! The dataset recipes: exactly which files every training, development and
//! confirmation set is built from, and how each is sampled. `tintype data
//! <plan>` runs one; `--dry-run` only resolves and prints the downloads.
use crate::fetch::{self, RemoteFile, Select, IMAGE_EXTS, VIDEO_EXTS};
use crate::ingest::{self, Item, ParquetSpec};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub const PLANS: [(&str, &str); 18] = [
    ("baselines", "released weights of the three reference detectors"),
    ("train-images", "phase-1 image training data"),
    ("train-video", "phase-1 video training data"),
    ("phase2", "phase-2 additions: more real photos and footage, more video generators"),
    ("phase3", "phase-3 additions: previously held-out generators, in-the-wild images"),
    ("phase4", "phase-4 additions for the video checkpoint: more video generators and low-resolution footage"),
    ("phase5", "phase-5 additions for the video checkpoint: recent commercial video generators, animation, gameplay"),
    ("phase6", "phase-6 photo additions: older GAN and diffusion families, face-identity generators, real faces"),
    ("dev", "development shards for sanity checks and calibration"),
    ("benchmarks", "development-visible benchmark sets"),
    ("confirm1", "first confirmation batch (images and video)"),
    ("confirm2", "second confirmation batch (images)"),
    ("confirm3", "third confirmation batch (images and video)"),
    ("confirm4", "fourth confirmation batch (video, for the video checkpoint)"),
    ("confirm5", "fifth confirmation batch (photos, for the balanced default and early exit)"),
    ("confirm6", "sixth confirmation batch (photos, for the phase-6 weights)"),
    ("confirm7", "seventh confirmation batch (photos, for the photo second opinion)"),
    ("all", "everything above, in order"),
];

struct Run {
    dry: bool,
    conns: usize,
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

impl Run {
    fn say(&self, what: &str) {
        println!("{}{what}", if self.dry { "[dry run] " } else { "" });
    }

    /// Download repository files matching `includes` into `data/raw/<dir>`.
    fn hf(&self, repo: &str, includes: &[&str], dir: &str) -> Result<Vec<PathBuf>> {
        let files: Vec<RemoteFile> = fetch::hf_list(repo)?.into_iter().filter(|f| includes.iter().any(|p| fetch::glob_match(p, &f.path))).collect();
        if files.is_empty() {
            bail!("nothing in {repo} matches {includes:?}");
        }
        self.hf_files(repo, &files, dir)
    }

    fn hf_files(&self, repo: &str, files: &[RemoteFile], dir: &str) -> Result<Vec<PathBuf>> {
        let out = Path::new("data/raw").join(dir);
        self.say(&format!("{repo}: {} files, {:.2} GB -> {}", files.len(), files.iter().map(|f| f.size).sum::<u64>() as f64 / 1e9, out.display()));
        if self.dry {
            return Ok(files.iter().map(|f| out.join(&f.path)).collect());
        }
        fetch::hf_download_files(repo, files, &out, self.conns)
    }

    /// Sorted files of a repository folder (for "the first N" style picks).
    fn hf_folder(&self, repo: &str, prefix: &str) -> Result<Vec<RemoteFile>> {
        Ok(fetch::hf_list(repo)?.into_iter().filter(|f| f.path.starts_with(prefix)).collect())
    }

    fn url(&self, url: &str, dest: &str) -> Result<PathBuf> {
        let dest = PathBuf::from(dest);
        self.say(&format!("{url} -> {}", dest.display()));
        if !self.dry {
            fetch::download(url, &dest, None, self.conns)?;
        }
        Ok(dest)
    }

    fn ingest(&self, inputs: &[PathBuf], out: &str, dataset: &str, spec: ParquetSpec) -> Result<()> {
        if self.dry {
            return Ok(());
        }
        let (n, skipped) = ingest::ingest_parquet(inputs, Path::new(out), dataset, &spec)?;
        println!("{dataset}: wrote {n} images, skipped {skipped} rows -> {out}");
        Ok(())
    }

    fn unpack(&self, archive: &Path, out: &str, sel: Select) -> Result<Vec<String>> {
        if self.dry {
            return Ok(vec![]);
        }
        let names = fetch::unpack(archive, Path::new(out), &sel)?;
        println!("{}: extracted {} files -> {out}", archive.display(), names.len());
        Ok(names)
    }

    fn index(&self, root: &str, dataset: &str, video: bool, rules: &[(&str, u8, &str)]) -> Result<()> {
        if self.dry {
            return Ok(());
        }
        let rules: Vec<(String, u8, String)> = rules.iter().map(|(a, b, c)| (a.to_string(), *b, c.to_string())).collect();
        let n = ingest::index_folder(Path::new(root), dataset, &rules, video)?;
        println!("{dataset}: indexed {n} files under {root}");
        Ok(())
    }

    fn rm(&self, dir: &str) {
        if !self.dry {
            let _ = std::fs::remove_dir_all(Path::new("data/raw").join(dir));
        }
    }

    /// Rewrite a manifest in place; `f` returns the generator to keep a row under, or `None` to drop it.
    fn rewrite(&self, manifest: &str, f: impl Fn(&str, u8, &str) -> Option<String>) -> Result<()> {
        if self.dry {
            return Ok(());
        }
        let text = std::fs::read_to_string(manifest)?;
        let mut out = String::new();
        for l in text.lines() {
            let c: Vec<&str> = l.split('\t').collect();
            if c.len() < 4 {
                continue;
            }
            if let Some(g) = f(c[0], c[1].parse().unwrap_or(0), c[2]) {
                out.push_str(&format!("{}\t{}\t{g}\t{}\n", c[0], c[1], c[3]));
            }
        }
        std::fs::write(manifest, out)?;
        Ok(())
    }

    /// Remove rows of `manifest` whose file is byte-identical to a file in `against`.
    fn drop_duplicates(&self, manifest: &str, against: &[&str]) -> Result<()> {
        if self.dry {
            return Ok(());
        }
        let mut seen: Vec<Item> = Vec::new();
        for m in against {
            if Path::new(m).exists() {
                seen.extend(ingest::read_manifest(Path::new(m))?);
            }
        }
        let test = ingest::read_manifest(Path::new(manifest))?;
        let dups: std::collections::HashSet<PathBuf> = ingest::exact_overlap(&seen, &test).into_iter().map(|(t, _)| t).collect();
        if !dups.is_empty() {
            let root = Path::new(manifest).parent().unwrap().to_path_buf();
            self.rewrite(manifest, |rel, _, g| (!dups.contains(&root.join(rel))).then(|| g.to_string()))?;
        }
        println!("{manifest}: removed {} files already present elsewhere", dups.len());
        Ok(())
    }
}

fn labelled(label_col: &str, fake: &[&str], real: &[&str], gen_col: Option<&str>, real_src: Option<&str>) -> ParquetSpec {
    ParquetSpec {
        image_col: "image".into(),
        label_col: Some(label_col.into()),
        fake_values: s(fake),
        real_values: s(real),
        gen_col: gen_col.map(Into::into),
        real_src_col: real_src.map(Into::into),
        ..Default::default()
    }
}

fn openfake_spec() -> ParquetSpec {
    labelled("label", &["fake"], &["real"], Some("model"), Some("model"))
}

fn sofake_ood_spec() -> ParquetSpec {
    labelled("label", &["FULL_SYNTHETIC"], &["REAL"], Some("generator"), Some("source_platform"))
}

/// Generators kept out of phases 1-2 so that they stayed unseen in So-Fake-OOD.
const HELD_OUT: [&str; 4] = ["recraft-v3-24-7-25", "seedream-3-24-7-25", "ideogram", "flux-1-pro"];

const AIGVD: &str = "AIGVDBench/AIGVDBench";
const KINETICS: &str = "kiyoonkim/kinetics-400-targz";
const SOFAKE_OOD: &str = "saberzl/So-Fake-OOD";
const WASITAI: &str = "wasitaigeneratedcom/frontier-image-video-bench-2026-10";
const GENVIDEO_VAL: &str = "https://modelscope.cn/datasets/cccnju/Gen-Video/resolve/master/GenVideo-Val.zip";
const GENVIDEO_GENERATORS: [&str; 10] = ["Crafter", "Gen2", "HotShot", "Lavie", "ModelScope", "MoonValley", "MorphStudio", "Show_1", "Sora", "WildScrape"];

impl Run {
    /// One AIGVDBench generator archive, cut down to the selection.
    fn aigvd(&self, zip_path: &str, out: &str, sel: Select) -> Result<()> {
        let got = self.hf(AIGVD, &[&format!("AIGVDBench/{zip_path}.zip")], "aigvd")?;
        self.unpack(&got[0], out, Select { exts: VIDEO_EXTS.to_vec(), flatten: true, ..sel })?;
        self.rm("aigvd");
        Ok(())
    }

    fn kinetics(&self, part: &str, out: &str, first: usize) -> Result<()> {
        let got = self.hf(KINETICS, &[part], "k400")?;
        self.unpack(&got[0], out, Select { exts: vec!["mp4"], first, flatten: first > 0, ..Default::default() })?;
        self.rm("k400");
        Ok(())
    }

    fn sofake_ood(&self, shards: &[&str], out: &str, dataset: &str) -> Result<()> {
        // One shard at a time: each is about 3 GB and is removed once its images are out.
        for x in shards {
            let got = self.hf(SOFAKE_OOD, &[&format!("data/test_image-{x}-of-00046.parquet")], "sofake")?;
            self.ingest(&got, out, dataset, sofake_ood_spec())?;
            self.rm("sofake");
        }
        Ok(())
    }

    /// An AIGIBench test archive with `0_real` / `1_fake` folders; `per_class > 0` samples evenly.
    fn aigibench(&self, name: &str, out: &str, dataset: &str, gen: &str, per_class: usize) -> Result<()> {
        self.aigibench_sample(name, out, dataset, gen, per_class, false)
    }

    /// As [`Run::aigibench`]; `second` takes the sample half a step away from the first one.
    fn aigibench_sample(&self, name: &str, out: &str, dataset: &str, gen: &str, per_class: usize, second: bool) -> Result<()> {
        let select = |class: &str| Select { contains: class.into(), exts: IMAGE_EXTS.to_vec(), evenly: per_class, offset_half: second, ..Default::default() };
        let classes = ["/0_real/", "/1_fake/"];
        if per_class > 0 {
            // A sampled set is read out of the remote archive; a whole one is downloaded first.
            let url = fetch::hf_url("HorizonTEL/AIGIBench", &format!("test/{name}.zip"));
            self.say(&format!("{url}: {per_class} files per class -> {out}"));
            if !self.dry {
                for class in classes {
                    let names = fetch::unpack_remote_zip(&url, Path::new(out), &select(class), self.conns)?;
                    println!("{url}: extracted {} files -> {out}", names.len());
                }
            }
        } else {
            let got = self.hf("HorizonTEL/AIGIBench", &[&format!("test/{name}.zip")], "aigibench")?;
            for class in classes {
                self.unpack(&got[0], out, select(class))?;
            }
            self.rm("aigibench");
        }
        self.index(out, dataset, false, &[("0_real", 0, "aigibench"), ("1_fake", 1, gen)])
    }

    /// GenVideo-Val clips: `n_real` real and `n_fake` per generator, evenly
    /// spaced; `exclude` lists members already used (the fresh sample then
    /// starts half a step in, as the first sample started at zero).
    fn genvideo_val(&self, out: &str, dataset: &str, n_real: usize, n_fake: usize, exclude: Vec<String>) -> Result<Vec<String>> {
        let zip = self.url(GENVIDEO_VAL, "data/raw/genvideo-val/GenVideo-Val.zip")?;
        if self.dry {
            return Ok(vec![]);
        }
        let fresh = !exclude.is_empty();
        let mut picked = Vec::new();
        let mut groups: Vec<(String, usize)> = vec![("GenVideo-Val/Real/".into(), n_real)];
        groups.extend(GENVIDEO_GENERATORS.iter().map(|g| (format!("GenVideo-Val/Fake/{g}/"), n_fake)));
        for (prefix, n) in groups {
            let all = fetch::list_archive(&zip, &Select { contains: prefix, exts: VIDEO_EXTS.to_vec(), exclude: exclude.clone(), ..Default::default() })?;
            picked.extend(fetch::evenly(&all, n, fresh));
        }
        let all_names = fetch::list_archive(&zip, &Select::default())?;
        let skip: Vec<String> = all_names.into_iter().filter(|n| !picked.contains(n)).collect();
        fetch::unpack(&zip, Path::new(out), &Select { exclude: skip, ..Default::default() })?;
        self.rm("genvideo-val");
        let mut rules: Vec<(String, u8, String)> = GENVIDEO_GENERATORS.iter().map(|g| (format!("Fake/{g}/"), 1, g.to_string())).collect();
        rules.push(("Real/".into(), 0, "msrvtt".into()));
        let n = ingest::index_folder(Path::new(out), dataset, &rules, true)?;
        println!("{dataset}: indexed {n} clips under {out}");
        Ok(picked)
    }

    fn train_images(&self) -> Result<()> {
        let got = self.hf("Rajarshi-Roy-research/Defactify_Image_Dataset", &["data/train-*.parquet"], "defactify")?;
        self.ingest(&got, "data/store/defactify", "defactify", ParquetSpec { image_col: "Image".into(), ..labelled("Label_A", &["1"], &["0"], Some("Label_B"), None) })?;
        self.rewrite("data/store/defactify/manifest.tsv", |_, label, g| Some(if label == 1 { format!("defactify-gen{g}") } else { g.to_string() }))?;
        self.rm("defactify");

        let got = self.hf("bitmind/nano-banana", &["data/train-0000[0-4]-of-00031.parquet"], "nano-banana")?;
        self.ingest(&got, "data/store/nano-banana", "nano-banana", ParquetSpec { image_col: "image".into(), const_label: Some(1), const_gen: Some("nano-banana".into()), ..Default::default() })?;
        self.rm("nano-banana");

        for (groups, store) in [(["00000", "00007", "00013", "00019", "00025", "00031"].as_slice(), "openfake"), (["00003", "00016", "00028"].as_slice(), "openfake2")] {
            for g in groups {
                let got = self.hf("ComplexDataLab/OpenFake", &[&format!("core/train-{g}-of-00032-00000.parquet")], "openfake")?;
                self.ingest(&got, &format!("data/store/{store}"), "openfake", openfake_spec())?;
                self.rm("openfake");
            }
        }

        let got = self.hf("ComplexDataLab/OpenFake", &["supplement/reddit/2026-11/supplement_reddit-2026-11-0000[0-4].parquet"], "openfake-reddit")?;
        self.ingest(&got, "data/store/openfake-reddit", "openfake-reddit", ParquetSpec { const_gen: Some("reddit-wild".into()), ..labelled("label", &["fake"], &["real"], None, None) })?;
        self.rm("openfake-reddit");

        let mut inc: Vec<String> = [
            "FLUX.2-pro", "FLUX.2_max", "GLM-Image", "GPT-Image-1", "HunyuanImage-3.0", "Imagen-4.0-Ultra", "Imagen-4.0", "Qwen-Image-2.0-pro", "Qwen-Image-2512", "Qwen-Image",
            "Seedream-4.0", "Seedream-4.5", "Seedream-5.0", "kling_v2_1", "nano-banana-2.0", "nano-banana-pro",
        ]
        .iter()
        .map(|g| format!("data/train-qwen-{g}.parquet"))
        .collect();
        inc.extend(s(&[
            "data/train-recent-Efficient-Large-Model-Sana_Sprint_1.6B_1024px_diffusers.parquet",
            "data/train-recent-Tongyi-MAI-Z-Image-Turbo.parquet",
            "data/train-recent-black-forest-labs-FLUX.2-klein-4B.parquet",
            "data/train-rapidata-flux2-0000[0-5]-of-00069.parquet",
        ]));
        let got = self.hf("Thermostatic/frontier-synthetic-images-2026", &inc.iter().map(String::as_str).collect::<Vec<_>>(), "frontier")?;
        self.ingest(&got, "data/store/frontier", "frontier", labelled("label", &["1"], &["0"], Some("generator"), None))?;
        self.rewrite("data/store/frontier/manifest.tsv", |_, _, g| (!HELD_OUT.contains(&g)).then(|| g.to_string()))?;
        self.rm("frontier");

        // Community Forensics (Small): shards spanning its generated and real sections.
        let shards = [3, 30, 60, 68, 70, 71, 73, 77, 78, 79, 82, 88, 116, 117, 118, 119, 120, 121, 122, 156];
        let inc: Vec<String> = shards.iter().map(|i| format!("data/HFCF_small_{i}.parquet")).collect();
        let got = self.hf("OwensLab/CommunityForensics-Small", &inc.iter().map(String::as_str).collect::<Vec<_>>(), "commfor-small")?;
        self.ingest(&got, "data/store/commfor-small", "commfor-small", ParquetSpec { image_col: "image_data".into(), ..labelled("label", &["1"], &["0"], Some("model_name"), Some("real_source")) })?;
        self.rewrite("data/store/commfor-small/manifest.tsv", |rel, label, g| {
            Some(if label == 1 {
                g.to_string()
            } else if rel.contains("_116/") {
                "real:ffhq".into()
            } else if rel.contains("_156/") {
                "real:lhq".into()
            } else {
                "real:coco".into()
            })
        })?;
        self.rm("commfor-small");
        Ok(())
    }

    fn train_video(&self) -> Result<()> {
        self.hf("kyoungjunpark/VidGuard-R1-sampled-video-10k", &["activitynet/*", "gen_hunyuan_activitynet/*", "gen_cogvideo_activitynet/*"], "../vstore/vidguard")?;
        self.index("data/vstore/vidguard", "vidguard", true, &[("gen_cogvideo", 1, "CogVideoX"), ("gen_hunyuan", 1, "HunyuanVideo"), ("activitynet/", 0, "activitynet")])?;

        let got = self.hf("l8cv/GenBuster-200K-mini", &["*.zip"], "genbuster")?;
        self.unpack(&got[0], "data/vstore/genbuster", Select::default())?;
        self.rm("genbuster");
        self.index("data/vstore/genbuster", "genbuster", true, &[("fake/easyanimate", 1, "EasyAnimate"), ("fake/cogvideox", 1, "CogVideoX"), ("fake/ltxvideo", 1, "LTX-Video"), ("fake/hunyuanvideo", 1, "HunyuanVideo"), ("train/real", 0, "openvid")])?;

        for g in ["OpenSora", "ZeroScope"] {
            let tar = self.url(&fetch::modelscope_url("cccnju/GenVideo-100K", &format!("{g}.tar.gz")), &format!("data/raw/genvideo-100k/{g}.tar.gz"))?;
            self.unpack(&tar, "data/vstore/genvideo100k", Select::default())?;
        }
        self.rm("genvideo-100k");
        self.index("data/vstore/genvideo100k", "genvideo100k", true, &[("OpenSora", 1, "OpenSora"), ("ZeroScope", 1, "ZeroScope")])?;

        for (zip, name) in [("ClosedSource/pika", "pika"), ("ClosedSource/vidu", "vidu"), ("ClosedSource/Luma", "Luma"), ("OpenSource/I2V/SVD", "SVD"), ("OpenSource/T2V/AnimateDiff", "AnimateDiff"), ("OpenSource/T2V/Pyramid-Flow", "Pyramid-Flow")] {
            self.aigvd(zip, &format!("data/vstore/aigvd/{name}"), Select { first: 1000, ..Default::default() })?;
        }
        self.index("data/vstore/aigvd", "aigvd", true, &[("pika/", 1, "Pika"), ("vidu/", 1, "Vidu"), ("Luma/", 1, "Luma"), ("SVD/", 1, "SVD"), ("AnimateDiff/", 1, "AnimateDiff"), ("Pyramid-Flow/", 1, "Pyramid-Flow")])?;

        for part in ["val/part_0.tar.gz", "val/part_1.tar.gz"] {
            self.kinetics(part, "data/vstore/k400", 0)?;
        }
        self.index("data/vstore/k400", "k400", true, &[("", 0, "kinetics400")])?;
        let got = self.hf("zengxianyu/open-sora-pexels-subset", &["videos/5000_0_shard_0000.tar"], "pexels")?;
        self.unpack(&got[0], "data/vstore/pexels", Select::default())?;
        self.rm("pexels");
        self.index("data/vstore/pexels", "pexels-video", true, &[("", 0, "pexels-video")])
    }

    fn phase2(&self) -> Result<()> {
        let got = self.hf("ComplexDataLab/OpenFake", &["core/test-00001-of-00013.parquet", "core/test-00002-of-00013.parquet"], "of-test")?;
        self.ingest(&got, "data/store/openfake-test-reals", "openfake-test-reals", labelled("label", &["__none__"], &["real"], None, Some("model")))?;
        self.rm("of-test");
        for part in ["val/part_2.tar.gz", "val/part_3.tar.gz"] {
            self.kinetics(part, "data/vstore/k400b", 0)?;
        }
        self.index("data/vstore/k400b", "k400", true, &[("", 0, "kinetics400")])?;
        for got in self.hf("zengxianyu/open-sora-pexels-subset", &["videos/5000_0_shard_000[1-4].tar"], "pexels")? {
            self.unpack(&got, "data/vstore/pexelsb", Select::default())?;
        }
        self.rm("pexels");
        self.index("data/vstore/pexelsb", "pexels-video", true, &[("", 0, "pexels-video")])?;
        for (zip, name, n) in [("OpenSource/T2V/Open-Sora", "Open-Sora", 1000), ("OpenSource/T2V/Cogvideox1.5", "Cogvideox1.5", 1000), ("OpenSource/T2V/HunyuanVideo", "HunyuanVideo", 700), ("ClosedSource/wan", "wan", 500), ("ClosedSource/kling", "kling", 500)] {
            self.aigvd(zip, &format!("data/vstore/aigvd2/{name}"), Select { first: n, ..Default::default() })?;
        }
        self.index("data/vstore/aigvd2", "aigvd", true, &[("Open-Sora/", 1, "OpenSora"), ("Cogvideox1.5/", 1, "CogVideoX"), ("HunyuanVideo/", 1, "HunyuanVideo"), ("wan/", 1, "Wan"), ("kling/", 1, "Kling")])
    }

    fn phase3(&self) -> Result<()> {
        let inc = ["data/train-qwen-GPT-Image-1.5.parquet", "data/train-qwen-gpt-image-2.parquet", "data/train-rapidata-recraft3-*.parquet", "data/train-rapidata-flux2-0000[0-5]-of-00069.parquet"];
        let got = self.hf("Thermostatic/frontier-synthetic-images-2026", &inc, "frontier")?;
        self.ingest(&got, "data/store/frontier-heldout", "frontier", labelled("label", &["1"], &["0"], Some("generator"), None))?;
        self.rewrite("data/store/frontier-heldout/manifest.tsv", |_, _, g| (HELD_OUT.contains(&g) || ["GPT-Image-1.5", "gpt-image-2", "recraft-v2"].contains(&g)).then(|| g.to_string()))?;
        self.rm("frontier");

        let inc: Vec<String> = (0..12).map(|i| format!("data/train-{:05}-of-02343.parquet", i * 200)).collect();
        let got = self.hf("saberzl/So-Fake-Set", &inc.iter().map(String::as_str).collect::<Vec<_>>(), "sofake-set")?;
        self.ingest(&got, "data/store/sofake-set", "sofake-set", labelled("label", &["full_synthetic"], &["real"], Some("generator"), None))?;
        self.rm("sofake-set");
        // No training file may also be an evaluation file.
        self.drop_duplicates("data/store/sofake-set/manifest.tsv", &EVAL_IMAGE_MANIFESTS)
    }

    fn dev(&self) -> Result<()> {
        let got = self.hf("ComplexDataLab/OpenFake", &["core/test-00000-of-00013.parquet"], "of-dev")?;
        self.ingest(&got, "data/store/dev-openfake-test", "dev-openfake-test", openfake_spec())?;
        self.rewrite("data/store/dev-openfake-test/manifest.tsv", |_, _, g| (!["gpt-image-1.5", "gpt-image-2", "recraft-v3", "recraft-v2", "ideogram-2.0"].contains(&g)).then(|| g.to_string()))?;
        self.rm("of-dev");
        self.drop_duplicates("data/store/dev-openfake-test/manifest.tsv", &["data/store/frontier/manifest.tsv", "data/store/openfake-test-reals/manifest.tsv"])?;

        self.kinetics("test/part_0.tar.gz", "data/vstore/vdev/real", 400)?;
        self.aigvd("OpenSource/I2V/SEINE", "data/vstore/vdev/seine", Select { first: 400, ..Default::default() })?;
        self.index("data/vstore/vdev", "vdev", true, &[("real/", 0, "kinetics400-test"), ("seine/", 1, "SEINE")])
    }

    fn benchmarks(&self) -> Result<()> {
        self.sofake_ood(&["00000", "00012", "00024", "00036"], "data/store/sofake-ood", "sofake-ood")?;

        let got = self.hf("marco-willi/synthbuster-plus", &["data/test-*", "data/validation-*"], "synthbuster")?;
        self.ingest(&got, "data/store/synthbuster", "synthbuster", labelled("label", &["1"], &["__none__"], Some("source"), None))?;
        self.rm("synthbuster");
        let zip = self.url("https://www.grip.unina.it/download/prog/B-Free/extended_synthbuster/real_RAISE_1k.zip", "data/raw/raise/real_RAISE_1k.zip")?;
        self.unpack(&zip, "data/store/synthbuster/raise1k", Select { exts: IMAGE_EXTS.to_vec(), ..Default::default() })?;
        self.rm("raise");
        if !self.dry {
            // Append the RAISE originals as the real class.
            let root = Path::new("data/store/synthbuster");
            let mut lines = std::fs::read_to_string(root.join("manifest.tsv"))?;
            for e in walkdir::WalkDir::new(root.join("raise1k")).sort_by_file_name().into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file() && ingest::is_image(e.path())) {
                lines.push_str(&format!("{}\t0\treal:raise1k\tsynthbuster\n", e.path().strip_prefix(root)?.display()));
            }
            std::fs::write(root.join("manifest.tsv"), lines)?;
        }

        self.aigibench("CommunityAI", "data/store/aigibench", "aigibench-communityai", "CommunityAI", 0)?;

        self.hf(WASITAI, &["images/*", "images_social/*", "videos/*", "videos_social/*", "*.csv"], "../vstore/wasitai")?;
        if !self.dry {
            wasitai_manifests(Path::new("data/vstore/wasitai"))?;
        }

        let files = self.hf_folder("OmerXYZ/comgenvid", "videos/")?;
        let mut pick = Vec::new();
        for folder in ["videos/fake/Sora/", "videos/fake/VEO3/", "videos/real/MSVD/"] {
            pick.extend(files.iter().filter(|f| f.path.starts_with(folder)).take(150).cloned());
        }
        self.hf_files("OmerXYZ/comgenvid", &pick, "../vstore/comgenvid")?;
        self.index("data/vstore/comgenvid", "comgenvid", true, &[("fake/Sora", 1, "Sora"), ("fake/VEO3", 1, "Veo3"), ("real/MSVD", 0, "msvd")])?;

        let picked = self.genvideo_val("data/vstore/genvideo-val", "genvideo-val", 300, 50, vec![])?;
        if !self.dry {
            std::fs::write("data/vstore/genvideo-val/pick.list", picked.join("\n") + "\n")?;
        }
        Ok(())
    }

    /// Video-only additions for the dedicated video checkpoint. The clips of
    /// Sora, Gen-2 and Jimeng that the first confirmation batch was scored on
    /// are removed from the training selection.
    fn phase4(&self) -> Result<()> {
        let picks = [
            ("ClosedSource/Sora", "Sora", 1200, "Sora"),
            ("ClosedSource/Gen2", "Gen2", 1000, "Gen2"),
            ("ClosedSource/Jimeng", "Jimeng", 1000, "Jimeng"),
            ("OpenSource/T2V/VideoCrafter", "VideoCrafter", 800, "VideoCrafter"),
            ("OpenSource/T2V/LTX", "LTX", 600, "LTX-Video"),
            ("OpenSource/T2V/EasyAnimate", "EasyAnimate", 600, "EasyAnimate"),
        ];
        for (zip, name, n, _) in picks {
            self.aigvd(zip, &format!("data/vstore/aigvd3/{name}"), Select { first: n, ..Default::default() })?;
        }
        let rules: Vec<(String, u8, &str)> = picks.iter().map(|(_, name, _, gen)| (format!("{name}/"), 1, *gen)).collect();
        self.index("data/vstore/aigvd3", "aigvd", true, &rules.iter().map(|(a, b, c)| (a.as_str(), *b, *c)).collect::<Vec<_>>())?;
        self.drop_duplicates("data/vstore/aigvd3/manifest.tsv", &["data/fresh/video/manifest.tsv", "data/fresh4/video/manifest.tsv"])?;

        for part in ["train/part_0.tar.gz", "train/part_1.tar.gz"] {
            self.kinetics(part, "data/vstore/k400c", 1000)?;
        }
        self.index("data/vstore/k400c", "k400", true, &[("", 0, "kinetics400")])?;
        self.drop_duplicates("data/vstore/k400c/manifest.tsv", &["data/fresh/video/manifest.tsv", "data/fresh4/video/manifest.tsv", "data/vstore/vdev/manifest.tsv"])?;

        // The released video weights also saw 1,500 UCF101 clips (AVI files, decoded
        // by an external tool at the time). This build reads video only through
        // the system decoder, which does not open AVI, so they are no longer fetched.
        Ok(())
    }

    /// Second round of video-only additions: current commercial generators
    /// from Rapidata's preference sets, two more AIGVDBench generators, and
    /// real footage that is not camera footage (animation, cartoons, games).
    /// A store that is already indexed is left alone, so the plan can resume.
    fn phase5(&self) -> Result<()> {
        let have = |store: &str| !self.dry && Path::new("data/vstore").join(store).join("manifest.tsv").exists();
        if !have("rapidata") {
            for repo in ["Rapidata/text-2-video-human-preferences-moonvalley-marey", "Rapidata/text-2-video-human-preferences-veo3.1"] {
                let files: Vec<RemoteFile> = fetch::hf_list(repo)?.into_iter().filter(|f| f.path.to_ascii_lowercase().ends_with(".mp4")).collect();
                self.hf_files(repo, &files, "../vstore/rapidata")?;
            }
            self.index(
                "data/vstore/rapidata",
                "rapidata-t2v",
                true,
                &[
                    ("/alpha_", 1, "Gen-3 Alpha"),
                    ("/hunyuan_", 1, "HunyuanVideo"),
                    ("/pika2.2_", 1, "Pika 2.2"),
                    ("/pika_", 1, "Pika"),
                    ("/ray2_", 1, "Luma Ray 2"),
                    ("/sora_", 1, "Sora"),
                    ("/veo2_", 1, "Veo 2"),
                    ("/veo31_", 1, "Veo 3.1"),
                    ("/veo3_", 1, "Veo 3"),
                    ("/wan2.1_", 1, "Wan"),
                    ("/mochi-1", 1, "Mochi 1"),
                    ("/marey", 1, "Marey"),
                    ("/kling-v2.1", 1, "Kling 2.1"),
                    ("/seedance-1-pro", 1, "Seedance 1 Pro"),
                ],
            )?;
        }

        if !have("aigvd4") {
            for (zip, name, n) in [("ClosedSource/Gen3", "Gen3", 1000), ("OpenSource/T2V/RepVideo", "RepVideo", 800)] {
                self.aigvd(zip, &format!("data/vstore/aigvd4/{name}"), Select { first: n, ..Default::default() })?;
            }
            self.index("data/vstore/aigvd4", "aigvd", true, &[("Gen3/", 1, "Gen-3"), ("RepVideo/", 1, "RepVideo")])?;
            self.drop_duplicates("data/vstore/aigvd4/manifest.tsv", &["data/fresh4/video/manifest.tsv"])?;
        }

        // Recorded gameplay: 20 clips of each of the twelve games, evenly spaced.
        if !have("gameplay") {
            let all: Vec<RemoteFile> = fetch::hf_list("Sunain-Labs/sunain-gameplay")?.into_iter().filter(|f| f.path.ends_with(".mp4")).collect();
            let mut games: Vec<&str> = all.iter().filter_map(|f| f.path.split('/').nth(1)).collect();
            games.sort();
            games.dedup();
            let mut pick = Vec::new();
            for g in games {
                let of_game: Vec<RemoteFile> = all.iter().filter(|f| f.path.split('/').nth(1) == Some(g)).cloned().collect();
                pick.extend(fetch::evenly(&of_game, 20, false));
            }
            self.hf_files("Sunain-Labs/sunain-gameplay", &pick, "../vstore/gameplay")?;
            self.index("data/vstore/gameplay", "sunain-gameplay", true, &[("", 0, "gameplay")])?;
        }

        // Hand-drawn animation: four shards of Sakugabooru clips.
        if !have("animation") {
            for got in self.hf("trojblue/sakugabooru2025", &["train/4.tar", "train/10.tar", "train/102.tar", "train/103.tar"], "sakuga")? {
                // MP4 only: the system decoder does not open the WebM clips.
                self.unpack(&got, "data/vstore/animation", Select { exts: vec!["mp4"], flatten: true, ..Default::default() })?;
            }
            self.rm("sakuga");
            self.index("data/vstore/animation", "sakugabooru", true, &[("", 0, "animation")])?;
        }

        // Western cartoons and 3D animation: the cartoon domain of the Human-Animal-Cartoon set.
        if !have("cartoons") {
            let got = self.hf("hdong51/Human-Animal-Cartoon", &["cartoon.zip"], "hac")?;
            // The archive also holds optical-flow renderings; only the videos are wanted.
            self.unpack(&got[0], "data/vstore/cartoons", Select { contains: "cartoon/videos/".into(), exts: VIDEO_EXTS.to_vec(), flatten: true, ..Default::default() })?;
            self.rm("hac");
            self.index("data/vstore/cartoons", "hac-cartoon", true, &[("", 0, "cartoon")])?;
        }
        Ok(())
    }

    /// Photo additions after the fifth confirmation batch, which showed the
    /// phase-3 weights finding fewer images than Community Forensics from
    /// older GAN and diffusion families and from face-identity adapters.
    /// A store that is already indexed is left alone, so the plan can resume.
    fn phase6(&self) -> Result<()> {
        let have = |store: &str| !self.dry && Path::new("data/store").join(store).join("manifest.tsv").exists();

        // 25 further shards of Community Forensics (Small): 15 generated (about 830
        // generators, GAN families among them) and 10 real.
        if !have("commfor-small2") {
            let shards = [0, 9, 15, 21, 27, 33, 39, 45, 51, 57, 63, 66, 75, 85, 91, 95, 99, 103, 107, 111, 114, 130, 145, 170, 180];
            for chunk in shards.chunks(5) {
                let inc: Vec<String> = chunk.iter().map(|i| format!("data/HFCF_small_{i}.parquet")).collect();
                let got = self.hf("OwensLab/CommunityForensics-Small", &inc.iter().map(String::as_str).collect::<Vec<_>>(), "commfor-small")?;
                self.ingest(&got, "data/store/commfor-small2", "commfor-small", ParquetSpec { image_col: "image_data".into(), ..labelled("label", &["1"], &["0"], Some("model_name"), Some("real_source")) })?;
                self.rm("commfor-small");
            }
            // The real shards (COCO, FFHQ, LHQ, VISION) carry no usable source name.
            self.rewrite("data/store/commfor-small2/manifest.tsv", |_, label, g| Some(if label == 0 { "real:commfor".into() } else { g.to_string() }))?;
        }

        // GenImage in miniature: ADM, BigGAN, GLIDE, VQDM, Wukong, SD 1.4/1.5, Midjourney, and ImageNet photos.
        if !have("tiny-genimage") {
            let got = self.hf("TheKernel01/Tiny-GenImage", &["data/train-*.parquet"], "tiny-genimage")?;
            self.ingest(&got, "data/store/tiny-genimage", "tiny-genimage", labelled("label", &["1", "fake"], &["0", "real"], Some("generator"), None))?;
            let names = ["Real", "ADM", "BigGAN", "GLIDE", "Midjourney", "SD14", "SD15", "VQDM", "Wukong"];
            self.rewrite("data/store/tiny-genimage/manifest.tsv", |_, label, g| {
                Some(if label == 0 { "real:imagenet".to_string() } else { g.parse::<usize>().ok().and_then(|i| names.get(i)).map_or(g.to_string(), |n| format!("genimage-{n}")) })
            })?;
            self.rm("tiny-genimage");
        }

        // StyleGAN3 and StyleGAN2 faces.
        if !have("stylegan3") {
            let got = self.hf("34data/StyleGAN3_1024x1024_images", &["*.parquet"], "stylegan3")?;
            self.ingest(&got, "data/store/stylegan3", "stylegan3", ParquetSpec { image_col: "image".into(), const_label: Some(1), const_gen: Some("StyleGAN3".into()), ..Default::default() })?;
            self.rm("stylegan3");
        }
        if !have("tpdne") {
            let all: Vec<RemoteFile> = fetch::hf_list("javi22/this-person-does-not-exist-10k")?.into_iter().filter(|f| f.path.to_ascii_lowercase().ends_with(".jpg")).collect();
            self.hf_files("javi22/this-person-does-not-exist-10k", &fetch::evenly(&all, 3000, false), "../store/tpdne")?;
            self.index("data/store/tpdne", "tpdne", false, &[("", 1, "StyleGAN2")])?;
        }

        // Face-identity adapters on SD 1.5 / SDXL and on SD 3.5.
        if !have("ip-adapter") {
            for (repo, gen) in [("34data/ip-adapter-faceid", "IP-Adapter FaceID"), ("34data/sd35-ip-adapter-faceid", "IP-Adapter FaceID SD3.5")] {
                let got = self.hf(repo, &["*.parquet"], "ip-adapter")?;
                self.ingest(&got, "data/store/ip-adapter", "ip-adapter-faceid", ParquetSpec { image_col: "image".into(), const_label: Some(1), const_gen: Some(gen.into()), max_per_file: Some(3000), ..Default::default() })?;
                self.rm("ip-adapter");
            }
        }

        // Real faces from a source no evaluation set uses.
        if !have("celeba-hq") {
            let got = self.hf("mattymchen/celeba-hq", &["*train-0000[0-2]-*.parquet"], "celeba-hq")?;
            self.ingest(&got, "data/store/celeba-hq", "celeba-hq", ParquetSpec { image_col: "image".into(), const_label: Some(0), max_per_file: Some(2500), ..Default::default() })?;
            self.rm("celeba-hq");
        }

        // These collections share source photos (ImageNet, COCO) with the
        // evaluation sets: drop every file that is identical to one of theirs.
        let mut scored: Vec<String> = EVAL_IMAGE_MANIFESTS.iter().chain(CONFIRM3_IMAGE_MANIFESTS.iter()).map(|m| m.to_string()).collect();
        scored.push("data/vstore/wasitai/images_social.tsv".into());
        scored.push("data/fresh6/sofake-ood/manifest.tsv".into());
        scored.extend(CONFIRM5_AIGIBENCH.iter().map(|(name, _)| format!("data/fresh6/aigibench-{name}/manifest.tsv")));
        for m in PHASE6_IMAGE_MANIFESTS {
            self.drop_duplicates(m, &scored.iter().map(String::as_str).collect::<Vec<_>>())?;
        }
        Ok(())
    }

    fn confirm1(&self) -> Result<()> {
        self.sofake_ood(&["00006", "00018"], "data/fresh/sofake-ood", "sofake-ood-fresh")?;
        self.drop_duplicates("data/fresh/sofake-ood/manifest.tsv", &["data/store/sofake-ood/manifest.tsv"])?;
        self.aigibench("SocialRF", "data/fresh/socialrf", "aigibench-socialrf", "SocialRF", 0)?;
        let got = self.hf("marco-willi/synthbuster-plus", &["data/train-0000[0-4]-of-00026.parquet"], "synthbuster")?;
        self.ingest(&got, "data/fresh/synthbuster", "synthbuster-fresh", labelled("label", &["1"], &["__none__"], Some("source"), None))?;
        self.rm("synthbuster");

        for g in ["Gen2", "Sora", "Jimeng"] {
            self.aigvd(&format!("ClosedSource/{g}"), &format!("data/fresh/video/aigvd-{g}"), Select { evenly: 150, ..Default::default() })?;
        }
        self.kinetics("test/part_1.tar.gz", "data/fresh/video/kinetics", 200)?;
        self.index("data/fresh/video", "aigvd-fresh", true, &[("aigvd-Gen2/", 1, "Gen2"), ("aigvd-Sora/", 1, "Sora (AIGVDBench)"), ("aigvd-Jimeng/", 1, "Jimeng"), ("kinetics/", 0, "kinetics400-test")])?;

        let files = self.hf_folder("OmerXYZ/comgenvid", "videos/")?;
        let mut pick = Vec::new();
        for (folder, n) in [("videos/fake/Sora/", 100), ("videos/fake/VEO3/", 100), ("videos/real/MSVD/", 200)] {
            pick.extend(files.iter().filter(|f| f.path.starts_with(folder)).skip(150).take(n).cloned());
        }
        self.hf_files("OmerXYZ/comgenvid", &pick, "../fresh/comgenvid")?;
        self.index("data/fresh/comgenvid", "comgenvid-fresh", true, &[("fake/Sora", 1, "Sora"), ("fake/VEO3", 1, "Veo3"), ("real/MSVD", 0, "msvd")])?;

        let used = std::fs::read_to_string("data/vstore/genvideo-val/pick.list").unwrap_or_default().lines().map(String::from).collect();
        let picked = self.genvideo_val("data/fresh/genvideo-val", "genvideo-val-fresh", 300, 50, used)?;
        if !self.dry {
            std::fs::write("data/fresh/genvideo-val/pick2.list", picked.join("\n") + "\n")?;
        }
        Ok(())
    }

    fn confirm2(&self) -> Result<()> {
        self.sofake_ood(&["00009", "00027"], "data/fresh3/sofake-ood", "sofake-ood-fresh2")?;
        self.drop_duplicates("data/fresh3/sofake-ood/manifest.tsv", &["data/store/sofake-ood/manifest.tsv", "data/fresh/sofake-ood/manifest.tsv"])?;
        self.aigibench("Midjourney", "data/fresh3/aigibench-Midjourney", "aigibench-midjourney", "Midjourney", 1500)?;
        self.aigibench("FLUX1-dev", "data/fresh3/aigibench-FLUX1-dev", "aigibench-flux1dev", "FLUX.1-dev", 1500)
    }

    /// The batch that confirms the phase-3 model. Nothing here was scored by
    /// any model before the final weights and thresholds were frozen.
    fn confirm3(&self) -> Result<()> {
        self.sofake_ood(&["00003", "00021", "00039"], "data/fresh4/sofake-ood", "sofake-ood-fresh3")?;
        self.drop_duplicates("data/fresh4/sofake-ood/manifest.tsv", &EVAL_IMAGE_MANIFESTS)?;
        self.drop_duplicates("data/fresh4/sofake-ood/manifest.tsv", &TRAIN_IMAGE_MANIFESTS)?;
        for (name, gen) in [("DALLE-3", "DALL-E 3"), ("SD3", "SD3"), ("SDXL", "SDXL"), ("Imagen3", "Imagen 3")] {
            let out = format!("data/fresh4/aigibench-{name}");
            self.aigibench(name, &out, &format!("aigibench-{}", name.to_lowercase()), gen, 1000)?;
            self.drop_duplicates(&format!("{out}/manifest.tsv"), &EVAL_IMAGE_MANIFESTS)?;
            self.drop_duplicates(&format!("{out}/manifest.tsv"), &TRAIN_IMAGE_MANIFESTS)?;
        }

        for g in ["Gen3", "causvid_24fps"] {
            self.aigvd(&format!("ClosedSource/{g}"), &format!("data/fresh4/video/aigvd-{g}"), Select { evenly: 100, ..Default::default() })?;
        }
        for g in ["RepVideo", "IPOC"] {
            self.aigvd(&format!("OpenSource/T2V/{g}"), &format!("data/fresh4/video/aigvd-{g}"), Select { evenly: 100, ..Default::default() })?;
        }
        self.kinetics("test/part_2.tar.gz", "data/fresh4/video/kinetics", 200)?;
        self.index(
            "data/fresh4/video",
            "aigvd-fresh2",
            true,
            &[("aigvd-Gen3/", 1, "Gen-3"), ("aigvd-causvid_24fps/", 1, "CausVid"), ("aigvd-RepVideo/", 1, "RepVideo"), ("aigvd-IPOC/", 1, "IPOC"), ("kinetics/", 0, "kinetics400-test")],
        )?;

        let files = self.hf_folder("OmerXYZ/comgenvid", "videos/")?;
        let mut pick = Vec::new();
        for (folder, n) in [("videos/fake/Sora/", 100), ("videos/fake/VEO3/", 100), ("videos/real/MSVD/", 200)] {
            pick.extend(files.iter().filter(|f| f.path.starts_with(folder)).skip(350).take(n).cloned());
        }
        self.hf_files("OmerXYZ/comgenvid", &pick, "../fresh4/comgenvid")?;
        self.index("data/fresh4/comgenvid", "comgenvid-fresh2", true, &[("fake/Sora", 1, "Sora"), ("fake/VEO3", 1, "Veo3"), ("real/MSVD", 0, "msvd")])?;

        let mut used: Vec<String> = Vec::new();
        for f in ["data/vstore/genvideo-val/pick.list", "data/fresh/genvideo-val/pick2.list"] {
            used.extend(std::fs::read_to_string(f).unwrap_or_default().lines().map(String::from));
        }
        let picked = self.genvideo_val("data/fresh4/genvideo-val", "genvideo-val-fresh2", 200, 30, used)?;
        if !self.dry {
            std::fs::write("data/fresh4/genvideo-val/pick3.list", picked.join("\n") + "\n")?;
        }
        Ok(())
    }

    /// The batch that confirms the video checkpoint. No clip here was scored
    /// by any model before that checkpoint and its threshold were frozen, and
    /// none of the AIGVDBench generators below is in any training data.
    fn confirm4(&self) -> Result<()> {
        // AccVideo is new; the other four get the clips half a step away from batch 2's.
        self.aigvd("OpenSource/T2V/AccVideo", "data/fresh5/video/aigvd-AccVideo", Select { evenly: 150, ..Default::default() })?;
        for g in ["Gen3", "causvid_24fps"] {
            self.aigvd(&format!("ClosedSource/{g}"), &format!("data/fresh5/video/aigvd-{g}"), Select { evenly: 100, offset_half: true, ..Default::default() })?;
        }
        for g in ["RepVideo", "IPOC"] {
            self.aigvd(&format!("OpenSource/T2V/{g}"), &format!("data/fresh5/video/aigvd-{g}"), Select { evenly: 100, offset_half: true, ..Default::default() })?;
        }
        self.kinetics("test/part_3.tar.gz", "data/fresh5/video/kinetics", 300)?;
        self.index(
            "data/fresh5/video",
            "aigvd-fresh3",
            true,
            &[("aigvd-AccVideo/", 1, "AccVideo"), ("aigvd-Gen3/", 1, "Gen-3"), ("aigvd-causvid_24fps/", 1, "CausVid"), ("aigvd-RepVideo/", 1, "RepVideo"), ("aigvd-IPOC/", 1, "IPOC"), ("kinetics/", 0, "kinetics400-test")],
        )?;
        let earlier: Vec<&str> = DEV_VIDEO_MANIFESTS.iter().chain(TRAIN_VIDEO_MANIFESTS.iter()).chain(PHASE4_VIDEO_MANIFESTS.iter()).copied().collect();
        self.drop_duplicates("data/fresh5/video/manifest.tsv", &earlier)?;

        let files = self.hf_folder("OmerXYZ/comgenvid", "videos/")?;
        let mut pick = Vec::new();
        for (folder, skip, n) in [("videos/fake/Sora/", 450, 150), ("videos/fake/VEO3/", 450, 150), ("videos/real/MSVD/", 550, 300)] {
            pick.extend(files.iter().filter(|f| f.path.starts_with(folder)).skip(skip).take(n).cloned());
        }
        self.hf_files("OmerXYZ/comgenvid", &pick, "../fresh5/comgenvid")?;
        self.index("data/fresh5/comgenvid", "comgenvid-fresh3", true, &[("fake/Sora", 1, "Sora"), ("fake/VEO3", 1, "Veo3"), ("real/MSVD", 0, "msvd")])?;
        self.drop_duplicates("data/fresh5/comgenvid/manifest.tsv", &earlier)?;

        let mut used: Vec<String> = Vec::new();
        for f in ["data/vstore/genvideo-val/pick.list", "data/fresh/genvideo-val/pick2.list", "data/fresh4/genvideo-val/pick3.list"] {
            used.extend(std::fs::read_to_string(f).unwrap_or_default().lines().map(String::from));
        }
        let picked = self.genvideo_val("data/fresh5/genvideo-val", "genvideo-val-fresh3", 300, 40, used)?;
        if !self.dry {
            std::fs::write("data/fresh5/genvideo-val/pick4.list", picked.join("\n") + "\n")?;
        }
        Ok(())
    }

    /// The batch that confirms the final photo configuration (balanced
    /// default threshold, early exit). Three more So-Fake-OOD shards and every
    /// AIGIBench test archive not used before, except the five face-swap
    /// ones (local edits are out of scope).
    fn confirm5(&self) -> Result<()> {
        let mut earlier: Vec<&str> = EVAL_IMAGE_MANIFESTS.to_vec();
        earlier.extend(CONFIRM3_IMAGE_MANIFESTS);
        // A set that is already indexed is left alone, so the plan can resume.
        let have = |dir: &str| !self.dry && Path::new(dir).join("manifest.tsv").exists();
        if !have("data/fresh6/sofake-ood") {
            self.sofake_ood(&["00015", "00030", "00042"], "data/fresh6/sofake-ood", "sofake-ood-fresh4")?;
            self.drop_duplicates("data/fresh6/sofake-ood/manifest.tsv", &earlier)?;
            self.drop_duplicates("data/fresh6/sofake-ood/manifest.tsv", &TRAIN_IMAGE_MANIFESTS)?;
        }
        for (name, gen) in CONFIRM5_AIGIBENCH {
            let out = format!("data/fresh6/aigibench-{name}");
            if have(&out) {
                continue;
            }
            self.aigibench(name, &out, &format!("aigibench-{}", name.to_lowercase()), gen, 400)?;
            self.drop_duplicates(&format!("{out}/manifest.tsv"), &earlier)?;
            self.drop_duplicates(&format!("{out}/manifest.tsv"), &TRAIN_IMAGE_MANIFESTS)?;
        }
        Ok(())
    }

    /// The batch that confirms the phase-6 photo weights. No image here was
    /// scored by any model before those weights and their thresholds were
    /// frozen, and none is in any training data.
    fn confirm6(&self) -> Result<()> {
        let mut earlier: Vec<String> = EVAL_IMAGE_MANIFESTS.iter().chain(CONFIRM3_IMAGE_MANIFESTS.iter()).map(|m| m.to_string()).collect();
        earlier.push("data/fresh6/sofake-ood/manifest.tsv".into());
        earlier.extend(CONFIRM5_AIGIBENCH.iter().map(|(name, _)| format!("data/fresh6/aigibench-{name}/manifest.tsv")));
        earlier.extend(TRAIN_IMAGE_MANIFESTS.iter().chain(PHASE6_IMAGE_MANIFESTS.iter()).map(|m| m.to_string()));
        let have = |dir: &str| !self.dry && Path::new(dir).join("manifest.tsv").exists();
        let dedupe = |manifest: String, seen: &[String]| -> Result<()> { self.drop_duplicates(&manifest, &seen.iter().map(String::as_str).collect::<Vec<_>>()) };

        // Current commercial generators in social-media photos: three unused shards.
        if !have("data/fresh7/sofake-ood") {
            self.sofake_ood(&["00001", "00033", "00045"], "data/fresh7/sofake-ood", "sofake-ood-fresh5")?;
            dedupe("data/fresh7/sofake-ood/manifest.tsv".into(), &earlier)?;
        }

        // The AIGIBench archives of batch 5 again: 400 per class, half a step
        // away from that batch's files. Anything shared with it, with an
        // earlier archive of this batch, or with training data is dropped.
        let mut seen = earlier.clone();
        for (name, gen) in CONFIRM5_AIGIBENCH {
            let out = format!("data/fresh7/aigibench-{name}");
            if !have(&out) {
                self.aigibench_sample(name, &out, &format!("aigibench-{}-b2", name.to_lowercase()), gen, 400, true)?;
                dedupe(format!("{out}/manifest.tsv"), &seen)?;
            }
            seen.push(format!("{out}/manifest.tsv"));
        }

        // The classic GAN test set (Wang et al., CVPR 2020): 300 per class of each family.
        let url = fetch::hf_url("sywang/CNNDetection", "CNN_synth_testset.zip");
        for (folder, gen) in CONFIRM6_CLASSIC {
            let out = format!("data/fresh7/classic-{folder}");
            if !have(&out) {
                self.say(&format!("{url}: {folder}, 300 files per class -> {out}"));
                if !self.dry {
                    let mut all = fetch::list_remote_zip(&url)?;
                    all.sort();
                    for class in ["0_real", "1_fake"] {
                        let of_class: Vec<String> = all.iter().filter(|n| n.starts_with(&format!("{folder}/")) && n.contains(&format!("/{class}/"))).cloned().collect();
                        let sel = Select { exts: IMAGE_EXTS.to_vec(), only: fetch::evenly(&of_class, 300, false), ..Default::default() };
                        let names = fetch::unpack_remote_zip(&url, Path::new(&out), &sel, self.conns)?;
                        println!("{url}: extracted {} files -> {out}", names.len());
                    }
                }
                self.index(&out, &format!("classic-{folder}"), false, &[("0_real", 0, "classic-gan-real"), ("1_fake", 1, gen)])?;
                dedupe(format!("{out}/manifest.tsv"), &seen)?;
            }
            seen.push(format!("{out}/manifest.tsv"));
        }
        Ok(())
    }
}

impl Run {
    /// `per_class` files per class of a remote archive with `0_real` /
    /// `1_fake` folders under `prefix`, evenly spaced over the files that the
    /// earlier samples in `taken` left out (`half` starts half a step in).
    /// Returns false, having written nothing, when fewer than 100 files of
    /// either class are left. `raw` says the earlier samples were counted
    /// over every archive entry of the class (as batch 6 did for the classic
    /// test set) and not over its image files only.
    #[allow(clippy::too_many_arguments)]
    fn sample_rest(&self, url: &str, prefix: &str, out: &str, dataset: &str, real_gen: &str, gen: &str, taken: &[(usize, bool)], per_class: usize, half: bool, raw: bool) -> Result<bool> {
        self.say(&format!("{url}: {prefix} {per_class} files per class not sampled before -> {out}"));
        if self.dry {
            return Ok(true);
        }
        let mut all = fetch::list_remote_zip(url)?;
        all.sort();
        let mut only = Vec::new();
        for class in ["/0_real/", "/1_fake/"] {
            let images: Vec<String> = fetch::members(&all, &Select { contains: class.into(), exts: IMAGE_EXTS.to_vec(), ..Default::default() }).into_iter().filter(|n| n.starts_with(prefix)).collect();
            let of_class: Vec<String> = if raw { all.iter().filter(|n| n.starts_with(prefix) && n.contains(class)).cloned().collect() } else { images.clone() };
            let rest: Vec<String> = fetch::unsampled(&of_class, taken).into_iter().filter(|n| images.contains(n)).collect();
            println!("{url}: {prefix}{class} has {} files, {} not sampled before", of_class.len(), rest.len());
            if rest.len() < 100 {
                println!("{out}: skipped, fewer than 100 unused files in {class}");
                return Ok(false);
            }
            only.extend(fetch::evenly(&rest, per_class, half));
        }
        let names = fetch::unpack_remote_zip(url, Path::new(out), &Select { exts: IMAGE_EXTS.to_vec(), only, ..Default::default() }, self.conns)?;
        println!("{url}: extracted {} files -> {out}", names.len());
        self.index(out, dataset, false, &[("0_real", 0, real_gen), ("1_fake", 1, gen)])?;
        Ok(true)
    }

    /// Seventh confirmation batch (photos): the first scoring of the photo
    /// second opinion. Fresh files from every collection the earlier photo
    /// batches drew on, and five families of the classic test set that no
    /// batch has used. Nothing here was downloaded before the band was fixed.
    fn confirm7(&self) -> Result<()> {
        let mut seen: Vec<String> = EVAL_IMAGE_MANIFESTS.iter().chain(CONFIRM3_IMAGE_MANIFESTS.iter()).map(|m| m.to_string()).collect();
        for batch in ["fresh6", "fresh7"] {
            seen.push(format!("data/{batch}/sofake-ood/manifest.tsv"));
            seen.extend(CONFIRM5_AIGIBENCH.iter().map(|(name, _)| format!("data/{batch}/aigibench-{name}/manifest.tsv")));
        }
        seen.extend(CONFIRM6_CLASSIC.iter().map(|(folder, _)| format!("data/fresh7/classic-{folder}/manifest.tsv")));
        seen.extend(TRAIN_IMAGE_MANIFESTS.iter().chain(PHASE6_IMAGE_MANIFESTS.iter()).map(|m| m.to_string()));
        let have = |dir: &str| !self.dry && Path::new(dir).join("manifest.tsv").exists();
        let dedupe = |manifest: &str, seen: &[String]| -> Result<()> { self.drop_duplicates(manifest, &seen.iter().map(String::as_str).collect::<Vec<_>>()) };

        // Current commercial generators in social-media photos: three unused shards.
        if !have("data/fresh8/sofake-ood") {
            self.sofake_ood(&CONFIRM7_SOFAKE_SHARDS, "data/fresh8/sofake-ood", "sofake-ood-fresh6")?;
            dedupe("data/fresh8/sofake-ood/manifest.tsv", &seen)?;
        }
        seen.push("data/fresh8/sofake-ood/manifest.tsv".into());

        // AIGIBench: 400 per class from what earlier batches left of each archive.
        for (name, gen, taken) in confirm7_aigibench() {
            let out = format!("data/fresh8/aigibench-{name}");
            let url = fetch::hf_url("HorizonTEL/AIGIBench", &format!("test/{name}.zip"));
            if !have(&out) && self.sample_rest(&url, "", &out, &format!("aigibench-{}-b3", name.to_lowercase()), "aigibench", gen, &taken, 400, false, false)? {
                dedupe(&format!("{out}/manifest.tsv"), &seen)?;
            }
            seen.push(format!("{out}/manifest.tsv"));
        }

        // The classic test set: 300 per class from what batch 6 left of its six
        // families, and from five families not used before. IMLE shares its real
        // photos with CRN, so its sample starts half a step in.
        let url = fetch::hf_url("sywang/CNNDetection", "CNN_synth_testset.zip");
        let again = CONFIRM6_CLASSIC.iter().map(|&(folder, gen)| (folder, gen, vec![(300, false)], false));
        let new = CONFIRM7_CLASSIC.iter().map(|&(folder, gen)| (folder, gen, vec![], folder == "imle"));
        for (folder, gen, taken, half) in again.chain(new) {
            let out = format!("data/fresh8/classic-{folder}");
            if !have(&out) && self.sample_rest(&url, &format!("{folder}/"), &out, &format!("classic-{folder}-b2"), "classic-gan-real", gen, &taken, 300, half, true)? {
                dedupe(&format!("{out}/manifest.tsv"), &seen)?;
            }
            seen.push(format!("{out}/manifest.tsv"));
        }
        Ok(())
    }
}

/// So-Fake-OOD shards of the seventh confirmation batch (no earlier plan uses them).
pub const CONFIRM7_SOFAKE_SHARDS: [&str; 3] = ["00004", "00020", "00037"];

/// Families of the classic test set first used in the seventh confirmation batch.
pub const CONFIRM7_CLASSIC: [(&str, &str); 5] = [("crn", "CRN"), ("imle", "IMLE"), ("san", "SAN"), ("seeingdark", "SITD"), ("deepfake", "FaceForensics++ Deepfakes")];

/// AIGIBench archives of the seventh confirmation batch with the samples
/// earlier batches took from each: (archive, generator label, samples).
pub fn confirm7_aigibench() -> Vec<(&'static str, &'static str, Vec<(usize, bool)>)> {
    let mut v: Vec<(&str, &str, Vec<(usize, bool)>)> = CONFIRM5_AIGIBENCH.iter().map(|&(name, gen)| (name, gen, vec![(400, false), (400, true)])).collect();
    v.extend([("Midjourney", "Midjourney", vec![(1500, false)]), ("FLUX1-dev", "FLUX.1-dev", vec![(1500, false)])]);
    v.extend([("DALLE-3", "DALL-E 3"), ("SD3", "SD3"), ("SDXL", "SDXL"), ("Imagen3", "Imagen 3")].map(|(name, gen)| (name, gen, vec![(1000, false)])));
    v
}

/// Families of the classic GAN test set used in the sixth confirmation batch.
pub const CONFIRM6_CLASSIC: [(&str, &str); 6] = [("stylegan", "StyleGAN"), ("stylegan2", "StyleGAN2"), ("biggan", "BigGAN"), ("cyclegan", "CycleGAN"), ("gaugan", "GauGAN"), ("stargan", "StarGAN")];

/// AIGIBench archives of the fifth confirmation batch: (archive, generator label).
pub const CONFIRM5_AIGIBENCH: [(&str, &str); 12] = [
    ("GLIDE", "GLIDE"),
    ("BLIP", "BLIP-Diffusion"),
    ("IP-Adapter", "IP-Adapter"),
    ("InstantID", "InstantID"),
    ("Infinite-ID", "Infinite-ID"),
    ("PhotoMaker", "PhotoMaker"),
    ("ProGAN", "ProGAN"),
    ("R3GAN", "R3GAN"),
    ("StyleGAN3", "StyleGAN3"),
    ("StyleGAN-XL", "StyleGAN-XL"),
    ("StyleSwim", "StyleSwim"),
    ("WFIR", "WFIR"),
];

/// Video sets scored before the fourth confirmation batch; their real clips
/// calibrate the video checkpoint.
pub const DEV_VIDEO_MANIFESTS: [&str; 10] = [
    "data/vstore/vdev/manifest.tsv",
    "data/vstore/wasitai/manifest.tsv",
    "data/vstore/comgenvid/manifest.tsv",
    "data/vstore/genvideo-val/manifest.tsv",
    "data/fresh/video/manifest.tsv",
    "data/fresh/comgenvid/manifest.tsv",
    "data/fresh/genvideo-val/manifest.tsv",
    "data/fresh4/video/manifest.tsv",
    "data/fresh4/comgenvid/manifest.tsv",
    "data/fresh4/genvideo-val/manifest.tsv",
];

/// Image sets of the third confirmation batch.
pub const CONFIRM3_IMAGE_MANIFESTS: [&str; 5] = [
    "data/fresh4/sofake-ood/manifest.tsv",
    "data/fresh4/aigibench-DALLE-3/manifest.tsv",
    "data/fresh4/aigibench-SD3/manifest.tsv",
    "data/fresh4/aigibench-SDXL/manifest.tsv",
    "data/fresh4/aigibench-Imagen3/manifest.tsv",
];

/// Released checkpoints of the reference detectors: (URL, local path).
pub const BASELINE_WEIGHTS: [(&str, &str); 4] = [
    ("https://huggingface.co/OwensLab/commfor-model-384/resolve/main/model.safetensors", "baselines/commfor-384.safetensors"),
    ("https://huggingface.co/OwensLab/commfor-model-224/resolve/main/model.safetensors", "baselines/commfor-224.safetensors"),
    ("https://huggingface.co/haywoodsloan/ai-image-detector-deploy/resolve/main/model.safetensors", "baselines/swinv2-detector.safetensors"),
    ("https://www.grip.unina.it/download/prog/WaveRep_SynthVideoDet/weights_dinov2_G4.ckpt", "baselines/waverep-g4.ckpt"),
];

pub const TRAIN_IMAGE_MANIFESTS: [&str; 10] = [
    "data/store/openfake/manifest.tsv",
    "data/store/openfake2/manifest.tsv",
    "data/store/openfake-reddit/manifest.tsv",
    "data/store/frontier/manifest.tsv",
    "data/store/frontier-heldout/manifest.tsv",
    "data/store/nano-banana/manifest.tsv",
    "data/store/defactify/manifest.tsv",
    "data/store/commfor-small/manifest.tsv",
    "data/store/openfake-test-reals/manifest.tsv",
    "data/store/sofake-set/manifest.tsv",
];

/// Added in phases 4 and 5, for the video checkpoint only.
pub const PHASE4_VIDEO_MANIFESTS: [&str; 7] = [
    "data/vstore/aigvd3/manifest.tsv",
    "data/vstore/k400c/manifest.tsv",
    "data/vstore/rapidata/manifest.tsv",
    "data/vstore/aigvd4/manifest.tsv",
    "data/vstore/animation/manifest.tsv",
    "data/vstore/gameplay/manifest.tsv",
    "data/vstore/cartoons/manifest.tsv",
];

/// Added in phase 6, for the photo weights.
pub const PHASE6_IMAGE_MANIFESTS: [&str; 6] = [
    "data/store/commfor-small2/manifest.tsv",
    "data/store/tiny-genimage/manifest.tsv",
    "data/store/stylegan3/manifest.tsv",
    "data/store/tpdne/manifest.tsv",
    "data/store/ip-adapter/manifest.tsv",
    "data/store/celeba-hq/manifest.tsv",
];

pub const TRAIN_VIDEO_MANIFESTS: [&str; 9] = [
    "data/vstore/vidguard/manifest.tsv",
    "data/vstore/genbuster/manifest.tsv",
    "data/vstore/genvideo100k/manifest.tsv",
    "data/vstore/aigvd/manifest.tsv",
    "data/vstore/k400/manifest.tsv",
    "data/vstore/pexels/manifest.tsv",
    "data/vstore/k400b/manifest.tsv",
    "data/vstore/pexelsb/manifest.tsv",
    "data/vstore/aigvd2/manifest.tsv",
];

/// Every image set a model has been scored on before the third confirmation batch.
pub const EVAL_IMAGE_MANIFESTS: [&str; 11] = [
    "data/store/dev-openfake-test/manifest.tsv",
    "data/store/sofake-ood/manifest.tsv",
    "data/store/synthbuster/manifest.tsv",
    "data/store/aigibench/manifest.tsv",
    "data/vstore/wasitai/images.tsv",
    "data/fresh/sofake-ood/manifest.tsv",
    "data/fresh/socialrf/manifest.tsv",
    "data/fresh/synthbuster/manifest.tsv",
    "data/fresh3/sofake-ood/manifest.tsv",
    "data/fresh3/aigibench-Midjourney/manifest.tsv",
    "data/fresh3/aigibench-FLUX1-dev/manifest.tsv",
];

/// Build the frontier benchmark's manifests from its `metadata.csv`
/// (`file,id,media,label,variant,generator,...`).
fn wasitai_manifests(root: &Path) -> Result<()> {
    let csv = std::fs::read_to_string(root.join("metadata.csv"))?;
    let mut out: std::collections::BTreeMap<&str, String> = Default::default();
    for l in csv.lines().skip(1) {
        let c: Vec<&str> = l.splitn(7, ',').collect();
        if c.len() < 6 {
            continue;
        }
        let (file, media, label, variant, gen) = (c[0], c[2], c[3], c[4], c[5]);
        let lab = (label == "ai") as u8;
        let gen = if lab == 1 { gen.to_string() } else { "real:wasitai".to_string() };
        let (manifest, dataset) = match (media, variant) {
            ("image", "original") => ("images.tsv", "wasitai-image".to_string()),
            ("image", v) => ("images_social.tsv", format!("wasitai-image-{v}")),
            ("video", "original") => ("manifest.tsv", "wasitai-video".to_string()),
            ("video", v) => ("videos_social.tsv", format!("wasitai-video-{v}")),
            _ => continue,
        };
        out.entry(manifest).or_default().push_str(&format!("{file}\t{lab}\t{gen}\t{dataset}\n"));
    }
    for (name, text) in out {
        std::fs::write(root.join(name), text)?;
    }
    Ok(())
}

pub fn run(plan: &str, dry: bool, conns: usize) -> Result<()> {
    let r = Run { dry, conns };
    match plan {
        "baselines" => {
            for (url, dest) in BASELINE_WEIGHTS {
                r.url(url, dest)?;
            }
            Ok(())
        }
        "train-images" => r.train_images(),
        "train-video" => r.train_video(),
        "phase2" => r.phase2(),
        "phase3" => r.phase3(),
        "phase4" => r.phase4(),
        "phase5" => r.phase5(),
        "phase6" => r.phase6(),
        "dev" => r.dev(),
        "benchmarks" => r.benchmarks(),
        "confirm1" => r.confirm1(),
        "confirm2" => r.confirm2(),
        "confirm3" => r.confirm3(),
        "confirm4" => r.confirm4(),
        "confirm5" => r.confirm5(),
        "confirm6" => r.confirm6(),
        "confirm7" => r.confirm7(),
        "all" => {
            for (p, _) in PLANS.iter().filter(|(p, _)| *p != "all" && *p != "baselines") {
                run(p, dry, conns)?;
            }
            Ok(())
        }
        other => bail!("unknown plan `{other}`; available: {}", PLANS.iter().map(|(p, _)| *p).collect::<Vec<_>>().join(", ")),
    }
}
