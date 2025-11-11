use iced::{
    Element, Subscription, Theme, time,
    widget::{column, text, text_input},
};
use std::time::{Duration, Instant};

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
}

impl Default for TypingGame {
    fn default() -> Self {
        Self::new()
    }
}

impl TypingGame {
    fn new() -> Self {
        Self {
            input: String::new(),
            target: "This is your first typing challenge!".to_string(),
            score: 0,
            start_time: None,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(value) => {
                self.input = value;
                self.score = self
                    .input
                    .chars()
                    .zip(self.target.chars())
                    .take_while(|(a, b)| a == b)
                    .count();
            }
            Message::Tick(now) => {
                if self.start_time.is_none() {
                    self.start_time = Some(now);
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            text(format!("Type this: {}", self.target)),
            text_input("Start typing...", &self.input).on_input(Message::InputChanged),
            text(format!("Score: {}", self.score)),
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(100)).map(Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::ALL[4].clone()
    }
}
