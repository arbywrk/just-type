use std::time::{Duration, Instant, SystemTime};

use iced::{
    Animation, Element, Length, Subscription, Theme,
    animation::Easing,
    keyboard::{self, Key},
    widget::{container, space, stack},
    window,
};

use crate::screen::{Screen, menu, results, typing};

use just_type_core::{paths::GamePaths, session::Session, settings::Appearance};

const ZOOM_STEP: f32 = 0.1;
/// How long the menu and results take to fade in.
const SCREEN_FADE: Duration = Duration::from_millis(200);

#[derive(Debug, Clone)]
pub enum Message {
    ZoomIn,
    ZoomOut,
    ZoomReset,
    /// A frame is about to be drawn while something is animating.
    Frame(Instant),
    Menu(menu::Message),
    Typing(typing::Message),
    Results(results::Message),
}

pub struct Game {
    screen: Screen,
    session: Session,
    /// When the current frame is drawn. Only kept up to date while something is animating.
    now: Instant,
    /// The current screen fading in. Done when `true`.
    screen_fade: Animation<bool>,
}

impl Game {
    pub fn new() -> Self {
        let session = Session::open(GamePaths::locate_or_create());

        Self {
            screen: Screen::Menu(menu::State::new(session.settings().gameplay.duration)),
            session,
            now: Instant::now(),
            screen_fade: Animation::new(true),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ZoomIn => self
                .session
                .change_settings(|settings| settings.appearance.zoom += ZOOM_STEP),
            Message::ZoomOut => self
                .session
                .change_settings(|settings| settings.appearance.zoom -= ZOOM_STEP),
            Message::ZoomReset => self
                .session
                .change_settings(|settings| settings.appearance.zoom = Appearance::default().zoom),
            Message::Frame(now) => self.now = now,
            Message::Menu(message) => {
                if let Screen::Menu(state) = &mut self.screen
                    && let Some(event) = state.update(message)
                {
                    match event {
                        menu::Event::SelectDuration(choice) => {
                            self.session
                                .change_settings(|settings| settings.gameplay.duration = choice);
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
                            let records = self.session.record(&score, SystemTime::now());
                            self.show(Screen::Results(results::State::new(score, records)));
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
        let screen = match &self.screen {
            Screen::Menu(state) => state.view().map(Message::Menu),
            Screen::Typing(state) => state.view(self.now).map(Message::Typing),
            Screen::Results(state) => state.view().map(Message::Results),
        };

        // The screen fades in from the background. The cover is always there, even when
        // fully transparent, so the screen's widgets keep their state when the fade ends.
        let cover = self.screen_fade.interpolate(1.0, 0.0, self.now);
        let cover = container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |theme: &Theme| container::background(theme.palette().background.scale_alpha(cover)));

        stack![screen, cover].into()
    }

    pub fn scale_factor(&self) -> f32 {
        self.session.settings().appearance.zoom
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

        // Frames are only needed while something moves, so the app is idle otherwise.
        let frames = if self.is_animating() {
            window::frames().map(Message::Frame)
        } else {
            Subscription::none()
        };

        Subscription::batch([zoom, screen, frames])
    }

    fn is_animating(&self) -> bool {
        self.screen_fade.is_animating(self.now)
            || matches!(&self.screen, Screen::Typing(state) if state.is_animating(self.now))
    }

    /// Switches to `screen`. The menu and results fade in, while a round shows up at
    /// once so typing can start right away.
    fn show(&mut self, screen: Screen) {
        self.screen_fade = if matches!(screen, Screen::Typing(_)) {
            Animation::new(true)
        } else {
            Animation::new(false)
                .duration(SCREEN_FADE)
                .easing(Easing::EaseOut)
                .go(true, Instant::now())
        };
        self.screen = screen;
    }

    fn show_menu(&mut self) {
        self.show(Screen::Menu(menu::State::new(
            self.session.settings().gameplay.duration,
        )));
    }

    fn start_round(&mut self) {
        let settings = self.session.settings();
        self.show(Screen::Typing(Box::new(typing::State::new(
            settings.gameplay.duration.as_duration(),
            settings.appearance.caret_glide(),
        ))));
    }
}
