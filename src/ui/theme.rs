//! Dark "Spotify-inspired" theme colors and styling constants.
//!
//! These constants define the Rustify visual identity.
//! The palette is inspired by Spotify's dark mode but with a unique twist.

use iced::Color;

// ─── Color Palette ───────────────────────────────────────────────────────────

/// Primary background (deep charcoal)
pub const BG_PRIMARY: Color = Color::from_rgb(0.07, 0.07, 0.09);

/// Secondary background (sidebar, cards)
pub const BG_SECONDARY: Color = Color::from_rgb(0.10, 0.10, 0.13);

/// Elevated surface (player bar, hovering cards)
pub const BG_ELEVATED: Color = Color::from_rgb(0.14, 0.14, 0.18);

/// Primary accent (Rustify green — slightly warmer than Spotify)
pub const ACCENT_PRIMARY: Color = Color::from_rgb(0.12, 0.84, 0.38);

/// Secondary accent (for hover states, subtle highlights)
pub const ACCENT_SECONDARY: Color = Color::from_rgb(0.18, 0.90, 0.48);

/// Text primary (bright white)
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.93, 0.93, 0.93);

/// Text secondary (muted gray)
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.60, 0.60, 0.65);

/// Text tertiary (very muted, timestamps, metadata)
pub const TEXT_TERTIARY: Color = Color::from_rgb(0.40, 0.40, 0.45);

/// Error / destructive action
pub const ERROR: Color = Color::from_rgb(0.90, 0.22, 0.22);

/// Warning
pub const WARNING: Color = Color::from_rgb(0.95, 0.75, 0.15);

/// Success
pub const SUCCESS: Color = Color::from_rgb(0.12, 0.84, 0.38);

// ─── Layout Constants ────────────────────────────────────────────────────────

/// Sidebar width in pixels
pub const SIDEBAR_WIDTH: u16 = 250;

/// Player bar height in pixels
pub const PLAYER_BAR_HEIGHT: u16 = 80;

/// Standard border radius
pub const BORDER_RADIUS: f32 = 8.0;

/// Small border radius (buttons, inputs)
pub const BORDER_RADIUS_SM: f32 = 4.0;

/// Standard spacing between elements
pub const SPACING: u16 = 8;

/// Large spacing (section gaps)
pub const SPACING_LG: u16 = 16;

/// Standard padding
pub const PADDING: u16 = 12;
