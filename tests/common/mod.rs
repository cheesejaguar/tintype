//! Helpers shared by the integration tests. Fixtures are generated here
//! rather than committed: a fixed formula gives the same pixels everywhere.
#![allow(dead_code)]
use std::path::{Path, PathBuf};

pub fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn photo_weights() -> PathBuf {
    repo().join("models/tintype.safetensors")
}

/// A fresh, empty directory for one test.
pub fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("tintype-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The fixture: a 320 x 256 RGB picture of smooth gradients, a few hard-edged
/// shapes and fine pseudo-random grain, from integer arithmetic only, so that
/// the same pixels come out on every platform. Large enough for every view of
/// the photo model (its native crops are 224 px), small enough to be quick.
pub fn fixture_pixels() -> image::RgbImage {
    image::RgbImage::from_fn(320, 256, |x, y| {
        // Grain: a 32-bit integer hash of the position.
        let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77);
        h ^= h >> 15;
        h = h.wrapping_mul(0x2C1B_3C6D);
        h ^= h >> 12;
        let grain = (h & 31) as i32 - 16;
        let (cx, cy) = (x as i32 - 160, y as i32 - 128);
        let disc = cx * cx + cy * cy < 70 * 70;
        let bar = (40..80).contains(&y) && x > 200;
        let base = [(x * 255 / 319) as i32, (y * 255 / 255) as i32, ((x + y) * 255 / 574) as i32];
        let c = |i: usize| -> u8 {
            let mut v = base[i] + grain;
            if disc {
                v = 255 - v / 2;
            }
            if bar {
                v = [30, 200, 90][i] + grain / 2;
            }
            v.clamp(0, 255) as u8
        };
        image::Rgb([c(0), c(1), c(2)])
    })
}

/// Write the fixture as a PNG (lossless, so decoding gives the pixels back exactly).
pub fn write_fixture(path: &Path) {
    fixture_pixels().save(path).unwrap();
}
