//! Custom Iced Task factories for async operations.
//!
//! These wrap async calls (Spotify API, audio resolving, etc.) into
//! `iced::Task<Message>` values that the update loop can dispatch.

use iced::Task;

use crate::app::{Message, PlaylistEntry, TrackEntry};

/// Fetch the user's playlists from Spotify (async).
pub fn fetch_playlists() -> Task<Message> {
    Task::perform(
        async {
            // TODO: Wire up to providers::spotify::get_user_playlists()
            tracing::info!("Fetching playlists from Spotify...");
            let playlists: Vec<PlaylistEntry> = Vec::new();
            playlists
        },
        Message::PlaylistsLoaded,
    )
}

/// Fetch tracks for a specific playlist from Spotify (async).
pub fn fetch_playlist_tracks(playlist_id: String) -> Task<Message> {
    Task::perform(
        async move {
            // TODO: Wire up to providers::spotify::get_playlist_tracks()
            tracing::info!("Fetching tracks for playlist {playlist_id}...");
            let tracks: Vec<TrackEntry> = Vec::new();
            tracks
        },
        Message::TracksLoaded,
    )
}

/// Search Spotify for tracks matching a query (async).
pub fn search_tracks(query: String) -> Task<Message> {
    Task::perform(
        async move {
            // TODO: Wire up to providers::spotify::search()
            tracing::info!("Searching Spotify for: {query}");
            let results: Vec<TrackEntry> = Vec::new();
            results
        },
        Message::SearchResults,
    )
}

/// Resolve a track to a playable stream URL (async).
pub fn resolve_track(track_id: String) -> Task<Message> {
    Task::perform(
        async move {
            // TODO: Wire up to providers::resolver::resolve_stream_url()
            tracing::info!("Resolving stream URL for track {track_id}...");
            Ok::<String, String>("https://placeholder.stream/audio".to_string())
        },
        Message::TrackResolved,
    )
}
