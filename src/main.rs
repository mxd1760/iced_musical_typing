use iced::{
    Element, Font, Length, Subscription, Task, Theme, time, widget::{Column, Row, Space, button, column, row, text, text_input}, window
};
use image::GenericImageView;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

mod spotify_controller;
mod text_controller;
mod char_controller;
use spotify_controller::Song;
use spotify_controller::SpotifyController;
use char_controller::CharController;

use crate::text_controller::TextController;

struct TypingGame {
    input: String,
    query: String,
    score: usize,
    spotify_controller_handle: SpotifyControllerHandle,
    spotify_data: SpotifyData,
    text_controller_handle: TextControllerHandle,
    text_controller_data: TextControllerData,
    char_controller_handle:CharControllerHandle,
    char_bonus:Option<(String,Vec<String>)>,
}

enum CharControllerHandle{
  Loading,
  Ready(CharController)
}

enum SpotifyControllerHandle {
    Loading,
    Ready(Arc<Mutex<SpotifyController>>),
    // Failed(String),
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
    // Failed(String),
}

#[derive(Debug, Clone)]
enum TextType {
    LRCLIB,
    // Github,
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
    Char(CharController),
}

#[derive(Debug, Clone)]
enum Message {
    InitializeStart,
    InitializeComplete(Result<InitializerObject, String>),
    InputChanged(String),
    InputSubmitted,
    Tick(Instant),
    QueryChanged(String),
    QuerySubmitted,
    SpotifyPlay,
    SpotifyPause,
    SpotifyDevices,
    HideDevices,
    SpotifySetDevice(String),
    APIResult(String, Result<(), String>),
    DevicesResult(Result<Vec<(String, String)>, String>),
    SpotifyChangeSong(Song),
    SetLRCLIBText,
    // SetGithubText,
    SetSourceFileText,
    NextLyricBatch,
    SkipLine,
    LoadNewText,
    UpdateText(TextControllerData),
    UpdateSongs(Option<Vec<Song>>),
    CheckForeignChars,
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
                query: String::new(),
                score: 0,
                spotify_controller_handle: SpotifyControllerHandle::Loading,
                spotify_data: SpotifyData::default(),
                text_controller_handle: TextControllerHandle::Loading,
                text_controller_data: TextControllerData::default(),
                char_controller_handle:CharControllerHandle::Loading,
                char_bonus:None
            },
            Task::done(Message::InitializeStart),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InitializeStart => {
                return Task::perform(
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
                    Message::InitializeComplete,
                );
            }
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
                    .chars().count()
                {
                    self.score += 1;
                    self.input = "".into();
                    if !self.text_controller_data.count_up() {
                        return Task::done(Message::NextLyricBatch);
                    }
                }
                return Task::done(Message::CheckForeignChars);
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
            Message::InitializeComplete(result) => match result {
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
                            Message::InitializeComplete,
                        );
                    }
                    InitializerObject::Text(tx) => {
                        self.text_controller_handle = TextControllerHandle::Ready(tx);
                        return Task::perform(
                            async {
                                let char_controller = CharController::init(
                                  vec![
                                    include_str!("../assets/kana_maps/hiragana.json"),
                                    include_str!("../assets/kana_maps/katakana.json"),
                                    include_str!("../assets/kana_maps/kanji-grade-1.json"),
                                    include_str!("../assets/kana_maps/kanji-grade-2.json"),
                                    include_str!("../assets/kana_maps/kanji-grade-3.json"),
                                    include_str!("../assets/kana_maps/kanji-grade-4.json"),
                                    include_str!("../assets/kana_maps/kanji-grade-5.json"),
                                    include_str!("../assets/kana_maps/kanji-grade-6.json"),
                                  ]
                                ).await;
                                Ok(InitializerObject::Char(char_controller))
                            },
                            Message::InitializeComplete,
                        );
                    }
                    InitializerObject::Char(cx)=>{
                      self.char_controller_handle = CharControllerHandle::Ready(cx);
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
                                Some(v) =>{
                                  if v.len()>0{
                                    v
                                  }else{
                                    vec![
                                      "No more lyrics".into(),
                                      "Please load some more".into(),
                                      "No more lyrics".into(),
                                      "Please load some more".into(),
                                      "No more lyrics".into(),
                                      "Please load some more".into(),
                                    ]
                                  }
                                },
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
                return Task::done(Message::CheckForeignChars)
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
                                // TextType::Github => todo!(),
                                TextType::ThisProject => None,
                            };
                            text_controller.load_lyrics(data.text_type, settings).await;
                        },
                        |_| Message::NextLyricBatch,
                    );
                }
            }
            Message::UpdateText(data) =>{
              self.text_controller_data = data;
              return Task::done(Message::CheckForeignChars);
            },
            Message::QueryChanged(query) => self.query = query,
            Message::QuerySubmitted => {
                if let SpotifyControllerHandle::Ready(controller) = &self.spotify_controller_handle
                {
                    let controller = controller.clone();
                    let query = self.query.clone();
                    return Task::perform(
                        async move { controller.lock().await.search(query).await },
                        Message::UpdateSongs,
                    );
                }
            }
            Message::UpdateSongs(result) => match result {
                Some(new_songs) => self.spotify_data.songs_list = new_songs,
                None => {}
            },
            Message::HideDevices => self.spotify_data.devices_list = vec![],
            Message::CheckForeignChars => {
              if let CharControllerHandle::Ready(cc) = &self.char_controller_handle{
                let line = self.text_controller_data.lyrics[self.text_controller_data.current_line]
                            .chars();
                let num_matching = self
                    .input
                    .chars()
                    .zip(line.clone())
                    .take_while(|(a, b)| a == b)
                    .count();
                let symbol:String = line.clone().nth(num_matching).unwrap_or(' ').into();
                self.char_bonus = match cc.get_play_char(symbol.as_str()){
                    Some(v) => {
                      if v.contains(&self.input.chars().skip(num_matching).collect()){
                        return Task::done(Message::InputChanged(self.input.chars().take(num_matching).collect::<String>() + symbol.as_str()))
                      }
                      Some((symbol,v.clone()))
                    },
                    None => None,
                }
              }
            },
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
      if self.text_controller_data.lyrics.len()>0{
        let num_matching = self
            .input
            .chars()
            .zip(self.text_controller_data.lyrics[self.text_controller_data.current_line].chars())
            .take_while(|(a, b)| a == b)
            .count();
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
        let matching_substr:String= target.chars().take(num_matching).collect();
        let remaining_substr:String = target.chars().skip(num_matching).collect();
        let mut info_row:Row<_> = row![text(format!("Score: {}", self.score)),Space::with_width(40)];
        match &self.char_bonus{
            Some((symbol,v)) => {
              info_row = info_row.push(text(format!("{} -> {:?}",symbol,v)));
            },
            None => (),
        }
        let mut songs_ui = Column::new().padding(10).spacing(10);
        for song in &self.spotify_data.songs_list {
            songs_ui = songs_ui.push(
                button(text(song.name.clone() + " by " + song.artist.as_str()))
                    .on_press(Message::SpotifyChangeSong(song.clone())),
            )
        }
        if let SpotifyControllerHandle::Ready(_controller) = &self.spotify_controller_handle {
            let mut devices_ui = Column::new().padding(10).spacing(10);
            for device in &self.spotify_data.devices_list {
                devices_ui = devices_ui.push(
                    button(text(device.0.clone()))
                        .on_press(Message::SpotifySetDevice(device.1.clone())),
                );
            }

            row![
                column![
                    text("Active Spotify Devices"),
                    row![button("Refresh Spotify Devices").on_press(Message::SpotifyDevices),button("Hide Devices").on_press(Message::HideDevices)],
                    devices_ui,
                    text("Spotify Playback Controller"),
                    row![
                        button("Play").on_press(Message::SpotifyPlay),
                        button("Pause").on_press(Message::SpotifyPause)
                    ],
                    text("Text Style"),
                    row![
                        button("LRCLIB").on_press(Message::SetLRCLIBText),
                        // button("Github").on_press(Message::SetGithubText),
                        button("Source File").on_press(Message::SetSourceFileText),
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
                    info_row,
                    row![button("Skip Line").on_press(Message::SkipLine)]
                ],
                column![
                    text("Songs"),
                    row![    
                        text_input("Search", &self.query)
                            .width(Length::Fixed(300.0))
                            .on_input(Message::QueryChanged)
                            .on_submit(Message::QuerySubmitted),
                        button("Search").on_press(Message::QuerySubmitted)
                    ],
                    songs_ui,
                ]
            ]
            .into()
        } else {
            row![
                column![
                    text("Text Style"),
                    row![
                        button("LRCLIB").on_press(Message::SetLRCLIBText),
                        // button("Github").on_press(Message::SetGithubText),
                        button("Source File").on_press(Message::SetSourceFileText),
                    ],
                    row![
                        text("Loading Spotify..."),
                        button("Retry").on_press(Message::InitializeStart)
                    ],
                    text(""),
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
                    text_input("Start typing...", &self.input).on_input(Message::InputChanged),
                    text(format!("Score: {}", self.score)),
                ],
                column![
                    // row![
                        text("Songs"),
                    //     text_input("Search", &self.query)
                    //         .on_input(Message::QueryChanged)
                    //         .on_submit(Message::QuerySubmitted),
                    //     button("Search").on_press(Message::QuerySubmitted)
                    // ],
                    songs_ui,
                ]
            ]
            .into()
        }
      } else {
        column![text("Loading")].into()
      }
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(100)).map(Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::ALL[5].clone()
    }
}
