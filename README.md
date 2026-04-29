# 🎵 Rustify

**The Rust-based Spotify client that set the music free.**

> ⚠️ **Disclaimer:** Rustify is an independent, community-driven client. It is **not** affiliated with, funded, or endorsed by Spotify AB. It uses the public Spotify Web API for metadata and retrieves audio from third-party providers.

---

## ✨ Features

- 🎧 **Native Spotify Client** — Browse your playlists, saved tracks, and search the Spotify catalog
- 🔊 **Free Audio Playback** — Resolves audio via ISRC/title matching through Piped/YouTube
- 🖥️ **60 FPS Native UI** — Built with [Iced](https://iced.rs), a Rust GUI framework inspired by Elm
- 🔐 **Secure OAuth** — PKCE authentication flow, tokens stored in Windows Credential Manager
- 🧩 **Plugin System** — Extend functionality with WASM plugins (lyrics, visualizers, etc.)
- 💾 **Offline Cache** — SQLite-backed local database for instant playlist loading

## 🏗️ Architecture

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Interface  │    │    Brain     │    │     Soul     │
│   (Iced UI)  │◄──►│  (Core API)  │◄──►│   (Audio)    │
│              │    │              │    │              │
│  • Sidebar   │    │  • Spotify   │    │  • Resolver  │
│  • Main View │    │  • Library   │    │  • Engine    │
│  • Player    │    │  • Commands  │    │  • Stream    │
└──────────────┘    └──────────────┘    └──────────────┘
```

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.88+
- A Spotify account (free or premium)
- Your own [Spotify Developer App](https://developer.spotify.com/dashboard) credentials

### Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/AsHfIEXE/rustify.git
   cd rustify
   ```

2. **Create a `.env` file** (or set environment variables):
   ```env
   RSPOTIFY_CLIENT_ID=your_spotify_client_id
   RSPOTIFY_REDIRECT_URI=https://localhost:8888/callback
   ```

3. **Build & Run:**
   ```bash
   cargo run --release
   ```

### Why do I need my own Client ID?

To protect the project from API quota abuse and potential bans, each user must register their own application at [developer.spotify.com](https://developer.spotify.com/dashboard). This is standard practice for open-source Spotify clients.

## 📁 Project Structure

```
rustify/
├── Cargo.toml
├── LICENSE (GPL-3.0)
├── README.md
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # Iced Application Logic
│   ├── config.rs            # App settings
│   ├── ui/                  # UI Components
│   ├── core/                # Business Logic
│   ├── providers/           # Spotify API + Audio Resolving
│   ├── audio/               # Playback Engine
│   └── plugins/             # WASM Plugin System
└── assets/
    └── icon.ico
```

## 🛠️ Tech Stack

| Component | Crate | Purpose |
|:---|:---|:---|
| GUI | `iced 0.14` | Native Elm-architecture UI |
| Async | `tokio` | Async runtime |
| Spotify | `rspotify 0.16` | OAuth PKCE + Web API |
| Audio | `rodio 0.22` | Playback with Symphonia decoders |
| HTTP | `reqwest` | Stream URL fetching |
| Database | `rusqlite` | Local caching |
| Plugins | `wasmtime` | WASM sandbox |
| Secrets | `keyring` | Windows Credential Manager |

## 📜 License

This project is licensed under the **GPL-3.0** license. See [LICENSE](LICENSE) for details.

If you fork and improve this code, you must share those improvements back to the community.

---

*Built with 🦀 Rust and ❤️ for music freedom.*
