use std::time::Duration;

use iced::{
    Alignment, Color, Element, Font, Length,
    widget::{button, center, column, radio, rich_text, row, space, span, text},
};

use crate::{
    game::{DurationChoice, InputResult, Message, Round},
    score::Score,
    words,
};

const FONT_SIZE: f32 = 28.0;
/// Max chars per line, trailing space included.
const LINE_CHARS: usize = 48;
const VISIBLE_LINES: usize = 3;
/// Monospace glyphs are roughly 0.6em wide; one extra glyph of slack avoids clipping.
const TEXT_WIDTH: f32 = (LINE_CHARS + 1) as f32 * FONT_SIZE * 0.6;
const MUTED: Color = Color::from_rgb(0.55, 0.55, 0.55);
const CORRECT: Color = Color::from_rgb(0.0, 0.8, 0.0);
const WRONG: Color = Color::from_rgb(0.9, 0.1, 0.1);

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
    let chars = round.text();
    let results = round.results();

    let visible_lines =
        words::visible_lines(chars, results.len(), LINE_CHARS, VISIBLE_LINES).map(|line| {
            let spans: Vec<text::Span<'_, ()>> = line
                .map(|i| {
                    let (color, ch) = match (results.get(i), chars[i]) {
                        (Some(InputResult::Correct), ch) => (CORRECT, ch),
                        (Some(InputResult::Wrong), ' ') => (WRONG, '_'),
                        (Some(InputResult::Wrong), ch) => (WRONG, ch),
                        (None, ch) => (MUTED, ch),
                    };
                    span(ch.to_owned()).color(color)
                })
                .collect();

            rich_text(spans)
                .font(Font::MONOSPACE)
                .size(FONT_SIZE)
                .wrapping(text::Wrapping::None)
                .into()
        });

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
        column![header, column(visible_lines).spacing(8)]
            .spacing(16)
            .width(Length::Fixed(TEXT_WIDTH)),
    )
    .padding(40)
    .into()
}
