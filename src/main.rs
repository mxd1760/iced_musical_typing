use anyhow::Error;
use iced::{
    Element, Subscription, Task, Theme, time, widget::{column, row, text, text_input}
};
use rand::Rng;
use std::{fs::File, io::{self, BufRead}, path::PathBuf, thread, time::{Duration, Instant}};

pub fn main() -> iced::Result {
    iced::application("Typing Game", TypingGame::update, TypingGame::view)
        .subscription(TypingGame::subscription)
        .theme(TypingGame::theme)
        .run()
}

struct TypingGame {
    input: String,
    target: String,
    score: usize,
    start_time: Option<Instant>,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    Tick(Instant),
    UpdateTarget(String),
}

impl Default for TypingGame {
    fn default() -> Self {
        Self::new()
    }
}

const MATCHING_COLOR:iced::Color = iced::Color::from_rgb(0.0, 0.5, 0.1);

impl TypingGame {
    fn new() -> Self {
        Self {
            input: String::new(),
            target: "This is your first typing challenge!".to_string(),
            score: 0,
            start_time: None,
        }
    }

    fn update(&mut self, message: Message) ->Task<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input = value;
                let num_matching = self
                    .input
                    .chars()
                    .zip(self.target.chars())
                    .take_while(|(a, b)| a == b)
                    .count();
                if num_matching >= self.target.len(){
                  self.score +=1;
                  return Task::perform(
                      async {
                          load_new_line(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")), rand::rng().random::<i32>() as usize)
                              .await
                              .unwrap_or_else(|_| "Failed to load line".to_string())
                      },
                      Message::UpdateTarget,
                  )
            Message::UpdateTarget(new_target) => {
                if is_valid_target_text(&new_target) {
                    self.target = new_target;
                    self.input = "".into();
                } else {
                    return Task::perform(
                        async {
                            load_new_line(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")), rand::rng().random::<i32>() as usize)
                                .await
                                .unwrap_or_else(|_| "Failed to load line".to_string())
                        },
                        Message::UpdateTarget,
                    );
                }
            }
                self.target = new_target;
                self.input = "".into();
              }else{
                return Task::perform(load_new_line(format!("{}/src/main.rs",env!("CARGO_MANIFEST_DIR")),rand::rng().random::<i32>() as usize),Message::UpdateTarget)
              }
            }
            }
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
        let matching_substr = &self.target[0..num_matching];
        let remaining_substr = &self.target[num_matching..self.target.len()];
        column![
            text("Type This:"),
            row![
                text(matching_substr).style(|_| text::Style { color: Some(MATCHING_COLOR) }),
                text(remaining_substr)
            ],
            text_input("Start typing...", &self.input).on_input(Message::InputChanged),
            text(format!("Score: {}", self.score)),
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(100)).map(Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::ALL[5].clone()
    }
}

async fn load_new_line(file_name: impl Into<PathBuf>,line_number:usize) -> anyhow::Result<String>{
    let file_path = file_name.into();
    let line_handle = thread::spawn(move || -> Result<String, io::Error> {
      let file = File::open(file_path)?;
      let reader = io::BufReader::new(file);
      let lines:Vec<String> = reader.lines().filter_map(Result::ok).collect();
      Ok(lines[line_number%lines.len()].clone())
    });
    
    if let Ok(result) = line_handle.join(){
      Ok(result?)
    }else{
      Err(anyhow::anyhow!("read file thread panicked"))
    }
}

fn is_valid_target_text(text:&str)->bool{
  let mut out = true;
  out = out && text.len()>3;
  out
}