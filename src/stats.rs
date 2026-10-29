//! Public totals for completed image jobs.
//!
//! The Turso token stays on the server. The nav renders the numbers, and
//! `GET /api/stats` only returns those totals. Without Turso env vars the
//! totals stay in `RESUMA_DATA_DIR/activity.json`.

use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

const APP: &str = "underkb";

static STATE: std::sync::LazyLock<Mutex<Activity>> =
    std::sync::LazyLock::new(|| Mutex::new(Activity::load_file()));
static REMOTE: OnceLock<libsql::Connection> = OnceLock::new();

#[derive(Clone, Copy)]
pub enum Kind {
    Compress,
    Convert,
    Resize,
    Background,
    Colors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub compress: u64,
    pub convert: u64,
    pub resize: u64,
    pub background: u64,
    pub colors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Activity {
    compress: u64,
    convert: u64,
    resize: u64,
    background: u64,
    colors: u64,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            compress: 0,
            convert: 0,
            resize: 0,
            background: 0,
            colors: 0,
        }
    }
}

impl Activity {
    fn load_file() -> Self {
        let path = store_path();
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        serde_json::from_str(&raw).unwrap_or_default()
    }

    fn max_with(&self, other: &Self) -> Self {
        Self {
            compress: self.compress.max(other.compress),
            convert: self.convert.max(other.convert),
            resize: self.resize.max(other.resize),
            background: self.background.max(other.background),
            colors: self.colors.max(other.colors),
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            compress: self.compress,
            convert: self.convert,
            resize: self.resize,
            background: self.background,
            colors: self.colors,
        }
    }

    fn add(&mut self, kind: Kind, n: u64) {
        if n == 0 {
            return;
        }
        match kind {
            Kind::Compress => self.compress = self.compress.saturating_add(n),
            Kind::Convert => self.convert = self.convert.saturating_add(n),
            Kind::Resize => self.resize = self.resize.saturating_add(n),
            Kind::Background => self.background = self.background.saturating_add(n),
            Kind::Colors => self.colors = self.colors.saturating_add(n),
        }
        self.flush();
        persist_remote(kind, n);
    }

    fn flush(&mut self) {
        let path = store_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let body = serde_json::json!({
            "compress": self.compress,
            "convert": self.convert,
            "resize": self.resize,
            "background": self.background,
            "colors": self.colors,
            "updated_at": now_secs(),
        });
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, body.to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

fn lock() -> std::sync::MutexGuard<'static, Activity> {
    STATE.lock().unwrap_or_else(|err| err.into_inner())
}

fn data_dir() -> PathBuf {
    PathBuf::from(std::env::var("RESUMA_DATA_DIR").unwrap_or_else(|_| ".resuma".into()))
}

fn store_path() -> PathBuf {
    data_dir().join("activity.json")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub async fn init() {
    let Some((url, token)) = turso_config() else {
        return;
    };
    let conn = match connect(&url, &token).await {
        Ok(conn) => conn,
        Err(_) => {
            eprintln!("usage counters: Turso unavailable, keeping the local file");
            return;
        }
    };
    let remote = match fetch(&conn).await {
        Ok(activity) => activity,
        Err(_) => {
            eprintln!("usage counters: Turso read failed, keeping the local file");
            return;
        }
    };
    let merged = {
        let mut guard = lock();
        let merged = guard.max_with(&remote);
        *guard = merged.clone();
        guard.flush();
        merged
    };
    if store_totals(&conn, &merged).await.is_err() {
        eprintln!("usage counters: Turso write failed, keeping the local file");
        return;
    }
    let _ = REMOTE.set(conn);
    eprintln!("usage counters: Turso connected");
}

pub fn record(kind: Kind) {
    lock().add(kind, 1);
}

pub fn record_n(kind: Kind, n: u64) {
    lock().add(kind, n);
}

pub fn snapshot() -> Snapshot {
    lock().snapshot()
}

pub fn grouped(n: u64) -> String {
    let raw = n.to_string();
    let mut out = String::with_capacity(raw.len() + raw.len() / 3);
    for (i, ch) in raw.chars().enumerate() {
        if i > 0 && (raw.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Compress => "compress",
        Kind::Convert => "convert",
        Kind::Resize => "resize",
        Kind::Background => "background",
        Kind::Colors => "colors",
    }
}

fn turso_config() -> Option<(String, String)> {
    let url = std::env::var("TURSO_DATABASE_URL").ok()?;
    let url = url.trim().to_string();
    if !(url.starts_with("libsql://") || url.starts_with("https://")) {
        return None;
    }
    let token = std::env::var("TURSO_AUTH_TOKEN").ok()?;
    let token = token.trim().to_string();
    if token.len() < 20 {
        return None;
    }
    Some((url, token))
}

async fn connect(url: &str, token: &str) -> Result<libsql::Connection, ()> {
    let db = libsql::Builder::new_remote(url.to_string(), token.to_string())
        .build()
        .await
        .map_err(|_| ())?;
    let conn = db.connect().map_err(|_| ())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS usage_counters (
            app TEXT NOT NULL,
            kind TEXT NOT NULL,
            total INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (app, kind)
        )",
        (),
    )
    .await
    .map_err(|_| ())?;
    Ok(conn)
}

async fn fetch(conn: &libsql::Connection) -> Result<Activity, ()> {
    let mut rows = conn
        .query(
            "SELECT kind, total FROM usage_counters WHERE app = ?1",
            [APP],
        )
        .await
        .map_err(|_| ())?;
    let mut activity = Activity::default();
    while let Some(row) = rows.next().await.map_err(|_| ())? {
        let kind: String = row.get(0).unwrap_or_default();
        let total = row.get::<i64>(1).unwrap_or(0).max(0) as u64;
        match kind.as_str() {
            "compress" => activity.compress = total,
            "convert" => activity.convert = total,
            "resize" => activity.resize = total,
            "background" => activity.background = total,
            "colors" => activity.colors = total,
            _ => {}
        }
    }
    Ok(activity)
}

async fn store_totals(conn: &libsql::Connection, activity: &Activity) -> Result<(), ()> {
    for (kind, total) in [
        ("compress", activity.compress),
        ("convert", activity.convert),
        ("resize", activity.resize),
        ("background", activity.background),
        ("colors", activity.colors),
    ] {
        conn.execute(
            "INSERT INTO usage_counters (app, kind, total, updated_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(app, kind) DO UPDATE SET
               total = MAX(usage_counters.total, excluded.total),
               updated_at = excluded.updated_at",
            libsql::params![APP, kind, total as i64, now_secs() as i64],
        )
        .await
        .map_err(|_| ())?;
    }
    Ok(())
}

fn persist_remote(kind: Kind, n: u64) {
    let Some(conn) = REMOTE.get().cloned() else {
        return;
    };
    let kind_name = kind_name(kind);
    let now = now_secs() as i64;
    let add = n as i64;
    tokio::spawn(async move {
        let _ = conn
            .execute(
                "INSERT INTO usage_counters (app, kind, total, updated_at) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(app, kind) DO UPDATE SET
                   total = total + excluded.total,
                   updated_at = excluded.updated_at",
                libsql::params![APP, kind_name, add, now],
            )
            .await;
    });
}
