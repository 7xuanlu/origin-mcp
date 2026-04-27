use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);
const RELEASES_URL: &str = "https://api.github.com/repos/7xuanlu/origin-mcp/releases/latest";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheEntry {
    latest_tag: String,
    checked_at_secs: u64,
}

fn cache_path() -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("origin-mcp");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("version-check.json"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn load_cache() -> Option<CacheEntry> {
    let path = cache_path()?;
    let bytes = std::fs::read(&path).ok()?;
    let entry: CacheEntry = serde_json::from_slice(&bytes).ok()?;
    if now_secs().saturating_sub(entry.checked_at_secs) < CACHE_TTL.as_secs() {
        Some(entry)
    } else {
        None
    }
}

fn store_cache(entry: &CacheEntry) {
    if let Some(path) = cache_path() {
        if let Ok(bytes) = serde_json::to_vec(entry) {
            let _ = std::fs::write(path, bytes);
        }
    }
}

async fn fetch_latest_tag() -> Option<String> {
    let resp = reqwest::Client::new()
        .get(RELEASES_URL)
        .header(
            "User-Agent",
            concat!("origin-mcp/", env!("CARGO_PKG_VERSION")),
        )
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .ok()?;
    let body: serde_json::Value = resp.json().await.ok()?;
    body["tag_name"]
        .as_str()
        .map(|s| s.trim_start_matches('v').to_string())
}

/// Check for a newer published release. Returns Some(message) if behind,
/// None otherwise. Uses a 24h on-disk cache so this never adds startup latency
/// after the first run.
pub async fn check() -> Option<String> {
    let mcp_version = env!("CARGO_PKG_VERSION");
    let mcp = Version::parse(mcp_version).ok()?;

    let latest_tag = match load_cache() {
        Some(entry) => entry.latest_tag,
        None => {
            let tag = fetch_latest_tag().await?;
            store_cache(&CacheEntry {
                latest_tag: tag.clone(),
                checked_at_secs: now_secs(),
            });
            tag
        }
    };

    let latest = Version::parse(&latest_tag).ok()?;
    if latest > mcp {
        Some(format!(
            "A newer origin-mcp is available (v{latest}, you are on v{mcp}). \
             Run `brew upgrade origin-mcp`."
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize tests that read/write the shared on-disk cache to prevent races.
    static CACHE_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn cache_path_under_user_cache_dir() {
        let p = cache_path().expect("cache dir should resolve on this platform");
        assert!(p.ends_with("origin-mcp/version-check.json"), "got {p:?}");
    }

    #[test]
    fn cache_round_trip_within_ttl() {
        let _guard = CACHE_LOCK.lock().unwrap();
        let entry = CacheEntry {
            latest_tag: "9.9.9".to_string(),
            checked_at_secs: now_secs(),
        };
        store_cache(&entry);
        let loaded = load_cache().expect("cache should load");
        assert_eq!(loaded.latest_tag, "9.9.9");
    }

    #[test]
    fn cache_expires_after_ttl() {
        let _guard = CACHE_LOCK.lock().unwrap();
        let entry = CacheEntry {
            latest_tag: "9.9.9".to_string(),
            checked_at_secs: now_secs().saturating_sub(CACHE_TTL.as_secs() + 60),
        };
        store_cache(&entry);
        assert!(
            load_cache().is_none(),
            "expired entry should not be returned"
        );
    }
}
