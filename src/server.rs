use crate::demo::{self, ClientAddress, Limiter, Origin};
use crate::detector::{Detector, Device};
use crate::review::{self, Review, ReviewArgs};
use crate::{image_io, video};
use anyhow::{bail, ensure, Context, Result};
use axum::{
    extract::{multipart::MultipartRejection, ConnectInfo, DefaultBodyLimit, Multipart, Request, State},
    http::{header, HeaderValue, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Args;
use std::time::Instant;

/// The W3C `Server-Timing` response header.
const SERVER_TIMING: header::HeaderName = header::HeaderName::from_static("server-timing");
use serde_json::{json, Value};
use std::borrow::Cow;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use subtle::ConstantTimeEq;
use tokio::sync::{mpsc, oneshot, Semaphore};

#[derive(Args)]
pub struct ServeArgs {
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: SocketAddr,
    #[arg(long)]
    weights: Option<PathBuf>,
    #[arg(long)]
    video_weights: Option<PathBuf>,
    #[arg(long, default_value = "auto")]
    device: String,
    /// Score photos with Tintype's own network only, even when the bundled
    /// second network is installed beside the photo weights (see
    /// `detect --no-photo-second-opinion`).
    #[arg(long)]
    no_photo_second_opinion: bool,
    /// Score photos in turbo mode (the eight-bit CPU engine, `cpu-int8`)
    /// unless a request asks for `?mode=standard`. Without it, requests
    /// opt in with `?mode=turbo`. Videos always use the standard engine.
    #[arg(long, conflicts_with = "device")]
    turbo: bool,
    #[command(flatten)]
    review: ReviewArgs,
    /// Run the public demo: no token, one photo per request, and rate
    /// limits that cannot be turned off (see the API documentation). Needs
    /// `--demo-origin`. Videos are not accepted and the video model is not
    /// loaded.
    #[arg(long, requires = "demo_origin")]
    demo: bool,
    /// The website origin whose pages may call the demo, such as
    /// https://example.org. Requests from other origins are refused.
    #[arg(long, requires = "demo")]
    demo_origin: Option<String>,
    /// Where the demo reads the caller's address for its rate limits:
    /// `peer` (the TCP connection) or `last-forwarded-for` (the last
    /// X-Forwarded-For entry, appended by a trusted front end such as
    /// Cloud Run's).
    #[arg(long, value_enum, default_value = "peer", requires = "demo")]
    client_address: ClientAddress,
}

/// The review bands in use, for photos and for clips. `None` leaves the
/// responses exactly as they are without the feature.
#[derive(Clone, Copy, Default)]
struct Bands {
    photo: Option<Review>,
    clip: Option<Review>,
}

const MAX_REQUEST: usize = 64 * 1024 * 1024;
const MAX_FILES: usize = 16;
const MAX_ACTIVE: usize = 4;
/// The most pixels an uploaded image may have, and the longest side. The
/// detector reads native-resolution crops, so images are never downscaled
/// to fit: this is a memory guard, sized for 48 MP phone photos.
pub const MAX_PIXELS: u64 = 50_000_000;
const MAX_SIDE: u32 = 16_384;

type ApiResult<T> = std::result::Result<T, ApiError>;

struct ApiError(StatusCode, &'static str, Cow<'static, str>);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = (
            self.0,
            Json(json!({"error": {"code": self.1, "message": self.2}})),
        )
            .into_response();
        if self.0 == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, "Bearer".parse().unwrap());
        }
        if self.0 == StatusCode::SERVICE_UNAVAILABLE {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, "1".parse().unwrap());
        }
        response
    }
}

fn bad_request(message: &'static str) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, "invalid_upload", message.into())
}

fn unavailable() -> ApiError {
    ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "busy",
        "inference is busy or unavailable".into(),
    )
}

#[derive(Clone)]
struct AppState {
    token: Option<Arc<str>>,
    active: Arc<Semaphore>,
    jobs: mpsc::Sender<Job>,
    /// Whether a request without `mode` is scored in turbo mode (`serve --turbo`).
    turbo_default: bool,
    /// The public demo (`serve --demo`), or `None` for the API.
    demo: Option<Arc<Demo>>,
}

/// The public demo's state: its limiter, the one origin it serves, and
/// aggregate counts since the process started (no per-caller data).
struct Demo {
    limiter: Limiter,
    origin: Origin,
    address: ClientAddress,
    scored: AtomicU64,
    refused: [AtomicU64; 3],
}

impl Demo {
    fn new(origin: &str, address: ClientAddress) -> Result<Self> {
        Ok(Self {
            // `LIMITS` only: there is no way to start the demo without them.
            limiter: Limiter::new(demo::LIMITS)?,
            origin: Origin::parse(origin)?,
            address,
            scored: AtomicU64::new(0),
            refused: Default::default(),
        })
    }

    /// The largest request body: one photo at the cap, plus the multipart framing.
    fn body_limit(&self) -> usize {
        self.limiter.limits().max_bytes + 64 * 1024
    }

    fn counts(&self) -> (u64, [u64; 3]) {
        (
            self.scored.load(Ordering::Relaxed),
            self.refused.each_ref().map(|c| c.load(Ordering::Relaxed)),
        )
    }

    /// A 429 that says which limit, with the rate-limit fields and `Retry-After`.
    fn refuse(&self, status: demo::Status) -> Response {
        if let Some(blocked) = status.blocked() {
            let index = blocked.limit as usize;
            self.refused[index].fetch_add(1, Ordering::Relaxed);
        }
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(status.refusal().unwrap_or_else(|| json!({"error": {"code": "rate_limited"}}))),
        )
            .into_response();
        if let Some(wait) = status.retry_after() {
            response.headers_mut().insert(header::RETRY_AFTER, wait.into());
        }
        response
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The demo caller's limiter key, from `demo_gate` to `demo_detect`.
#[derive(Clone, Copy)]
struct Caller(demo::Key);

struct Job {
    files: Vec<Upload>,
    /// Score the photos with the eight-bit engine.
    turbo: bool,
    response: oneshot::Sender<(Vec<Value>, WorkTimes)>,
    queued: Instant,
    _slot: tokio::sync::OwnedSemaphorePermit,
}

/// Where a request's time went, for its `Server-Timing` header: durations
/// only, nothing about the caller or the photo. The worker scores one job
/// at a time, so what the work counters ([`crate::stats`]) gain while it
/// runs is that job's.
#[derive(Clone, Copy, Default)]
struct WorkTimes {
    /// Waiting for the worker.
    queue: Duration,
    /// Decoding, preparing views and the network, each added up over the
    /// threads that did it.
    decode_ns: u64,
    views_ns: u64,
    network_ns: u64,
    /// The worker's time on the job.
    score: Duration,
}

impl WorkTimes {
    /// `Server-Timing` (W3C) entries in milliseconds, `upload` being the time
    /// spent reading the request body.
    fn header(&self, upload: Duration) -> HeaderValue {
        let ms = |d: Duration| d.as_secs_f64() * 1e3;
        let ns = |n: u64| n as f64 / 1e6;
        let value = format!(
            "upload;dur={:.1}, queue;dur={:.1}, decode;dur={:.1}, views;dur={:.1}, network;dur={:.1}, total;dur={:.1}",
            ms(upload),
            ms(self.queue),
            ns(self.decode_ns),
            ns(self.views_ns),
            ns(self.network_ns),
            ms(self.queue + self.score)
        );
        HeaderValue::from_str(&value).expect("ASCII")
    }
}

impl Job {
    fn execute(self, photo: &Detector, second: Option<&Detector>, clip: Option<&Detector>, frames: usize, bands: Bands) {
        use crate::stats::{CONVERT_TIME, DECODE_TIME, MODEL_TIME, VIEWS_TIME};
        // Turbo mode scores photos with the first stage alone, as `detect --turbo` does.
        let second = second.filter(|_| !self.turbo);
        let start = Instant::now();
        let before = [DECODE_TIME.get() + CONVERT_TIME.get(), VIEWS_TIME.get(), MODEL_TIME.get()];
        if let Some(mut rows) = score_uploads(photo, second, clip, frames, bands, self.files, || {
            self.response.is_closed()
        }) {
            let after = [DECODE_TIME.get() + CONVERT_TIME.get(), VIEWS_TIME.get(), MODEL_TIME.get()];
            let times = WorkTimes {
                queue: start - self.queued,
                decode_ns: after[0] - before[0],
                views_ns: after[1] - before[1],
                network_ns: after[2] - before[2],
                score: start.elapsed(),
            };
            if self.turbo {
                // Photos scored in turbo mode say so; clips use the standard engine.
                for row in rows.iter_mut().filter(|r| r["kind"] == "image") {
                    row["mode"] = "turbo".into();
                }
            }
            let _ = self.response.send((rows, times));
        }
    }
}

/// The photo detectors of the worker: the standard one, and the eight-bit
/// one for turbo mode, loaded on the first turbo request (or at start with
/// `serve --turbo`). When the server's own engine is `cpu-int8`, both are
/// the same detector.
struct Photos {
    weights: PathBuf,
    standard: Detector,
    turbo: Option<Detector>,
    /// Whether the turbo kernel has been named on standard error.
    announced: bool,
}

impl Photos {
    fn new(weights: PathBuf, device: Device, preload_turbo: bool) -> Result<Self> {
        let standard = Detector::load(&weights, None, 32, device)?;
        let mut photos = Self { weights, standard, turbo: None, announced: false };
        if preload_turbo {
            photos.get(true)?;
        }
        Ok(photos)
    }

    fn get(&mut self, turbo: bool) -> Result<&Detector> {
        if !turbo || matches!(self.standard.device(), Device::CpuInt8) {
            return Ok(&self.standard);
        }
        if self.turbo.is_none() {
            if !self.standard.device().is_cpu() {
                eprintln!("note: turbo mode scores photos on the eight-bit CPU engine; on this Mac the GPU engine (mode=standard) is faster");
            }
            self.turbo = Some(Detector::load(&self.weights, None, 32, Device::CpuInt8)?);
        }
        if !self.announced {
            // Says which kernel turbo runs here, so a host's CPU can be checked from its logs.
            let kernel = self.turbo.as_ref().unwrap_or(&self.standard).int8_kernel();
            eprintln!("turbo mode: photos run on the eight-bit engine, kernel: {}", kernel.unwrap_or("none (simulated)"));
            self.announced = true;
        }
        Ok(self.turbo.as_ref().unwrap())
    }
}

struct Upload {
    name: String,
    bytes: Vec<u8>,
}

pub fn run(
    args: ServeArgs,
    resolve_weights: impl FnOnce(Option<PathBuf>) -> Result<PathBuf>,
) -> Result<()> {
    let token = match std::env::var("TINTYPE_API_TOKEN") {
        Ok(token) => Some(token),
        Err(std::env::VarError::NotPresent) => None,
        Err(error) => return Err(error).context("reading TINTYPE_API_TOKEN"),
    };
    if let Some(token) = &token {
        ensure!(
            !token.is_empty()
                && token.len() <= 512
                && token.trim_end_matches('=').bytes().all(|b| {
                    b.is_ascii_alphanumeric()
                        || matches!(b, b'-' | b'.' | b'_' | b'~' | b'+' | b'/')
                })
                && !token.starts_with('='),
            "TINTYPE_API_TOKEN must be a bearer token of 1 to 512 ASCII bytes"
        );
    }
    let demo = match (args.demo, &args.demo_origin) {
        (true, Some(origin)) => {
            ensure!(
                token.is_none(),
                "demo mode is public: unset TINTYPE_API_TOKEN, and run the API as a separate service"
            );
            Some(Arc::new(Demo::new(origin, args.client_address)?))
        }
        (true, None) => bail!("demo mode needs --demo-origin"),
        (false, _) => None,
    };
    ensure!(
        args.bind.ip().is_loopback() || token.is_some() || demo.is_some(),
        "set TINTYPE_API_TOKEN when binding outside loopback"
    );
    let device = match args.device.as_str() {
        "auto" => Device::best(),
        "cpu" => Device::Cpu,
        "gpu" => Device::Gpu,
        "gpu-f32" => Device::GpuF32,
        "cpu-int8" => Device::CpuInt8,
        _ => bail!("--device must be auto, cpu, cpu-int8, gpu, or gpu-f32"),
    };
    let photo = resolve_weights(args.weights)?;
    let clip = args
        .video_weights
        .unwrap_or_else(|| crate::detector::video_weights_for(&photo));
    // A response has nowhere to carry a warning: experimental weights whose
    // calibration was never fitted are not served at all.
    for weights in [&photo, &clip] {
        let unfitted = crate::detector::ModelSpec::load_for(weights)
            .ok()
            .and_then(|spec| spec.uncalibrated_warning(weights));
        if let Some(warning) = unfitted {
            bail!("refusing to serve uncalibrated weights. {warning}");
        }
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let (jobs, mut receive) = mpsc::channel::<Job>(MAX_ACTIVE);
    let (ready, startup) = oneshot::channel();
    let review = args.review;
    let turbo_default = args.turbo;
    let no_photo_second = args.no_photo_second_opinion;
    // The demo takes photos only: its video model is never loaded, so the
    // video path, which writes the upload to a temporary file, cannot run.
    let photos_only = demo.is_some();
    // Videos stay on the standard engine in turbo mode, so `--turbo` keeps
    // the best engine here for clips and loads the eight-bit one for photos.
    let worker = std::thread::spawn(move || {
        let setup = (|| -> Result<_> {
            // The photo second opinion: the bundled network beside the photo
            // weights, loaded once for the life of the server.
            let second = match crate::detector::ModelSpec::load_for(&photo)
                .ok()
                .and_then(|spec| crate::detector::bundled_photo_second(&photo, &spec))
                .filter(|_| !no_photo_second)
            {
                Some(path) => Some(crate::detector::load_photo_second(&path, 32, device)?),
                None => None,
            };
            let photos = Photos::new(photo, device, turbo_default)?;
            let photo = &photos.standard;
            ensure!(!photo.spec.views.is_empty(), "model must declare at least one view");
            let (clip, frames) = if photos_only {
                (None, 0)
            } else {
                let mut clip = Detector::load(&clip, None, 64, device)?;
                let frames = clip.use_video_profile();
                ensure!(!clip.spec.views.is_empty(), "model must declare at least one view");
                ensure!(
                    (1..=32).contains(&frames),
                    "video model must sample 1 to 32 frames"
                );
                (Some(clip), frames)
            };
            // Flags first, then the band stored with each model.
            let bands = Bands {
                photo: review.resolve(photo.spec.review),
                clip: clip
                    .as_ref()
                    .and_then(|c| review.resolve(c.spec.video.as_ref().and_then(|v| v.review))),
            };
            Ok((photos, second, clip, frames, bands))
        })();
        let (mut photos, second, clip, frames, bands) = match setup {
            Ok(models) => {
                let _ = ready.send(Ok(()));
                models
            }
            Err(error) => {
                let _ = ready.send(Err(error));
                return;
            }
        };
        while let Some(job) = receive.blocking_recv() {
            match photos.get(job.turbo) {
                Ok(photo) => job.execute(photo, second.as_ref(), clip.as_ref(), frames, bands),
                // Dropping the job answers 503: the eight-bit engine could not load.
                Err(error) => eprintln!("turbo mode unavailable: {error:#}"),
            }
        }
    });
    let result = runtime.block_on(async {
        startup.await.context("starting inference worker")??;
        let state = AppState {
            token: token.map(Arc::from),
            active: Arc::new(Semaphore::new(MAX_ACTIVE)),
            jobs,
            turbo_default,
            demo: demo.clone(),
        };
        let listener = tokio::net::TcpListener::bind(args.bind).await?;
        // A host that gives the container fewer processors than it shows
        // (Cloud Run's second generation) needs RAYON_NUM_THREADS set to its
        // share: more threads than processors there halves the speed.
        eprintln!(
            "inference threads: {} ({} processors visible{})",
            rayon::current_num_threads(),
            std::thread::available_parallelism().map_or(0, usize::from),
            std::env::var("RAYON_NUM_THREADS").map(|n| format!(", RAYON_NUM_THREADS={n}")).unwrap_or_default()
        );
        if let Some(demo) = demo {
            let l = demo.limiter.limits();
            eprintln!(
                "Tintype demo listening on {}: photos only, {} per {} s and {} per day per network, {} per day in total, {} bytes at most",
                listener.local_addr()?, l.short, l.short_window_s, l.daily_per_ip, l.daily_total, l.max_bytes
            );
            tokio::spawn(hourly_summary(demo.clone()));
            // Where the host only gives CPU during requests (Cloud Run), this
            // runs when the server next runs; an idle instance is stopped anyway.
            tokio::spawn(async move {
                let mut tick = tokio::time::interval(Duration::from_secs(60));
                loop {
                    tick.tick().await;
                    demo.limiter.expire(unix_now());
                }
            });
        } else {
            eprintln!("Tintype HTTP listening on {}", listener.local_addr()?);
        }
        axum::serve(listener, app(state).into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(shutdown())
            .await?;
        Ok(())
    });
    worker
        .join()
        .map_err(|_| anyhow::anyhow!("inference worker panicked"))?;
    result
}

/// One aggregate line an hour when the demo did anything: counts only.
async fn hourly_summary(demo: Arc<Demo>) {
    let mut tick = tokio::time::interval(Duration::from_secs(3600));
    tick.tick().await;
    let mut last = demo.counts();
    loop {
        tick.tick().await;
        let now = demo.counts();
        let refused: [u64; 3] = std::array::from_fn(|i| now.1[i] - last.1[i]);
        if now != last {
            eprintln!(
                "demo, last hour: scored {} photos; refused {} over the short-window limit, {} over the daily limit per network, {} over the daily cap",
                now.0 - last.0, refused[0], refused[1], refused[2]
            );
        }
        last = now;
    }
}

fn app(state: AppState) -> Router {
    if let Some(demo) = state.demo.clone() {
        return Router::new()
            .route("/healthz", get(health))
            // Cloud Run answers `/healthz` itself (it reserves paths ending in
            // `z`), so the page wakes the server through `/health`.
            .route("/health", get(health))
            .route(
                "/v1/detect",
                post(demo_detect).route_layer(middleware::from_fn_with_state(state.clone(), demo_gate)),
            )
            .layer(DefaultBodyLimit::max(demo.body_limit()))
            .layer(middleware::from_fn_with_state(demo, demo_cors))
            .with_state(state);
    }
    let inference = Router::new()
        .route("/v1/detect", post(detect))
        .route("/v1/batch", post(batch))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            authorize_upload,
        ))
        .layer(DefaultBodyLimit::max(MAX_REQUEST));
    Router::new()
        .route("/healthz", get(health))
        .route("/health", get(health))
        .merge(inference)
        .with_state(state)
}

async fn shutdown() {
    #[cfg(unix)]
    if let Ok(mut terminate) =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    {
        tokio::select! {
            _ = terminate.recv() => {},
            _ = tokio::signal::ctrl_c() => {},
        }
        return;
    }
    let _ = tokio::signal::ctrl_c().await;
}

async fn authorize_upload(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if let Some(token) = &state.token {
        let provided = request
            .headers()
            .get(header::AUTHORIZATION)
            .map(|v| v.as_bytes())
            .and_then(|v| v.split_first_chunk::<7>())
            .filter(|(prefix, _)| prefix.eq_ignore_ascii_case(b"Bearer "))
            .map(|(_, v)| v);
        if !provided.is_some_and(|v| bool::from(v.ct_eq(token.as_bytes()))) {
            return ApiError(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "supply the configured bearer token".into(),
            )
            .into_response();
        }
    }
    let length = request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    if length.is_some_and(|length| length > MAX_REQUEST as u64) {
        return upload_too_large().into_response();
    }
    next.run(request).await
}

/// Every demo response: the one origin allowed to read it, the headers that
/// page may read, and no caching anywhere. A request from any other origin
/// is refused before it is read.
async fn demo_cors(State(demo): State<Arc<Demo>>, request: Request, next: Next) -> Response {
    let mut response = if !demo.origin.allows(request.headers()) {
        ApiError(
            StatusCode::FORBIDDEN,
            "origin_not_allowed",
            "this demo answers its own website and scripts that send no Origin".into(),
        )
        .into_response()
    } else if request.method() == Method::OPTIONS {
        let mut preflight = StatusCode::NO_CONTENT.into_response();
        let headers = preflight.headers_mut();
        headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("POST"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("content-type"));
        headers.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static("600"));
        preflight
    } else {
        next.run(request).await
    };
    let headers = response.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, demo.origin.value().clone());
    headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, HeaderValue::from_static(demo::EXPOSED));
    headers.append(header::VARY, HeaderValue::from_static("Origin"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    response
}

/// The demo's rate limits around `POST /v1/detect`: refuse a caller over a
/// limit before reading the upload, and tell every caller where they stand.
async fn demo_gate(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let demo = state.demo.clone().expect("the demo gate runs in demo mode only");
    let peer = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|c| c.0);
    // No address at all only happens off the network (tests); such callers share one key.
    let ip = demo
        .address
        .resolve(request.headers(), peer)
        .unwrap_or(std::net::Ipv4Addr::UNSPECIFIED.into());
    let key = demo.limiter.key(ip);
    let status = demo.limiter.status(key, unix_now());
    let mut response = if status.blocked().is_some() {
        demo.refuse(status)
    } else {
        let length = request
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok());
        if length.is_some_and(|length| length > demo.body_limit() as u64) {
            photo_too_large().into_response()
        } else {
            request.extensions_mut().insert(Caller(key));
            next.run(request).await
        }
    };
    for (name, value) in demo.limiter.status(key, unix_now()).headers() {
        if let Ok(value) = HeaderValue::from_str(&value) {
            response.headers_mut().insert(name, value);
        }
    }
    response
}

fn photo_too_large() -> ApiError {
    ApiError(
        StatusCode::PAYLOAD_TOO_LARGE,
        "photo_too_large",
        format!("the demo accepts one photo of up to {} MB", demo::LIMITS.max_bytes / 1_000_000).into(),
    )
}

/// One photo, counted against the caller's limits once it is known to be a
/// photo the demo accepts.
async fn demo_detect(
    State(state): State<AppState>,
    axum::Extension(Caller(key)): axum::Extension<Caller>,
    multipart: std::result::Result<Multipart, MultipartRejection>,
) -> ApiResult<Response> {
    let demo = state.demo.clone().expect("demo mode");
    let multipart =
        multipart.map_err(|_| bad_request("expected multipart/form-data with a boundary"))?;
    let slot = state.active.clone().try_acquire_owned().map_err(|_| unavailable())?;
    let reading = Instant::now();
    let mut files = tokio::time::timeout(
        Duration::from_secs(20),
        parse_upload(multipart, 1, "send one photo per request"),
    )
    .await
    .map_err(|_| ApiError(StatusCode::REQUEST_TIMEOUT, "upload_timeout", "upload timed out".into()))?
    .map_err(|error| if error.0 == StatusCode::PAYLOAD_TOO_LARGE { photo_too_large() } else { error })?;
    let upload = reading.elapsed();
    let file = &mut files[0];
    if file.bytes.len() > demo.limiter.limits().max_bytes {
        return Err(photo_too_large());
    }
    // Photos only, by their bytes. The name is replaced, so nothing the
    // caller calls the file can route it anywhere else.
    let format = image::guess_format(&file.bytes).ok();
    file.name = match format {
        Some(image::ImageFormat::Jpeg) => "photo.jpg",
        Some(image::ImageFormat::Png) => "photo.png",
        Some(image::ImageFormat::WebP) => "photo.webp",
        _ => {
            return Err(ApiError(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported_media",
                "the demo accepts one JPEG, PNG or WebP photo; videos and other files are not accepted".into(),
            ))
        }
    }
    .to_owned();
    // Too many pixels to decode: say so plainly, before counting the photo.
    // The header alone gives the size; nothing is decoded here.
    let size = image::ImageReader::with_format(std::io::Cursor::new(&file.bytes), format.unwrap())
        .into_dimensions()
        .ok();
    if let Some((w, h)) = size.filter(|&(w, h)| u64::from(w) * u64::from(h) > MAX_PIXELS || w.max(h) > MAX_SIDE) {
        return Err(ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            "photo_too_many_pixels",
            format!(
                "this photo is {w} by {h} pixels; the demo accepts photos of up to {} megapixels and {MAX_SIDE} pixels a side. Install Tintype to check larger photos on your own computer",
                MAX_PIXELS / 1_000_000
            )
            .into(),
        ));
    }
    if let Err(status) = demo.limiter.take(key, unix_now()) {
        return Ok(demo.refuse(status));
    }
    demo.scored.fetch_add(1, Ordering::Relaxed);
    let (mut results, times) = queue(&state, files, state.turbo_default, slot).await?;
    let result = results.remove(0);
    let status = if result.get("error").is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    Ok((status, [(SERVER_TIMING, times.header(upload))], Json(result)).into_response())
}

async fn health(State(state): State<AppState>) -> ApiResult<Json<Value>> {
    if state.jobs.is_closed() {
        return Err(unavailable());
    }
    let mut body = json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")});
    if let Some(demo) = &state.demo {
        let (scored, refused) = demo.counts();
        body["demo"] = json!({"scored": scored, "refused": refused.iter().sum::<u64>()});
    }
    Ok(Json(body))
}

async fn detect(
    State(state): State<AppState>,
    uri: Uri,
    multipart: std::result::Result<Multipart, MultipartRejection>,
) -> ApiResult<impl IntoResponse> {
    let (mut results, timing) = submit(state, &uri, multipart, 1).await?;
    let result = results.remove(0);
    let status = if result.get("error").is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    Ok((status, [(SERVER_TIMING, timing)], Json(result)))
}

async fn batch(
    State(state): State<AppState>,
    uri: Uri,
    multipart: std::result::Result<Multipart, MultipartRejection>,
) -> ApiResult<impl IntoResponse> {
    let (results, timing) = submit(state, &uri, multipart, MAX_FILES).await?;
    Ok(([(SERVER_TIMING, timing)], Json(json!({"results": results}))))
}

/// `?mode=turbo` or `?mode=standard`; without it, the server's default.
fn turbo_requested(uri: &Uri, default: bool) -> ApiResult<bool> {
    let mut turbo = default;
    for pair in uri.query().unwrap_or("").split('&').filter(|p| !p.is_empty()) {
        match pair {
            "mode=turbo" => turbo = true,
            "mode=standard" => turbo = false,
            _ if pair.starts_with("mode=") || pair == "mode" => {
                return Err(ApiError(StatusCode::BAD_REQUEST, "invalid_mode", "mode must be standard or turbo".into()))
            }
            _ => {}
        }
    }
    Ok(turbo)
}

async fn submit(
    state: AppState,
    uri: &Uri,
    multipart: std::result::Result<Multipart, MultipartRejection>,
    count: usize,
) -> ApiResult<(Vec<Value>, HeaderValue)> {
    let turbo = turbo_requested(uri, state.turbo_default)?;
    let multipart =
        multipart.map_err(|_| bad_request("expected multipart/form-data with a boundary"))?;
    let slot = state
        .active
        .clone()
        .try_acquire_owned()
        .map_err(|_| unavailable())?;
    let reading = Instant::now();
    let files = tokio::time::timeout(Duration::from_secs(20), parse_upload(multipart, count, "too many file fields"))
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::REQUEST_TIMEOUT,
                "upload_timeout",
                "upload timed out".into(),
            )
        })??;
    let upload = reading.elapsed();
    let (rows, times) = queue(&state, files, turbo, slot).await?;
    Ok((rows, times.header(upload)))
}

/// Hand uploads to the inference worker and wait for its rows.
async fn queue(
    state: &AppState,
    files: Vec<Upload>,
    turbo: bool,
    slot: tokio::sync::OwnedSemaphorePermit,
) -> ApiResult<(Vec<Value>, WorkTimes)> {
    let (response, reply) = oneshot::channel();
    state
        .jobs
        .try_send(Job {
            files,
            turbo,
            response,
            queued: Instant::now(),
            _slot: slot,
        })
        .map_err(|_| unavailable())?;
    tokio::time::timeout(Duration::from_secs(90), reply)
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::GATEWAY_TIMEOUT,
                "inference_timeout",
                "inference timed out".into(),
            )
        })?
        .map_err(|_| unavailable())
}

fn multipart_error(error: axum::extract::multipart::MultipartError) -> ApiError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        upload_too_large()
    } else {
        bad_request("could not read multipart upload")
    }
}

fn upload_too_large() -> ApiError {
    ApiError(
        StatusCode::PAYLOAD_TOO_LARGE,
        "upload_too_large",
        "upload exceeds 64 MiB".into(),
    )
}

async fn parse_upload(
    mut multipart: Multipart,
    max: usize,
    too_many: &'static str,
) -> ApiResult<Vec<Upload>> {
    let mut files = Vec::new();
    while let Some(mut field) = multipart.next_field().await.map_err(multipart_error)? {
        if field.name() != Some("file") {
            return Err(bad_request("expected multipart fields named file"));
        }
        if files.len() >= max {
            return Err(bad_request(too_many));
        }
        let name = field
            .file_name()
            .filter(|n| !n.is_empty() && n.len() <= 255)
            .ok_or_else(|| bad_request("each file needs a filename of at most 255 bytes"))?
            .to_owned();
        let mut bytes = Vec::new();
        while let Some(chunk) = field.chunk().await.map_err(multipart_error)? {
            bytes.extend_from_slice(&chunk);
        }
        files.push(Upload { name, bytes });
    }
    if files.is_empty() {
        return Err(bad_request("upload at least one file"));
    }
    Ok(files)
}

/// `second` is the network of the photo second opinion: with it, every photo
/// inside the photo model's band gets the blended score `detect` gives.
fn score_uploads(
    photo: &Detector,
    second: Option<&Detector>,
    clip: Option<&Detector>,
    frames: usize,
    bands: Bands,
    files: Vec<Upload>,
    cancelled: impl Fn() -> bool + Sync,
) -> Option<Vec<Value>> {
    if cancelled() {
        return None;
    }
    let mut results: Vec<_> = files
        .iter()
        .map(|f| json!({"name": f.name, "error": {"code": "unreadable_media", "message": "media is unsupported, exceeds decode limits, or could not be decoded"}}))
        .collect();
    let mut images = Vec::new();
    for (i, file) in files.into_iter().enumerate() {
        if cancelled() {
            return None;
        }
        let ext = std::path::Path::new(&file.name)
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if matches!(ext.as_str(), "mp4" | "mov" | "m4v") {
            // Without a video model (the demo), a video stays unreadable.
            if let Some(Ok(value)) = clip.map(|clip| score_video(clip, frames, bands.clip, &file, &ext)) {
                results[i] = value;
            }
        } else {
            images.push((i, file));
        }
    }
    if cancelled() {
        return None;
    }
    if !images.is_empty() {
        let exit = photo
            .spec
            .early_exit
            .as_ref()
            .filter(|e| e.covers(photo.spec.views.len()));
        let (scores, _) = match exit {
            Some(exit) => {
                photo.score_staged(images.len(), 2, exit, 1, |i| {
                    if cancelled() {
                        return None;
                    }
                    decode_image(&images[i].1.bytes)
                        .ok()
                        .filter(|_| !cancelled())
                        .map(|image| vec![image])
                })
            }
            None => {
                let (scores, time) = photo.score_stream(images.len(), 2, |i| {
                    if cancelled() {
                        return None;
                    }
                    decode_image(&images[i].1.bytes)
                        .ok()
                        .filter(|_| !cancelled())
                        .map(|image| vec![image])
                });
                (
                    scores
                        .into_iter()
                        .map(|s| s.map(|mut v| v.remove(0)))
                        .collect(),
                    time,
                )
            }
        };
        if cancelled() {
            return None;
        }
        // The second opinion: the photos inside the band, scored once more by
        // the second network and blended, as `detect` does.
        let mut blended: Vec<Option<f32>> = vec![None; images.len()];
        if let Some((second, band)) = second.zip(photo.spec.second_opinion) {
            let first: Vec<Option<f32>> = scores
                .iter()
                .map(|s| s.as_ref().map(|l| photo.calibrated_logit(l)))
                .collect();
            let pick: Vec<usize> = (0..images.len())
                .filter(|&k| first[k].is_some_and(|z| band.wanted(z)))
                .collect();
            if !pick.is_empty() {
                let (res, _) = second.score_stream(pick.len(), 2, |j| {
                    if cancelled() {
                        return None;
                    }
                    decode_image(&images[pick[j]].1.bytes)
                        .ok()
                        .filter(|_| !cancelled())
                        .map(|image| vec![image])
                });
                if cancelled() {
                    return None;
                }
                for (&k, r) in pick.iter().zip(res) {
                    if let (Some(z1), Some(z2)) = (first[k], r.and_then(|mut v| v.pop()).and_then(|v| v.first().copied())) {
                        blended[k] = Some(band.probability(z1, z2)).filter(|p| p.is_finite());
                    }
                }
            }
        }
        for (k, ((i, file), scores)) in images.into_iter().zip(scores).enumerate() {
            if let Some(scores) = scores {
                let score = blended[k].unwrap_or_else(|| photo.probability(&scores));
                if score.is_finite() {
                    results[i] = json!({
                        "name": file.name, "kind": "image", "profile": "default", "score": score,
                        "verdict": verdict(score), "views": scores.len(),
                        "logit": video::aggregate(&scores), "threshold_logit": photo.threshold_logit()
                    });
                    if second.is_some() {
                        results[i]["second_opinion"] = blended[k].is_some().into();
                    }
                    if let Some(band) = bands.photo {
                        // From the view logits already scored: no further inference.
                        // With a second opinion the margin is on the blended score.
                        let reason = band.reason(
                            blended[k].map_or_else(|| photo.calibrated_logit(&scores), review::logit_of),
                            &scores,
                            photo.threshold_logit(),
                        );
                        review::annotate(&mut results[i], reason);
                    }
                }
            }
        }
    }
    Some(results)
}

fn decode_image(bytes: &[u8]) -> Result<image_io::Rgb> {
    let reader = || {
        let mut reader =
            image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()?;
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(MAX_SIDE);
        limits.max_image_height = Some(MAX_SIDE);
        // A 50 MP image is 150 MB as 8-bit RGB, and twice that while a
        // decoder holds 16-bit or alpha samples.
        limits.max_alloc = Some(512 * 1024 * 1024);
        reader.limits(limits);
        Ok::<_, std::io::Error>(reader)
    };
    let (w, h) = reader()?.into_dimensions()?;
    ensure!(
        w > 0 && h > 0 && u64::from(w) * u64::from(h) <= MAX_PIXELS,
        "image too large"
    );
    ensure!(w.max(h) <= w.min(h) * 32, "image aspect ratio too large");
    let start = Instant::now();
    let image = reader()?.decode()?.into_rgb8();
    crate::stats::DECODE_TIME.since(start);
    Ok(image_io::Rgb {
        w,
        h,
        data: image.into_raw(),
    })
}

fn score_video(
    detector: &Detector,
    frames: usize,
    band: Option<Review>,
    file: &Upload,
    ext: &str,
) -> Result<Value> {
    use std::io::Write;
    let mut temporary = tempfile::Builder::new()
        .suffix(&format!(".{ext}"))
        .tempfile()?;
    temporary.write_all(&file.bytes)?;
    crate::swdec::validate_upload(temporary.path())?;
    let sampling = video::Sampling {
        frames,
        ..Default::default()
    };
    let exit = detector
        .spec
        .video
        .as_ref()
        .and_then(|profile| profile.early_exit.as_ref());
    let (mut result, _) =
        detector.score_videos_with_coverage(&[temporary.path().to_owned()], sampling, 1, exit);
    let result = result.pop().flatten().context("unreadable video")?;
    let values: Vec<_> = result
        .frame_logits
        .iter()
        .map(|v| video::aggregate(v))
        .collect();
    let logit = video::aggregate(&values);
    let score = detector.probability(&[logit]);
    ensure!(score.is_finite(), "invalid video score");
    let c = result.coverage;
    let verdict = if c.complete {
        verdict(score)
    } else {
        "inconclusive"
    };
    let mut object = json!({
        "name":file.name,"kind":"video","score":score,"profile":"video","verdict":verdict,
        "logit":logit,"threshold_logit":detector.threshold_logit(),
        "frames":c.frames,"frames_expected":c.frames_expected,"duration_s":c.duration,
        "analysed_until_s":c.until,"complete":c.complete
    });
    if let Some(band) = band {
        // Over the frame logits already scored; `inconclusive` stays what it is.
        let reason = band.reason(
            detector.calibrated_logit(&[logit]),
            &values,
            detector.threshold_logit(),
        );
        review::annotate(&mut object, reason);
    }
    Ok(object)
}

fn verdict(score: f32) -> &'static str {
    if score > 0.5 {
        "ai-generated"
    } else {
        "likely-real"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use std::io::Cursor;
    use tower::ServiceExt;

    fn request(method: &str, uri: &str, auth: Option<&str>, files: &[(&str, &[u8])]) -> Request {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(auth) = auth {
            request = request.header(header::AUTHORIZATION, auth);
        }
        if files.is_empty() {
            return request.body(Body::empty()).unwrap();
        }
        let mut body = Vec::new();
        for (name, bytes) in files {
            body.extend_from_slice(format!("--tintype-test\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\n\r\n").as_bytes());
            body.extend_from_slice(bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(b"--tintype-test--\r\n");
        request
            .header(
                header::CONTENT_TYPE,
                "multipart/form-data; boundary=tintype-test",
            )
            .body(Body::from(body))
            .unwrap()
    }

    /// The `Server-Timing` entries of a response, by name, in milliseconds.
    fn server_timing(response: &Response) -> Vec<(String, f64)> {
        let value = response.headers()[SERVER_TIMING].to_str().unwrap();
        value
            .split(", ")
            .map(|entry| {
                let (name, dur) = entry.split_once(";dur=").expect("name;dur=ms");
                (name.to_string(), dur.parse().expect("milliseconds"))
            })
            .collect()
    }

    fn assert_server_timing(response: &Response) {
        let entries = server_timing(response);
        let names: Vec<&str> = entries.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["upload", "queue", "decode", "views", "network", "total"]);
        assert!(entries.iter().all(|(_, ms)| ms.is_finite() && *ms >= 0.0), "{entries:?}");
    }

    async fn json(response: Response) -> Value {
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
        serde_json::from_slice(&to_bytes(response.into_body(), 1 << 20).await.unwrap()).unwrap()
    }

    #[test]
    fn routes_enforce_auth_limits_and_readiness() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let (jobs, mut worker) = mpsc::channel(MAX_ACTIVE);
                let active = Arc::new(Semaphore::new(MAX_ACTIVE));
                let app = app(AppState {
                    token: Some(Arc::from("secret")),
                    active: active.clone(),
                    jobs,
                    turbo_default: false,
                    demo: None,
                });

                let health = app
                    .clone()
                    .oneshot(request("GET", "/healthz", None, &[]))
                    .await
                    .unwrap();
                assert_eq!(health.status(), StatusCode::OK);
                assert_eq!(json(health).await["status"], "ok");
                let head = app
                    .clone()
                    .oneshot(request("HEAD", "/healthz", None, &[]))
                    .await
                    .unwrap();
                assert_eq!(head.status(), StatusCode::OK);
                assert!(to_bytes(head.into_body(), 0).await.unwrap().is_empty());

                let rejected = app
                    .clone()
                    .oneshot(request("POST", "/v1/detect", None, &[]))
                    .await
                    .unwrap();
                assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
                assert_eq!(rejected.headers()[header::WWW_AUTHENTICATE], "Bearer");
                assert_eq!(json(rejected).await["error"]["code"], "unauthorized");

                let auth = Some("bearer secret");
                let bad = app
                    .clone()
                    .oneshot(request("POST", "/v1/detect", auth, &[]))
                    .await
                    .unwrap();
                assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
                assert_eq!(json(bad).await["error"]["code"], "invalid_upload");
                let files: &[(&str, &[u8])] = &[("a.jpg", b"bad"), ("b.jpg", b"bad")];
                let bad = app
                    .clone()
                    .oneshot(request("POST", "/v1/detect", auth, files))
                    .await
                    .unwrap();
                assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
                assert_eq!(json(bad).await["error"]["code"], "invalid_upload");
                assert!(worker.try_recv().is_err());

                let mut large = request("POST", "/v1/detect", auth, &files[..1]);
                large.headers_mut().insert(
                    header::CONTENT_LENGTH,
                    (MAX_REQUEST + 1).to_string().parse().unwrap(),
                );
                let large = app.clone().oneshot(large).await.unwrap();
                assert_eq!(large.status(), StatusCode::PAYLOAD_TOO_LARGE);
                assert_eq!(json(large).await["error"]["code"], "upload_too_large");

                let held = active.acquire_many(MAX_ACTIVE as u32).await.unwrap();
                let busy = app
                    .clone()
                    .oneshot(request("POST", "/v1/detect", auth, &files[..1]))
                    .await
                    .unwrap();
                assert_eq!(busy.status(), StatusCode::SERVICE_UNAVAILABLE);
                assert_eq!(busy.headers()[header::RETRY_AFTER], "1");
                drop(held);
                drop(worker);
                let health = app
                    .oneshot(request("GET", "/healthz", None, &[]))
                    .await
                    .unwrap();
                assert_eq!(health.status(), StatusCode::SERVICE_UNAVAILABLE);
            });
    }

    #[test]
    fn http_batch_scores_real_model_fixtures_in_upload_order() {
        let model_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("models");
        let (jobs, mut receive) = mpsc::channel::<Job>(MAX_ACTIVE);
        let worker = std::thread::spawn(move || {
            let mut photos = Photos::new(model_dir.join("tintype.safetensors"), Device::Cpu, false).unwrap();
            let mut clip = Detector::load(
                &model_dir.join("tintype-video.safetensors"),
                None,
                64,
                Device::Cpu,
            )
            .unwrap();
            let frames = clip.use_video_profile();
            while let Some(job) = receive.blocking_recv() {
                let photo = photos.get(job.turbo).unwrap();
                job.execute(photo, None, Some(&clip), frames, Bands::default());
            }
        });
        let photo = image::RgbImage::from_pixel(224, 224, image::Rgb([47, 127, 211]));
        let mut png = Vec::new();
        photo
            .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let mp4 = include_bytes!("../tests/fixtures/video.mp4");
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let app = app(AppState {
                    token: None,
                    active: Arc::new(Semaphore::new(MAX_ACTIVE)),
                    jobs,
                    turbo_default: false,
                    demo: None,
                });
                let batch = app
                    .clone()
                    .oneshot(request(
                        "POST",
                        "/v1/batch",
                        None,
                        &[
                            ("clip.mp4", mp4),
                            ("broken.png", b"broken"),
                            ("photo.png", &png),
                        ],
                    ))
                    .await
                    .unwrap();
                assert_eq!(batch.status(), StatusCode::OK);
                assert_server_timing(&batch);
                let result = json(batch).await;
                let results = result["results"].as_array().unwrap();
                assert_eq!(results.len(), 3);
                assert_eq!(results[0]["name"], "clip.mp4");
                assert_eq!(results[0]["kind"], "video", "{}", results[0]);
                assert_eq!(results[0]["frames_expected"], 16);
                assert!(results[0]["score"].is_number());
                assert!(
                    results[0]["logit"].is_number() && results[0]["threshold_logit"].is_number()
                );
                assert_eq!(results[1]["name"], "broken.png");
                assert_eq!(results[1]["error"]["code"], "unreadable_media");
                assert_eq!(results[2]["name"], "photo.png");
                assert_eq!(results[2]["kind"], "image");
                assert!(results[2]["score"].is_number());
                assert!(
                    results[2]["logit"].is_number() && results[2]["threshold_logit"].is_number()
                );
                // The standard engine unless asked: no `mode` field.
                assert!(results.iter().all(|r| r.get("mode").is_none()), "{result}");

                // Turbo: photos on the eight-bit engine, marked; clips as before.
                let turbo = app
                    .clone()
                    .oneshot(request(
                        "POST",
                        "/v1/batch?mode=turbo",
                        None,
                        &[("clip.mp4", mp4), ("photo.png", &png)],
                    ))
                    .await
                    .unwrap();
                assert_eq!(turbo.status(), StatusCode::OK);
                let turbo = json(turbo).await;
                let turbo = turbo["results"].as_array().unwrap();
                assert_eq!(turbo[0]["kind"], "video");
                assert!(turbo[0].get("mode").is_none(), "{}", turbo[0]);
                assert_eq!(turbo[0]["score"], results[0]["score"]);
                assert_eq!(turbo[1]["mode"], "turbo", "{}", turbo[1]);
                assert_eq!(turbo[1]["threshold_logit"], results[2]["threshold_logit"]);
                let (a, b) = (turbo[1]["logit"].as_f64().unwrap(), results[2]["logit"].as_f64().unwrap());
                assert!((a - b).abs() < 1.0, "turbo logit {a} against {b}");

                let standard = app
                    .clone()
                    .oneshot(request("POST", "/v1/detect?mode=standard", None, &[("photo.png", &png)]))
                    .await
                    .unwrap();
                assert_server_timing(&standard);
                // The photo went through the network (its decode, of a tiny
                // fixture, can round to nothing).
                let timing = server_timing(&standard);
                assert!(timing[4].1 > 0.0, "{timing:?}");
                assert_eq!(json(standard).await, results[2]);
                let unknown = app
                    .clone()
                    .oneshot(request("POST", "/v1/detect?mode=fastest", None, &[("photo.png", &png)]))
                    .await
                    .unwrap();
                assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
                assert_eq!(json(unknown).await["error"]["code"], "invalid_mode");

                let bad = app
                    .oneshot(request(
                        "POST",
                        "/v1/detect",
                        None,
                        &[("broken.png", b"broken")],
                    ))
                    .await
                    .unwrap();
                assert_eq!(bad.status(), StatusCode::UNPROCESSABLE_ENTITY);
                assert_eq!(json(bad).await["error"]["code"], "unreadable_media");
            });
        worker.join().unwrap();
    }

    #[test]
    fn the_mode_comes_from_the_query_or_the_server_default() {
        let uri = |q: &str| q.parse::<Uri>().unwrap();
        assert!(!turbo_requested(&uri("/v1/detect"), false).ok().unwrap());
        assert!(turbo_requested(&uri("/v1/detect"), true).ok().unwrap());
        assert!(turbo_requested(&uri("/v1/detect?mode=turbo"), false).ok().unwrap());
        assert!(!turbo_requested(&uri("/v1/detect?mode=standard"), true).ok().unwrap());
        assert!(turbo_requested(&uri("/v1/batch?x=1&mode=turbo"), false).ok().unwrap());
        for bad in ["/v1/detect?mode=fast", "/v1/detect?mode=", "/v1/detect?mode"] {
            assert!(turbo_requested(&uri(bad), false).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_review_band_adds_fields_and_changes_nothing_else() {
        let model_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("models");
        let photo = Detector::load(&model_dir.join("tintype.safetensors"), None, 6, Device::Cpu).unwrap();
        let mut clip = Detector::load(&model_dir.join("tintype-video.safetensors"), None, 64, Device::Cpu).unwrap();
        let frames = clip.use_video_profile();
        let uploads = || {
            vec![
                Upload { name: "demo.png".into(), bytes: include_bytes!("../site/assets/demo.png").to_vec() },
                Upload { name: "clip.mp4".into(), bytes: include_bytes!("../tests/fixtures/video.mp4").to_vec() },
                Upload { name: "broken.png".into(), bytes: b"broken".to_vec() },
            ]
        };
        let run = |bands: Bands| score_uploads(&photo, None, Some(&clip), frames, bands, uploads(), || false).unwrap();
        let plain = run(Bands::default());
        // Off: no review keys anywhere.
        assert!(plain.iter().all(|r| r.get("review").is_none() && r.get("review_reason").is_none()), "{plain:?}");

        // A band wide enough to hold both results, and one too narrow to hold either.
        let wide = Review { margin: 50.0, view_disagreement: None };
        let narrow = Review { margin: 1e-6, view_disagreement: None };
        for (band, expect) in [(wide, true), (narrow, false)] {
            let with = run(Bands { photo: Some(band), clip: Some(band) });
            for (a, b) in plain.iter().zip(&with) {
                if a.get("error").is_some() {
                    // An unreadable file has no verdict to review.
                    assert_eq!(a, b);
                    continue;
                }
                assert_eq!(b["review"], expect, "{b}");
                assert_eq!(b.get("review_reason").and_then(|r| r.as_str()), expect.then_some("margin"), "{b}");
                // Everything else, the verdict included, is what it was.
                let mut stripped = b.clone();
                let object = stripped.as_object_mut().unwrap();
                object.remove("review");
                object.remove("review_reason");
                assert_eq!(&stripped, a);
            }
        }
        // Disagreement alone: consistent with the logits behind the score.
        let split = Review { margin: 0.0, view_disagreement: Some(1e-6) };
        let with = run(Bands { photo: Some(split), clip: None });
        assert!(with[1].get("review").is_none(), "no band for clips: {}", with[1]);
        let reason = with[0].get("review_reason").and_then(|r| r.as_str());
        assert!(reason.is_none() || reason == Some("view_disagreement"), "{}", with[0]);
        assert_eq!(with[0]["review"], reason.is_some());
    }

    #[test]
    fn abandoned_upload_stops_before_decoding_remaining_files() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let weights =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("models/tintype.safetensors");
        let detector = Detector::load(&weights, None, 6, Device::Cpu).unwrap();
        let files = ["invalid.mp4", "not-needed.mp4", "not-needed.png"]
            .into_iter()
            .map(|name| Upload {
                name: name.into(),
                bytes: b"invalid".to_vec(),
            })
            .collect();
        let checks = AtomicUsize::new(0);
        let output = score_uploads(&detector, None, Some(&detector), 1, Bands::default(), files, || {
            // The request closes after the first video reaches the decoder.
            checks.fetch_add(1, Ordering::Relaxed) >= 2
        });
        assert!(output.is_none());
        assert_eq!(checks.load(Ordering::Relaxed), 3);
    }

/// Just a PNG's signature and header, declaring `w` by `h`: enough to be
    /// sized without allocating the pixels.
    fn png_header(w: u32, h: u32) -> Vec<u8> {
        let chunk = |png: &mut Vec<u8>, kind: &[u8], data: &[u8]| {
            png.extend_from_slice(&(data.len() as u32).to_be_bytes());
            let mut crc = flate2::Crc::new();
            crc.update(kind);
            crc.update(data);
            png.extend_from_slice(kind);
            png.extend_from_slice(data);
            png.extend_from_slice(&crc.sum().to_be_bytes());
        };
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        let ihdr = [&w.to_be_bytes()[..], &h.to_be_bytes(), &[8, 2, 0, 0, 0]].concat();
        chunk(&mut png, b"IHDR", &ihdr);
        chunk(&mut png, b"IDAT", &[]);
        chunk(&mut png, b"IEND", &[]);
        png
    }

    #[test]
    fn phone_photos_over_twelve_megapixels_decode_at_full_size() {
        // 4032 by 3024: a 12 MP phone photo, just over the old 12,000,000 pixel guard.
        let mut jpeg = Vec::new();
        image::RgbImage::from_fn(4032, 3024, |x, y| image::Rgb([(x % 251) as u8, (y % 241) as u8, ((x ^ y) % 239) as u8]))
            .write_to(&mut Cursor::new(&mut jpeg), image::ImageFormat::Jpeg)
            .unwrap();
        let image = decode_image(&jpeg).unwrap();
        assert_eq!((image.w, image.h), (4032, 3024), "decoded at full resolution, not resized");
        let refused = decode_image(&png_header(9000, 6000)).err().expect("54 MP is over the guard");
        assert!(format!("{refused:#}").contains("too large"), "{refused:#}");
    }

    fn png() -> Vec<u8> {
        let photo = image::RgbImage::from_pixel(64, 48, image::Rgb([47, 127, 211]));
        let mut png = Vec::new();
        photo.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
        png
    }

    /// A demo app whose worker answers every job with a fixed row and
    /// reports the names it was given.
    fn demo_app() -> (Router, Arc<Demo>, std::sync::mpsc::Receiver<Vec<String>>) {
        let (jobs, mut receive) = mpsc::channel::<Job>(MAX_ACTIVE);
        let (names, seen) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            while let Some(job) = receive.blocking_recv() {
                let rows = job.files.iter().map(|f| json!({"name": f.name, "kind": "image", "score": 0.25, "verdict": "likely-real"})).collect();
                names.send(job.files.iter().map(|f| f.name.clone()).collect()).unwrap();
                let _ = job.response.send((rows, WorkTimes::default()));
            }
        });
        let demo = Arc::new(Demo::new("https://cheesejaguar.github.io", ClientAddress::LastForwardedFor).unwrap());
        let state = AppState {
            token: None,
            active: Arc::new(Semaphore::new(MAX_ACTIVE)),
            jobs,
            turbo_default: false,
            demo: Some(demo.clone()),
        };
        (app(state), demo, seen)
    }

    fn from(mut request: Request, origin: Option<&str>, forwarded: &str) -> Request {
        if let Some(origin) = origin {
            request.headers_mut().insert(header::ORIGIN, origin.parse().unwrap());
        }
        request.headers_mut().insert("x-forwarded-for", forwarded.parse().unwrap());
        request
    }

    const SITE: Option<&str> = Some("https://cheesejaguar.github.io");

    fn header<'a>(response: &'a Response, name: &str) -> &'a str {
        response.headers().get(name).map_or("", |v| v.to_str().unwrap())
    }

    #[test]
    fn the_demo_answers_its_origin_counts_photos_and_says_where_the_caller_stands() {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let (app, demo, seen) = demo_app();
            let png = png();
            let send = |origin: Option<&'static str>, forwarded: &'static str, files: Vec<(&'static str, Vec<u8>)>| {
                let app = app.clone();
                async move {
                    let files: Vec<(&str, &[u8])> = files.iter().map(|(n, b)| (*n, b.as_slice())).collect();
                    app.oneshot(from(request("POST", "/v1/detect", None, &files), origin, forwarded)).await.unwrap()
                }
            };

            // Preflight: the site's origin, POST, cached for ten minutes.
            let preflight = app
                .clone()
                .oneshot(from(request("OPTIONS", "/v1/detect", None, &[]), SITE, "203.0.113.1"))
                .await
                .unwrap();
            assert_eq!(preflight.status(), StatusCode::NO_CONTENT);
            assert_eq!(header(&preflight, "access-control-allow-origin"), SITE.unwrap());
            assert_eq!(header(&preflight, "access-control-allow-methods"), "POST");
            assert_eq!(header(&preflight, "cache-control"), "no-store");

            // Another site's page is refused before anything is read or counted.
            let foreign = send(Some("https://elsewhere.example"), "203.0.113.1", vec![("a.png", png.clone())]).await;
            assert_eq!(foreign.status(), StatusCode::FORBIDDEN);
            assert_eq!(json(foreign).await["error"]["code"], "origin_not_allowed");

            // Five photos in the window, whatever the caller puts in front of
            // the address the front end appended.
            for i in 0..5u32 {
                let forged = ["203.0.113.1", "9.9.9.9, 203.0.113.1", "1.1.1.1,2.2.2.2, 203.0.113.1"][i as usize % 3];
                let ok = send(SITE, forged, vec![("holiday.mp4", png.clone())]).await;
                assert_eq!(ok.status(), StatusCode::OK);
                assert_eq!(header(&ok, "access-control-allow-origin"), SITE.unwrap());
                let exposed = header(&ok, "access-control-expose-headers");
                for name in ["RateLimit", "RateLimit-Policy", "RateLimit-Remaining", "RateLimit-Reset", "Retry-After", "Server-Timing"] {
                    assert!(exposed.split(", ").any(|e| e == name), "{name} in {exposed}");
                }
                assert_server_timing(&ok);
                assert_eq!(header(&ok, "cache-control"), "no-store");
                assert_eq!(header(&ok, "ratelimit-remaining"), (4 - i).to_string());
                assert!(header(&ok, "ratelimit").starts_with(&format!("\"ip-short\";r={};t=", 4 - i)), "{}", header(&ok, "ratelimit"));
                assert_eq!(header(&ok, "ratelimit-policy"), format!("\"ip-short\";q=5;w=600, \"ip-daily\";q=20;w=86400, \"global-daily\";q={};w=86400", demo::LIMITS.daily_total));
                assert!(ok.headers().get(header::RETRY_AFTER).is_none());
                assert_eq!(json(ok).await["verdict"], "likely-real");
                // The caller's file name never reaches the worker: a video name cannot route a photo to the video path.
                assert_eq!(seen.try_recv().unwrap(), ["photo.png"]);
            }
            let limited = send(SITE, "8.8.8.8, 203.0.113.1", vec![("a.png", png.clone())]).await;
            assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
            let wait: u64 = header(&limited, "retry-after").parse().unwrap();
            assert!((590..=600).contains(&wait), "{wait}");
            assert_eq!(header(&limited, "ratelimit-remaining"), "0");
            assert_eq!(header(&limited, "access-control-allow-origin"), SITE.unwrap());
            let body = json(limited).await;
            assert_eq!(body["error"]["code"], "rate_limited");
            assert_eq!(body["error"]["limit"], "ip_short");
            assert!(body["error"]["message"].as_str().unwrap().starts_with("You have used the 5 photos allowed per 10 minutes"));
            assert!(seen.try_recv().is_err());

            // Another network has its own allowance; the shared cap shows the six.
            let other = send(SITE, "203.0.113.2", vec![("b.jpg", png.clone())]).await;
            assert_eq!(other.status(), StatusCode::OK);
            assert_eq!(header(&other, "ratelimit-remaining"), "4");
            assert!(header(&other, "ratelimit").contains(&format!("\"global-daily\";r={};t=", demo::LIMITS.daily_total - 6)), "{}", header(&other, "ratelimit"));
            seen.try_recv().unwrap();

            // Refused uploads are not counted: too large, not a photo, more than one.
            let before = demo.limiter.status(demo.limiter.key("203.0.113.3".parse().unwrap()), unix_now());
            let big = send(SITE, "203.0.113.3", vec![("big.png", [png.clone(), vec![0; demo::LIMITS.max_bytes]].concat())]).await;
            assert_eq!(big.status(), StatusCode::PAYLOAD_TOO_LARGE);
            let body = json(big).await;
            assert_eq!(body["error"]["code"], "photo_too_large");
            assert_eq!(body["error"]["message"], "the demo accepts one photo of up to 10 MB");
            let mut declared = from(request("POST", "/v1/detect", None, &[("a.png", &png)]), SITE, "203.0.113.3");
            declared.headers_mut().insert(header::CONTENT_LENGTH, (demo.body_limit() + 1).to_string().parse().unwrap());
            assert_eq!(app.clone().oneshot(declared).await.unwrap().status(), StatusCode::PAYLOAD_TOO_LARGE);
            let mp4 = include_bytes!("../tests/fixtures/video.mp4").to_vec();
            for (name, bytes) in [("clip.mp4", mp4.clone()), ("photo.jpg", mp4), ("notes.txt", b"hello".to_vec()), ("photo.gif", b"GIF89a....".to_vec())] {
                let refused = send(SITE, "203.0.113.3", vec![(name, bytes)]).await;
                assert_eq!(refused.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE, "{name}");
                assert_eq!(header(&refused, "ratelimit-remaining"), "5");
                assert_eq!(json(refused).await["error"]["code"], "unsupported_media");
            }
            let refused = send(SITE, "203.0.113.3", vec![("huge.png", png_header(9000, 6000))]).await;
            assert_eq!(refused.status(), StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(header(&refused, "ratelimit-remaining"), "5");
            let body = json(refused).await;
            assert_eq!(body["error"]["code"], "photo_too_many_pixels");
            assert!(body["error"]["message"].as_str().unwrap().starts_with("this photo is 9000 by 6000 pixels; the demo accepts photos of up to 50 megapixels"), "{body}");
            let two = send(SITE, "203.0.113.3", vec![("a.png", png.clone()), ("b.png", png.clone())]).await;
            assert_eq!(two.status(), StatusCode::BAD_REQUEST);
            assert_eq!(json(two).await["error"]["message"], "send one photo per request");
            let after = demo.limiter.status(demo.limiter.key("203.0.113.3".parse().unwrap()), unix_now());
            assert_eq!(before.quotas[0].remaining, after.quotas[0].remaining);
            assert!(seen.try_recv().is_err(), "nothing refused reached the worker");

            // A script without an Origin may call; the API's batch route does not exist here.
            assert_eq!(send(None, "203.0.113.4", vec![("a.webp", png.clone())]).await.status(), StatusCode::OK);
            let batch = app.clone().oneshot(from(request("POST", "/v1/batch", None, &[("a.png", &png)]), SITE, "203.0.113.4")).await.unwrap();
            assert_eq!(batch.status(), StatusCode::NOT_FOUND);
            assert_eq!(header(&batch, "cache-control"), "no-store");
            // `/health` is the same answer (Cloud Run keeps `/healthz` for itself),
            // and asking it counts toward nothing.
            for path in ["/health", "/healthz"] {
                let health = app.clone().oneshot(request("GET", path, None, &[])).await.unwrap();
                let health = json(health).await;
                assert_eq!(health["demo"]["scored"], 7, "{path}");
                assert_eq!(health["demo"]["refused"], 1, "{path}");
            }
        });
    }
}
