use iced::{
    Color, Element, Subscription, keyboard,
    widget::{column, radio, rich_text, row, span, text},
};
use rand::seq::IndexedRandom;
use serde::Deserialize;
use std::{sync::LazyLock, time::Instant};
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

/// `keystrokes` and `correct_keystrokes` count every key press, including ones later
/// erased with backspace; `correct_chars` counts only correct chars left in the text.
fn compute_score(
    keystrokes: usize,
    correct_keystrokes: usize,
    correct_chars: usize,
    elapsed: std::time::Duration,
) -> Score {
    let raw_wpm = wpm(keystrokes, elapsed);
    let net_wpm = wpm(correct_chars, elapsed);
    let accuracy = if keystrokes > 0 {
        correct_keystrokes as f64 / keystrokes as f64 * 100.0
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
    Tick,
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
    start_time: Option<Instant>,
    duration_choice: DurationChoice,
    menu: bool,
    game_over: bool,
    started: bool,
    text: Vec<char>,
    results: Vec<InputResult>,
    keystrokes: usize,
    correct_keystrokes: usize,
}

impl Game {
    fn boot() -> Self {
        Self {
            start_time: None,
            duration_choice: DurationChoice::default(),
            game_over: false,
            text: vec![],
            results: vec![],
            keystrokes: 0,
            correct_keystrokes: 0,
            started: false,
            menu: true,
        }
    }

    /// The full duration until the first keystroke, then counts down to zero.
    fn remaining_time(&self) -> Duration {
        match self.start_time {
            Some(start_time) => self.duration_choice.as_duration().saturating_sub(start_time.elapsed()),
            None => self.duration_choice.as_duration(),
        }
    }

    /// Resets the round state and generates a fresh text.
    fn new_round(&mut self) {
        self.start_time = None;
        self.game_over = false;
        self.started = false;
        self.text = vec![];
        self.results = vec![];
        self.keystrokes = 0;
        self.correct_keystrokes = 0;

        extend_text(&mut self.text, 200);
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                if self.remaining_time().is_zero() {
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
                if self.start_time.is_none() {
                    self.start_time = Some(Instant::now());
                }
                if let Some(expected) = self.text.get(self.results.len()) {
                    let correct = c == *expected;
                    self.keystrokes += 1;
                    if correct {
                        self.correct_keystrokes += 1;
                    }
                    self.results.push(correct.into());
                }
                extend_text(&mut self.text, self.results.len() + 200);
            }
            Message::Restart => {
                self.new_round();
            }
            Message::Start => {
                self.menu = false;
                self.new_round();
            }
            Message::UpdateTime(choice) => {
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
                self.keystrokes,
                self.correct_keystrokes,
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

        column![info, text(self.remaining_time().as_secs_f64().ceil().to_string()), rich_text(spans), button].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = if self.game_over || !self.started {
            Subscription::none()
        } else {
            iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
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
