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

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Input(char),
    Backspace,
    Start,
    Restart,
    SelectDuration(DurationChoice),
    BackToMenu,
    KeyboardModifiersChanged { ctrl: bool },
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

enum Screen {
    Menu,
    Typing,
    Results,
}

pub struct Round {
    text: Vec<char>,
    results: Vec<InputResult>,
    start_time: Option<Instant>,
    keystrokes: usize,
    correct_keystrokes: usize,
}

impl Round {
    fn new() -> Self {
        let mut text = vec![];
        words::extend_text(&mut text, TEXT_BUFFER);

        Self {
            text,
            results: vec![],
            start_time: None,
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

    fn backspace(&mut self, ctrl: bool) {
        self.results.pop();
        if ctrl {
            while !self.results.is_empty() && self.text[self.results.len() - 1] != ' ' {
                self.results.pop();
            }
        }
    }

    /// The full duration until the first keystroke, then counts down to zero.
    fn remaining(&self, total: Duration) -> Duration {
        self.start_time
            .map_or(total, |start| total.saturating_sub(start.elapsed()))
    }

    fn score(&self, elapsed: Duration) -> Score {
        let correct_chars = self
            .results
            .iter()
            .filter(|&&r| r == InputResult::Correct)
            .count();

        compute_score(
            self.keystrokes,
            self.correct_keystrokes,
            correct_chars,
            elapsed,
        )
    }
}

pub struct Game {
    screen: Screen,
    duration_choice: DurationChoice,
    round: Round,
    ctrl_pressed: bool,
}

impl Game {
    pub fn boot() -> Self {
        Self {
            screen: Screen::Menu,
            duration_choice: DurationChoice::default(),
            round: Round::new(),
            ctrl_pressed: false,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                if self.remaining_time().is_zero() {
                    self.screen = Screen::Results;
                }
            }
            Message::Input(c) => {
                if let Screen::Typing = self.screen {
                    self.round.type_char(c);
                }
            }
            Message::Backspace => {
                if let Screen::Typing = self.screen {
                    self.round.backspace(self.ctrl_pressed);
                }
            }
            Message::KeyboardModifiersChanged { ctrl } => {
                self.ctrl_pressed = ctrl;
            }
            Message::Start | Message::Restart => {
                self.round = Round::new();
                self.screen = Screen::Typing;
            }
            Message::SelectDuration(choice) => {
                self.duration_choice = choice;
            }
            Message::BackToMenu => {
                self.screen = Screen::Menu;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Menu => view::menu(self.duration_choice),
            Screen::Typing => view::typing(&self.round, self.remaining_time()),
            Screen::Results => view::results(
                self.round.score(self.duration_choice.as_duration()),
                self.duration_choice,
            ),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let Screen::Typing = self.screen else {
            return Subscription::none();
        };

        let tick = if self.round.started() {
            iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        };

        let keyboard = keyboard::listen().filter_map(|event| match event {
            keyboard::Event::ModifiersChanged(modifiers) => {
                Some(Message::KeyboardModifiersChanged {
                    ctrl: modifiers.control()
                })
            },
            keyboard::Event::KeyPressed {
                key: Key::Named(Named::Backspace),
                ..
            } => Some(Message::Backspace),
            keyboard::Event::KeyPressed { text: Some(t), .. } => t
                .chars()
                .next()
                .filter(|c| !c.is_control())
                .map(Message::Input),
            _ => None,
        });

        Subscription::batch([tick, keyboard])
    }

    fn remaining_time(&self) -> Duration {
        self.round.remaining(self.duration_choice.as_duration())
    }
}
