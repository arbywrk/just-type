use iced::{
    Alignment, Element, Font,
    widget::{button, center, column, radio, row, text},
};

use crate::{
    game::DurationChoice,
    style::{BODY_SIZE, MUTED, SMALL_SIZE},
};

const TITLE_SIZE: f32 = 48.0;

pub struct State;

#[derive(Debug, Clone)]
pub enum Message {
    SelectDuration(DurationChoice),
    Start,
}

pub enum Event {
    SelectDuration(DurationChoice),
    Start,
}

impl State {
    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::SelectDuration(choice) => Some(Event::SelectDuration(choice)),
            Message::Start => Some(Event::Start),
        }
    }

    /// `duration` is the current setting, which the app owns.
    pub fn view(&self, duration: DurationChoice) -> Element<'_, Message> {
        let duration_radio = |label, choice| {
            radio(label, choice, Some(duration), Message::SelectDuration)
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
                    text("Pick a duration, then start typing.")
                        .size(SMALL_SIZE)
                        .color(MUTED),
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
}
