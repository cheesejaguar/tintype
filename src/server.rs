use crate::detector::{Detector, Device};
use crate::review::{self, Review, ReviewArgs};
use crate::{image_io, video};
use anyhow::{bail, ensure, Context, Result};
use axum::{
    extract::{multipart::MultipartRejection, DefaultBodyLimit, Multipart, Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Args;
use serde_json::{json, Value};
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
    #[command(flatten)]
    review: ReviewArgs,
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

type ApiResult<T> = std::result::Result<T, ApiError>;

struct ApiError(StatusCode, &'static str, &'static str);

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
    ApiError(StatusCode::BAD_REQUEST, "invalid_upload", message)
}

fn unavailable() -> ApiError {
    ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "busy",
        "inference is busy or unavailable",
    )
}

#[derive(Clone)]
struct AppState {
    token: Option<Arc<str>>,
    active: Arc<Semaphore>,
    jobs: mpsc::Sender<Job>,
}

struct Job {
    files: Vec<Upload>,
    response: oneshot::Sender<Vec<Value>>,
    _slot: tokio::sync::OwnedSemaphorePermit,
}

impl Job {
    fn execute(self, photo: &Detector, clip: &Detector, frames: usize, bands: Bands) {
        if let Some(rows) = score_uploads(photo, clip, frames, bands, self.files, || {
            self.response.is_closed()
        }) {
            let _ = self.response.send(rows);
        }
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
    ensure!(
        args.bind.ip().is_loopback() || token.is_some(),
        "set TINTYPE_API_TOKEN when binding outside loopback"
    );
    let device = match args.device.as_str() {
        "auto" => Device::best(),
        "cpu" => Device::Cpu,
        "gpu" => Device::Gpu,
        "gpu-f32" => Device::GpuF32,
        _ => bail!("--device must be auto, cpu, gpu, or gpu-f32"),
    };
    let photo = resolve_weights(args.weights)?;
    let clip = args
        .video_weights
        .unwrap_or_else(|| crate::detector::video_weights_for(&photo));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let (jobs, mut receive) = mpsc::channel::<Job>(MAX_ACTIVE);
    let (ready, startup) = oneshot::channel();
    let review = args.review;
    let worker = std::thread::spawn(move || {
        let setup = (|| -> Result<_> {
            let photo = Detector::load(&photo, None, 32, device)?;
            let mut clip = Detector::load(&clip, None, 64, device)?;
            let frames = clip.use_video_profile();
            ensure!(
                !photo.spec.views.is_empty() && !clip.spec.views.is_empty(),
                "model must declare at least one view"
            );
            ensure!(
                (1..=32).contains(&frames),
                "video model must sample 1 to 32 frames"
            );
            // Flags first, then the band stored with each model.
            let bands = Bands {
                photo: review.resolve(photo.spec.review),
                clip: review.resolve(clip.spec.video.as_ref().and_then(|v| v.review)),
            };
            Ok((photo, clip, frames, bands))
        })();
        let (photo, clip, frames, bands) = match setup {
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
            job.execute(&photo, &clip, frames, bands);
        }
    });
    let result = runtime.block_on(async {
        startup.await.context("starting inference worker")??;
        let state = AppState {
            token: token.map(Arc::from),
            active: Arc::new(Semaphore::new(MAX_ACTIVE)),
            jobs,
        };
        let listener = tokio::net::TcpListener::bind(args.bind).await?;
        eprintln!("Tintype HTTP listening on {}", listener.local_addr()?);
        axum::serve(listener, app(state))
            .with_graceful_shutdown(shutdown())
            .await?;
        Ok(())
    });
    worker
        .join()
        .map_err(|_| anyhow::anyhow!("inference worker panicked"))?;
    result
}

fn app(state: AppState) -> Router {
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
                "supply the configured bearer token",
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

async fn health(State(state): State<AppState>) -> ApiResult<Json<Value>> {
    if state.jobs.is_closed() {
        return Err(unavailable());
    }
    Ok(Json(
        json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}),
    ))
}

async fn detect(
    State(state): State<AppState>,
    multipart: std::result::Result<Multipart, MultipartRejection>,
) -> ApiResult<impl IntoResponse> {
    let mut results = submit(state, multipart, 1).await?;
    let result = results.remove(0);
    let status = if result.get("error").is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    Ok((status, Json(result)))
}

async fn batch(
    State(state): State<AppState>,
    multipart: std::result::Result<Multipart, MultipartRejection>,
) -> ApiResult<Json<Value>> {
    let results = submit(state, multipart, MAX_FILES).await?;
    Ok(Json(json!({"results": results})))
}

async fn submit(
    state: AppState,
    multipart: std::result::Result<Multipart, MultipartRejection>,
    count: usize,
) -> ApiResult<Vec<Value>> {
    let multipart =
        multipart.map_err(|_| bad_request("expected multipart/form-data with a boundary"))?;
    let slot = state
        .active
        .try_acquire_owned()
        .map_err(|_| unavailable())?;
    let files = tokio::time::timeout(Duration::from_secs(20), parse_upload(multipart, count))
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::REQUEST_TIMEOUT,
                "upload_timeout",
                "upload timed out",
            )
        })??;
    let (response, reply) = oneshot::channel();
    state
        .jobs
        .try_send(Job {
            files,
            response,
            _slot: slot,
        })
        .map_err(|_| unavailable())?;
    tokio::time::timeout(Duration::from_secs(90), reply)
        .await
        .map_err(|_| {
            ApiError(
                StatusCode::GATEWAY_TIMEOUT,
                "inference_timeout",
                "inference timed out",
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
        "upload exceeds 64 MiB",
    )
}

async fn parse_upload(mut multipart: Multipart, max: usize) -> ApiResult<Vec<Upload>> {
    let mut files = Vec::new();
    while let Some(mut field) = multipart.next_field().await.map_err(multipart_error)? {
        if field.name() != Some("file") {
            return Err(bad_request("expected multipart fields named file"));
        }
        if files.len() >= max {
            return Err(bad_request("too many file fields"));
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

fn score_uploads(
    photo: &Detector,
    clip: &Detector,
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
            if let Ok(value) = score_video(clip, frames, bands.clip, &file, &ext) {
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
                let pre = photo.preprocessor();
                photo.score_staged(images.len(), 2, exit, photo.spec.views.len(), |i| {
                    if cancelled() {
                        return None;
                    }
                    decode_image(&images[i].1.bytes)
                        .ok()
                        .filter(|_| !cancelled())
                        .map(|image| pre.crops(&image))
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
        for ((i, file), scores) in images.into_iter().zip(scores) {
            if let Some(scores) = scores {
                let score = photo.probability(&scores);
                if score.is_finite() {
                    results[i] = json!({
                        "name": file.name, "kind": "image", "profile": "default", "score": score,
                        "verdict": verdict(score), "views": scores.len(),
                        "logit": video::aggregate(&scores), "threshold_logit": photo.threshold_logit()
                    });
                    if let Some(band) = bands.photo {
                        // From the view logits already scored: no further inference.
                        let reason = band.reason(
                            photo.calibrated_logit(&scores),
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
        limits.max_image_width = Some(8192);
        limits.max_image_height = Some(8192);
        limits.max_alloc = Some(128 * 1024 * 1024);
        reader.limits(limits);
        Ok::<_, std::io::Error>(reader)
    };
    let (w, h) = reader()?.into_dimensions()?;
    ensure!(
        w > 0 && h > 0 && u64::from(w) * u64::from(h) <= 12_000_000,
        "image too large"
    );
    ensure!(w.max(h) <= w.min(h) * 32, "image aspect ratio too large");
    let image = reader()?.decode()?.into_rgb8();
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
            let photo = Detector::load(
                &model_dir.join("tintype.safetensors"),
                None,
                32,
                Device::Cpu,
            )
            .unwrap();
            let mut clip = Detector::load(
                &model_dir.join("tintype-video.safetensors"),
                None,
                64,
                Device::Cpu,
            )
            .unwrap();
            let frames = clip.use_video_profile();
            while let Some(job) = receive.blocking_recv() {
                job.execute(&photo, &clip, frames, Bands::default());
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
        let run = |bands: Bands| score_uploads(&photo, &clip, frames, bands, uploads(), || false).unwrap();
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
        let output = score_uploads(&detector, &detector, 1, Bands::default(), files, || {
            // The request closes after the first video reaches the decoder.
            checks.fetch_add(1, Ordering::Relaxed) >= 2
        });
        assert!(output.is_none());
        assert_eq!(checks.load(Ordering::Relaxed), 3);
    }
}
