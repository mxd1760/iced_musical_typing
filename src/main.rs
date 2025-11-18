use iced::{
    Element, Subscription, Task, Theme, time,
    widget::{button, column, row, text, text_input},
};
use rand::Rng;
use std::{
    fs::File, io::{self, BufRead}, path::PathBuf, thread, time::{Duration, Instant}
};
use std::sync::Arc;
use tokio::sync::Mutex;

mod spotify_controller;
mod text_controller;
use spotify_controller::SpotifyController;


struct TypingGame {
    input: String,
    target: String,
    score: usize,
    spotify_controller_state:SpotifyControllerState
}

#[derive(Debug, Clone)]
enum Message {
    Initialized(Result<Arc<Mutex<SpotifyController>>,String>),
    InputChanged(String),
    Tick(Instant),
    UpdateTarget(String),
    SpotifyPlay,
    SpotifyPause,
    APIResult(Result<(),String>)
}

enum SpotifyControllerState{
  Loading,
  Ready(Arc<Mutex<SpotifyController>>),
  Failed(String)
}

const MATCHING_COLOR: iced::Color = iced::Color::from_rgb(0.0, 0.5, 0.1);

impl Default for TypingGame {
    fn default() -> Self {
        Self::new().0
    }
}

pub fn main() -> iced::Result {
    env_logger::init();
    iced::application("Typing Game", TypingGame::update, TypingGame::view)
        .subscription(TypingGame::subscription)
        .theme(TypingGame::theme)
        .run_with(TypingGame::new)
}


impl TypingGame {
    fn new() -> (Self,Task<Message>) {
        (
          Self {
            input: String::new(),
            target: "This is your first typing challenge!".to_string(),
            score: 0,
            spotify_controller_state:SpotifyControllerState::Loading
          },
          Task::perform(async {
            let spotify_controller:SpotifyController = match SpotifyController::init_from_env(
                rspotify::OAuth {
                    redirect_uri: "http://127.0.0.1:3000".to_string(),
                    scopes: rspotify::scopes!(
                        "user-read-playback-state",
                        "user-modify-playback-state",
                        "user-read-currently-playing",
                        "streaming"
                    ),
                    ..Default::default()
            }).await{
                Ok(item) => item,
                Err(e) => return Err(e.to_string()),
            };
            Ok(Arc::new(Mutex::new(spotify_controller)))   
        },Message::Initialized))

    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input = value;
                let num_matching = self
                    .input
                    .chars()
                    .zip(self.target.chars())
                    .take_while(|(a, b)| a == b)
                    .count();
                if num_matching >= self.target.len() {
                    self.score += 1;
                    let line_number = rand::rng().random::<i32>() as usize;
                    return Task::perform(
                        async move {
                            load_new_line(
                                format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")),
                                line_number,
                            )
                            .await
                            .unwrap_or_else(|_| "Failed to load line".to_string())
                        },
                        Message::UpdateTarget,
                    );
                }
            }
            Message::UpdateTarget(new_target) => {
                if is_valid_target_text(&new_target) {
                    self.target = new_target;
                    self.input = "".into();
                } else {
                    let line_number = rand::rng().random::<i32>() as usize;
                    return Task::perform(
                        async move {
                            load_new_line(
                                format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")),
                                line_number,
                            )
                            .await
                            .unwrap_or_else(|_| "Failed to load line".to_string())
                        },
                        Message::UpdateTarget,
                    );
                }
            },
            Message::SpotifyPlay => {
              if let SpotifyControllerState::Ready(controller) = &self.spotify_controller_state {
                let controller = controller.clone();
                return Task::perform(
                  async move {
                    controller.lock().await.play(None).await.map_err(|e| e.to_string())
                  },
                  Message::APIResult
                );
              }
            },
            Message::SpotifyPause => {
              if let SpotifyControllerState::Ready(controller) = &self.spotify_controller_state {
                let controller = controller.clone();
                return Task::perform(
                  async move {
                    controller.lock().await.pause().await.map_err(|e| e.to_string())
                  },
                  Message::APIResult
                );
              }
            },
            Message::APIResult(result) => match result{
              Err(e) => println!("API Error: {}",e),
              _=>{}
            },
            Message::Initialized(result) => match result{
                Ok(mutex) => self.spotify_controller_state = SpotifyControllerState::Ready(mutex),
                Err(_) => todo!(),
            },
            Message::Tick(instant) => (),
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let num_matching = self
            .input
            .chars()
            .zip(self.target.chars())
            .take_while(|(a, b)| a == b)
            .count();
        if let SpotifyControllerState::Ready(_controller) = &self.spotify_controller_state {
            let matching_substr = &self.target[0..num_matching];
            let remaining_substr = &self.target[num_matching..self.target.len()];
            column![
                row![
                    button("Play").on_press(Message::SpotifyPlay),
                    button("Pause").on_press(Message::SpotifyPause)
                ],
                row![
                    text(matching_substr).style(|_| text::Style {
                        color: Some(MATCHING_COLOR)
                    }),
                    text(remaining_substr)
                ],
                text_input("Start typing...", &self.input).on_input(Message::InputChanged),
                text(format!("Score: {}", self.score)),
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

async fn load_new_line(
    file_name: impl Into<PathBuf>,
    line_number: usize,
) -> anyhow::Result<String> {
    let file_path = file_name.into();
    let line_handle = thread::spawn(move || -> Result<String, io::Error> {
        let file = File::open(file_path)?;
        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
        Ok(lines[line_number % lines.len()].trim().to_string().clone())
    });

    if let Ok(result) = line_handle.join() {
        Ok(result?)
    } else {
        Err(anyhow::anyhow!("read file thread panicked"))
    }
}

fn is_valid_target_text(text: &str) -> bool {
    let mut out = true;
    out = out && text.len() > 3;
    out
}
