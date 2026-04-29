//! Plugin API — Functions exposed to WASM plugins.
//!
//! These are the host functions that plugins can call to interact
//! with the Rustify application. They run in the host context and
//! enforce security boundaries (e.g., URL allowlists).

use serde::{Deserialize, Serialize};

/// Track information exposed to plugins (JSON-serializable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub isrc: Option<String>,
}

/// Domains that plugins are allowed to fetch from.
const ALLOWED_DOMAINS: &[&str] = &[
    "api.lyrics.ovh",
    "lrclib.net",
    "api.genius.com",
    "musicbrainz.org",
];

/// Check if a URL is in the plugin allowlist.
pub fn is_url_allowed(url: &str) -> bool {
    ALLOWED_DOMAINS.iter().any(|domain| {
        url.contains(domain)
    })
}

/// Get current track info as JSON string (for plugin consumption).
pub fn get_current_track_json(track: &PluginTrackInfo) -> String {
    serde_json::to_string(track).unwrap_or_else(|_| "{}".into())
}

/// Fetch a URL on behalf of a plugin (with allowlist enforcement).
pub async fn plugin_fetch_url(url: &str) -> Result<Vec<u8>, String> {
    if !is_url_allowed(url) {
        return Err(format!("Domain not in allowlist: {url}"));
    }

    reqwest::get(url)
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_allowlist() {
        assert!(is_url_allowed("https://lrclib.net/api/get?artist=Queen"));
        assert!(is_url_allowed("https://api.lyrics.ovh/v1/Queen/Bohemian%20Rhapsody"));
        assert!(!is_url_allowed("https://evil.com/steal-data"));
        assert!(!is_url_allowed("https://google.com"));
    }

    #[test]
    fn test_track_json() {
        let track = PluginTrackInfo {
            title: "Bohemian Rhapsody".into(),
            artist: "Queen".into(),
            album: "A Night at the Opera".into(),
            duration_ms: 354000,
            isrc: Some("GBUM71029604".into()),
        };
        let json = get_current_track_json(&track);
        assert!(json.contains("Bohemian Rhapsody"));
        assert!(json.contains("GBUM71029604"));
    }
}
