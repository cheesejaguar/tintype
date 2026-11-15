//! Work counters for diagnostics: how many frames were decoded and converted,
//! how many views were prepared and how many went through the network, how
//! long each of those took, and where the time inside a CPU forward pass goes.
//!
//! They are process-wide atomics (a relaxed add per frame or view, nothing
//! per pixel) and are only ever printed when `TINTYPE_STATS` is set, on
//! standard error, so the normal output of every command is unchanged. They
//! exist so that a change that claims to save work can show that it does.
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

macro_rules! counters {
    ($($(#[$doc:meta])* $name:ident => $label:literal,)*) => {
        $($(#[$doc])* pub static $name: Counter = Counter(AtomicU64::new(0));)*
        const ALL: &[(&Counter, &str)] = &[$((&$name, $label)),*];
    };
}

pub struct Counter(AtomicU64);

impl Counter {
    pub fn add(&self, n: usize) {
        self.0.fetch_add(n as u64, Ordering::Relaxed);
    }

    /// Add the time since `start`, for the counters that hold nanoseconds.
    /// Time spent on several threads at once adds up.
    pub fn since(&self, start: Instant) {
        self.0.fetch_add(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
    }

    /// Add nanoseconds measured elsewhere.
    pub fn add_nanos(&self, nanos: u64) {
        self.0.fetch_add(nanos, Ordering::Relaxed);
    }

    /// Run `f` and add the time it took.
    pub fn time<R>(&self, f: impl FnOnce() -> R) -> R {
        let start = Instant::now();
        let r = f();
        self.since(start);
        r
    }

    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

counters! {
    /// Video frames a decoder handed back, whether or not they were sampled.
    DECODED_FRAMES => "frames_decoded",
    /// Decoded frames converted from YCbCr to RGB.
    CONVERTED_FRAMES => "frames_converted",
    /// Views cropped or resized out of an image or frame.
    MATERIALISED_VIEWS => "views_materialised",
    /// Views that went through the network (padding is not counted).
    SCORED_VIEWS => "views_scored",
    /// Views whose score was taken from an identical view of the same image.
    REUSED_VIEWS => "views_reused",
    /// Clips opened a second time to decode their second half.
    REOPENED_CLIPS => "clips_reopened",
    /// Nanoseconds decoding image files and video frames (not converting them).
    DECODE_TIME => "decode_ms",
    /// Nanoseconds converting decoded frames to RGB.
    CONVERT_TIME => "convert_ms",
    /// Nanoseconds ranking windows, cutting views out and putting them in the engine's form.
    VIEWS_TIME => "views_ms",
    /// Nanoseconds in the network.
    MODEL_TIME => "model_ms",
    /// Nanoseconds in CPU forward passes, each pass timed on the thread that
    /// ran it: the same wall time as `model_ms` when a batch goes through as
    /// one pass, more where the views of a batch run side by side (the
    /// pure-Rust kernels). The counters after it divide this one up; what
    /// they leave over is allocating buffers and gathering the class tokens.
    FORWARD_TIME => "forward_ms",
    /// Of the forward pass: unfolding and normalising pixels, the patch embedding and the position embedding.
    PATCH_TIME => "patch_ms",
    /// Of the forward pass: layer normalisation.
    NORM_TIME => "norm_ms",
    /// Of the forward pass: the query, key and value projection of every block.
    QKV_TIME => "qkv_ms",
    /// Of the forward pass: the query-key and attention-value products.
    ATTENTION_TIME => "attention_ms",
    /// Of the forward pass: the softmax over the query-key products.
    SOFTMAX_TIME => "softmax_ms",
    /// Of the forward pass: the rest of attention, which is copying each head's queries, keys and values together and its output back.
    GATHER_TIME => "gather_ms",
    /// Of the forward pass: the projection after attention.
    PROJ_TIME => "proj_ms",
    /// Of the forward pass: the first linear layer of every MLP.
    FC1_TIME => "fc1_ms",
    /// Of the forward pass: GELU.
    GELU_TIME => "gelu_ms",
    /// Of the forward pass: the second linear layer of every MLP.
    FC2_TIME => "fc2_ms",
    /// Of the forward pass: the classification head.
    HEAD_TIME => "head_ms",
    /// Of the forward pass: adding each half-block's output to the residual stream.
    RESIDUAL_TIME => "residual_ms",
}

/// Whether `TINTYPE_STATS` asks for the counters (any value but empty or `0`).
pub fn wanted() -> bool {
    std::env::var_os("TINTYPE_STATS").is_some_and(|v| !v.is_empty() && v != "0")
}

/// One line: `stats: name=value ...`, in a fixed order. Times are whole
/// milliseconds, added up over the threads that spent them.
pub fn line() -> String {
    let mut s = String::from("stats:");
    for (c, label) in ALL {
        let v = if label.ends_with("_ms") { (c.get() + 500_000) / 1_000_000 } else { c.get() };
        s.push_str(&format!(" {label}={v}"));
    }
    s
}

/// Print the counters on standard error when `TINTYPE_STATS` is set.
pub fn report() {
    if wanted() {
        eprintln!("{}", line());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_names_every_counter_once_in_order() {
        let l = line();
        assert!(l.starts_with("stats: frames_decoded="));
        let names: Vec<&str> = l.split_whitespace().skip(1).map(|f| f.split('=').next().unwrap()).collect();
        assert_eq!(names, ["frames_decoded", "frames_converted", "views_materialised", "views_scored", "views_reused", "clips_reopened", "decode_ms", "convert_ms", "views_ms", "model_ms", "forward_ms", "patch_ms", "norm_ms", "qkv_ms", "attention_ms", "softmax_ms", "gather_ms", "proj_ms", "fc1_ms", "gelu_ms", "fc2_ms", "head_ms", "residual_ms"]);
        // Counting moves a counter by exactly what was added (other tests may add too).
        let before = REOPENED_CLIPS.get();
        REOPENED_CLIPS.add(3);
        assert!(REOPENED_CLIPS.get() >= before + 3);
        // Times are kept in nanoseconds and shown in milliseconds.
        let start = Instant::now() - std::time::Duration::from_millis(40);
        CONVERT_TIME.since(start);
        let shown: u64 = line().split_whitespace().find_map(|f| f.strip_prefix("convert_ms=")).unwrap().parse().unwrap();
        assert!(shown >= 40, "{shown}");
        // A timed closure hands back its value and adds what it took.
        let before = HEAD_TIME.get();
        assert_eq!(HEAD_TIME.time(|| { std::thread::sleep(std::time::Duration::from_millis(2)); 7 }), 7);
        assert!(HEAD_TIME.get() >= before + 2_000_000);
        HEAD_TIME.add_nanos(5);
        assert!(HEAD_TIME.get() >= before + 2_000_005);
    }
}
