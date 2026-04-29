//! Audio Resolver — The "Secret Sauce".
//!
//! Bridges Spotify track metadata to a playable audio stream URL by:
//! 1. Searching Piped using ISRC (primary) or "Artist - Title" (fallback).
//! 2. Filtering by duration match (±2 seconds tolerance).
//! 3. Selecting the highest-bitrate audio-only stream.

use crate::app::TrackEntry;
use crate::providers::piped::{PipedClient, PipedError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("Piped error: {0}")]
    Piped(#[from] PipedError),
    #[error("No matching track found for: {0}")]
    NoMatch(String),
}

/// Duration tolerance for matching (in milliseconds).
const DURATION_TOLERANCE_MS: i64 = 2_000;

/// Resolved audio stream result.
#[derive(Debug, Clone)]
pub struct ResolvedStream {
    pub stream_url: String,
    pub bitrate: u32,
    pub codec: String,
    pub video_title: String,
}

/// Resolve a Spotify track to a playable stream URL.
pub async fn resolve_stream_url(
    track: &TrackEntry,
    piped: &PipedClient,
) -> Result<ResolvedStream, ResolveError> {
    let target_duration = track.duration_ms as i64;

    // Strategy 1: Search by ISRC (unique recording identifier)
    if let Some(isrc) = &track.isrc {
        if !isrc.is_empty() {
            tracing::info!("Resolving via ISRC: {isrc}");
            if let Ok(results) = piped.search(isrc).await {
                for item in &results {
                    if item.item_type.as_deref() != Some("stream") {
                        continue;
                    }
                    if let Some(item_duration_secs) = item.duration {
                        let item_duration_ms = item_duration_secs * 1000;
                        let diff = (target_duration - item_duration_ms).abs();
                        if diff > DURATION_TOLERANCE_MS {
                            continue;
                        }
                    }
                    if let Some(ref video_url) = item.url {
                        tracing::info!("Duration match found for ISRC: {:?}", item.title);
                        if let Ok(audio) = piped.get_best_audio_url(video_url).await {
                            return Ok(ResolvedStream {
                                stream_url: audio.url,
                                bitrate: audio.bitrate.unwrap_or(0),
                                codec: audio.codec.unwrap_or_else(|| "unknown".into()),
                                video_title: item.title.clone().unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Strategy 2: Search by "Artist - Track Name"
    let fallback_query = format!("{} - {}", track.artist, track.name);
    tracing::info!("Resolving via title: {fallback_query}");
    
    let results = piped.search(&fallback_query).await?;
    for item in &results {
        if item.item_type.as_deref() != Some("stream") {
            continue;
        }

        if let Some(item_duration_secs) = item.duration {
            let item_duration_ms = item_duration_secs * 1000;
            let diff = (target_duration - item_duration_ms).abs();

            if diff > DURATION_TOLERANCE_MS {
                tracing::debug!(
                    "Duration mismatch: {} vs {} (diff: {}ms)",
                    target_duration, item_duration_ms, diff
                );
                continue;
            }
        }

        if let Some(ref video_url) = item.url {
            tracing::info!("Duration match found for Title: {:?}", item.title);
            match piped.get_best_audio_url(video_url).await {
                Ok(audio) => {
                    return Ok(ResolvedStream {
                        stream_url: audio.url,
                        bitrate: audio.bitrate.unwrap_or(0),
                        codec: audio.codec.unwrap_or_else(|| "unknown".into()),
                        video_title: item.title.clone().unwrap_or_default(),
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to get audio for {:?}: {e}", item.title);
                    continue;
                }
            }
        }
    }

    Err(ResolveError::NoMatch(format!("{} - {}", track.artist, track.name)))
}
