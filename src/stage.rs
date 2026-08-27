//! In-memory download staging. Random IDs, Arc bytes, 64 slots / 30 min.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

const TTL: Duration = Duration::from_secs(1800);
const MAX_SLOTS: usize = 64;

pub struct Staged {
    pub bytes: Arc<[u8]>,
    pub mime: String,
    pub filename: String,
}

struct Slot {
    bytes: Arc<[u8]>,
    mime: String,
    filename: String,
    at: Instant,
}

fn map() -> &'static Mutex<HashMap<String, Slot>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Slot>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn prune(cache: &mut HashMap<String, Slot>) {
    let now = Instant::now();
    cache.retain(|_, v| now.duration_since(v.at) < TTL);
    while cache.len() > MAX_SLOTS {
        let oldest = cache.iter().min_by_key(|(_, v)| v.at).map(|(k, _)| k.clone());
        if let Some(k) = oldest {
            cache.remove(&k);
        } else {
            break;
        }
    }
}

pub fn put(bytes: Vec<u8>, mime: &str, filename: &str) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("empty result".into());
    }
    let mut raw = [0u8; 16];
    getrandom::fill(&mut raw).map_err(|_| "could not mint download id".to_string())?;
    let id = format!(
        "u_{}",
        raw.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
    let bytes: Arc<[u8]> = bytes.into();
    let mut cache = map().lock().map_err(|_| "staging busy".to_string())?;
    prune(&mut cache);
    cache.insert(
        id.clone(),
        Slot {
            bytes,
            mime: mime.to_string(),
            filename: filename.to_string(),
            at: Instant::now(),
        },
    );
    Ok(id)
}

pub fn get(id: &str) -> Option<Staged> {
    let mut cache = map().lock().ok()?;
    prune(&mut cache);
    cache.get(id).map(|s| Staged {
        bytes: s.bytes.clone(),
        mime: s.mime.clone(),
        filename: s.filename.clone(),
    })
}

pub fn is_id(id: &str) -> bool {
    let rest = id.strip_prefix("u_").unwrap_or("");
    rest.len() == 32 && rest.chars().all(|c| c.is_ascii_hexdigit())
}
