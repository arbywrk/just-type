use std::time::Duration;

use iced::{
    Alignment, Color, Element, Font,
    widget::{button, center, column, radio, row, space, text},
};

use crate::{
    game::{DurationChoice, Message, Round},
    score::Score,
    typing_text::{self, TypingText},
};

const FONT_SIZE: f32 = 28.0;
/// Max width of the typing area. Lines wrap to fit it.
const TEXT_WIDTH: f32 = 820.0;
const VISIBLE_LINES: usize = 3;
const MUTED: Color = Color::from_rgb(0.55, 0.55, 0.55);
const CORRECT: Color = Color::from_rgb(0.0, 0.8, 0.0);
const WRONG: Color = Color::from_rgb(0.9, 0.1, 0.1);
const CARET: Color = Color::from_rgb(0.95, 0.75, 0.2);

pub fn menu(duration: DurationChoice) -> Element<'static, Message> {
    let duration_radio = |label, choice| {
        radio(label, choice, Some(duration), Message::SelectDuration)
            .size(18)
            .text_size(18)
    };
    let time_options = row![
        duration_radio("15s", DurationChoice::Secs15),
        duration_radio("30s", DurationChoice::Secs30),
        duration_radio("60s", DurationChoice::Secs60),
    ]
    .spacing(32);

    let start = button(text("Start").size(18))
        .padding([10, 40])
        .on_press(Message::Start);

    center(
        column![
            column![
                text("type-game").size(48).font(Font::MONOSPACE),
                text("Pick a duration, then start typing.")
                    .size(16)
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

pub fn results(score: Score, duration: DurationChoice) -> Element<'static, Message> {
    let stat = |label, value: String| {
        column![
            text(label).size(16).color(MUTED),
            text(value).size(56).font(Font::MONOSPACE),
        ]
        .spacing(4)
        .align_x(Alignment::Center)
    };
    let stats = row![
        stat("wpm", format!("{:.0}", score.net_wpm)),
        stat("raw", format!("{:.0}", score.raw_wpm)),
        stat("acc", format!("{:.0}%", score.accuracy)),
    ]
    .spacing(64);

    let test_info = text(format!("{}s test", duration.as_duration().as_secs()))
        .size(16)
        .color(MUTED);

    let buttons = row![
        button(text("Restart").size(18))
            .padding([10, 32])
            .on_press(Message::Restart),
        button(text("Menu").size(18))
            .padding([10, 32])
            .style(button::secondary)
            .on_press(Message::BackToMenu),
    ]
    .spacing(16);

    center(
        column![
            column![stats, test_info]
                .spacing(16)
                .align_x(Alignment::Center),
            buttons,
        ]
        .spacing(48)
        .align_x(Alignment::Center),
    )
    .padding(40)
    .into()
}

pub fn typing(round: &Round, remaining: Duration) -> Element<'_, Message> {
    let typing_text = TypingText::new(
        round.text(),
        round.results(),
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

    let hint: Element<'_, Message> = if round.started() {
        space().into()
    } else {
        text("start typing to begin").size(16).color(MUTED).into()
    };
    let timer = text(remaining.as_secs_f64().ceil().to_string())
        .size(FONT_SIZE)
        .font(Font::MONOSPACE)
        .style(text::primary);
    let header = row![hint, space::horizontal(), timer].align_y(Alignment::Center);

    center(
        column![header, typing_text]
            .spacing(16)
            .max_width(TEXT_WIDTH),
    )
    .padding(40)
    .into()
}
