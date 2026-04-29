//! Player bar component — Bottom playback controls.
//!
//! Renders the now-playing info, transport controls, and progress bar.

use crate::app::NowPlaying;

/// Format playback progress as "current / total".
pub fn format_progress(np: &NowPlaying) -> String {
    let current = format_time(np.progress_ms);
    let total = format_time(np.duration_ms);
    format!("{current} / {total}")
}

/// Format milliseconds to "M:SS".
fn format_time(ms: u64) -> String {
    let minutes = ms / 60_000;
    let seconds = (ms % 60_000) / 1_000;
    format!("{minutes}:{seconds:02}")
}

/// Calculate the progress as a 0.0 - 1.0 fraction.
pub fn progress_fraction(np: &NowPlaying) -> f64 {
    if np.duration_ms == 0 {
        return 0.0;
    }
    (np.progress_ms as f64 / np.duration_ms as f64).min(1.0)
}
