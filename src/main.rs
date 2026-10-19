use iced::{
    Alignment, Color, Element, Font, Length, Subscription, keyboard,
    widget::{button, center, column, radio, rich_text, row, space, span, text},
};
use rand::seq::IndexedRandom;
use serde::Deserialize;
use std::{ops::Range, sync::LazyLock, time::Instant};
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

const FONT_SIZE: f32 = 28.0;
/// Max chars per line, trailing space included.
const LINE_CHARS: usize = 48;
const VISIBLE_LINES: usize = 3;
/// Monospace glyphs are roughly 0.6em wide; one extra glyph of slack avoids clipping.
const TEXT_WIDTH: f32 = (LINE_CHARS + 1) as f32 * FONT_SIZE * 0.6;
const MUTED: Color = Color::from_rgb(0.55, 0.55, 0.55);

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

/// Greedily splits `text` into lines of at most `width` chars, breaking after spaces.
/// Earlier lines never change when words are appended, so lines don't jump while typing.
fn wrap_lines(text: &[char], width: usize) -> Vec<Range<usize>> {
    let mut lines = vec![];
    let mut line_start = 0;
    let mut word_start = 0;

    for i in 0..=text.len() {
        let word_end = if i == text.len() {
            i
        } else if text[i] == ' ' {
            i + 1
        } else {
            continue;
        };

        if word_end - line_start > width && word_start > line_start {
            lines.push(line_start..word_start);
            line_start = word_start;
        }
        word_start = word_end;
    }
    if line_start < text.len() {
        lines.push(line_start..text.len());
    }

    lines
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
            return self.view_results();
        }
        if self.menu {
            return self.view_menu();
        }
        self.view_typing()
    }

    fn view_menu(&self) -> Element<'_, Message> {
        let duration_radio = |label, choice| {
            radio(label, choice, Some(self.duration_choice), Message::UpdateTime)
                .size(18)
                .text_size(18)
        };
        let time_options = row![
            duration_radio("15s", DurationChoice::Secs15),
            duration_radio("30s", DurationChoice::Secs30),
            duration_radio("60s", DurationChoice::Secs60),
        ]
        .spacing(32);

        let start = button(text("Start").size(18))
            .padding([10, 40])
            .on_press(Message::Start);

        center(
            column![
                column![
                    text("type-game").size(48).font(Font::MONOSPACE),
                    text("Pick a duration, then start typing.").size(16).color(MUTED),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
                time_options,
                start,
            ]
            .spacing(40)
            .align_x(Alignment::Center),
        )
        .padding(40)
        .into()
    }

    fn view_results(&self) -> Element<'_, Message> {
        let score = compute_score(
            self.keystrokes,
            self.correct_keystrokes,
            self.results
                .iter()
                .map(|b| if *b == InputResult::Correct { 1 } else { 0 })
                .sum(),
            self.duration_choice.as_duration(),
        );

        let stat = |label, value: String| {
            column![
                text(label).size(16).color(MUTED),
                text(value).size(56).font(Font::MONOSPACE),
            ]
            .spacing(4)
            .align_x(Alignment::Center)
        };
        let stats = row![
            stat("wpm", format!("{:.0}", score.net_wpm)),
            stat("raw", format!("{:.0}", score.raw_wpm)),
            stat("acc", format!("{:.0}%", score.accuracy)),
        ]
        .spacing(64);

        let test_info = text(format!(
            "{}s test",
            self.duration_choice.as_duration().as_secs()
        ))
        .size(16)
        .color(MUTED);

        let buttons = row![
            button(text("Restart").size(18))
                .padding([10, 32])
                .on_press(Message::Restart),
            button(text("Menu").size(18))
                .padding([10, 32])
                .style(button::secondary)
                .on_press(Message::Continue),
        ]
        .spacing(16);

        center(
            column![column![stats, test_info].spacing(16).align_x(Alignment::Center), buttons]
                .spacing(48)
                .align_x(Alignment::Center),
        )
        .padding(40)
        .into()
    }

    fn view_typing(&self) -> Element<'_, Message> {
        let lines = wrap_lines(&self.text, LINE_CHARS);
        let cursor = self.results.len();
        let current_line = lines
            .iter()
            .position(|line| line.contains(&cursor))
            .unwrap_or(lines.len().saturating_sub(1));
        // Keep the current line in the middle once past the first, so there's always one line ahead.
        let first_visible = current_line.saturating_sub(1);

        let visible_lines = lines
            .into_iter()
            .skip(first_visible)
            .take(VISIBLE_LINES)
            .map(|line| {
                let spans: Vec<iced::widget::text::Span<'_, ()>> = line
                    .map(|i| {
                        let color = match self.results.get(i) {
                            Some(InputResult::Correct) => Color::from_rgb(0.0, 0.8, 0.0),
                            Some(InputResult::Wrong) => Color::from_rgb(0.9, 0.1, 0.1),
                            None => MUTED,
                        };
                        span(self.text[i].to_string()).color(color)
                    })
                    .collect();

                rich_text(spans)
                    .font(Font::MONOSPACE)
                    .size(FONT_SIZE)
                    .wrapping(iced::widget::text::Wrapping::None)
                    .into()
            });

        let hint: Element<'_, Message> = if self.started {
            space().into()
        } else {
            text("start typing to begin").size(16).color(MUTED).into()
        };
        let timer = text(self.remaining_time().as_secs_f64().ceil().to_string())
            .size(FONT_SIZE)
            .font(Font::MONOSPACE)
            .style(text::primary);
        let header = row![hint, space::horizontal(), timer].align_y(Alignment::Center);

        center(
            column![header, column(visible_lines).spacing(8)]
                .spacing(16)
                .width(Length::Fixed(TEXT_WIDTH)),
        )
        .padding(40)
        .into()
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
