// TODO: fix default duration properly (It should be correctly set at boot, It should be implemented
// in such a way that when/if persistance is added it can remember user prefference, it should match
// the selected radio button).
// Also one radio button must be required to be set
use iced::{
    Color, Element, Subscription, keyboard,
    widget::{column, radio, rich_text, row, span, text},
};
use rand::seq::IndexedRandom;
use serde::Deserialize;
use std::sync::LazyLock;
use std::time::Duration;

#[derive(Deserialize)]
struct Language {
    words: Vec<String>,
}

static WORDS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let lang: Language = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/languages/english.json"
    )))
    .expect("failed to parse english.json");

    lang.words
});

#[derive(Eq, PartialEq)]
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

/// Appends random space-separated words until `text` has at least `min_len` chars.
fn extend_text(text: &mut Vec<char>, min_len: usize) {
    let mut rng = rand::rng();

    while text.len() < min_len {
        let word = WORDS.choose(&mut rng).expect("word list is empty");

        if !text.is_empty() {
            text.push(' ');
        }
        text.extend(word.chars());
    }
}

struct Score {
    raw_wpm: f64,
    net_wpm: f64,
    accuracy: f64,
}

fn wpm(chars: usize, elapsed: std::time::Duration) -> f64 {
    let minutes = elapsed.as_secs_f64() / 60.0;
    if minutes == 0.0 {
        return 0.0;
    }
    (chars as f64 / 5.0) / minutes
}

fn compute_score(total_typed: usize, correct_typed: usize, elapsed: std::time::Duration) -> Score {
    let raw_wpm = wpm(total_typed, elapsed);
    let net_wpm = wpm(correct_typed, elapsed);
    let accuracy = if total_typed > 0 {
        correct_typed as f64 / total_typed as f64 * 100.0
    } else {
        100.0
    };

    Score {
        raw_wpm,
        net_wpm,
        accuracy,
    }
}


#[derive(Debug, Clone)]
enum Message {
    Tick(std::time::Instant),
    Input(char),
    Restart,
    Start,
    UpdateTime(DurationChoice),
    Continue,
}

#[derive(Default, Clone, Debug, Copy, Eq, PartialEq)]
enum DurationChoice {
    #[default]
    Secs15,
    Secs30,
    Secs60,
}

impl DurationChoice {
    fn as_duration(self) -> Duration {
        match self {
            DurationChoice::Secs15 => Duration::from_secs(15),
            DurationChoice::Secs30 => Duration::from_secs(30),
            DurationChoice::Secs60 => Duration::from_secs(60),
        }
    }
}

struct Game {
    time: Duration,
    duration_choice: DurationChoice,
    menu: bool,
    game_over: bool,
    started: bool,
    text: Vec<char>,
    results: Vec<InputResult>,
}

impl Game {
    fn boot() -> Self {
        Self {
            time: DurationChoice::default().as_duration(),
            duration_choice: DurationChoice::default(),
            game_over: false,
            text: vec![],
            results: vec![],
            started: false,
            menu: true,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(time_elapsed) => {
                self.time = self.time.saturating_sub(time_elapsed.elapsed());
                if self.time.is_zero() {
                    self.game_over = true;
                }
            }
            Message::Input(c) => {
                if self.game_over {
                    return;
                }
                if c == 8 as char {
                    // Backspace
                    self.results.pop();
                    return;
                }
                self.started = true;
                if let Some(expected) = self.text.get(self.results.len()) {
                    self.results.push((c == *expected).into());
                }
                extend_text(&mut self.text, self.results.len() + 200);
            }
            Message::Restart => {
                self.time = self.duration_choice.as_duration();
                self.game_over = false;
                self.started = false;
                self.results = vec![];
            }
            Message::Start => {
                self.menu = false;

                self.text = vec![];
                self.results = vec![];

                extend_text(&mut self.text, 200);
            }
            Message::UpdateTime(choice) => {
                self.time = choice.as_duration();
                self.duration_choice = choice;
            }
            Message::Continue => {
                self.game_over = false;
                self.started = false;
                self.menu = true;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        if self.game_over && self.started {
            let score = compute_score(
                self.results.len(),
                self.results
                    .iter()
                    .map(|b| if *b == InputResult::Correct { 1 } else { 0 })
                    .sum(),
                self.duration_choice.as_duration(),
            );
            let raw: Element<Message> = text(format!("Raw: {:.2} wpm", score.raw_wpm)).into();
            let net: Element<Message> = text(format!("Net: {:.2} wpm", score.net_wpm)).into();
            let accuracy: Element<Message> =
                text(format!("Accuracy: {:.2} %", score.accuracy)).into();

            let button = iced::widget::button("Continue").on_press(Message::Continue);

            return column![row![raw, net, accuracy].spacing(10), button].into();
        }
        if self.menu {
            let time_option: Element<Message> = row![
                radio("15s", DurationChoice::Secs15, Some(self.duration_choice), Message::UpdateTime),
                radio("30s", DurationChoice::Secs30, Some(self.duration_choice), Message::UpdateTime),
                radio("60s", DurationChoice::Secs60, Some(self.duration_choice), Message::UpdateTime),
            ]
            .spacing(10)
            .into();

            let button = iced::widget::button("Start").on_press(Message::Start);

            return column![time_option, button].into();
        }

        let info: iced::Element<Message> = if !self.started {
            iced::widget::text("The game will start when you start typing.").into()
        } else {
            iced::widget::space().into()
        };

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
            iced::widget::button("Restart")
                .on_press(Message::Restart)
                .into()
        } else {
            iced::widget::space().into()
        };

        column![info, text(self.time.as_secs_f64().ceil().to_string()), rich_text(spans), button].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = if self.game_over || !self.started {
            Subscription::none()
        } else {
            iced::time::every(Duration::from_secs(1)).map(Message::Tick)
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

fn main() -> iced::Result {
    iced::application(Game::boot, Game::update, Game::view)
        .subscription(Game::subscription)
        .run()
}
