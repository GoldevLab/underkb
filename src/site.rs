//! Contact and measurement extras (optional env).

use resuma::prelude::*;

pub fn contact_email() -> Option<String> {
    std::env::var("CONTACT_EMAIL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| s.contains('@') && !s.contains(' '))
}

pub fn chrome_store_url() -> Option<String> {
    std::env::var("CHROME_STORE_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with("https://"))
}

/// Send `underkb.fly.dev` HTML traffic to the canonical domain.
#[middleware]
async fn redirect_legacy_fly_host(req: FlowRequest) -> Result<FlowRequest> {
    const LEGACY: &str = "underkb.fly.dev";
    let host = req
        .header("x-forwarded-host")
        .or_else(|| req.header("host"))
        .unwrap_or("")
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if host != LEGACY {
        return Ok(req);
    }
    let origin = crate::landing::public_origin()
        .trim_end_matches('/')
        .to_string();
    if origin.is_empty() || origin.contains("fly.dev") {
        return Ok(req);
    }
    let path = if req.path.is_empty() {
        "/"
    } else {
        req.path.as_str()
    };
    let mut loc = format!("{origin}{path}");
    if !req.query.is_empty() {
        let mut first = true;
        for (k, v) in &req.query {
            loc.push(if first { '?' } else { '&' });
            first = false;
            loc.push_str(&encode_query(k));
            loc.push('=');
            loc.push_str(&encode_query(v));
        }
    }
    Err(ResumaError::Redirect(loc))
}

fn encode_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn head_extras() -> String {
    let mut out = String::new();
    if let Ok(v) = std::env::var("GSC_VERIFICATION") {
        let v = v.trim();
        if !v.is_empty()
            && v.len() < 120
            && v
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            out.push_str(&format!(
                r#"<meta name="google-site-verification" content="{v}" />"#
            ));
        }
    }
    if let Ok(id) = std::env::var("GA4_ID") {
        let id = id.trim();
        if id.starts_with("G-")
            && id.len() < 20
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
                        out.push_str(&format!(
                r#"<link rel="dns-prefetch" href="https://www.googletagmanager.com" />
<script>
(function(){{
  var id={id:?};
  function load(){{
    if(window.__goldevGaLoaded)return;
    window.__goldevGaLoaded=1;
    var s=document.createElement('script');
    s.async=true;
    s.fetchPriority='low';
    s.src='https://www.googletagmanager.com/gtag/js?id='+id;
    s.onload=function(){{
      window.dataLayer=window.dataLayer||[];
      function gtag(){{dataLayer.push(arguments);}}
      window.gtag=gtag;
      gtag('js',new Date());
      gtag('config',id,{{send_page_view:true}});
    }};
    document.head.appendChild(s);
  }}
  function arm(){{
    var start=function(){{load();}};
    ['pointerdown','keydown','touchstart','scroll'].forEach(function(e){{
      window.addEventListener(e,start,{{once:true,passive:true}});
    }});
    if('requestIdleCallback' in window)requestIdleCallback(start,{{timeout:12000}});
    else window.addEventListener('load',function(){{setTimeout(start,8000);}},{{once:true}});
  }}
  if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',arm,{{once:true}});
  else arm();
}})();
</script>"#
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
                        part.strip_prefix("ukb_pro=")
                            .map(|s| cookie_unescape(s.trim()))
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
