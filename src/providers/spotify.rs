//! Spotify API wrapper using rspotify.
//!
//! Handles OAuth PKCE authentication and wraps the Spotify Web API
//! endpoints: user playlists, playlist tracks, search, and ISRC extraction.

use rspotify::{prelude::*, scopes, AuthCodePkceSpotify, Credentials, OAuth};
use thiserror::Error;
use crate::app::{PlaylistEntry, TrackEntry};

#[derive(Debug, Error)]
pub enum SpotifyError {
    #[error("Auth failed: {0}")]
    AuthError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Client error: {0}")]
    ClientError(#[from] rspotify::ClientError),
}

/// Manages the authenticated Spotify client.
pub struct SpotifyClient {
    client: AuthCodePkceSpotify,
}

impl SpotifyClient {
    pub fn new(client_id: &str, redirect_uri: &str) -> Self {
        let creds = Credentials::new_pkce(client_id);
        let oauth = OAuth {
            redirect_uri: redirect_uri.to_string(),
            scopes: scopes!(
                "user-read-playback-state",
                "user-library-read",
                "playlist-read-private",
                "playlist-read-collaborative"
            ),
            ..Default::default()
        };
        Self { client: AuthCodePkceSpotify::new(creds, oauth) }
    }

    /// Start OAuth PKCE flow — opens browser, catches callback.
    pub async fn authenticate(&mut self) -> Result<(), SpotifyError> {
        let url = self.client.get_authorize_url(None)
            .map_err(|e| SpotifyError::AuthError(e.to_string()))?;
        let _ = open::that(&url);
        self.client.prompt_for_token(&url).await
            .map_err(|e| SpotifyError::AuthError(e.to_string()))?;
        Ok(())
    }

    /// Fetch user playlists (paginated).
    pub async fn get_user_playlists(&self) -> Result<Vec<PlaylistEntry>, SpotifyError> {
        let mut out = Vec::new();
        let mut offset = 0;
        loop {
            let page = self.client
                .current_user_playlists_manual(Some(50), Some(offset)).await
                .map_err(|e| SpotifyError::ApiError(e.to_string()))?;
            for item in &page.items {
                let id_str = item.id.id().to_string();
                out.push(PlaylistEntry {
                    id: id_str,
                    name: item.name.clone(),
                    track_count: item.items.total,
                });
            }
            if page.next.is_none() { break; }
            offset += 50;
        }
        Ok(out)
    }

    /// Fetch tracks for a playlist (paginated).
    pub async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<TrackEntry>, SpotifyError> {
        use rspotify::model::PlaylistId;
        let pid = PlaylistId::from_id(playlist_id)
            .map_err(|e| SpotifyError::ApiError(e.to_string()))?;
        let mut out = Vec::new();
        let mut offset = 0;
        loop {
            let page = self.client
                .playlist_items_manual(pid.clone(), None, None, Some(100), Some(offset)).await
                .map_err(|e| SpotifyError::ApiError(e.to_string()))?;
            for item in &page.items {
                if let Some(rspotify::model::PlayableItem::Track(ref t)) = item.item {
                    let id_str = match &t.id {
                        Some(id) => id.id().to_string(),
                        None => String::new(),
                    };
                    out.push(TrackEntry {
                        id: id_str,
                        name: t.name.clone(),
                        artist: t.artists.first().map(|a| a.name.clone()).unwrap_or_default(),
                        album: t.album.name.clone(),
                        duration_ms: t.duration.num_milliseconds() as u64,
                        isrc: t.external_ids.get("isrc").cloned(),
                    });
                }
            }
            if page.next.is_none() { break; }
            offset += 100;
        }
        Ok(out)
    }

    /// Search Spotify catalog.
    pub async fn search(&self, query: &str) -> Result<Vec<TrackEntry>, SpotifyError> {
        use rspotify::model::SearchType;
        let results = self.client
            .search(query, SearchType::Track, None, None, Some(20), Some(0)).await
            .map_err(|e| SpotifyError::ApiError(e.to_string()))?;
        let mut out = Vec::new();
        if let rspotify::model::SearchResult::Tracks(page) = results {
            for t in &page.items {
                let id_str = match &t.id {
                    Some(id) => id.id().to_string(),
                    None => String::new(),
                };
                out.push(TrackEntry {
                    id: id_str,
                    name: t.name.clone(),
                    artist: t.artists.first().map(|a| a.name.clone()).unwrap_or_default(),
                    album: t.album.name.clone(),
                    duration_ms: t.duration.num_milliseconds() as u64,
                    isrc: t.external_ids.get("isrc").cloned(),
                });
            }
        }
        Ok(out)
    }
}
