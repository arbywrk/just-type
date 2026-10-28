use std::time::Duration;

use iced::{
    Element, Subscription,
    keyboard::{self, Key},
};

use crate::screen::{Screen, menu, results, typing};

const ZOOM_STEP: f32 = 0.1;
const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 2.0;

#[derive(Debug, Clone)]
pub enum Message {
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Menu(menu::Message),
    Typing(typing::Message),
    Results(results::Message),
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

    /// The next longer choice, wrapping around to the shortest.
    pub fn next(self) -> Self {
        match self {
            DurationChoice::Secs15 => DurationChoice::Secs30,
            DurationChoice::Secs30 => DurationChoice::Secs60,
            DurationChoice::Secs60 => DurationChoice::Secs15,
        }
    }

    /// The next shorter choice, wrapping around to the longest.
    pub fn previous(self) -> Self {
        match self {
            DurationChoice::Secs15 => DurationChoice::Secs60,
            DurationChoice::Secs30 => DurationChoice::Secs15,
            DurationChoice::Secs60 => DurationChoice::Secs30,
        }
    }
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
            Message::ZoomIn => {
                self.settings.zoom = (self.settings.zoom + ZOOM_STEP).min(MAX_ZOOM);
            }
            Message::ZoomOut => {
                self.settings.zoom = (self.settings.zoom - ZOOM_STEP).max(MIN_ZOOM);
            }
            Message::ZoomReset => {
                self.settings.zoom = 1.0;
            }
            Message::Menu(message) => {
                if let Screen::Menu(state) = &mut self.screen
                    && let Some(event) = state.update(message)
                {
                    match event {
                        menu::Event::SelectDuration(choice) => self.settings.duration = choice,
                        menu::Event::Start => self.start_round(),
                    }
                }
            }
            Message::Typing(message) => {
                if let Screen::Typing(state) = &mut self.screen
                    && let Some(event) = state.update(message)
                {
                    match event {
                        typing::Event::Finished(score) => self.screen = Screen::Results(results::State::new(score)),
                        typing::Event::Exit => self.show_menu(),
                    }
                }
            }
            Message::Results(message) => {
                if let Screen::Results(state) = &mut self.screen
                    && let Some(event) = state.update(message)
                {
                    match event {
                        results::Event::Restart => self.start_round(),
                        results::Event::BackToMenu => self.show_menu(),
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Menu(state) => state.view().map(Message::Menu),
            Screen::Typing(state) => state.view().map(Message::Typing),
            Screen::Results(state) => state.view().map(Message::Results),
        }
    }

    pub fn scale_factor(&self) -> f32 {
        self.settings.zoom
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Zoom works on every screen, on top of whatever the screen listens to.
        let zoom = keyboard::listen().filter_map(|event| match event {
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
            _ => None,
        });

        let screen = match &self.screen {
            Screen::Menu(state) => state.subscription().map(Message::Menu),
            Screen::Typing(state) => state.subscription().map(Message::Typing),
            Screen::Results(state) => state.subscription().map(Message::Results),
        };

        Subscription::batch([zoom, screen])
    }

    fn show_menu(&mut self) {
        self.screen = Screen::Menu(menu::State::new(self.settings.duration));
    }

    fn start_round(&mut self) {
        self.screen = Screen::Typing(typing::State::new(self.settings.duration.as_duration()));
    }
}
