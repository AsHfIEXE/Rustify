//! Iced Application Logic — State, Messages, Update loop, and View.
//!
//! This is the "brain" of the GUI. It follows the Elm architecture:
//! Boot → State + Task → Update(Message) → View(State)

use iced::widget::{button, center, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Subscription, Task, Theme};

use crate::config::AppConfig;

// ─── State ───────────────────────────────────────────────────────────────────

/// Root application state.
#[derive(Debug)]
pub struct Rustify {
    /// Application configuration.
    pub config: AppConfig,

    /// Current navigation screen.
    pub screen: Screen,

    /// Whether the user is authenticated with Spotify.
    pub authenticated: bool,

    /// Spotify Client ID input (setup screen).
    pub client_id_input: String,

    /// Currently playing track info (if any).
    pub now_playing: Option<NowPlaying>,

    /// Current playback state.
    pub playback: PlaybackState,

    /// Search query input.
    pub search_query: String,

    /// List of user playlists (simplified).
    pub playlists: Vec<PlaylistEntry>,

    /// Tracks in the currently selected playlist.
    pub tracks: Vec<TrackEntry>,

    /// Status bar message.
    pub status_message: String,
}

/// Current screen/navigation state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// First-time setup: enter Spotify Client ID.
    Setup,
    /// OAuth login in progress.
    Authenticating,
    /// Main library view.
    Library,
    /// Search results view.
    Search,
    /// Settings panel.
    Settings,
}

/// Currently playing track information.
#[derive(Debug, Clone)]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub progress_ms: u64,
    pub cover_url: Option<String>,
}

/// Playback state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Loading,
}

/// Simplified playlist entry for the sidebar.
#[derive(Debug, Clone)]
pub struct PlaylistEntry {
    pub id: String,
    pub name: String,
    pub track_count: u32,
}

/// Simplified track entry for the main view.
#[derive(Debug, Clone)]
pub struct TrackEntry {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub isrc: Option<String>,
}

// ─── Messages ────────────────────────────────────────────────────────────────

/// All possible user/system interactions.
#[derive(Debug, Clone)]
pub enum Message {
    // ── Setup ──
    ClientIdChanged(String),
    SaveClientId,

    // ── Authentication ──
    StartLogin,
    LoginComplete(Result<(), String>),

    // ── Navigation ──
    NavigateTo(Screen),

    // ── Library ──
    PlaylistsLoaded(Vec<PlaylistEntry>),
    SelectPlaylist(String),
    TracksLoaded(Vec<TrackEntry>),

    // ── Search ──
    SearchQueryChanged(String),
    PerformSearch,
    SearchResults(Vec<TrackEntry>),

    // ── Playback ──
    PlayTrack(String),
    TogglePlayPause,
    Stop,
    SkipNext,
    SkipPrev,
    SeekTo(f64),
    TrackResolved(Result<String, String>),
    PlaybackTick,

    // ── Settings ──
    PipedInstanceChanged(String),
    SaveSettings,

    // ── System ──
    StatusMessage(String),
    Noop,
}

// ─── Boot ────────────────────────────────────────────────────────────────────

/// Initialize application state.
pub fn boot() -> (Rustify, Task<Message>) {
    let config = AppConfig::load();

    let screen = if config.has_spotify_credentials() {
        Screen::Authenticating
    } else {
        Screen::Setup
    };

    let state = Rustify {
        config,
        screen,
        authenticated: false,
        client_id_input: String::new(),
        now_playing: None,
        playback: PlaybackState::Stopped,
        search_query: String::new(),
        playlists: Vec::new(),
        tracks: Vec::new(),
        status_message: "Welcome to Rustify 🎵".to_string(),
    };

    // If credentials exist, auto-start authentication
    let task = if state.screen == Screen::Authenticating {
        Task::perform(async { Ok::<(), String>(()) }, Message::LoginComplete)
    } else {
        Task::none()
    };

    (state, task)
}

// ─── Update ──────────────────────────────────────────────────────────────────

/// Handle messages and update state.
pub fn update(state: &mut Rustify, message: Message) -> Task<Message> {
    match message {
        // ── Setup ──
        Message::ClientIdChanged(id) => {
            state.client_id_input = id;
            Task::none()
        }
        Message::SaveClientId => {
            state.config.spotify_client_id = state.client_id_input.clone();
            let _ = state.config.save();
            state.status_message = "Client ID saved!".to_string();
            state.screen = Screen::Authenticating;
            // Trigger login after saving
            Task::perform(async { Ok::<(), String>(()) }, Message::LoginComplete)
        }

        // ── Authentication ──
        Message::StartLogin => {
            state.screen = Screen::Authenticating;
            state.status_message = "Opening browser for Spotify login...".to_string();
            // TODO: Implement actual OAuth PKCE flow
            Task::perform(async { Ok::<(), String>(()) }, Message::LoginComplete)
        }
        Message::LoginComplete(result) => {
            match result {
                Ok(()) => {
                    state.authenticated = true;
                    state.screen = Screen::Library;
                    state.status_message = "Logged in to Spotify ✓".to_string();
                    // TODO: Fetch playlists
                    Task::none()
                }
                Err(e) => {
                    state.status_message = format!("Login failed: {e}");
                    state.screen = Screen::Setup;
                    Task::none()
                }
            }
        }

        // ── Navigation ──
        Message::NavigateTo(screen) => {
            state.screen = screen;
            Task::none()
        }

        // ── Library ──
        Message::PlaylistsLoaded(playlists) => {
            state.playlists = playlists;
            state.status_message = format!("{} playlists loaded", state.playlists.len());
            Task::none()
        }
        Message::SelectPlaylist(id) => {
            state.status_message = format!("Loading playlist {id}...");
            // TODO: Fetch tracks for selected playlist
            Task::none()
        }
        Message::TracksLoaded(tracks) => {
            state.tracks = tracks;
            state.status_message = format!("{} tracks loaded", state.tracks.len());
            Task::none()
        }

        // ── Search ──
        Message::SearchQueryChanged(query) => {
            state.search_query = query;
            Task::none()
        }
        Message::PerformSearch => {
            state.status_message = format!("Searching: {}...", state.search_query);
            state.screen = Screen::Search;
            // TODO: Perform Spotify search
            Task::none()
        }
        Message::SearchResults(results) => {
            state.tracks = results;
            state.status_message = format!("{} results found", state.tracks.len());
            Task::none()
        }

        // ── Playback ──
        Message::PlayTrack(track_id) => {
            state.playback = PlaybackState::Loading;
            state.status_message = format!("Resolving audio for track {track_id}...");
            // TODO: Resolve track → stream URL → play
            Task::none()
        }
        Message::TogglePlayPause => {
            state.playback = match &state.playback {
                PlaybackState::Playing => PlaybackState::Paused,
                PlaybackState::Paused => PlaybackState::Playing,
                other => other.clone(),
            };
            Task::none()
        }
        Message::Stop => {
            state.playback = PlaybackState::Stopped;
            state.now_playing = None;
            Task::none()
        }
        Message::SkipNext | Message::SkipPrev => {
            state.status_message = "Skip not yet implemented".to_string();
            Task::none()
        }
        Message::SeekTo(_position) => {
            // TODO: Implement seeking
            Task::none()
        }
        Message::TrackResolved(result) => {
            match result {
                Ok(url) => {
                    state.playback = PlaybackState::Playing;
                    state.status_message = format!("Playing: {url}");
                    // TODO: Feed URL to audio engine
                }
                Err(e) => {
                    state.playback = PlaybackState::Stopped;
                    state.status_message = format!("Resolution failed: {e}");
                }
            }
            Task::none()
        }
        Message::PlaybackTick => {
            if let Some(ref mut np) = state.now_playing {
                if state.playback == PlaybackState::Playing {
                    np.progress_ms += 1000;
                }
            }
            Task::none()
        }

        // ── Settings ──
        Message::PipedInstanceChanged(url) => {
            state.config.piped_instance = url;
            Task::none()
        }
        Message::SaveSettings => {
            let _ = state.config.save();
            state.status_message = "Settings saved ✓".to_string();
            Task::none()
        }

        // ── System ──
        Message::StatusMessage(msg) => {
            state.status_message = msg;
            Task::none()
        }
        Message::Noop => Task::none(),
    }
}

// ─── Theme ───────────────────────────────────────────────────────────────────

/// Returns the application theme based on user preference.
pub fn theme(state: &Rustify) -> Theme {
    match state.config.theme {
        crate::config::ThemePreference::Dark => Theme::Dark,
        crate::config::ThemePreference::Light => Theme::Light,
        crate::config::ThemePreference::System => Theme::Dark, // Default to dark
    }
}

// ─── Subscription ────────────────────────────────────────────────────────────

/// Global subscriptions (keyboard shortcuts, playback ticks, etc.)
pub fn subscription(state: &Rustify) -> Subscription<Message> {
    if state.playback == PlaybackState::Playing {
        iced::time::every(std::time::Duration::from_secs(1))
            .map(|_| Message::PlaybackTick)
    } else {
        Subscription::none()
    }
}

// ─── View ────────────────────────────────────────────────────────────────────

/// Render the entire UI based on current state.
pub fn view(state: &Rustify) -> Element<Message> {
    let content: Element<Message> = match state.screen {
        Screen::Setup => view_setup(state),
        Screen::Authenticating => view_authenticating(state),
        Screen::Library => view_main(state),
        Screen::Search => view_main(state),
        Screen::Settings => view_settings(state),
    };

    // Wrap everything in a dark container with status bar
    let status_bar = container(
        row![
            text(&state.status_message).size(12),
            Space::new().width(Length::Fill),
            text(match &state.playback {
                PlaybackState::Playing => "▶ Playing",
                PlaybackState::Paused => "⏸ Paused",
                PlaybackState::Loading => "⏳ Loading",
                PlaybackState::Stopped => "⏹ Stopped",
            })
            .size(12),
        ]
        .spacing(10)
        .padding(8),
    )
    .width(Length::Fill);

    column![content, status_bar]
        .height(Length::Fill)
        .into()
}

/// Setup screen — first-time configuration.
fn view_setup(state: &Rustify) -> Element<Message> {
    let content = column![
        Space::new().height(Length::FillPortion(1)),
        text("🎵 Welcome to Rustify").size(32),
        Space::new().height(20),
        text("To get started, you need a Spotify Client ID.").size(16),
        text("Visit developer.spotify.com/dashboard to create one.").size(14),
        Space::new().height(20),
        text_input("Paste your Client ID here...", &state.client_id_input)
            .on_input(Message::ClientIdChanged)
            .on_submit(Message::SaveClientId)
            .padding(12)
            .width(400),
        Space::new().height(10),
        button(text("Continue →").size(16))
            .on_press(Message::SaveClientId)
            .padding([10, 30]),
        Space::new().height(Length::FillPortion(2)),
    ]
    .spacing(8)
    .align_x(iced::Alignment::Center)
    .width(Length::Fill);

    center(content).into()
}

/// Authenticating screen — waiting for OAuth.
fn view_authenticating(_state: &Rustify) -> Element<Message> {
    center(
        column![
            text("🔐 Authenticating with Spotify...").size(24),
            Space::new().height(20),
            text("Check your browser to complete login.").size(14),
        ]
        .spacing(8)
        .align_x(iced::Alignment::Center),
    )
    .into()
}

/// Main layout: Sidebar + Content + Player Bar
fn view_main(state: &Rustify) -> Element<Message> {
    // ── Sidebar ──
    let sidebar = {
        let header = column![
            text("Rustify").size(20),
            Space::new().height(10),
            button(text("🏠 Home").size(14))
                .on_press(Message::NavigateTo(Screen::Library))
                .width(Length::Fill)
                .padding(8),
            button(text("🔍 Search").size(14))
                .on_press(Message::NavigateTo(Screen::Search))
                .width(Length::Fill)
                .padding(8),
            button(text("⚙ Settings").size(14))
                .on_press(Message::NavigateTo(Screen::Settings))
                .width(Length::Fill)
                .padding(8),
            Space::new().height(20),
            text("Your Playlists").size(14),
        ]
        .spacing(4);

        let playlist_list: Element<Message> = if state.playlists.is_empty() {
            text("No playlists loaded yet")
                .size(12)
                .into()
        } else {
            let items: Vec<Element<Message>> = state
                .playlists
                .iter()
                .map(|pl| {
                    button(
                        text(&pl.name).size(13),
                    )
                    .on_press(Message::SelectPlaylist(pl.id.clone()))
                    .width(Length::Fill)
                    .padding(6)
                    .into()
                })
                .collect();

            scrollable(column(items).spacing(2)).into()
        };

        container(
            column![header, playlist_list]
                .spacing(8)
                .padding(16),
        )
        .width(250)
        .height(Length::Fill)
    };

    // ── Main Content ──
    let main_content = {
        let search_bar = row![
            text_input("Search Spotify...", &state.search_query)
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::PerformSearch)
                .padding(10)
                .width(Length::Fill),
            button(text("Search").size(14))
                .on_press(Message::PerformSearch)
                .padding([10, 20]),
        ]
        .spacing(8);

        let track_list: Element<Message> = if state.tracks.is_empty() {
            center(
                text("Select a playlist or search for music").size(16),
            )
            .height(Length::Fill)
            .into()
        } else {
            let header = row![
                text("#").size(12).width(40),
                text("Title").size(12).width(Length::FillPortion(3)),
                text("Artist").size(12).width(Length::FillPortion(2)),
                text("Album").size(12).width(Length::FillPortion(2)),
                text("Duration").size(12).width(80),
            ]
            .spacing(8)
            .padding([0, 8]);

            let items: Vec<Element<Message>> = state
                .tracks
                .iter()
                .enumerate()
                .map(|(i, track)| {
                    let duration_str = format!(
                        "{}:{:02}",
                        track.duration_ms / 60000,
                        (track.duration_ms % 60000) / 1000
                    );
                    button(
                        row![
                            text(format!("{}", i + 1)).size(13).width(40),
                            text(&track.name).size(13).width(Length::FillPortion(3)),
                            text(&track.artist).size(13).width(Length::FillPortion(2)),
                            text(&track.album).size(13).width(Length::FillPortion(2)),
                            text(duration_str).size(13).width(80),
                        ]
                        .spacing(8),
                    )
                    .on_press(Message::PlayTrack(track.id.clone()))
                    .width(Length::Fill)
                    .padding(6)
                    .into()
                })
                .collect();

            scrollable(
                column![header, column(items).spacing(1)]
                    .spacing(4),
            )
            .height(Length::Fill)
            .into()
        };

        container(
            column![search_bar, track_list]
                .spacing(16)
                .padding(16),
        )
        .width(Length::Fill)
        .height(Length::Fill)
    };

    // ── Player Bar ──
    let player_bar = {
        let (title, artist) = match &state.now_playing {
            Some(np) => (np.title.as_str(), np.artist.as_str()),
            None => ("Not Playing", "—"),
        };

        let play_pause_label = match state.playback {
            PlaybackState::Playing => "⏸",
            _ => "▶",
        };

        container(
            row![
                // Track info (left)
                column![
                    text(title).size(14),
                    text(artist).size(12),
                ]
                .width(Length::FillPortion(1))
                .spacing(2),

                // Playback controls (center)
                row![
                    button(text("⏮").size(16))
                        .on_press(Message::SkipPrev)
                        .padding(8),
                    button(text(play_pause_label).size(18))
                        .on_press(Message::TogglePlayPause)
                        .padding([8, 16]),
                    button(text("⏭").size(16))
                        .on_press(Message::SkipNext)
                        .padding(8),
                    button(text("⏹").size(16))
                        .on_press(Message::Stop)
                        .padding(8),
                ]
                .spacing(8)
                .width(Length::FillPortion(1))
                .align_y(iced::Alignment::Center),

                // Volume / extras (right)
                Space::new().width(Length::Fill)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(16)
            .padding(12)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(70)
    };

    // ── Compose Layout ──
    column![
        row![sidebar, main_content].height(Length::Fill),
        player_bar,
    ]
    .height(Length::Fill)
    .into()
}

/// Settings screen.
fn view_settings(state: &Rustify) -> Element<Message> {
    center(
        column![
            text("⚙ Settings").size(24),
            Space::new().height(20),
            text("Piped API Instance:").size(14),
            text_input("https://pipedapi.kavin.rocks", &state.config.piped_instance)
                .on_input(Message::PipedInstanceChanged)
                .padding(10)
                .width(400),
            Space::new().height(10),
            text("Spotify Client ID:").size(14),
            text(&state.config.spotify_client_id).size(13),
            Space::new().height(20),
            row![
                button(text("Save").size(14))
                    .on_press(Message::SaveSettings)
                    .padding([10, 30]),
                button(text("← Back").size(14))
                    .on_press(Message::NavigateTo(Screen::Library))
                    .padding([10, 30]),
            ]
            .spacing(10),
        ]
        .spacing(8)
        .align_x(iced::Alignment::Center)
        .width(Length::Fill),
    )
    .into()
}
