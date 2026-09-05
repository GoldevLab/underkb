//! POST /api/compress — multipart file + target_kb + format.
//! GET /d/{id} — staged download.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Multipart, Path, Query};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use crate::compress::{
    clamp_target, compress, reject_unsupported, stem_filename, with_cpu_deadline, OutFormat,
    CPU_TIMEOUT_MSG, MAX_UPLOAD_BYTES,
};
use crate::ops::{self, FitMode, ImageOut};
use crate::stage;

static COMPRESS_SLOTS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));
static IN_FLIGHT: LazyLock<Arc<AtomicUsize>> =
    LazyLock::new(|| Arc::new(AtomicUsize::new(0)));
/// 2 accepted jobs + 1 leftover after a 504. More than this OOMs the 512 MB VM.
const MAX_IN_FLIGHT: usize = 3;
const SLOT_WAIT: Duration = Duration::from_secs(8);
/// HTTP backstop. The blocking job also gets `CPU_BUDGET` so it usually
/// stops itself and frees the slot on the normal return path.
const CPU_WATCHDOG: Duration = Duration::from_secs(25);
const CPU_BUDGET: Duration = Duration::from_secs(22);
const RATE_MAX: usize = 12;
const RATE_WINDOW: Duration = Duration::from_secs(60);

#[derive(Serialize)]
struct OkBody {
    ok: bool,
    original_bytes: usize,
    result_bytes: usize,
    width: u32,
    height: u32,
    original_width: u32,
    original_height: u32,
    target_kb: u32,
    format: String,
    mime: String,
    filename: String,
    url: String,
    over_budget: bool,
}

#[derive(Serialize)]
struct ErrBody {
    ok: bool,
    error: String,
}

#[derive(Serialize)]
struct ColorsBody {
    ok: bool,
    width: u32,
    height: u32,
    colors: Vec<ops::Swatch>,
}

struct Parts {
    file: Option<(Vec<u8>, String)>,
    files: Vec<(Vec<u8>, String)>,
    fields: HashMap<String, String>,
}

async fn take_parts(multipart: &mut Multipart) -> Result<Parts, Response> {
    let mut parts = Parts {
        file: None,
        files: Vec::new(),
        fields: HashMap::new(),
    };
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => {
                return Err(fail(
                    StatusCode::BAD_REQUEST,
                    format!("Upload failed: {e}"),
                ));
            }
        };
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" | "upload" | "files" => {
                let filename = field.file_name().unwrap_or("image").to_string();
                match field.bytes().await {
                    Ok(b) => {
                        let total: usize = parts.files.iter().map(|(d, _)| d.len()).sum();
                        if total.saturating_add(b.len()) > crate::site::PRO_MAX_BYTES {
                            return Err(fail(
                                StatusCode::PAYLOAD_TOO_LARGE,
                                "Upload is over 50 MB total.".into(),
                            ));
                        }
                        let item = (b.to_vec(), filename);
                        parts.files.push(item.clone());
                        parts.file = Some(item);
                    }
                    Err(e) => {
                        return Err(fail(
                            StatusCode::BAD_REQUEST,
                            format!("Could not read the file: {e}"),
                        ));
                    }
                }
            }
            _ => {
                if let Ok(t) = field.text().await {
                    parts.fields.insert(name, t);
                }
            }
        }
    }
    Ok(parts)
}

fn field_u32(parts: &Parts, key: &str) -> Option<u32> {
    parts.fields.get(key).and_then(|t| t.trim().parse().ok())
}

fn field_str<'a>(parts: &'a Parts, key: &str) -> &'a str {
    parts.fields.get(key).map(|s| s.as_str()).unwrap_or("")
}

fn fail(status: StatusCode, error: String) -> Response {
    cors(status, Json(ErrBody { ok: false, error }))
}

pub async fn preflight() -> Response {
    cors(StatusCode::NO_CONTENT, ())
}

pub async fn compress_upload(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let ip = client_ip(&headers, Some(addr));
    let parts = match take_parts(&mut multipart).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let target_kb = field_u32(&parts, "target_kb").unwrap_or(200);
    let format = OutFormat::parse(field_str(&parts, "format"));
    let orig_bytes = field_u32(&parts, "orig_bytes").map(|n| n as usize);
    let orig_w = field_u32(&parts, "orig_width");
    let orig_h = field_u32(&parts, "orig_height");
    let pro = crate::site::is_pro(&headers);
    let max_bytes = if pro {
        crate::site::PRO_MAX_BYTES
    } else {
        crate::site::FREE_MAX_BYTES
    };
    let (bytes, filename) = match gate_file(parts.file, &ip, max_bytes) {
        Ok(v) => v,
        Err(r) => return r,
    };

    let target = clamp_target(target_kb);
    let result = match run_cpu(move || compress(&bytes, target, format)).await {
        Ok(r) => r,
        Err(r) => return r,
    };

    let out_name = format!(
        "{}-{}kb.{}",
        stem_filename(&filename),
        (result.bytes.len() / 1024).max(1),
        result.format.ext()
    );
    staged_ok(
        result.bytes,
        result.format,
        out_name,
        orig_bytes,
        orig_w,
        orig_h,
        result.original_bytes,
        result.original_width,
        result.original_height,
        result.width,
        result.height,
        (target / 1024) as u32,
        result.over_budget,
    )
}

pub async fn compress_batch(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let ip = client_ip(&headers, Some(addr));
    let parts = match take_parts(&mut multipart).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let pro = crate::site::is_pro(&headers);
    let max_files = if pro {
        crate::site::PRO_BATCH
    } else {
        crate::site::FREE_BATCH
    };
    let max_bytes = if pro {
        crate::site::PRO_MAX_BYTES
    } else {
        crate::site::FREE_MAX_BYTES
    };
    let target_kb = field_u32(&parts, "target_kb").unwrap_or(200);
    let format = OutFormat::parse(field_str(&parts, "format"));
    let target = clamp_target(target_kb);
    let files = if parts.files.is_empty() {
        parts.file.into_iter().collect::<Vec<_>>()
    } else {
        parts.files
    };
    if files.is_empty() {
        return fail(
            StatusCode::BAD_REQUEST,
            "Attach at least one image in the file field.".into(),
        );
    }
    if files.len() > max_files {
        return fail(
            StatusCode::PAYMENT_REQUIRED,
            if pro {
                "Max 20 files per ZIP.".into()
            } else {
                "Free is one file. A Pro key on /pricing allows a ZIP of up to 20.".into()
            },
        );
    }
    if files.len() == 1 {
        let (bytes, filename) = match gate_file(Some(files.into_iter().next().unwrap()), &ip, max_bytes)
        {
            Ok(v) => v,
            Err(r) => return r,
        };
        let result = match run_cpu(move || compress(&bytes, target, format)).await {
            Ok(r) => r,
            Err(r) => return r,
        };
        let out_name = format!(
            "{}-{}kb.{}",
            stem_filename(&filename),
            (result.bytes.len() / 1024).max(1),
            result.format.ext()
        );
        return staged_ok(
            result.bytes,
            result.format,
            out_name,
            None,
            None,
            None,
            result.original_bytes,
            result.original_width,
            result.original_height,
            result.width,
            result.height,
            (target / 1024) as u32,
            result.over_budget,
        );
    }

    let mut gated = Vec::with_capacity(files.len());
    for item in files {
        match gate_file(Some(item), &ip, max_bytes) {
            Ok(v) => gated.push(v),
            Err(r) => return r,
        }
    }
    let n = gated.len();
    let packed = match run_cpu_batch(move || zip_compressed(gated, target, format)).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let id = match stage::put(packed.bytes, "application/zip", "underkb-batch.zip") {
        Ok(id) => id,
        Err(_) => {
            return fail(
                StatusCode::INSUFFICIENT_STORAGE,
                "Could not stage the download.".into(),
            );
        }
    };
    cors(
        StatusCode::OK,
        Json(OkBody {
            ok: true,
            original_bytes: packed.original_bytes,
            result_bytes: packed.result_bytes,
            width: 0,
            height: 0,
            original_width: 0,
            original_height: 0,
            target_kb: (target / 1024) as u32,
            format: "zip".into(),
            mime: "application/zip".into(),
            filename: format!("underkb-{n}-files.zip"),
            url: format!("/d/{id}"),
            over_budget: packed.over_budget,
        }),
    )
}

struct ZipPack {
    bytes: Vec<u8>,
    original_bytes: usize,
    result_bytes: usize,
    over_budget: bool,
}

fn zip_compressed(
    files: Vec<(Vec<u8>, String)>,
    target: usize,
    format: OutFormat,
) -> Result<ZipPack, String> {
    use std::collections::HashSet;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let mut original_bytes = 0usize;
    let mut over_budget = false;
    let mut used = HashSet::new();
    let mut entries = Vec::with_capacity(files.len());
    for (bytes, filename) in files {
        original_bytes += bytes.len();
        let result = compress(&bytes, target, format)?;
        drop(bytes);
        over_budget |= result.over_budget;
        let mut name = format!(
            "{}-{}kb.{}",
            stem_filename(&filename),
            (result.bytes.len() / 1024).max(1),
            result.format.ext()
        );
        if !used.insert(name.clone()) {
            for n in 2..10_000 {
                let cand = format!(
                    "{}-{n}-{}kb.{}",
                    stem_filename(&filename),
                    (result.bytes.len() / 1024).max(1),
                    result.format.ext()
                );
                if used.insert(cand.clone()) {
                    name = cand;
                    break;
                }
            }
        }
        entries.push((name, result.bytes));
    }

    let mut zip = ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, data) in entries {
        zip.start_file(name, opts).map_err(|e| e.to_string())?;
        zip.write_all(&data).map_err(|e| e.to_string())?;
    }
    let cursor = zip.finish().map_err(|e| e.to_string())?;
    let bytes = cursor.into_inner();
    let result_bytes = bytes.len();
    Ok(ZipPack {
        bytes,
        original_bytes,
        result_bytes,
        over_budget,
    })
}

const BATCH_WATCHDOG: Duration = Duration::from_secs(55);
const BATCH_BUDGET: Duration = Duration::from_secs(50);

async fn run_cpu_batch<T, E>(job: impl FnOnce() -> Result<T, E> + Send + 'static) -> Result<T, Response>
where
    T: Send + 'static,
    E: ToString + Send + 'static,
{
    run_cpu_on(
        &COMPRESS_SLOTS,
        job,
        SLOT_WAIT,
        BATCH_WATCHDOG,
        BATCH_BUDGET,
        Arc::clone(&IN_FLIGHT),
        MAX_IN_FLIGHT,
    )
    .await
}

pub async fn convert_upload(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let ip = client_ip(&headers, Some(addr));
    let parts = match take_parts(&mut multipart).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let format = OutFormat::parse_prefer_webp(field_str(&parts, "format"));
    let quality = ops::clamp_quality(field_u32(&parts, "quality").unwrap_or(80));
    let orig = orig_meta(&parts);
    let pro = crate::site::is_pro(&headers);
    let max_bytes = if pro {
        crate::site::PRO_MAX_BYTES
    } else {
        crate::site::FREE_MAX_BYTES
    };
    let (bytes, filename) = match gate_file(parts.file, &ip, max_bytes) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let result = match run_cpu(move || ops::convert(&bytes, format, quality)).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    image_ok(result, &filename, orig)
}

pub async fn resize_upload(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let ip = client_ip(&headers, Some(addr));
    let parts = match take_parts(&mut multipart).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let format = OutFormat::parse(field_str(&parts, "format"));
    let quality = ops::clamp_quality(field_u32(&parts, "quality").unwrap_or(82));
    let mode = FitMode::parse(field_str(&parts, "mode"));
    let width = field_u32(&parts, "width").filter(|w| *w > 0);
    let height = field_u32(&parts, "height").filter(|h| *h > 0);
    let orig = orig_meta(&parts);
    let pro = crate::site::is_pro(&headers);
    let max_bytes = if pro {
        crate::site::PRO_MAX_BYTES
    } else {
        crate::site::FREE_MAX_BYTES
    };
    let (bytes, filename) = match gate_file(parts.file, &ip, max_bytes) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let result = match run_cpu(move || ops::resize(&bytes, width, height, mode, format, quality))
        .await
    {
        Ok(r) => r,
        Err(r) => return r,
    };
    image_ok(result, &filename, orig)
}

pub async fn remove_bg_upload(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let ip = client_ip(&headers, Some(addr));
    let parts = match take_parts(&mut multipart).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let format = OutFormat::parse_prefer_png(field_str(&parts, "format"));
    let tolerance = field_u32(&parts, "tolerance").unwrap_or(32).clamp(8, 90) as u8;
    let orig = orig_meta(&parts);
    let pro = crate::site::is_pro(&headers);
    let max_bytes = if pro {
        crate::site::PRO_MAX_BYTES
    } else {
        crate::site::FREE_MAX_BYTES
    };
    let (bytes, filename) = match gate_file(parts.file, &ip, max_bytes) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let result = match run_cpu(move || ops::remove_background(&bytes, tolerance, format)).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    image_ok(result, &filename, orig)
}

pub async fn colors_upload(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let ip = client_ip(&headers, Some(addr));
    let parts = match take_parts(&mut multipart).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let count = field_u32(&parts, "count").unwrap_or(6) as usize;
    let pro = crate::site::is_pro(&headers);
    let max_bytes = if pro {
        crate::site::PRO_MAX_BYTES
    } else {
        crate::site::FREE_MAX_BYTES
    };
    let (bytes, _) = match gate_file(parts.file, &ip, max_bytes) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let (colors, width, height) = match run_cpu(move || ops::extract_colors(&bytes, count)).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    cors(
        StatusCode::OK,
        Json(ColorsBody {
            ok: true,
            width,
            height,
            colors,
        }),
    )
}

struct OrigMeta {
    bytes: Option<usize>,
    w: Option<u32>,
    h: Option<u32>,
}

fn orig_meta(parts: &Parts) -> OrigMeta {
    OrigMeta {
        bytes: field_u32(parts, "orig_bytes").map(|n| n as usize),
        w: field_u32(parts, "orig_width"),
        h: field_u32(parts, "orig_height"),
    }
}

fn image_ok(result: ImageOut, filename: &str, orig: OrigMeta) -> Response {
    let out_name = format!("{}.{}", stem_filename(filename), result.format.ext());
    staged_ok(
        result.bytes,
        result.format,
        out_name,
        orig.bytes,
        orig.w,
        orig.h,
        result.original_bytes,
        result.original_width,
        result.original_height,
        result.width,
        result.height,
        0,
        false,
    )
}

fn gate_file(
    file: Option<(Vec<u8>, String)>,
    ip: &str,
    max_bytes: usize,
) -> Result<(Vec<u8>, String), Response> {
    let Some((bytes, filename)) = file else {
        return Err(fail(
            StatusCode::BAD_REQUEST,
            "Attach an image in the file field.".into(),
        ));
    };
    if bytes.is_empty() {
        return Err(fail(StatusCode::BAD_REQUEST, "Empty file.".into()));
    }
    if bytes.len() > max_bytes {
        let mb = max_bytes / (1024 * 1024);
        return Err(fail(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("File is over {mb} MB. A Pro key on /pricing allows 50 MB."),
        ));
    }
    if let Err(e) = reject_unsupported(&bytes) {
        return Err(fail(StatusCode::UNPROCESSABLE_ENTITY, e));
    }
    if !rate_allow(ip) {
        return Err(fail(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Wait a minute and try again.".into(),
        ));
    }
    Ok((bytes, filename))
}

fn busy() -> Response {
    fail(
        StatusCode::SERVICE_UNAVAILABLE,
        "The server is busy. Try again in a few seconds.".into(),
    )
}

fn too_slow() -> Response {
    fail(StatusCode::GATEWAY_TIMEOUT, CPU_TIMEOUT_MSG.into())
}

async fn run_cpu<T, E>(job: impl FnOnce() -> Result<T, E> + Send + 'static) -> Result<T, Response>
where
    T: Send + 'static,
    E: ToString + Send + 'static,
{
    run_cpu_on(
        &COMPRESS_SLOTS,
        job,
        SLOT_WAIT,
        CPU_WATCHDOG,
        CPU_BUDGET,
        Arc::clone(&IN_FLIGHT),
        MAX_IN_FLIGHT,
    )
    .await
}

fn try_begin_inflight(inflight: &AtomicUsize, max_inflight: usize) -> bool {
    loop {
        let n = inflight.load(Ordering::SeqCst);
        if n >= max_inflight {
            return false;
        }
        if inflight
            .compare_exchange(n, n + 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return true;
        }
    }
}

async fn run_cpu_on<T, E>(
    slots: &Semaphore,
    job: impl FnOnce() -> Result<T, E> + Send + 'static,
    slot_wait: Duration,
    watchdog: Duration,
    budget: Duration,
    inflight: Arc<AtomicUsize>,
    max_inflight: usize,
) -> Result<T, Response>
where
    T: Send + 'static,
    E: ToString + Send + 'static,
{
    let permit = match tokio::time::timeout(slot_wait, slots.acquire()).await {
        Ok(Ok(p)) => p,
        Ok(Err(_)) | Err(_) => return Err(busy()),
    };
    if !try_begin_inflight(inflight.as_ref(), max_inflight) {
        drop(permit);
        return Err(busy());
    }

    // Keep the permit on this future, not inside spawn_blocking. Dropping the
    // JoinHandle cannot abort a blocking codec; holding the slot there made
    // "busy" last until the thread finished — longer than the 504 claimed.
    let mut work = tokio::task::spawn_blocking(move || {
        with_cpu_deadline(Instant::now() + budget, job)
    });

    match tokio::time::timeout(watchdog, &mut work).await {
        Ok(Ok(Ok(r))) => {
            inflight.fetch_sub(1, Ordering::SeqCst);
            drop(permit);
            Ok(r)
        }
        Ok(Ok(Err(e))) => {
            inflight.fetch_sub(1, Ordering::SeqCst);
            drop(permit);
            let msg = e.to_string();
            if msg == CPU_TIMEOUT_MSG {
                Err(too_slow())
            } else {
                Err(fail(StatusCode::UNPROCESSABLE_ENTITY, msg))
            }
        }
        Ok(Err(_)) => {
            inflight.fetch_sub(1, Ordering::SeqCst);
            drop(permit);
            Err(fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Processing failed.".into(),
            ))
        }
        Err(_) => {
            drop(permit);
            tokio::spawn(async move {
                let _ = work.await;
                inflight.fetch_sub(1, Ordering::SeqCst);
            });
            Err(too_slow())
        }
    }
}

fn staged_ok(
    bytes: Vec<u8>,
    format: OutFormat,
    out_name: String,
    orig_bytes: Option<usize>,
    orig_w: Option<u32>,
    orig_h: Option<u32>,
    fallback_bytes: usize,
    fallback_w: u32,
    fallback_h: u32,
    width: u32,
    height: u32,
    target_kb: u32,
    over_budget: bool,
) -> Response {
    let result_bytes = bytes.len();
    let original_bytes = orig_bytes
        .filter(|n| *n > 0 && *n <= MAX_UPLOAD_BYTES)
        .unwrap_or(fallback_bytes);
    let original_width = orig_w
        .filter(|w| *w > 0 && *w <= 20_000)
        .unwrap_or(fallback_w);
    let original_height = orig_h
        .filter(|h| *h > 0 && *h <= 20_000)
        .unwrap_or(fallback_h);
    let ext = format.ext();
    let mime = format.mime();
    let id = match stage::put(bytes, mime, &out_name) {
        Ok(id) => id,
        Err(_) => {
            return fail(
                StatusCode::INSUFFICIENT_STORAGE,
                "Could not stage the download.".into(),
            );
        }
    };
    cors(
        StatusCode::OK,
        Json(OkBody {
            ok: true,
            original_bytes,
            result_bytes,
            width,
            height,
            original_width,
            original_height,
            target_kb,
            format: ext.into(),
            mime: mime.into(),
            filename: out_name,
            url: format!("/d/{id}"),
            over_budget,
        }),
    )
}

#[derive(Deserialize)]
pub struct DownloadQuery {
    pub dl: Option<String>,
}

pub async fn download(Path(id): Path<String>, Query(q): Query<DownloadQuery>) -> Response {
    if !stage::is_id(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(file) = stage::get(&id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mut headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&file.mime) {
        headers.insert(header::CONTENT_TYPE, v);
    }
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=300"),
    );
    let kind = if q.dl.as_deref() == Some("1") {
        "attachment"
    } else {
        "inline"
    };
    if let Some(disp) = content_disposition(kind, &file.filename) {
        headers.insert(header::CONTENT_DISPOSITION, disp);
    }
    (StatusCode::OK, headers, file.bytes.to_vec()).into_response()
}

fn content_disposition(kind: &str, filename: &str) -> Option<HeaderValue> {
    let safe: String = filename
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect();
    if safe.is_empty() {
        return HeaderValue::from_str(kind).ok();
    }
    HeaderValue::from_str(&format!("{kind}; filename=\"{safe}\"")).ok()
}

fn client_ip(headers: &HeaderMap, connect: Option<SocketAddr>) -> String {
    // Fly overwrites this hop; clients cannot spoof it. X-Forwarded-For is appended.
    if let Some(fly) = headers
        .get("fly-client-ip")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|ip| !ip.is_empty() && ip.parse::<std::net::IpAddr>().is_ok())
    {
        return fly.to_string();
    }
    resuma::server::client_ip_from_parts(headers, connect)
}

fn rate_allow(ip: &str) -> bool {
    fn buckets() -> &'static Mutex<HashMap<String, Vec<Instant>>> {
        static MAP: OnceLock<Mutex<HashMap<String, Vec<Instant>>>> = OnceLock::new();
        MAP.get_or_init(|| Mutex::new(HashMap::new()))
    }
    let Ok(mut map) = buckets().lock() else {
        return true;
    };
    let now = Instant::now();
    if map.len() > 4096 {
        map.retain(|_, hits| hits.iter().any(|t| now.duration_since(*t) < RATE_WINDOW));
    }
    let hits = map.entry(ip.to_string()).or_default();
    hits.retain(|t| now.duration_since(*t) < RATE_WINDOW);
    if hits.len() >= RATE_MAX {
        return false;
    }
    hits.push(now);
    true
}

fn cors<T: IntoResponse>(status: StatusCode, body: T) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("content-type"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    (status, headers, body).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn prefers_fly_client_ip_over_spoofed_xff() {
        let mut headers = HeaderMap::new();
        headers.insert("fly-client-ip", HeaderValue::from_static("203.0.113.9"));
        headers.insert("x-forwarded-for", HeaderValue::from_static("1.2.3.4"));
        let peer = "127.0.0.1:1".parse().unwrap();
        assert_eq!(client_ip(&headers, Some(peer)), "203.0.113.9");
    }

    #[test]
    fn ignores_garbage_fly_client_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("fly-client-ip", HeaderValue::from_static("not-an-ip"));
        let peer: SocketAddr = "10.0.0.9:1".parse().unwrap();
        assert_eq!(client_ip(&headers, Some(peer)), "10.0.0.9");
    }

    #[tokio::test]
    async fn timeout_frees_slot_while_blocking_work_continues() {
        let slots = Semaphore::new(1);
        let started = Arc::new(AtomicBool::new(false));
        let flag = started.clone();
        let inflight = Arc::new(AtomicUsize::new(0));
        let err = run_cpu_on(
            &slots,
            move || {
                flag.store(true, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(200));
                Ok::<(), String>(())
            },
            Duration::from_millis(50),
            Duration::from_millis(25),
            Duration::from_secs(5),
            Arc::clone(&inflight),
            8,
        )
        .await
        .unwrap_err();
        assert_eq!(err.status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(slots.available_permits(), 1);
        assert!(started.load(Ordering::SeqCst));

        let n = run_cpu_on(
            &slots,
            || Ok::<_, String>(7u8),
            Duration::from_millis(40),
            Duration::from_millis(200),
            Duration::from_secs(5),
            Arc::clone(&inflight),
            8,
        )
        .await
        .expect("slot should be free after the 504");
        assert_eq!(n, 7);
    }

    #[tokio::test]
    async fn slot_wait_times_out_when_full() {
        let slots = Semaphore::new(1);
        let _held = slots.acquire().await.unwrap();
        let err = run_cpu_on(
            &slots,
            || Ok::<_, String>(()),
            Duration::from_millis(20),
            Duration::from_millis(50),
            Duration::from_secs(1),
            Arc::new(AtomicUsize::new(0)),
            8,
        )
        .await
        .unwrap_err();
        assert_eq!(err.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn inflight_cap_rejects_before_a_fourth_blocking_job() {
        let slots = Arc::new(Semaphore::new(4));
        let inflight = Arc::new(AtomicUsize::new(0));
        let slots_a = Arc::clone(&slots);
        let inflight_a = Arc::clone(&inflight);
        let first = tokio::spawn(async move {
            run_cpu_on(
                slots_a.as_ref(),
                || {
                    std::thread::sleep(Duration::from_millis(80));
                    Ok::<(), String>(())
                },
                Duration::from_millis(50),
                Duration::from_millis(400),
                Duration::from_secs(5),
                inflight_a,
                1,
            )
            .await
        });
        let give_up = Instant::now() + Duration::from_millis(200);
        while inflight.load(Ordering::SeqCst) == 0 {
            if Instant::now() > give_up {
                panic!("first job never took an inflight slot");
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        let err = run_cpu_on(
            slots.as_ref(),
            || Ok::<_, String>(()),
            Duration::from_millis(50),
            Duration::from_millis(200),
            Duration::from_secs(5),
            Arc::clone(&inflight),
            1,
        )
        .await
        .unwrap_err();
        assert_eq!(err.status(), StatusCode::SERVICE_UNAVAILABLE);
        first.await.unwrap().expect("first job should finish");
        assert_eq!(inflight.load(Ordering::SeqCst), 0);
    }
}
