use std::time::{Duration, Instant};

use iced::{
    Alignment, Border, Element, Font, Subscription, Theme,
    keyboard::{self, Key, key::Named},
    widget::{button, center, center_x, column, mouse_area, row, space, svg, text},
};

use crate::{
    score::{Score, compute_score},
    screen::{CARET, CORRECT, MUTED, SMALL_SIZE, WRONG},
    typing_text::{self, TypingText},
    words,
};

const TEXT_BUFFER: usize = 200;
/// The typing text and timer.
const FONT_SIZE: f32 = 28.0;
const ICON_SIZE: f32 = FONT_SIZE;
/// Space between an icon and its button border, which also enlarges the hitbox.
const ICON_PADDING: f32 = 8.0;
const CORNER_RADIUS: f32 = 8.0;
/// Max width of the typing area. Lines wrap to fit it.
const TEXT_WIDTH: f32 = 820.0;
const VISIBLE_LINES: usize = 3;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Input(char),
    Backspace { word: bool },
    RestartHovered(bool),
    ToggleRestartSelected,
    Confirm,
    Restart,
}

pub enum Event {
    Finished(Score),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InputResult {
    Correct,
    Wrong,
}

impl From<bool> for InputResult {
    fn from(correct: bool) -> Self {
        if correct { Self::Correct } else { Self::Wrong }
    }
}

struct Round {
    text: Vec<char>,
    results: Vec<InputResult>,
    start_time: Option<Instant>,
    /// Copied from the settings when the round starts, so changing them can't affect it.
    duration: Duration,
    keystrokes: usize,
    correct_keystrokes: usize,
}

impl Round {
    fn new(duration: Duration) -> Self {
        let mut text = vec![];
        words::extend_text(&mut text, TEXT_BUFFER);

        Self {
            text,
            results: vec![],
            start_time: None,
            duration,
            keystrokes: 0,
            correct_keystrokes: 0,
        }
    }

    fn text(&self) -> &[char] {
        &self.text
    }

    fn results(&self) -> &[InputResult] {
        &self.results
    }

    fn started(&self) -> bool {
        self.start_time.is_some()
    }

    fn type_char(&mut self, c: char) {
        self.start_time.get_or_insert_with(Instant::now);
        if let Some(&expected) = self.text.get(self.results.len()) {
            let correct = c == expected;
            self.keystrokes += 1;
            self.correct_keystrokes += usize::from(correct);
            self.results.push(correct.into());
        }
        words::extend_text(&mut self.text, self.results.len() + TEXT_BUFFER);
    }

    /// Deletes the last typed character.
    /// If `word` is true, it deletes everything back to the previous space.
    fn backspace(&mut self, word: bool) {
        self.results.pop();
        if word {
            while !self.results.is_empty() && self.text[self.results.len() - 1] != ' ' {
                self.results.pop();
            }
        }
    }

    /// The full duration until the first keystroke, then counts down to zero.
    fn remaining(&self) -> Duration {
        self.start_time
            .map_or(self.duration, |start| self.duration.saturating_sub(start.elapsed()))
    }

    fn score(&self) -> Score {
        let correct_chars = self.results.iter().filter(|&&r| r == InputResult::Correct).count();

        compute_score(self.keystrokes, self.correct_keystrokes, correct_chars, self.duration)
    }
}

/// Hover and keyboard selection of the restart button in the typing view.
#[derive(Default, Clone, Copy)]
struct RestartButton {
    hovered: bool,
    selected: bool,
}

pub struct State {
    round: Round,
    restart_button: RestartButton,
}

impl State {
    pub fn new(duration: Duration) -> Self {
        Self {
            round: Round::new(duration),
            restart_button: RestartButton::default(),
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::Tick => self
                .round
                .remaining()
                .is_zero()
                .then(|| Event::Finished(self.round.score())),
            Message::Input(c) => {
                self.round.type_char(c);
                self.restart_button.selected = false;
                None
            }
            Message::Backspace { word } => {
                self.round.backspace(word);
                None
            }
            Message::RestartHovered(hovered) => {
                self.restart_button.hovered = hovered;
                None
            }
            Message::ToggleRestartSelected => {
                self.restart_button.selected = !self.restart_button.selected;
                None
            }
            Message::Confirm => {
                if self.restart_button.selected {
                    self.restart();
                }
                None
            }
            Message::Restart => {
                self.restart();
                None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let typing_text = TypingText::new(
            self.round.text(),
            self.round.results(),
            typing_text::Style {
                pending: MUTED,
                correct: CORRECT,
                wrong: WRONG,
                caret: CARET,
            },
        )
        .font(Font::MONOSPACE)
        .size(FONT_SIZE)
        .line_spacing(8.0)
        .visible_lines(VISIBLE_LINES);

        let hint: Element<'_, Message> = if self.round.started() {
            space().into()
        } else {
            text("start typing to begin").size(SMALL_SIZE).color(MUTED).into()
        };
        let timer = text(self.round.remaining().as_secs_f64().ceil().to_string())
            .size(FONT_SIZE)
            .font(Font::MONOSPACE)
            .style(text::primary);
        let header = row![hint, space::horizontal(), timer].align_y(Alignment::Center);

        // Hidden mid-round so it doesn't distract, full contrast when hovered or selected.
        let highlighted = self.restart_button.hovered || self.restart_button.selected;
        let opacity = if self.round.started() && !highlighted { 0.0 } else { 1.0 };
        let restart_color = move |theme: &Theme| {
            if highlighted { theme.palette().text } else { MUTED }
        };

        let restart_icon = svg(svg::Handle::from_memory(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/restart.svg"
        ))))
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .opacity(opacity)
        .style(move |theme: &Theme, _| svg::Style {
            color: Some(restart_color(theme)),
        });
        let restart_button = mouse_area(
            button(restart_icon)
                .padding(ICON_PADDING)
                .style(move |theme: &Theme, _| {
                    let color = restart_color(theme).scale_alpha(opacity);
                    button::Style {
                        background: None,
                        text_color: color,
                        border: Border {
                            color,
                            width: 1.0,
                            radius: CORNER_RADIUS.into(),
                        },
                        ..button::Style::default()
                    }
                })
                .on_press(Message::Restart),
        )
        .on_enter(Message::RestartHovered(true))
        .on_exit(Message::RestartHovered(false));

        center(
            column![header, typing_text, center_x(restart_button)]
                .spacing(16)
                .max_width(TEXT_WIDTH),
        )
        .padding(40)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let tick = if self.round.started() {
            iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        };

        let keyboard = keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: Key::Named(Named::Backspace),
                modifiers,
                ..
            } => Some(Message::Backspace {
                word: modifiers.control(),
            }),
            keyboard::Event::KeyPressed {
                key: Key::Named(Named::Tab),
                ..
            } => Some(Message::ToggleRestartSelected),
            keyboard::Event::KeyPressed {
                key: Key::Named(Named::Enter),
                ..
            } => Some(Message::Confirm),
            // Shortcuts like zoom are handled by the app, not typed.
            keyboard::Event::KeyPressed { modifiers, .. } if modifiers.command() => None,
            keyboard::Event::KeyPressed { text: Some(t), .. } => {
                t.chars().next().filter(|c| !c.is_control()).map(Message::Input)
            }
            _ => None,
        });

        Subscription::batch([tick, keyboard])
    }

    /// Starts a new round of the same length, keeping the button's hover since the
    /// cursor is still over it after a click.
    fn restart(&mut self) {
        self.round = Round::new(self.round.duration);
        self.restart_button.selected = false;
    }
}
