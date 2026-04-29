use iced::widget::{button, center, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Subscription, Task, Theme};
use crate::config::AppConfig;
use crate::ui::theme;
use crate::audio::engine::AudioEngine;
use crate::providers::spotify::SpotifyClient;
use crate::providers::piped::PipedClient;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

pub struct Rustify {
    pub config: AppConfig,
    pub screen: Screen,
    pub authenticated: bool,
    pub client_id_input: String,
    pub now_playing: Option<NowPlaying>,
    pub playback: PlaybackState,
    pub search_query: String,
    pub new_playlist_name: String,
    pub playlists: Vec<PlaylistEntry>,
    pub tracks: Vec<TrackEntry>,
    pub status_message: String,
    pub spotify: Arc<AsyncMutex<Option<SpotifyClient>>>,
    pub piped: Arc<PipedClient>,
    pub audio: Arc<AudioEngine>,
}

#[derive(Debug, Clone)]
pub struct PlaylistEntry {
    pub id: String,
    pub name: String,
    pub track_count: u32,
}

#[derive(Debug, Clone)]
pub struct TrackEntry {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub isrc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Setup,
    Authenticating,
    Library,
    Search,
    Settings,
}

#[derive(Debug, Clone)]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub progress_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Loading,
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
pub enum Message {
    ClientIdChanged(String),
    SaveClientId,
    StartLogin,
    LoginComplete(Result<(), String>),
    NavigateTo(Screen),
    PlaylistsLoaded(Vec<PlaylistEntry>),
    NewPlaylistNameChanged(String),
    CreatePlaylist,
    PlaylistCreated(Result<(), String>),
    SelectPlaylist(String),
    TracksLoaded(Vec<TrackEntry>),
    SearchQueryChanged(String),
    PerformSearch,
    SearchResults(Vec<TrackEntry>),
    PlayTrack(String),
    TogglePlayPause,
    Stop,
    SkipNext,
    SkipPrev,
    SeekTo(u64),
    TrackResolved(Result<crate::providers::resolver::ResolvedStream, String>),
    AudioFinished,
    PipedInstanceChanged(String),
    SaveSettings,
}

pub fn theme(_state: &Rustify) -> Theme {
    Theme::Dark
}

pub fn subscription(_state: &Rustify) -> Subscription<Message> {
    Subscription::none()
}

pub fn boot() -> (Rustify, Task<Message>) {
    let config = AppConfig::load();
    let screen = if config.has_spotify_credentials() {
        Screen::Authenticating
    } else {
        Screen::Setup
    };

    let piped = Arc::new(PipedClient::new(&config.piped_instance));
    let audio = Arc::new(AudioEngine::new().unwrap_or_else(|_| panic!("Failed to init audio device")));
    let spotify = if config.has_spotify_credentials() {
        Arc::new(AsyncMutex::new(Some(SpotifyClient::new(
            &config.spotify_client_id,
            &config.spotify_redirect_uri,
        ))))
    } else {
        Arc::new(AsyncMutex::new(None))
    };

    let state = Rustify {
        config,
        screen,
        authenticated: false,
        client_id_input: String::new(),
        now_playing: None,
        playback: PlaybackState::Stopped,
        search_query: String::new(),
        new_playlist_name: String::new(),
        playlists: Vec::new(),
        tracks: Vec::new(),
        status_message: "Welcome to Rustify 🎵".to_string(),
        spotify,
        piped,
        audio,
    };

    let task = if state.screen == Screen::Authenticating {
        let spotify_arc = state.spotify.clone();
        Task::perform(async move {
            let mut guard = spotify_arc.lock().await;
            if let Some(client) = guard.as_mut() {
                client.authenticate().await.map_err(|e| e.to_string())
            } else {
                Err("No client".to_string())
            }
        }, Message::LoginComplete)
    } else {
        Task::none()
    };

    (state, task)
}

pub fn update(state: &mut Rustify, message: Message) -> Task<Message> {
    match message {
        Message::ClientIdChanged(id) => {
            state.client_id_input = id;
            Task::none()
        }
        Message::SaveClientId => {
            state.config.spotify_client_id = state.client_id_input.clone();
            let _ = state.config.save();
            state.status_message = "Client ID saved!".to_string();
            state.screen = Screen::Authenticating;
            
            let client = SpotifyClient::new(
                &state.config.spotify_client_id,
                &state.config.spotify_redirect_uri,
            );
            let spotify_arc = state.spotify.clone();
            Task::perform(async move {
                let mut guard = spotify_arc.lock().await;
                *guard = Some(client);
                guard.as_mut().unwrap().authenticate().await.map_err(|e| e.to_string())
            }, Message::LoginComplete)
        }
        Message::StartLogin => {
            state.screen = Screen::Authenticating;
            state.status_message = "Opening browser for Spotify login...".to_string();
            let spotify_arc = state.spotify.clone();
            Task::perform(async move {
                let mut guard = spotify_arc.lock().await;
                if let Some(client) = guard.as_mut() {
                    client.authenticate().await.map_err(|e| e.to_string())
                } else {
                    Err("No Spotify client configured".to_string())
                }
            }, Message::LoginComplete)
        }
        Message::LoginComplete(result) => match result {
            Ok(()) => {
                state.authenticated = true;
                state.screen = Screen::Library;
                state.status_message = "Logged in to Spotify ✓".to_string();
                let spotify_arc = state.spotify.clone();
                Task::perform(async move {
                    let guard = spotify_arc.lock().await;
                    if let Some(client) = guard.as_ref() {
                        client.get_user_playlists().await.unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                }, Message::PlaylistsLoaded)
            }
            Err(e) => {
                state.status_message = format!("Login failed: {e}");
                state.screen = Screen::Setup;
                Task::none()
            }
        },
        Message::NavigateTo(screen) => {
            state.screen = screen;
            Task::none()
        }
        Message::NewPlaylistNameChanged(name) => {
            state.new_playlist_name = name;
            Task::none()
        }
        Message::CreatePlaylist => {
            if state.new_playlist_name.is_empty() {
                return Task::none();
            }
            state.status_message = format!("Creating playlist '{}'...", state.new_playlist_name);
            let name = state.new_playlist_name.clone();
            let spotify_arc = state.spotify.clone();
            Task::perform(async move {
                let guard = spotify_arc.lock().await;
                if let Some(client) = guard.as_ref() {
                    client.create_playlist(&name).await.map_err(|e| e.to_string())
                } else {
                    Err("No client".to_string())
                }
            }, Message::PlaylistCreated)
        }
        Message::PlaylistCreated(result) => {
            match result {
                Ok(()) => {
                    state.status_message = "Playlist created!".to_string();
                    state.new_playlist_name.clear();
                    // Refresh playlists
                    let spotify_arc = state.spotify.clone();
                    Task::perform(async move {
                        let guard = spotify_arc.lock().await;
                        if let Some(client) = guard.as_ref() {
                            client.get_user_playlists().await.unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    }, Message::PlaylistsLoaded)
                }
                Err(e) => {
                    state.status_message = format!("Failed to create playlist: {e}");
                    Task::none()
                }
            }
        }
        Message::PlaylistsLoaded(playlists) => {
            state.playlists = playlists;
            state.status_message = format!("{} playlists loaded", state.playlists.len());
            Task::none()
        }
        Message::SelectPlaylist(id) => {
            state.status_message = format!("Loading playlist {id}...");
            let spotify_arc = state.spotify.clone();
            Task::perform(async move {
                let guard = spotify_arc.lock().await;
                if let Some(client) = guard.as_ref() {
                    client.get_playlist_tracks(&id).await.unwrap_or_default()
                } else {
                    Vec::new()
                }
            }, Message::TracksLoaded)
        }
        Message::TracksLoaded(tracks) => {
            state.tracks = tracks;
            state.status_message = format!("{} tracks loaded", state.tracks.len());
            Task::none()
        }
        Message::SearchQueryChanged(query) => {
            state.search_query = query;
            Task::none()
        }
        Message::PerformSearch => {
            state.status_message = format!("Searching: {}...", state.search_query);
            state.screen = Screen::Search;
            let query = state.search_query.clone();
            let spotify_arc = state.spotify.clone();
            Task::perform(async move {
                let guard = spotify_arc.lock().await;
                if let Some(client) = guard.as_ref() {
                    client.search(&query).await.unwrap_or_default()
                } else {
                    Vec::new()
                }
            }, Message::SearchResults)
        }
        Message::SearchResults(results) => {
            state.tracks = results;
            state.status_message = format!("{} results found", state.tracks.len());
            Task::none()
        }
        Message::PlayTrack(track_id) => {
            if let Some(track) = state.tracks.iter().find(|t| t.id == track_id).cloned() {
                state.playback = PlaybackState::Loading;
                state.status_message = format!("Resolving audio for {}...", track.name);
                state.now_playing = Some(NowPlaying {
                    title: track.name.clone(),
                    artist: track.artist.clone(),
                    album: track.album.clone(),
                    duration_ms: track.duration_ms,
                    progress_ms: 0,
                });
                let piped_arc = state.piped.clone();
                Task::perform(async move {
                    crate::providers::resolver::resolve_stream_url(&track, &piped_arc).await
                        .map_err(|e| e.to_string())
                }, Message::TrackResolved)
            } else {
                Task::none()
            }
        }
        Message::TrackResolved(result) => match result {
            Ok(resolved) => {
                state.playback = PlaybackState::Playing;
                state.status_message = format!("Playing: {}", resolved.video_title);
                let audio_arc = state.audio.clone();
                let url = resolved.stream_url;
                Task::perform(async move {
                    let _ = audio_arc.play_url(&url).await;
                }, |_| Message::AudioFinished)
            }
            Err(e) => {
                state.playback = PlaybackState::Stopped;
                state.status_message = format!("Failed to resolve: {e}");
                Task::none()
            }
        },
        Message::TogglePlayPause => {
            state.playback = match &state.playback {
                PlaybackState::Playing => {
                    state.audio.pause();
                    PlaybackState::Paused
                }
                PlaybackState::Paused => {
                    state.audio.resume();
                    PlaybackState::Playing
                }
                other => other.clone(),
            };
            Task::none()
        }
        Message::Stop => {
            state.audio.stop();
            state.playback = PlaybackState::Stopped;
            state.now_playing = None;
            Task::none()
        }
        Message::AudioFinished => {
            state.playback = PlaybackState::Stopped;
            state.now_playing = None;
            Task::none()
        }
        Message::SkipNext | Message::SkipPrev => {
            state.status_message = "Skip not yet implemented".to_string();
            Task::none()
        }
        Message::SeekTo(_position) => Task::none(),
        Message::PipedInstanceChanged(url) => {
            state.config.piped_instance = url;
            Task::none()
        }
        Message::SaveSettings => {
            let _ = state.config.save();
            state.status_message = "Settings saved".to_string();
            Task::none()
        }
    }
}

pub fn view(state: &Rustify) -> Element<'_, Message> {
    let content = match state.screen {
        Screen::Setup => view_setup(state),
        Screen::Authenticating => view_authenticating(state),
        Screen::Library | Screen::Search => view_main(state),
        Screen::Settings => view_settings(state),
    };

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

    column![content, status_bar].height(Length::Fill).into()
}

fn view_setup(state: &Rustify) -> Element<'_, Message> {
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

fn view_authenticating(_state: &Rustify) -> Element<'_, Message> {
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

fn view_main(state: &Rustify) -> Element<'_, Message> {
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
            row![
                text_input("New Playlist...", &state.new_playlist_name)
                    .on_input(Message::NewPlaylistNameChanged)
                    .on_submit(Message::CreatePlaylist)
                    .padding(4),
                button(text("+")).on_press(Message::CreatePlaylist).padding(4)
            ].spacing(4),
            Space::new().height(10),
            text("Your Playlists").size(14),
        ]
        .spacing(4);

        let playlist_list: Element<Message> = if state.playlists.is_empty() {
            text("No playlists loaded yet").size(12).into()
        } else {
            let items: Vec<Element<Message>> = state
                .playlists
                .iter()
                .map(|pl| {
                    button(text(&pl.name).size(13))
                        .on_press(Message::SelectPlaylist(pl.id.clone()))
                        .width(Length::Fill)
                        .padding(6)
                        .into()
                })
                .collect();
            scrollable(column(items).spacing(2)).into()
        };

        container(column![header, playlist_list].spacing(8).padding(16))
            .width(250)
            .height(Length::Fill)
    };

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
            center(text("Select a playlist or search for music").size(16))
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

            scrollable(column![header, column(items).spacing(1)].spacing(4))
                .height(Length::Fill)
                .into()
        };

        container(column![search_bar, track_list].spacing(16).padding(16))
            .width(Length::Fill)
            .height(Length::Fill)
    };

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
                column![text(title).size(14), text(artist).size(12)]
                    .width(Length::FillPortion(1))
                    .spacing(2),
                row![
                    button(text("⏮").size(16)).on_press(Message::SkipPrev).padding(8),
                    button(text(play_pause_label).size(18)).on_press(Message::TogglePlayPause).padding([8, 16]),
                    button(text("⏭").size(16)).on_press(Message::SkipNext).padding(8),
                    button(text("⏹").size(16)).on_press(Message::Stop).padding(8),
                ]
                .spacing(8)
                .width(Length::FillPortion(1))
                .align_y(iced::Alignment::Center),
                Space::new().width(Length::Fill).width(Length::FillPortion(1)),
            ]
            .spacing(16)
            .padding(12)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(70)
    };

    column![row![sidebar, main_content].height(Length::Fill), player_bar,]
        .height(Length::Fill)
        .into()
}

fn view_settings(state: &Rustify) -> Element<'_, Message> {
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
                button(text("Save").size(14)).on_press(Message::SaveSettings).padding([10, 30]),
                button(text("← Back").size(14)).on_press(Message::NavigateTo(Screen::Library)).padding([10, 30]),
            ]
            .spacing(10),
        ]
        .align_x(iced::Alignment::Center),
    )
    .into()
}
