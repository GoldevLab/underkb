//! Contact and measurement extras (optional env).

pub fn contact_email() -> Option<String> {
    std::env::var("CONTACT_EMAIL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| s.contains('@') && !s.contains(' '))
}

pub fn head_extras() -> String {
    let mut out = String::new();
    if let Ok(v) = std::env::var("GSC_VERIFICATION") {
        let v = v.trim();
        if !v.is_empty() && v.len() < 120 && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            out.push_str(&format!(
                r#"<meta name="google-site-verification" content="{v}" />"#
            ));
        }
    }
    if let Ok(id) = std::env::var("GA4_ID") {
        let id = id.trim();
        if id.starts_with("G-") && id.len() < 20 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            out.push_str(&format!(
                r#"<script async src="https://www.googletagmanager.com/gtag/js?id={id}"></script>
<script>window.dataLayer=window.dataLayer||[];function gtag(){{dataLayer.push(arguments);}}gtag('js',new Date());gtag('config','{id}');</script>"#
            ));
        }
    }
    if let Ok(domain) = std::env::var("PLAUSIBLE_DOMAIN") {
        let domain = domain.trim();
        if !domain.is_empty() && domain.len() < 80 && !domain.contains('<') {
            out.push_str(&format!(
                r#"<script defer data-domain="{domain}" src="https://plausible.io/js/script.js"></script>"#
            ));
        }
    }
    out
}

fn cookie_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(v) = u8::from_str_radix(hex, 16) {
                out.push(v as char);
                i += 3;
                continue;
            }
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

pub fn configured_pro_keys() -> Vec<String> {
    std::env::var("UNDERKB_PRO_KEYS")
        .ok()
        .or_else(|| std::env::var("API_KEY").ok())
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() >= 16)
        .collect()
}

pub fn is_pro(headers: &axum::http::HeaderMap) -> bool {
    let provided = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers.get("authorization").and_then(|v| v.to_str().ok()).and_then(|v| {
                v.strip_prefix("Bearer ")
                    .or_else(|| v.strip_prefix("bearer "))
                    .map(|s| s.trim().to_string())
            })
        })
        .or_else(|| {
            headers
                .get(axum::http::header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|c| {
                    c.split(';').find_map(|part| {
                        let part = part.trim();
                        part.strip_prefix("ukb_pro=").map(|s| cookie_unescape(s.trim()))
                    })
                })
        });
    let Some(provided) = provided.filter(|s| !s.is_empty()) else {
        return false;
    };
    configured_pro_keys().iter().any(|k| k == &provided)
}

pub const FREE_MAX_BYTES: usize = 20 * 1024 * 1024;
pub const PRO_MAX_BYTES: usize = 50 * 1024 * 1024;
pub const FREE_BATCH: usize = 1;
pub const PRO_BATCH: usize = 20;
