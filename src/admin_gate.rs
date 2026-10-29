//! Private `/admin` session. The password stays in this binary.
//! The cookie is an HMAC of an expiry, not the password.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

pub const ADMIN_COOKIE: &str = "ukb_admin";
pub const ADMIN_SESSION_SECS: u64 = 12 * 60 * 60;
const LOGIN_MAX: usize = 8;
const LOGIN_WINDOW_SECS: u64 = 600;

const ADMIN_USER: &str = "admin";
const ADMIN_PASSWORD: &str = "10203042";

struct Hits {
    start: u64,
    count: usize,
}

static LOGIN_HITS: Mutex<HashMap<String, Hits>> = Mutex::new(HashMap::new());
static SECRET: std::sync::LazyLock<[u8; 32]> = std::sync::LazyLock::new(load_or_create_secret);

pub fn allow_login(ip: &str) -> bool {
    let now = now_secs();
    let mut map = LOGIN_HITS.lock().unwrap_or_else(|err| err.into_inner());
    if map.len() > 4_000 {
        map.retain(|_, hit| now.saturating_sub(hit.start) <= LOGIN_WINDOW_SECS);
    }
    let hit = map
        .entry(ip.to_string())
        .or_insert(Hits { start: now, count: 0 });
    if now.saturating_sub(hit.start) > LOGIN_WINDOW_SECS {
        hit.start = now;
        hit.count = 0;
    }
    if hit.count >= LOGIN_MAX {
        return false;
    }
    hit.count += 1;
    true
}

pub fn client_ip(req: &resuma::prelude::FlowRequest) -> String {
    req.header("fly-client-ip")
        .or_else(|| req.header("x-real-ip"))
        .or_else(|| {
            req.header("x-forwarded-for")
                .and_then(|v| v.split(',').next())
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

/// Server-side check. The browser never receives this password.
pub fn admin_login_ok(user: &str, password: &str) -> bool {
    ct_eq(user.trim(), ADMIN_USER) && ct_eq(password, ADMIN_PASSWORD)
}

pub fn issue_admin_session() -> String {
    let exp = now_secs().saturating_add(ADMIN_SESSION_SECS);
    let mac = hmac_hex(format!("ukb-admin|{exp}").as_bytes());
    format!("{exp}.{mac}")
}

pub fn admin_session_valid(raw: &str) -> bool {
    let Some((exp_s, mac)) = raw.split_once('.') else {
        return false;
    };
    if exp_s.is_empty() || mac.len() != 64 {
        return false;
    }
    let Ok(exp) = exp_s.parse::<u64>() else {
        return false;
    };
    let now = now_secs();
    if exp <= now || exp > now.saturating_add(ADMIN_SESSION_SECS).saturating_add(120) {
        return false;
    }
    hmac_ok(format!("ukb-admin|{exp}").as_bytes(), mac)
}

pub fn cookie_session_valid(cookie_header: Option<&str>) -> bool {
    let Some(raw) = cookie_header else {
        return false;
    };
    raw.split(';').any(|part| {
        part.trim()
            .strip_prefix("ukb_admin=")
            .is_some_and(|value| admin_session_valid(value.trim()))
    })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn secret_path() -> PathBuf {
    let dir = std::env::var("RESUMA_DATA_DIR").unwrap_or_else(|_| ".resuma".into());
    PathBuf::from(dir).join("admin.key")
}

fn load_or_create_secret() -> [u8; 32] {
    if let Ok(raw) = std::env::var("GATE_SECRET") {
        let t = raw.trim();
        if let Some(bytes) = decode_hex32(t) {
            return bytes;
        }
        if !t.is_empty() {
            return sha256_32(t.as_bytes());
        }
    }
    let path = secret_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Some(bytes) = decode_hex32(s.trim()) {
            return bytes;
        }
    }
    let mut bytes = [0u8; 32];
    if getrandom::fill(&mut bytes).is_err() {
        bytes = sha256_32(now_secs().to_string().as_bytes());
    }
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(&path, hex_encode(&bytes));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    bytes
}

fn sha256_32(data: &[u8]) -> [u8; 32] {
    let d = Sha256::digest(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&d);
    out
}

fn hmac_hex(data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(&*SECRET).expect("hmac key");
    mac.update(data);
    hex_encode(&mac.finalize().into_bytes())
}

fn hmac_ok(data: &[u8], sig_hex: &str) -> bool {
    let mut mac = HmacSha256::new_from_slice(&*SECRET).expect("hmac key");
    mac.update(data);
    let got = mac.finalize().into_bytes();
    let Some(want) = decode_hex(sig_hex) else {
        return false;
    };
    if got.len() != want.len() {
        return false;
    }
    got.iter()
        .zip(want.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

fn ct_eq(left: &str, right: &str) -> bool {
    let a = left.as_bytes();
    let b = right.as_bytes();
    let mut diff = (a.len() ^ b.len()) as u8;
    let n = a.len().max(b.len()).max(1);
    for i in 0..n {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= x ^ y;
    }
    diff == 0
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

fn decode_hex32(s: &str) -> Option<[u8; 32]> {
    let bytes = decode_hex(s)?;
    if bytes.len() != 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Some(out)
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
