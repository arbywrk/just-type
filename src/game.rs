use std::time::{Duration, Instant};

use iced::{
    Element, Subscription,
    keyboard::{self, Key, key::Named},
};

use crate::{
    score::{Score, compute_score},
    view, words,
};

const TEXT_BUFFER: usize = 200;
const ZOOM_STEP: f32 = 0.1;
const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 2.0;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Input(char),
    Backspace { word: bool },
    Start,
    Restart,
    SelectDuration(DurationChoice),
    BackToMenu,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    RestartHovered(bool),
    ToggleRestartSelected,
    Confirm,
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

#[derive(Default, Clone, Debug, Copy, Eq, PartialEq)]
pub enum DurationChoice {
    #[default]
    Secs15,
    Secs30,
    Secs60,
}

impl DurationChoice {
    pub fn as_duration(self) -> Duration {
        match self {
            DurationChoice::Secs15 => Duration::from_secs(15),
            DurationChoice::Secs30 => Duration::from_secs(30),
            DurationChoice::Secs60 => Duration::from_secs(60),
        }
    }
}

#[derive(Default)]
enum Screen {
    #[default]
    Menu,
    Typing {
        round: Round,
        restart_button: RestartButton,
    },
    Results {
        score: Score,
    },
}

pub struct Round {
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

    pub fn text(&self) -> &[char] {
        &self.text
    }

    pub fn results(&self) -> &[InputResult] {
        &self.results
    }

    pub fn started(&self) -> bool {
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
pub struct RestartButton {
    pub hovered: bool,
    pub selected: bool,
}

struct Settings {
    duration: DurationChoice,
    /// Scale factor applied to the whole UI, on top of the OS one.
    zoom: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            duration: Default::default(),
            zoom: 1.0,
        }
    }
}

#[derive(Default)]
pub struct Game {
    screen: Screen,
    settings: Settings,
}

impl Game {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                if let Screen::Typing { round, .. } = &self.screen
                    && round.remaining().is_zero()
                {
                    self.screen = Screen::Results { score: round.score() };
                }
            }
            Message::Input(c) => {
                if let Screen::Typing { round, restart_button } = &mut self.screen {
                    round.type_char(c);
                    restart_button.selected = false;
                }
            }
            Message::Backspace { word } => {
                if let Screen::Typing { round, .. } = &mut self.screen {
                    round.backspace(word);
                }
            }
            Message::Start | Message::Restart => self.start_round(),
            Message::SelectDuration(choice) => {
                self.settings.duration = choice;
            }
            Message::BackToMenu => {
                self.screen = Screen::Menu;
            }
            Message::ZoomIn => {
                self.settings.zoom = (self.settings.zoom + ZOOM_STEP).min(MAX_ZOOM);
            }
            Message::ZoomOut => {
                self.settings.zoom = (self.settings.zoom - ZOOM_STEP).max(MIN_ZOOM);
            }
            Message::ZoomReset => {
                self.settings.zoom = 1.0;
            }
            Message::RestartHovered(hovered) => {
                if let Screen::Typing { restart_button, .. } = &mut self.screen {
                    restart_button.hovered = hovered;
                }
            }
            Message::ToggleRestartSelected => {
                if let Screen::Typing { restart_button, .. } = &mut self.screen {
                    restart_button.selected = !restart_button.selected;
                }
            }
            Message::Confirm => {
                if let Screen::Typing { restart_button, .. } = &self.screen
                    && restart_button.selected
                {
                    self.start_round();
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Menu => view::menu(self.settings.duration),
            Screen::Typing { round, restart_button } => view::typing(round, round.remaining(), *restart_button),
            Screen::Results { score } => view::results(*score),
        }
    }

    pub fn scale_factor(&self) -> f32 {
        self.settings.zoom
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Zoom works on every screen. Typing input is ignored by `update` outside a round.
        let tick = if let Screen::Typing { round, .. } = &self.screen
            && round.started()
        {
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
            keyboard::Event::KeyPressed {
                modified_key,
                modifiers,
                ..
            } if modifiers.command() => match modified_key.as_ref() {
                Key::Character("+" | "=") => Some(Message::ZoomIn),
                Key::Character("-") => Some(Message::ZoomOut),
                Key::Character("0") => Some(Message::ZoomReset),
                _ => None,
            },
            keyboard::Event::KeyPressed { text: Some(t), .. } => {
                t.chars().next().filter(|c| !c.is_control()).map(Message::Input)
            }
            _ => None,
        });

        Subscription::batch([tick, keyboard])
    }

    fn start_round(&mut self) {
        let duration = self.settings.duration.as_duration();
        if let Screen::Typing { round, restart_button } = &mut self.screen {
            *round = Round::new(duration);
            restart_button.selected = false;
        } else {
            self.screen = Screen::Typing {
                round: Round::new(duration),
                restart_button: RestartButton::default(),
            };
        }
    }
}
