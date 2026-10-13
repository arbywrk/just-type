// I want to build a typeing game
//
// The game (MVP):
// It will show an infinit text (beautifully formatted for fast reading, mutted grey collor).
// Once the player starts typeing the text a countdown for 15 seconds will start.
// When a letter is written it will be colored with:
// - white if the input was correct
// - red if the input was incorrect (it will not show the actually typped letter at the moment)
//
// Once the timer is done the game is over and it will change to a stats screen,
// the stats screen will show the avrage WPM
// and will have a button for playing again.
use std::time::Duration;

use iced::widget::{column, rich_text, span, text};
use iced::{Color, Element, Subscription, keyboard};

const TEXT: &str = "The quick brown fox jumps over the lazy dog The quick brown fox jumps over the lazy dog The quick brown fox jumps over the lazy dog";

enum InputResult {
    Correct,
    Wrong,
}

impl From<bool> for InputResult {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Correct,
            false => Self::Wrong,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Input(char),
    Restart,
}

struct App {
    time: i64,
    game_over: bool,
    started: bool,
    text: Vec<char>,
    results: Vec<InputResult>,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.time -= 1;
                if self.time == 0 {
                    self.game_over = true;
                }
            }
            Message::Input(c) => {
                if self.game_over {
                    return;
                }
                self.started = true;
                if let Some(expected) = self.text.get(self.results.len()) {
                    self.results.push((c == *expected).into());
                }
            }
            Message::Restart => {
                self.time = 15;
                self.game_over = false;
                self.started = false;
                self.results = vec![];
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let spans: Vec<iced::widget::text::Span<'_, ()>> = self
            .text
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let color = match self.results.get(i) {
                    Some(InputResult::Correct) => Color::from_rgb(0.0, 0.8, 0.0),
                    Some(InputResult::Wrong) => Color::from_rgb(0.9, 0.1, 0.1),
                    None => Color::from_rgb(0.55, 0.55, 0.55),
                };
                span(c.to_string()).color(color)
            })
            .collect();

        let button: iced::Element<Message> = if self.game_over {
            iced::widget::button("Restart").on_press(Message::Restart).into()
        } else {
            iced::widget::space().into()
        };

        column![text(self.time.to_string()), rich_text(spans), button].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = if self.game_over || !self.started {
            Subscription::none()
        } else {
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        };

        let keyboard = if self.game_over {
            Subscription::none()
        } else {
            keyboard::listen().filter_map(|event| match event {
                keyboard::Event::KeyPressed { text: Some(t), .. } => {
                    t.chars().next().map(Message::Input)
                }
                _ => None,
            })
        };

        Subscription::batch([tick, keyboard])
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            time: 15,
            game_over: false,
            text: TEXT.to_owned().chars().collect(),
            results: vec![],
            started: false,
        }
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .run()
}
