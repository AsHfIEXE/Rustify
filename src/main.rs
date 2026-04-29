//! # Rustify
//!
//! The Rust-based Spotify client that set the music free.
//! Entry point: loads config, initializes tracing, and launches the Iced GUI.

mod app;
mod config;

mod ui;
mod core;
mod providers;
mod audio;
mod plugins;

use tracing_subscriber::{fmt, EnvFilter};

fn main() -> iced::Result {
    // Initialize structured logging
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("rustify=info")),
        )
        .init();

    tracing::info!("🎵 Rustify starting up...");

    // Launch the Iced application using the 0.14 builder pattern
    iced::application(app::boot, app::update, app::view)
        .title("Rustify — Spotify Client")
        .theme(app::theme)
        .subscription(app::subscription)
        .window_size(iced::Size::new(1280.0, 800.0))
        .run()
}
