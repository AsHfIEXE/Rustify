use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::io::{BufReader, Cursor};
use std::sync::atomic::{AtomicU32, Ordering};
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

pub struct AudioEngine {
    player: Player,
    _device: rodio::MixerDeviceSink,
    volume: AtomicU32,
}

impl AudioEngine {
    pub fn new() -> Result<Self, AudioError> {
        let device = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| AudioError::Stream(e.to_string()))?;
        let player = Player::connect_new(&device.mixer());

        Ok(Self {
            player,
            _device: device,
            volume: AtomicU32::new(f32::to_bits(1.0)),
        })
    }

    pub async fn play_url(&self, url: &str) -> Result<(), AudioError> {
        self.player.stop();

        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;

        let cursor = Cursor::new(bytes);
        let reader = BufReader::new(cursor);
        let source = Decoder::new(reader)
            .map_err(|e| AudioError::Decode(e.to_string()))?;

        self.player.append(source);
        self.player.set_volume(self.volume());
        self.player.play();

        Ok(())
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
    }

    pub fn stop(&self) {
        self.player.stop();
    }

    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn is_empty(&self) -> bool {
        self.player.empty()
    }

    pub fn set_volume(&self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        self.volume.store(f32::to_bits(clamped), Ordering::Relaxed);
        self.player.set_volume(clamped);
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }
}
