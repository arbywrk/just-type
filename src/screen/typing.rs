use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use iced::{
    Alignment, Element, Font, Subscription,
    keyboard::{self, Key, key::Named},
    widget::{center, center_x, column, row, space, svg, text},
};

use crate::{
    score::{Score, compute_score},
    style::{CARET, CORRECT, MUTED, SMALL_SIZE, WRONG},
    typing_text::{self, TypingText},
    widget::icon_button,
    words,
};

const TEXT_BUFFER: usize = 200;
/// The typing text and timer.
const FONT_SIZE: f32 = 28.0;
const ICON_SIZE: f32 = FONT_SIZE;
/// Max width of the typing area. Lines wrap to fit it.
const TEXT_WIDTH: f32 = 820.0;
const VISIBLE_LINES: usize = 3;

/// Loaded once and reused, instead of on every redraw.
static RESTART_ICON: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/icons/restart.svg"
    )))
});

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Input(char),
    Backspace {
        word: bool,
    },
    RestartHovered(bool),
    ToggleRestartSelected,
    /// Presses the restart button if it is selected.
    ActivateSelected,
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
            Message::ActivateSelected => {
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

        // Hidden mid-round so it doesn't distract, unless hovered or selected.
        let restart_button = icon_button(
            RESTART_ICON.clone(),
            ICON_SIZE,
            !self.round.started(),
            self.restart_button.hovered || self.restart_button.selected,
            Message::Restart,
            Message::RestartHovered,
        );

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

        let keyboard = keyboard::listen().filter_map(key_to_message);

        Subscription::batch([tick, keyboard])
    }

    /// Starts a new round of the same length, keeping the button's hover since the
    /// cursor is still over it after a click.
    fn restart(&mut self) {
        self.round = Round::new(self.round.duration);
        self.restart_button.selected = false;
    }
}

fn key_to_message(event: keyboard::Event) -> Option<Message> {
    match event {
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
        } => Some(Message::ActivateSelected),
        // Shortcuts like zoom are handled by the app, not typed.
        keyboard::Event::KeyPressed { modifiers, .. } if modifiers.command() => None,
        keyboard::Event::KeyPressed { text: Some(t), .. } => {
            t.chars().next().filter(|c| !c.is_control()).map(Message::Input)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use iced::keyboard::{
        Location, Modifiers,
        key::{NativeCode, Physical},
    };

    use super::*;

    fn press(key: Key, modifiers: Modifiers, text: Option<&str>) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Unidentified(NativeCode::Unidentified),
            location: Location::Standard,
            modifiers,
            text: text.map(Into::into),
            repeat: false,
        }
    }

    fn character(c: &str, modifiers: Modifiers) -> keyboard::Event {
        press(Key::Character(c.into()), modifiers, Some(c))
    }

    #[test]
    fn letters_are_typed() {
        let message = key_to_message(character("a", Modifiers::empty()));
        assert!(matches!(message, Some(Message::Input('a'))));
    }

    #[test]
    fn backspace_deletes_a_char() {
        let message = key_to_message(press(Key::Named(Named::Backspace), Modifiers::empty(), None));
        assert!(matches!(message, Some(Message::Backspace { word: false })));
    }

    #[test]
    fn ctrl_backspace_deletes_a_word() {
        let message = key_to_message(press(Key::Named(Named::Backspace), Modifiers::CTRL, None));
        assert!(matches!(message, Some(Message::Backspace { word: true })));
    }

    #[test]
    fn tab_toggles_the_restart_selection() {
        let message = key_to_message(press(Key::Named(Named::Tab), Modifiers::empty(), Some("\t")));
        assert!(matches!(message, Some(Message::ToggleRestartSelected)));
    }

    #[test]
    fn enter_activates_the_selection() {
        let message = key_to_message(press(Key::Named(Named::Enter), Modifiers::empty(), Some("\r")));
        assert!(matches!(message, Some(Message::ActivateSelected)));
    }

    #[test]
    fn shortcuts_are_not_typed() {
        assert!(key_to_message(character("=", Modifiers::COMMAND)).is_none());
    }

    #[test]
    fn control_characters_are_not_typed() {
        let message = key_to_message(press(Key::Named(Named::Escape), Modifiers::empty(), Some("\u{1b}")));
        assert!(message.is_none());
    }
}
