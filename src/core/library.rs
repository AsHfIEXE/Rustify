//! Local SQLite database for caching playlists, tracks, and resolved URLs.
//!
//! This avoids hammering the Spotify API on every launch and provides
//! instant-load playlists even when offline.

use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;

use crate::app::{PlaylistEntry, TrackEntry};

/// Manages the local SQLite cache.
pub struct Library {
    conn: Connection,
}

impl Library {
    /// Open (or create) the database at the given path.
    pub fn open(path: &Path) -> SqlResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        let lib = Self { conn };
        lib.initialize_tables()?;
        Ok(lib)
    }

    /// Create tables if they don't already exist.
    fn initialize_tables(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS playlists (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                track_count INTEGER DEFAULT 0,
                updated_at  TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS tracks (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                artist      TEXT NOT NULL,
                album       TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                isrc        TEXT,
                playlist_id TEXT,
                FOREIGN KEY (playlist_id) REFERENCES playlists(id)
            );

            CREATE TABLE IF NOT EXISTS resolved_urls (
                track_id    TEXT PRIMARY KEY,
                stream_url  TEXT NOT NULL,
                resolved_at TEXT DEFAULT (datetime('now')),
                expires_at  TEXT,
                FOREIGN KEY (track_id) REFERENCES tracks(id)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    // ─── Playlists ───────────────────────────────────────────────────────

    /// Upsert a playlist into the cache.
    pub fn upsert_playlist(&self, playlist: &PlaylistEntry) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO playlists (id, name, track_count, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))",
            params![playlist.id, playlist.name, playlist.track_count],
        )?;
        Ok(())
    }

    /// Fetch all cached playlists.
    pub fn get_playlists(&self) -> SqlResult<Vec<PlaylistEntry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, track_count FROM playlists ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(PlaylistEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                track_count: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    // ─── Tracks ──────────────────────────────────────────────────────────

    /// Upsert a track into the cache.
    pub fn upsert_track(&self, track: &TrackEntry, playlist_id: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tracks (id, name, artist, album, duration_ms, isrc, playlist_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                track.id,
                track.name,
                track.artist,
                track.album,
                track.duration_ms,
                track.isrc,
                playlist_id
            ],
        )?;
        Ok(())
    }

    /// Fetch all tracks for a given playlist.
    pub fn get_tracks_for_playlist(&self, playlist_id: &str) -> SqlResult<Vec<TrackEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, artist, album, duration_ms, isrc
             FROM tracks WHERE playlist_id = ?1 ORDER BY rowid",
        )?;
        let rows = stmt.query_map(params![playlist_id], |row| {
            Ok(TrackEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_ms: row.get(4)?,
                isrc: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    // ─── Resolved URLs ───────────────────────────────────────────────────

    /// Cache a resolved stream URL for a track.
    pub fn cache_resolved_url(
        &self,
        track_id: &str,
        stream_url: &str,
        ttl_hours: i64,
    ) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO resolved_urls (track_id, stream_url, resolved_at, expires_at)
             VALUES (?1, ?2, datetime('now'), datetime('now', '+' || ?3 || ' hours'))",
            params![track_id, stream_url, ttl_hours],
        )?;
        Ok(())
    }

    /// Look up a cached stream URL (only if not expired).
    pub fn get_cached_url(&self, track_id: &str) -> SqlResult<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT stream_url FROM resolved_urls
             WHERE track_id = ?1 AND (expires_at IS NULL OR expires_at > datetime('now'))",
        )?;
        let mut rows = stmt.query_map(params![track_id], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(url)) => Ok(Some(url)),
            _ => Ok(None),
        }
    }

    /// Purge all expired cached URLs.
    pub fn purge_expired_urls(&self) -> SqlResult<usize> {
        self.conn.execute(
            "DELETE FROM resolved_urls WHERE expires_at IS NOT NULL AND expires_at <= datetime('now')",
            [],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_db() -> Library {
        Library::open(&PathBuf::from(":memory:")).unwrap()
    }

    #[test]
    fn test_playlist_roundtrip() {
        let db = test_db();
        let pl = PlaylistEntry {
            id: "pl1".into(),
            name: "Test Playlist".into(),
            track_count: 42,
        };
        db.upsert_playlist(&pl).unwrap();
        let playlists = db.get_playlists().unwrap();
        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].name, "Test Playlist");
        assert_eq!(playlists[0].track_count, 42);
    }

    #[test]
    fn test_track_roundtrip() {
        let db = test_db();
        let pl = PlaylistEntry {
            id: "pl1".into(),
            name: "Test".into(),
            track_count: 1,
        };
        db.upsert_playlist(&pl).unwrap();

        let track = TrackEntry {
            id: "t1".into(),
            name: "Bohemian Rhapsody".into(),
            artist: "Queen".into(),
            album: "A Night at the Opera".into(),
            duration_ms: 354000,
            isrc: Some("GBUM71029604".into()),
        };
        db.upsert_track(&track, "pl1").unwrap();
        let tracks = db.get_tracks_for_playlist("pl1").unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].name, "Bohemian Rhapsody");
    }

    #[test]
    fn test_resolved_url_cache() {
        let db = test_db();
        db.cache_resolved_url("t1", "https://example.com/audio.opus", 6).unwrap();
        let url = db.get_cached_url("t1").unwrap();
        assert!(url.is_some());
        assert_eq!(url.unwrap(), "https://example.com/audio.opus");
    }
}
