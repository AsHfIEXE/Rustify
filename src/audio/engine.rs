//! Audio Engine — Wrapper around rodio for playback control.
//!
//! Provides Play, Pause, Stop, Volume, and Seek controls.
//! Uses rodio's Sink with symphonia decoders for Opus/AAC/MP3 support.

use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::io::{BufReader, Cursor};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Playback error: {0}")]
    Playback(String),
    #[error("Decode error: {0}")]
    Decode(String),
    #[error("Stream error: {0}")]
    Stream(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

/// The audio playback engine.
pub struct AudioEngine {
    player: Player,
    _device: rodio::MixerDeviceSink,
    volume: f32,
}

impl AudioEngine {
    /// Initialize the audio engine with the default output device.
    pub fn new() -> Result<Self, AudioError> {
        let device = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| AudioError::Stream(e.to_string()))?;
        let player = Player::connect_new(&device.mixer());

        Ok(Self {
            player,
            _device: device,
            volume: 1.0,
        })
    }

    /// Play audio from a URL by downloading the full buffer first.
    pub async fn play_url(&self, url: &str) -> Result<(), AudioError> {
        self.player.stop();

        tracing::info!("Downloading audio from: {url}");
        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;

        tracing::info!("Downloaded {} bytes, decoding...", bytes.len());
        let cursor = Cursor::new(bytes);
        let reader = BufReader::new(cursor);
        let source = Decoder::new(reader)
            .map_err(|e| AudioError::Decode(e.to_string()))?;

        self.player.append(source);
        self.player.set_volume(self.volume);
        self.player.play();

        tracing::info!("Playback started");
        Ok(())
    }

    /// Pause playback.
    pub fn pause(&self) {
        self.player.pause();
    }

    /// Resume playback.
    pub fn resume(&self) {
        self.player.play();
    }

    /// Stop playback and clear the queue.
    pub fn stop(&self) {
        self.player.stop();
    }

    /// Check if playback is currently paused.
    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    /// Check if the player is empty (nothing playing or queued).
    pub fn is_empty(&self) -> bool {
        self.player.empty()
    }

    /// Set volume (0.0 = mute, 1.0 = full).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.player.set_volume(self.volume);
    }

    /// Get current volume.
    pub fn volume(&self) -> f32 {
        self.volume
    }
}
