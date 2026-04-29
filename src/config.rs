//! Application configuration and settings management.
//!
//! Handles loading/saving user preferences, Spotify credentials,
//! and Piped instance URLs from a local config file.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Spotify OAuth Client ID (user must provide their own).
    pub spotify_client_id: String,

    /// Redirect URI for OAuth callback.
    pub spotify_redirect_uri: String,

    /// Piped API base URL for audio resolving.
    pub piped_instance: String,

    /// Audio quality preference.
    pub audio_quality: AudioQuality,

    /// UI theme preference.
    pub theme: ThemePreference,

    /// Path to the plugins directory.
    pub plugins_dir: PathBuf,

    /// Whether to cache resolved stream URLs.
    pub cache_streams: bool,
}

/// Audio quality preference for stream selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioQuality {
    /// Highest available bitrate (Opus 160kbps preferred)
    High,
    /// Medium quality (~128kbps)
    Medium,
    /// Low quality for bandwidth-constrained environments
    Low,
}

/// UI theme preference.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemePreference {
    Dark,
    Light,
    System,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            spotify_client_id: String::new(),
            spotify_redirect_uri: "http://localhost:8888/callback".to_string(),
            piped_instance: "https://pipedapi.kavin.rocks".to_string(),
            audio_quality: AudioQuality::High,
            theme: ThemePreference::Dark,
            plugins_dir: Self::default_plugins_dir(),
            cache_streams: true,
        }
    }
}

impl AppConfig {
    /// Returns the path to the config file.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustify")
            .join("config.json")
    }

    /// Returns the default plugins directory.
    fn default_plugins_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustify")
            .join("plugins")
    }

    /// Returns the database file path.
    pub fn database_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustify")
            .join("library.db")
    }

    /// Load config from disk, falling back to defaults.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    serde_json::from_str(&contents).unwrap_or_else(|e| {
                        tracing::warn!("Failed to parse config: {e}, using defaults");
                        Self::default()
                    })
                }
                Err(e) => {
                    tracing::warn!("Failed to read config: {e}, using defaults");
                    Self::default()
                }
            }
        } else {
            tracing::info!("No config found, creating defaults at {}", path.display());
            let config = Self::default();
            let _ = config.save(); // Best-effort save
            config
        }
    }

    /// Persist config to disk.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        tracing::info!("Config saved to {}", path.display());
        Ok(())
    }

    /// Check if Spotify credentials are configured.
    pub fn has_spotify_credentials(&self) -> bool {
        !self.spotify_client_id.is_empty()
    }
}
