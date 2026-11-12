use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use iced::{
    Alignment, Animation, Element,
    Length::Fill,
    Subscription,
    animation::Easing,
    keyboard::{self, Key, key::Named},
    widget::{center, center_x, column, container, opaque, radio, row, space, stack, svg, text},
};

use crate::{
    style::{self, BODY_SIZE},
    theme::Palette,
    typing_text::{self, TypingText},
    widget::icon_button,
};

use just_type_core::{
    score::{Round, Score},
    settings::DurationChoice,
};

/// The typing text and timer.
const FONT_SIZE: f32 = 28.0;
/// Same size as the text, so the icons' strokes are as thick as the font's.
const ICON_SIZE: f32 = FONT_SIZE;
/// Distance of the exit button from the window's corner.
const CORNER_PADDING: f32 = 16.0;
/// Max width of the typing area. Lines wrap to fit it.
const TEXT_WIDTH: f32 = 820.0;
const VISIBLE_LINES: usize = 3;
/// How long the buttons take to fade out when typing starts, and back in on restart.
const CONTROLS_FADE: Duration = Duration::from_millis(300);
/// How long a button takes to light up when hovered or selected.
const HIGHLIGHT_FADE: Duration = Duration::from_millis(150);

/// Loaded once and reused, instead of on every redraw.
static RESTART_ICON: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/icons/arrow-clockwise-bold.svg"
    )))
});

#[derive(Debug, Clone, PartialEq)]
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
    SelectDuration(DurationChoice),
}

pub enum Event {
    Finished(Score),
    SelectDuration(DurationChoice),
}

/// Hover and keyboard selection of a button, with its highlight fading in and out.
struct ButtonState {
    hovered: bool,
    selected: bool,
    highlight: Animation<bool>,
}

impl ButtonState {
    fn new() -> Self {
        Self {
            hovered: false,
            selected: false,
            highlight: Animation::new(false).duration(HIGHLIGHT_FADE).easing(Easing::EaseOut),
        }
    }

    fn set_hovered(&mut self, hovered: bool, now: Instant) {
        self.hovered = hovered;
        self.highlight.go_mut(self.hovered || self.selected, now);
    }

    fn set_selected(&mut self, selected: bool, now: Instant) {
        self.selected = selected;
        self.highlight.go_mut(self.hovered || self.selected, now);
    }

    /// From 0 when plain to 1 when highlighted.
    fn highlight(&self, now: Instant) -> f32 {
        self.highlight.interpolate(0.0, 1.0, now)
    }
}

pub struct State {
    round: Round,
    /// Shown before the round starts and faded out while typing.
    controls_visibility: Animation<bool>,
    duration_choice: DurationChoice,
    restart_button: ButtonState,
    /// The exit button can't be selected with Tab, so it is only ever hovered.
    exit_button: ButtonState,
    /// How long the caret takes to glide, or `None` when it jumps.
    caret_glide: Option<Duration>,
}

impl State {
    pub fn new(choice: DurationChoice, caret_glide: Option<Duration>) -> Self {
        Self {
            round: Round::new(choice.as_duration()),
            controls_visibility: Animation::new(true).duration(CONTROLS_FADE).easing(Easing::EaseOut),
            restart_button: ButtonState::new(),
            exit_button: ButtonState::new(),
            caret_glide,
            duration_choice: choice,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        let now = Instant::now();
        match message {
            Message::Tick => self
                .round
                .remaining()
                .is_zero()
                .then(|| Event::Finished(self.round.score())),
            Message::Input(c) => {
                self.round.type_char(c);
                self.controls_visibility.go_mut(false, now);
                self.restart_button.set_selected(false, now);
                None
            }
            Message::Backspace { word } => {
                self.round.backspace(word);
                None
            }
            Message::RestartHovered(hovered) => {
                self.restart_button.set_hovered(hovered, now);
                None
            }
            Message::ToggleRestartSelected => {
                self.restart_button.set_selected(!self.restart_button.selected, now);
                None
            }
            Message::ActivateSelected => {
                if self.restart_button.selected {
                    self.restart(now);
                }
                None
            }
            Message::Restart => {
                self.restart(now);
                None
            }
            Message::SelectDuration(choice) => {
                if self.round.started() {
                    return None;
                }

                self.select(choice, now)
            }
        }
    }

    fn select(&mut self, duration_choice: DurationChoice, now: Instant) -> Option<Event> {
        if duration_choice == self.duration_choice {
            return None;
        }
        self.duration_choice = duration_choice;
        self.restart(now);
        Some(Event::SelectDuration(duration_choice))
    }

    pub fn view(&self, palette: Palette, now: Instant) -> Element<'_, Message> {
        let typing_text = TypingText::new(
            self.round.text(),
            self.round.results(),
            typing_text::Style {
                pending: palette.muted,
                correct: palette.correct,
                wrong: palette.wrong,
                caret: palette.accent,
            },
        )
        .font(style::MONO)
        .size(FONT_SIZE)
        .line_spacing(8.0)
        .visible_lines(VISIBLE_LINES)
        .glide(self.caret_glide);

        let visibility = self.controls_visibility.interpolate(0.0, 1.0, now);

        let duration_radio = |label, choice| {
            stack![
                radio(label, choice, Some(self.duration_choice), Message::SelectDuration)
                    .size(BODY_SIZE)
                    .text_size(BODY_SIZE)
                    .style(move |theme: &iced::Theme, status| {
                        let style = radio::default(theme, status);
                        radio::Style {
                            background: style.background.scale_alpha(visibility),
                            dot_color: style.dot_color.scale_alpha(visibility),
                            border_color: style.border_color.scale_alpha(visibility),
                            text_color: Some(theme.palette().text.scale_alpha(visibility)),
                            ..style
                        }
                    }),
                self.round.started().then(|| opaque(space().width(Fill).height(Fill))),
            ]
        };
        let time_options = row![
            duration_radio("15s", DurationChoice::Secs15),
            duration_radio("30s", DurationChoice::Secs30),
            duration_radio("60s", DurationChoice::Secs60),
        ]
        .spacing(32);

        let top_bar = center_x(time_options);

        let timer = text(self.round.remaining().as_secs_f64().ceil().to_string())
            .size(FONT_SIZE)
            .font(style::MONO)
            .style(text::primary);
        let header = row![timer].align_y(Alignment::Center);

        // Hidden during the round so they don't distract, unless hovered or selected.
        let restart_button = icon_button(
            RESTART_ICON.clone(),
            ICON_SIZE,
            visibility,
            self.restart_button.highlight(now),
            Message::Restart,
            Message::RestartHovered,
            palette.muted,
        );

        let main = column![header, typing_text, center_x(restart_button)]
            .spacing(16)
            .max_width(TEXT_WIDTH);

        let content = stack![center(main), container(top_bar).center_x(Fill).padding(40)];

        content.into()
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

    /// Whether a button is still fading, so the app keeps drawing frames.
    pub fn is_animating(&self, now: Instant) -> bool {
        [
            &self.controls_visibility,
            &self.restart_button.highlight,
            &self.exit_button.highlight,
        ]
        .iter()
        .any(|animation| animation.is_animating(now))
    }

    /// Starts a new round with the same settings.
    fn restart(&mut self, now: Instant) {
        self.round = Round::new(self.duration_choice.as_duration());
        self.controls_visibility.go_mut(true, now);
        self.restart_button.set_selected(false, now);
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
        keyboard::Event::KeyPressed { modifiers, .. } if modifiers.command() => None,
        keyboard::Event::KeyPressed { text: Some(t), .. } => {
            t.chars().next().filter(|c| !c.is_control()).map(Message::Input)
        }
        _ => None,
    }
}
