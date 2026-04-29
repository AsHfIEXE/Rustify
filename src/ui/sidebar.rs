//! Sidebar component — Playlist navigation list.
//!
//! Renders the left sidebar with navigation buttons and the user's playlists.

use crate::app::{PlaylistEntry, Message};

/// Helper to format playlist display text.
pub fn format_playlist_label(playlist: &PlaylistEntry) -> String {
    if playlist.track_count > 0 {
        format!("{} ({})", playlist.name, playlist.track_count)
    } else {
        playlist.name.clone()
    }
}

/// Generate the list of sidebar items as Messages to emit on click.
pub fn playlist_messages(playlists: &[PlaylistEntry]) -> Vec<(String, Message)> {
    playlists
        .iter()
        .map(|pl| {
            (
                format_playlist_label(pl),
                Message::SelectPlaylist(pl.id.clone()),
            )
        })
        .collect()
}
