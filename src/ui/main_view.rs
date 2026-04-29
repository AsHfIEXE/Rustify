//! Main view component — Track list and search results.
//!
//! Displays the currently selected playlist's tracks or search results
//! in a scrollable table format.

use crate::app::TrackEntry;

/// Format a duration in milliseconds to "M:SS" display string.
pub fn format_duration(ms: u64) -> String {
    let minutes = ms / 60_000;
    let seconds = (ms % 60_000) / 1_000;
    format!("{minutes}:{seconds:02}")
}

/// Truncate text to a maximum length with ellipsis.
pub fn truncate(text: &str, max_len: usize) -> String {
    if text.len() > max_len {
        format!("{}…", &text[..max_len.saturating_sub(1)])
    } else {
        text.to_string()
    }
}

/// Build a display row from a TrackEntry.
pub fn track_display(index: usize, track: &TrackEntry) -> TrackRow {
    TrackRow {
        number: index + 1,
        title: truncate(&track.name, 50),
        artist: truncate(&track.artist, 30),
        album: truncate(&track.album, 30),
        duration: format_duration(track.duration_ms),
        track_id: track.id.clone(),
    }
}

/// Displayable track row data.
pub struct TrackRow {
    pub number: usize,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: String,
    pub track_id: String,
}
