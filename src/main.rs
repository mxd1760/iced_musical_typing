use iced::{
    Element, Font, Settings, Subscription, Task, Theme, time,
    widget::{Column, Text, button, column, row, text, text_input},
    window::{self, icon::Icon},
};
use image::GenericImageView;
use rand::Rng;
use std::sync::Arc;
use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

mod spotify_controller;
mod text_controller;
use spotify_controller::Song;
use spotify_controller::SpotifyController;

use crate::text_controller::TextController;

struct TypingGame {
    input: String,
    score: usize,
    spotify_controller_handle: SpotifyControllerHandle,
    spotify_data: SpotifyData,
    text_controller_handle: TextControllerHandle,
    text_controller_data: TextControllerData,
}

enum SpotifyControllerHandle {
    Loading,
    Ready(Arc<Mutex<SpotifyController>>),
    Failed(String),
}

struct SpotifyData {
    pub is_playing: bool,
    pub devices_list: Vec<(String, String)>,
    pub songs_list: Vec<Song>,
    pub current_song: Option<Song>,
}
impl Default for SpotifyData {
    fn default() -> Self {
        Self {
            is_playing: Default::default(),
            devices_list: Default::default(),
            songs_list: Song::mock_songs(),
            current_song: None,
        }
    }
}

enum TextControllerHandle {
    Loading,
    Ready(Arc<Mutex<TextController>>),
    Failed(String),
}

#[derive(Debug, Clone)]
enum TextType {
    LRCLIB,
    Github,
    ThisProject,
}
#[derive(Debug, Clone)]
struct TextControllerData {
    pub text_type: TextType,
    pub lyrics: Vec<String>,
    current_line: usize,
    pub next_fetch_line: i32,
}
impl TextControllerData {
    pub fn count_up(&mut self) -> bool {
        self.current_line += 1;
        if self.current_line >= self.lyrics.len() {
            self.current_line = 0;
            false
        } else {
            true
        }
    }
}
impl Default for TextControllerData {
    fn default() -> Self {
        Self {
            text_type: TextType::ThisProject,
            lyrics: vec![
                "First you need to ".into(),
                "Select a song ".into(),
                "So get on that ".into(),
                "Or you won't be able to start! ".into(),
            ],
            current_line: 0,
            next_fetch_line: 0,
        }
    }
}
#[derive(Debug, Clone)]
enum InitializerObject {
    Spotify(Arc<Mutex<SpotifyController>>),
    Text(Arc<Mutex<TextController>>),
}

#[derive(Debug, Clone)]
enum Message {
    Initialized(Result<InitializerObject, String>),
    InputChanged(String),
    InputSubmitted,
    Tick(Instant),
    SpotifyPlay,
    SpotifyPause,
    SpotifyDevices,
    SpotifySetDevice(String),
    APIResult(String, Result<(), String>),
    DevicesResult(Result<Vec<(String, String)>, String>),
    SpotifyChangeSong(Song),
    SetLRCLIBText,
    SetGithubText,
    SetSourceFileText,
    NextLyricBatch,
    SkipLine,
    LoadNewText,
    UpdateText(TextControllerData),
}

const COMPLETED_COLOR: iced::Color = iced::Color::from_rgb(0.0, 0.5, 0.1);
const MATCHING_COLOR: iced::Color = iced::Color::from_rgb(0.5, 0.8, 1.0);
const PREPARE_COLOR: iced::Color = iced::Color::from_rgb(1.0, 1.0, 0.6);
const UPCOMING_COLOR: iced::Color = iced::Color::from_rgb(1.0, 1.0, 1.0);

impl Default for TypingGame {
    fn default() -> Self {
        Self::new().0
    }
}

const MY_FONT: Font = Font::with_name("Noto Sans CJK JP");

pub fn main() -> iced::Result {
    env_logger::init();
    let full_font = include_bytes!("../assets/fonts/NotoSansCJKjp-Regular.otf").as_slice();

    let icon_bytes = include_bytes!("../assets/icon.png");
    let icon_img = image::load_from_memory(icon_bytes).expect("Failed to load icon");
    let icon_raw = icon_img.to_rgba8().into_raw();
    let (width, height) = icon_img.dimensions();
    println!("debug: w={} h={} raw_len={}", width, height, icon_raw.len());
    let icon = iced::window::icon::from_rgba(icon_raw, width, height).expect("Invalid icon");

    iced::application("Typing Game", TypingGame::update, TypingGame::view)
        .font(full_font)
        .default_font(MY_FONT)
        .subscription(TypingGame::subscription)
        .window(window::Settings {
            icon: Some(icon),
            ..Default::default()
        })
        .theme(TypingGame::theme)
        .run_with(TypingGame::new)
}

impl TypingGame {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                input: String::new(),
                score: 0,
                spotify_controller_handle: SpotifyControllerHandle::Loading,
                spotify_data: SpotifyData::default(),
                text_controller_handle: TextControllerHandle::Loading,
                text_controller_data: TextControllerData::default(),
            },
            Task::perform(
                async {
                    let spotify_controller: SpotifyController =
                        match SpotifyController::init_from_env(rspotify::OAuth {
                            redirect_uri: "http://127.0.0.1:3000".to_string(),
                            scopes: rspotify::scopes!(
                                "user-read-playback-state",
                                "user-modify-playback-state",
                                "user-read-currently-playing",
                                "streaming"
                            ),
                            ..Default::default()
                        })
                        .await
                        {
                            Ok(item) => item,
                            Err(e) => return Err(e.to_string()),
                        };
                    Ok(InitializerObject::Spotify(Arc::new(Mutex::new(
                        spotify_controller,
                    ))))
                },
                Message::Initialized,
            ),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input = value;
                if self.input.trim().is_empty() {
                    self.input = "".into();
                }
                let num_matching = self
                    .input
                    .chars()
                    .zip(
                        self.text_controller_data.lyrics[self.text_controller_data.current_line]
                            .chars(),
                    )
                    .take_while(|(a, b)| a == b)
                    .count();
                if num_matching
                    >= self.text_controller_data.lyrics[self.text_controller_data.current_line]
                        .len()
                {
                    self.score += 1;
                    self.input = "".into();
                    if !self.text_controller_data.count_up() {
                        return Task::done(Message::NextLyricBatch);
                    }
                }
            }
            Message::InputSubmitted => {
                let mut v = self.input.clone();
                v.push(' ');
                return Task::done(Message::InputChanged(v));
            }
            Message::SpotifyPlay => {
                if let SpotifyControllerHandle::Ready(controller) = &self.spotify_controller_handle
                {
                    let controller = controller.clone();
                    return Task::perform(
                        async move {
                            controller
                                .lock()
                                .await
                                .play()
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |out| Message::APIResult("play".into(), out),
                    );
                }
            }
            Message::SpotifyPause => {
                if let SpotifyControllerHandle::Ready(controller) = &self.spotify_controller_handle
                {
                    let controller = controller.clone();
                    return Task::perform(
                        async move {
                            controller
                                .lock()
                                .await
                                .pause()
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |out| Message::APIResult("pause".into(), out),
                    );
                }
            }
            Message::SpotifyDevices => {
                if let SpotifyControllerHandle::Ready(controller) = &self.spotify_controller_handle
                {
                    let controller = controller.clone();
                    return Task::perform(
                        async move {
                            controller
                                .lock()
                                .await
                                .get_devices()
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::DevicesResult,
                    );
                }
            }
            Message::APIResult(kind, result) => match result {
                Err(e) => println!("API Error: {}", e),
                Ok(_) => match kind.as_str() {
                    "play" => self.spotify_data.is_playing = true,
                    "pause" => self.spotify_data.is_playing = false,
                    _ => {}
                },
            },
            Message::Initialized(result) => match result {
                Ok(obj) => match obj {
                    InitializerObject::Spotify(sp) => {
                        self.spotify_controller_handle = SpotifyControllerHandle::Ready(sp);
                        return Task::perform(
                            async {
                                let text_controller = TextController::init().await;
                                Ok(InitializerObject::Text(Arc::new(Mutex::new(
                                    text_controller,
                                ))))
                            },
                            Message::Initialized,
                        );
                    }
                    InitializerObject::Text(tx) => {
                        self.text_controller_handle = TextControllerHandle::Ready(tx);
                    }
                },
                Err(_) => todo!(),
            },
            Message::Tick(_instant) => (),
            Message::DevicesResult(items) => match items {
                Ok(items) => self.spotify_data.devices_list = items,
                Err(e) => log::error!("could not fetch devices {}", e),
            },
            Message::SpotifySetDevice(new_device_id) => {
                if let SpotifyControllerHandle::Ready(controller) = &self.spotify_controller_handle
                {
                    let controller = controller.clone();
                    return Task::perform(
                        async move {
                            controller.lock().await.set_device_id(new_device_id);
                        },
                        |_| Message::APIResult("set_device".into(), Ok(())),
                    );
                }
            }
            Message::SpotifyChangeSong(new_song) => {
                self.spotify_data.current_song = Some(new_song.clone());
                if let SpotifyControllerHandle::Ready(controller) = &self.spotify_controller_handle
                {
                    let new_song_id = new_song.id.clone();
                    let controller = controller.clone();
                    return Task::perform(
                        async move {
                            controller.lock().await.set_song_id(new_song_id);
                        },
                        |_| Message::APIResult("set_song_id".into(), Ok(())),
                    );
                }
            }
            Message::SetLRCLIBText => {
                self.text_controller_data.text_type = TextType::LRCLIB;
                return Task::done(Message::LoadNewText);
            }
            Message::SetGithubText => {
                self.text_controller_data.text_type = TextType::Github;
                return Task::done(Message::LoadNewText);
            }
            Message::SetSourceFileText => {
                self.text_controller_data.text_type = TextType::ThisProject;
                return Task::done(Message::LoadNewText);
            }
            Message::NextLyricBatch => {
                if let TextControllerHandle::Ready(controller) = &self.text_controller_handle {
                    let controller = controller.clone();
                    let data = self.text_controller_data.clone();
                    return Task::perform(
                        async move {
                            let mut text_controller = controller.lock().await;
                            let lyrics = match text_controller
                                .fetch_lyrics(data.next_fetch_line as usize)
                                .await
                            {
                                Some(v) => v,
                                None => vec![
                                    "No more lyrics".into(),
                                    "Please load some more".into(),
                                    "No more lyrics".into(),
                                    "Please load some more".into(),
                                    "No more lyrics".into(),
                                    "Please load some more".into(),
                                ],
                            };
                            TextControllerData {
                                text_type: data.text_type,
                                lyrics,
                                current_line: 0,
                                next_fetch_line: data.next_fetch_line
                                    + text_controller::NUM_LINES as i32,
                            }
                        },
                        Message::UpdateText,
                    );
                }
            }
            Message::SkipLine => {
                self.input = "".into();
                if !self.text_controller_data.count_up() {
                    return Task::done(Message::NextLyricBatch);
                }
            }
            Message::LoadNewText => {
                self.text_controller_data.current_line = 0;
                self.text_controller_data.next_fetch_line = 0;
                if let TextControllerHandle::Ready(controller) = &self.text_controller_handle {
                    let controller = controller.clone();
                    let data = self.text_controller_data.clone();
                    let current_song = self.spotify_data.current_song.clone();
                    return Task::perform(
                        async move {
                            let mut text_controller = controller.lock().await;
                            let settings = match data.text_type {
                                TextType::LRCLIB => match current_song {
                                    Some(song) => Some(song.name + " " + song.artist.as_str()),
                                    None => None,
                                },
                                TextType::Github => todo!(),
                                TextType::ThisProject => None,
                            };
                            text_controller.load_lyrics(data.text_type, settings).await;
                        },
                        |_| Message::NextLyricBatch,
                    );
                }
            }
            Message::UpdateText(data) => self.text_controller_data = data,
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let num_matching = self
            .input
            .chars()
            .zip(self.text_controller_data.lyrics[self.text_controller_data.current_line].chars())
            .take_while(|(a, b)| a == b)
            .count();
        if let SpotifyControllerHandle::Ready(_controller) = &self.spotify_controller_handle {
            let pre: Column<_> = self.text_controller_data.lyrics
                [0..self.text_controller_data.current_line]
                .iter()
                .fold(Column::new(), |col, v| {
                    col.push(text(v).style(|_| text::Style {
                        color: Some(COMPLETED_COLOR),
                    }))
                });
            let post: Column<_> = self.text_controller_data.lyrics
                [self.text_controller_data.current_line + 1..]
                .iter()
                .fold(Column::new(), |col, v| {
                    col.push(text(v).style(|_| text::Style {
                        color: Some(UPCOMING_COLOR),
                    }))
                });
            let target = &self.text_controller_data.lyrics[self.text_controller_data.current_line];
            let matching_substr = &target[0..num_matching];
            let remaining_substr = &target[num_matching..];
            let mut devices_ui = Column::new().padding(10).spacing(10);
            for device in &self.spotify_data.devices_list {
                devices_ui = devices_ui.push(
                    button(text(device.0.clone()))
                        .on_press(Message::SpotifySetDevice(device.1.clone())),
                );
            }
            let mut songs_ui = Column::new().padding(10).spacing(10);
            for song in &self.spotify_data.songs_list {
                songs_ui = songs_ui.push(
                    button(text(song.name.clone()))
                        .on_press(Message::SpotifyChangeSong(song.clone())),
                )
            }
            row![
                column![
                    text("Text Style"),
                    row![
                        button("LRCLIB").on_press(Message::SetLRCLIBText),
                        // button("Github").on_press(Message::SetGithubText),
                        button("Source File").on_press(Message::SetSourceFileText),
                    ],
                    text("Spotify Playback Controller"),
                    row![
                        button("Play").on_press(Message::SpotifyPlay),
                        button("Pause").on_press(Message::SpotifyPause)
                    ],
                    pre,
                    row![
                        text(matching_substr).style(|_| text::Style {
                            color: Some(MATCHING_COLOR)
                        }),
                        text(remaining_substr).style(|_| text::Style {
                            color: Some(PREPARE_COLOR)
                        }),
                    ],
                    post,
                    text_input("Start typing...", &self.input)
                        .on_input(Message::InputChanged)
                        .on_submit(Message::InputSubmitted),
                    text(format!("Score: {}", self.score)),
                    row![button("Skip Line").on_press(Message::SkipLine)]
                ],
                column![
                    text("Songs"),
                    songs_ui,
                    text("Devices"),
                    button("Refresh Spotify Devices").on_press(Message::SpotifyDevices),
                    devices_ui
                ]
            ]
            .into()
        } else {
            column![
                text("Loading Spotify..."),
                text_input("Start typing...", &self.input).on_input(Message::InputChanged),
                text(format!("Score: {}", self.score)),
            ]
            .into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(100)).map(Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::ALL[5].clone()
    }
}

fn is_valid_target_text(text: &str) -> bool {
    let mut out = true;
    out = out && text.len() > 3;
    out
}
