use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

use iced::{
    Element, Subscription,
    keyboard::{self, Key},
};
use serde::{Deserialize, Serialize};

use crate::{
    database::Database,
    paths::Paths,
    score::{Records, Score},
    screen::{Screen, menu, results, typing},
    settings::Settings,
};

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

#[derive(Default, Clone, Debug, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum DurationChoice {
    #[default]
    #[serde(rename = "15s")]
    Secs15,
    #[serde(rename = "30s")]
    Secs30,
    #[serde(rename = "60s")]
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

pub struct Game {
    screen: Screen,
    settings: Settings,
    settings_path: Option<PathBuf>,
    /// `None` when the history couldn't be opened.
    /// Rounds still work without records.
    database: Option<Database>,
}

impl Game {
    pub fn new() -> Self {
        let paths = Paths::new();
        let mut settings = paths.settings.as_deref().map(Settings::load).unwrap_or_default();
        settings.zoom = settings.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        let database = paths.database.as_deref().and_then(|path| {
            Database::open(path)
                .inspect_err(|error| eprintln!("Couldn't open the history at {}: {error}", path.display()))
                .ok()
        });

        Self {
            screen: Screen::Menu(menu::State::new(settings.duration)),
            settings,
            settings_path: paths.settings,
            database,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ZoomIn => self.set_zoom(self.settings.zoom + ZOOM_STEP),
            Message::ZoomOut => self.set_zoom(self.settings.zoom - ZOOM_STEP),
            Message::ZoomReset => self.set_zoom(1.0),
            Message::Menu(message) => {
                if let Screen::Menu(state) = &mut self.screen
                    && let Some(event) = state.update(message)
                {
                    match event {
                        menu::Event::SelectDuration(choice) => {
                            self.settings.duration = choice;
                            self.save_settings();
                        }
                        menu::Event::Start => self.start_round(),
                    }
                }
            }
            Message::Typing(message) => {
                if let Screen::Typing(state) = &mut self.screen
                    && let Some(event) = state.update(message)
                {
                    match event {
                        typing::Event::Finished(score) => {
                            let records = self.record(&score);
                            self.screen = Screen::Results(results::State::new(score, records));
                        }
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
        // zoom works on every screen, on top of whatever the screen listens to.
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

    fn set_zoom(&mut self, zoom: f32) {
        self.settings.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        self.save_settings();
    }

    fn save_settings(&self) {
        if let Some(path) = &self.settings_path
            && let Err(error) = self.settings.save(path)
        {
            eprintln!("Couldn't save the settings to {}: {error}", path.display());
        }
    }

    /// Saves the round to the history. `None` when there is no history to compare with.
    fn record(&mut self, score: &Score) -> Option<Records> {
        self.database
            .as_mut()?
            .record(score, SystemTime::now())
            .inspect_err(|error| eprintln!("Couldn't save the result: {error}"))
            .ok()
    }

    fn show_menu(&mut self) {
        self.screen = Screen::Menu(menu::State::new(self.settings.duration));
    }

    fn start_round(&mut self) {
        self.screen = Screen::Typing(typing::State::new(self.settings.duration.as_duration()));
    }
}
