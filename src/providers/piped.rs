//! Piped API client — Interfaces with a Piped instance to search
//! and fetch audio stream URLs from YouTube without direct YT API usage.

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipedError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("No results found")]
    NoResults,
    #[error("No audio streams available")]
    NoAudioStreams,
}

/// A search result item from Piped.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipedSearchItem {
    pub url: Option<String>,
    pub title: Option<String>,
    pub duration: Option<i64>,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
}

/// Audio stream info from a Piped video.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipedAudioStream {
    pub url: String,
    pub bitrate: Option<u32>,
    pub mime_type: Option<String>,
    pub codec: Option<String>,
    pub quality: Option<String>,
}

/// Full stream response from Piped.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipedStreamResponse {
    pub title: Option<String>,
    pub duration: Option<i64>,
    pub audio_streams: Option<Vec<PipedAudioStream>>,
}

/// Search response wrapper.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipedSearchResponse {
    pub items: Option<Vec<PipedSearchItem>>,
}

/// Client for communicating with a Piped API instance.
pub struct PipedClient {
    http: Client,
    base_url: String,
}

impl PipedClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Search Piped for videos matching a query (ISRC or "Artist - Title").
    pub async fn search(&self, query: &str) -> Result<Vec<PipedSearchItem>, PipedError> {
        let url = format!("{}/search?q={}&filter=music_songs", self.base_url, urlencoding(query));
        let resp: PipedSearchResponse = self.http.get(&url).send().await?.json().await?;
        resp.items.ok_or(PipedError::NoResults)
    }

    /// Get audio streams for a specific video by its URL path (e.g., "/watch?v=xxx").
    pub async fn get_streams(&self, video_path: &str) -> Result<PipedStreamResponse, PipedError> {
        let video_id = video_path
            .split("v=")
            .nth(1)
            .unwrap_or(video_path.trim_start_matches('/'));
        let url = format!("{}/streams/{}", self.base_url, video_id);
        let resp: PipedStreamResponse = self.http.get(&url).send().await?.json().await?;
        Ok(resp)
    }

    /// Get the best audio-only stream URL for a video.
    pub async fn get_best_audio_url(&self, video_path: &str) -> Result<PipedAudioStream, PipedError> {
        let streams = self.get_streams(video_path).await?;
        let audio_streams = streams.audio_streams.ok_or(PipedError::NoAudioStreams)?;

        // Prefer highest bitrate audio stream
        audio_streams
            .into_iter()
            .max_by_key(|s| s.bitrate.unwrap_or(0))
            .ok_or(PipedError::NoAudioStreams)
    }
}

/// Simple URL encoding for query parameters.
fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
