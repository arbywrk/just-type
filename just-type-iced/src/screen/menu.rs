use iced::{
    Alignment, Element, Font, Subscription,
    keyboard::{self, Key, key::Named},
    widget::{button, center, column, radio, row, text},
};

use crate::{
    game::DurationChoice,
    style::{BODY_SIZE, MUTED, SMALL_SIZE},
};

const TITLE_SIZE: f32 = 48.0;

pub struct State {
    /// Copied from the settings, and reported back through `Event::SelectDuration`.
    duration: DurationChoice,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    SelectDuration(DurationChoice),
    PreviousDuration,
    NextDuration,
    Start,
}

pub enum Event {
    SelectDuration(DurationChoice),
    Start,
}

impl State {
    pub fn new(duration: DurationChoice) -> Self {
        Self { duration }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::SelectDuration(choice) => self.select(choice),
            Message::PreviousDuration => self.select(self.duration.previous()),
            Message::NextDuration => self.select(self.duration.next()),
            Message::Start => Some(Event::Start),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let duration_radio = |label, choice| {
            radio(label, choice, Some(self.duration), Message::SelectDuration)
                .size(BODY_SIZE)
                .text_size(BODY_SIZE)
        };
        let time_options = row![
            duration_radio("15s", DurationChoice::Secs15),
            duration_radio("30s", DurationChoice::Secs30),
            duration_radio("60s", DurationChoice::Secs60),
        ]
        .spacing(32);

        let start = button(text("Start").size(BODY_SIZE))
            .padding([10, 40])
            .on_press(Message::Start);

        center(
            column![
                column![
                    text("just-type").size(TITLE_SIZE).font(Font::MONOSPACE),
                    text("Set the time, then start typing.").size(SMALL_SIZE).color(MUTED),
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

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(key_to_message)
    }

    fn select(&mut self, duration: DurationChoice) -> Option<Event> {
        self.duration = duration;
        Some(Event::SelectDuration(duration))
    }
}

fn key_to_message(event: keyboard::Event) -> Option<Message> {
    let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
        return None;
    };

    match key {
        Key::Named(Named::Enter) => Some(Message::Start),
        Key::Named(Named::Tab) if modifiers.shift() => Some(Message::PreviousDuration),
        Key::Named(Named::Tab) => Some(Message::NextDuration),
        Key::Named(Named::ArrowLeft) => Some(Message::PreviousDuration),
        Key::Named(Named::ArrowRight) => Some(Message::NextDuration),
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
        #[case::enter(Named::Enter, Modifiers::empty(), Message::Start)]
        #[case::tab(Named::Tab, Modifiers::empty(), Message::NextDuration)]
        #[case::shift_tab(Named::Tab, Modifiers::SHIFT, Message::PreviousDuration)]
        #[case::left(Named::ArrowLeft, Modifiers::empty(), Message::PreviousDuration)]
        #[case::right(Named::ArrowRight, Modifiers::empty(), Message::NextDuration)]
        fn maps_key_to_its_message(#[case] key: Named, #[case] modifiers: Modifiers, #[case] expected: Message) {
            let event = helpers::press(Key::Named(key), modifiers, None);

            assert_eq!(key_to_message(event), Some(expected));
        }

        #[rstest]
        #[case::letter(helpers::character("a", Modifiers::empty()))]
        #[case::escape(helpers::press(Key::Named(Named::Escape), Modifiers::empty(), None))]
        fn ignores_key(#[case] event: keyboard::Event) {
            assert_eq!(key_to_message(event), None);
        }
    }

    mod update {
        use super::*;

        #[rstest]
        #[case::shorter(DurationChoice::Secs30, Message::PreviousDuration, DurationChoice::Secs15)]
        #[case::longer(DurationChoice::Secs30, Message::NextDuration, DurationChoice::Secs60)]
        #[case::wraps_to_the_longest(DurationChoice::Secs15, Message::PreviousDuration, DurationChoice::Secs60)]
        #[case::wraps_to_the_shortest(DurationChoice::Secs60, Message::NextDuration, DurationChoice::Secs15)]
        fn changes_the_duration(
            #[case] from: DurationChoice,
            #[case] message: Message,
            #[case] expected: DurationChoice,
        ) {
            let mut state = State::new(from);

            let event = state.update(message);

            assert_eq!(state.duration, expected);
            assert!(matches!(event, Some(Event::SelectDuration(d)) if d == expected));
        }
    }
}
