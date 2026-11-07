use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use iced::{
    Alignment, Animation, Element, Font, Subscription,
    animation::Easing,
    keyboard::{self, Key, key::Named},
    widget::{center, center_x, column, container, row, space, stack, svg, text},
};

use crate::{
    style::{CARET, CORRECT, MUTED, SMALL_SIZE, WRONG},
    typing_text::{self, TypingText},
    widget::icon_button,
};

use just_type_core::score::{Round, Score};

/// The typing text and timer.
const FONT_SIZE: f32 = 28.0;
const ICON_SIZE: f32 = FONT_SIZE;
/// Smaller than the restart button, since leaving is a secondary action.
const EXIT_ICON_SIZE: f32 = ICON_SIZE * 0.75;
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
        "/resources/icons/view-refresh-symbolic.svg"
    )))
});
static EXIT_ICON: LazyLock<svg::Handle> = LazyLock::new(|| {
    svg::Handle::from_memory(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/icons/go-previous-symbolic.svg"
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
    ExitHovered(bool),
    Exit,
    /// Clears the restart button's selection, or exits when nothing is selected.
    Cancel,
}

pub enum Event {
    Finished(Score),
    Exit,
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
    controls: Animation<bool>,
    restart_button: ButtonState,
    /// The exit button can't be selected with Tab, so it is only ever hovered.
    exit_button: ButtonState,
    /// How long the caret takes to glide, or `None` when it jumps.
    caret_glide: Option<Duration>,
}

impl State {
    pub fn new(duration: Duration, caret_glide: Option<Duration>) -> Self {
        Self {
            round: Round::new(duration),
            controls: Animation::new(true).duration(CONTROLS_FADE).easing(Easing::EaseOut),
            restart_button: ButtonState::new(),
            exit_button: ButtonState::new(),
            caret_glide,
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
                self.controls.go_mut(false, now);
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
            Message::ExitHovered(hovered) => {
                self.exit_button.set_hovered(hovered, now);
                None
            }
            Message::Exit => Some(Event::Exit),
            Message::Cancel => {
                if self.restart_button.selected {
                    self.restart_button.set_selected(false, now);
                    None
                } else {
                    Some(Event::Exit)
                }
            }
        }
    }

    pub fn view(&self, now: Instant) -> Element<'_, Message> {
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
        .visible_lines(VISIBLE_LINES)
        .glide(self.caret_glide);

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

        // Hidden mid-round so they don't distract, unless hovered or selected.
        let controls = self.controls.interpolate(0.0, 1.0, now);
        let restart_button = icon_button(
            RESTART_ICON.clone(),
            ICON_SIZE,
            controls,
            self.restart_button.highlight(now),
            Message::Restart,
            Message::RestartHovered,
        );
        let exit_button = icon_button(
            EXIT_ICON.clone(),
            EXIT_ICON_SIZE,
            controls,
            self.exit_button.highlight(now),
            Message::Exit,
            Message::ExitHovered,
        );

        let content = center(
            column![header, typing_text, center_x(restart_button)]
                .spacing(16)
                .max_width(TEXT_WIDTH),
        )
        .padding(40);

        // Layered over the content so it sits in the window's corner without moving the text.
        stack![content, container(exit_button).padding(CORNER_PADDING)].into()
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
            &self.controls,
            &self.restart_button.highlight,
            &self.exit_button.highlight,
        ]
        .iter()
        .any(|animation| animation.is_animating(now))
    }

    /// Starts a new round of the same length, keeping the button's hover since the
    /// cursor is still over it after a click.
    fn restart(&mut self, now: Instant) {
        self.round = Round::new(self.round.duration);
        self.controls.go_mut(true, now);
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
        keyboard::Event::KeyPressed {
            key: Key::Named(Named::Escape),
            ..
        } => Some(Message::Cancel),
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
    use iced::keyboard::Modifiers;
    use rstest::rstest;

    use super::*;
    use crate::screen::test_helpers as helpers;

    mod key_to_message {
        use super::*;

        #[rstest]
        #[case::letter(helpers::character("a", Modifiers::empty()), Message::Input('a'))]
        #[case::backspace(
            helpers::press(Key::Named(Named::Backspace), Modifiers::empty(), None),
            Message::Backspace { word: false },
        )]
        #[case::ctrl_backspace(
            helpers::press(Key::Named(Named::Backspace), Modifiers::CTRL, None),
            Message::Backspace { word: true },
        )]
        #[case::tab(
            helpers::press(Key::Named(Named::Tab), Modifiers::empty(), Some("\t")),
            Message::ToggleRestartSelected
        )]
        #[case::enter(
            helpers::press(Key::Named(Named::Enter), Modifiers::empty(), Some("\r")),
            Message::ActivateSelected
        )]
        #[case::escape(
            helpers::press(Key::Named(Named::Escape), Modifiers::empty(), Some("\u{1b}")),
            Message::Cancel
        )]
        fn maps_key_to_its_message(#[case] event: keyboard::Event, #[case] expected: Message) {
            assert_eq!(key_to_message(event), Some(expected));
        }

        #[rstest]
        #[case::shortcut(helpers::character("=", Modifiers::COMMAND))]
        #[case::control_character(helpers::press(Key::Named(Named::Delete), Modifiers::empty(), Some("\u{7f}")))]
        fn ignores_key(#[case] event: keyboard::Event) {
            assert_eq!(key_to_message(event), None);
        }
    }

    mod update {
        use super::*;

        #[rstest]
        fn escape_clears_the_selection_before_exiting() {
            let mut state = State::new(Duration::from_secs(15), None);
            state.update(Message::ToggleRestartSelected);

            assert!(state.update(Message::Cancel).is_none());
            assert!(!state.restart_button.selected);
            assert!(matches!(state.update(Message::Cancel), Some(Event::Exit)));
        }

        #[rstest]
        fn typing_hides_the_controls_and_restarting_shows_them() {
            let mut state = State::new(Duration::from_secs(15), None);

            state.update(Message::Input('a'));
            assert!(!state.controls.value());

            state.update(Message::Restart);
            assert!(state.controls.value());
        }

        #[rstest]
        fn controls_fade_instead_of_vanishing() {
            let mut state = State::new(Duration::from_secs(15), None);
            let start = Instant::now();

            state.update(Message::Input('a'));

            assert!(state.is_animating(start));
            assert!(!state.is_animating(start + CONTROLS_FADE * 2));
            assert_eq!(state.controls.interpolate(0.0, 1.0, start + CONTROLS_FADE * 2), 0.0);
        }
    }
}
