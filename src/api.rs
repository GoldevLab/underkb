//! POST /api/compress — multipart file + target_kb + format.
//! GET /d/{id} — staged download.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Multipart, Path, Query};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use crate::compress::{
    clamp_target, compress, reject_unsupported, stem_filename, OutFormat, MAX_UPLOAD_BYTES,
};
use crate::stage;

static COMPRESS_SLOTS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));
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

pub async fn preflight() -> Response {
    cors(StatusCode::NO_CONTENT, ())
}

pub async fn compress_upload(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let ip = client_ip(&headers, Some(addr));

    let mut file: Option<(Vec<u8>, String)> = None;
    let mut target_kb: u32 = 200;
    let mut format = OutFormat::Jpeg;
    let mut orig_bytes: Option<usize> = None;
    let mut orig_w: Option<u32> = None;
    let mut orig_h: Option<u32> = None;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => {
                return cors(
                    StatusCode::BAD_REQUEST,
                    Json(ErrBody {
                        ok: false,
                        error: format!("Upload failed: {e}"),
                    }),
                );
            }
        };
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" | "upload" => {
                let filename = field.file_name().unwrap_or("image").to_string();
                match field.bytes().await {
                    Ok(b) => file = Some((b.to_vec(), filename)),
                    Err(e) => {
                        return cors(
                            StatusCode::BAD_REQUEST,
                            Json(ErrBody {
                                ok: false,
                                error: format!("Could not read the file: {e}"),
                            }),
                        );
                    }
                }
            }
            "target_kb" => {
                if let Ok(t) = field.text().await {
                    if let Ok(n) = t.trim().parse::<u32>() {
                        target_kb = n;
                    }
                }
            }
            "format" => {
                if let Ok(t) = field.text().await {
                    format = OutFormat::parse(&t);
                }
            }
            "orig_bytes" => {
                if let Ok(t) = field.text().await {
                    orig_bytes = t.trim().parse().ok();
                }
            }
            "orig_width" => {
                if let Ok(t) = field.text().await {
                    orig_w = t.trim().parse().ok();
                }
            }
            "orig_height" => {
                if let Ok(t) = field.text().await {
                    orig_h = t.trim().parse().ok();
                }
            }
            _ => {}
        }
    }

    let Some((bytes, filename)) = file else {
        return cors(
            StatusCode::BAD_REQUEST,
            Json(ErrBody {
                ok: false,
                error: "Attach an image in the file field.".into(),
            }),
        );
    };
    if bytes.is_empty() {
        return cors(
            StatusCode::BAD_REQUEST,
            Json(ErrBody {
                ok: false,
                error: "Empty file.".into(),
            }),
        );
    }
    if bytes.len() > MAX_UPLOAD_BYTES {
        return cors(
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ErrBody {
                ok: false,
                error: "File is over 20 MB.".into(),
            }),
        );
    }
    if let Err(e) = reject_unsupported(&bytes) {
        return cors(
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrBody { ok: false, error: e }),
        );
    }
    if !rate_allow(&ip) {
        return cors(
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrBody {
                ok: false,
                error: "Too many compresses. Wait a minute and try again.".into(),
            }),
        );
    }

    let Ok(permit) = tokio::time::timeout(Duration::from_secs(8), COMPRESS_SLOTS.acquire()).await
    else {
        return cors(
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrBody {
                ok: false,
                error: "The compressor is busy. Try again in a few seconds.".into(),
            }),
        );
    };
    let permit = match permit {
        Ok(p) => p,
        Err(_) => {
            return cors(
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrBody {
                    ok: false,
                    error: "The compressor is busy. Try again in a few seconds.".into(),
                }),
            );
        }
    };

    let target = clamp_target(target_kb);
    let work = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        compress(&bytes, target, format)
    });
    let result = match tokio::time::timeout(Duration::from_secs(25), work).await {
        Err(_) => {
            return cors(
                StatusCode::GATEWAY_TIMEOUT,
                Json(ErrBody {
                    ok: false,
                    error: "That image took too long. Try a smaller file or JPG.".into(),
                }),
            );
        }
        Ok(Err(_)) => {
            return cors(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrBody {
                    ok: false,
                    error: "Compress failed.".into(),
                }),
            );
        }
        Ok(Ok(Err(e))) => {
            return cors(
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrBody { ok: false, error: e }),
            );
        }
        Ok(Ok(Ok(r))) => r,
    };

    let out_name = format!(
        "{}-{}kb.{}",
        stem_filename(&filename),
        (result.bytes.len() / 1024).max(1),
        result.format.ext()
    );
    let original_bytes = orig_bytes
        .filter(|n| *n >= result.bytes.len() && *n <= MAX_UPLOAD_BYTES)
        .unwrap_or(result.original_bytes);
    let original_width = orig_w
        .filter(|w| *w > 0 && *w <= 20_000)
        .unwrap_or(result.original_width);
    let original_height = orig_h
        .filter(|h| *h > 0 && *h <= 20_000)
        .unwrap_or(result.original_height);
    let over_budget = result.over_budget;
    let result_bytes = result.bytes.len();
    let width = result.width;
    let height = result.height;
    let ext = result.format.ext();
    let mime = result.format.mime();
    let id = match stage::put(result.bytes, mime, &out_name) {
        Ok(id) => id,
        Err(_) => {
            return cors(
                StatusCode::INSUFFICIENT_STORAGE,
                Json(ErrBody {
                    ok: false,
                    error: "Could not stage the download.".into(),
                }),
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
            target_kb: (target / 1024) as u32,
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
}
