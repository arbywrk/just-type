use std::time::Duration;

use iced::{
    Alignment, Border, Color, Element, Font, Theme,
    widget::{button, center, center_x, column, mouse_area, radio, row, space, svg, text},
};

use crate::{
    game::{DurationChoice, Message, RestartButton, Round},
    score::Score,
    typing_text::{self, TypingText},
};

/// Hints and labels.
const SMALL_SIZE: f32 = 16.0;
/// Buttons and options.
const BODY_SIZE: f32 = 18.0;
/// The typing text and timer.
const FONT_SIZE: f32 = 28.0;
const TITLE_SIZE: f32 = 48.0;
const STAT_SIZE: f32 = 56.0;
const ICON_SIZE: f32 = FONT_SIZE;
/// Space between an icon and its button border, which also enlarges the hitbox.
const ICON_PADDING: f32 = 8.0;
const CORNER_RADIUS: f32 = 8.0;
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

pub fn results(score: Score, duration: DurationChoice) -> Element<'static, Message> {
    let stat = |label, value: String| {
        column![
            text(label).size(SMALL_SIZE).color(MUTED),
            text(value).size(STAT_SIZE).font(Font::MONOSPACE),
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
        .size(SMALL_SIZE)
        .color(MUTED);

    let buttons = row![
        button(text("Restart").size(BODY_SIZE))
            .padding([10, 32])
            .on_press(Message::Restart),
        button(text("Menu").size(BODY_SIZE))
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

pub fn typing(round: &Round, remaining: Duration, restart: RestartButton) -> Element<'_, Message> {
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
        text("start typing to begin")
            .size(SMALL_SIZE)
            .color(MUTED)
            .into()
    };
    let timer = text(remaining.as_secs_f64().ceil().to_string())
        .size(FONT_SIZE)
        .font(Font::MONOSPACE)
        .style(text::primary);
    let header = row![hint, space::horizontal(), timer].align_y(Alignment::Center);

    // Hidden mid-round so it doesn't distract, full contrast when hovered or selected.
    let highlighted = restart.hovered || restart.selected;
    let opacity = if round.started() && !highlighted {
        0.0
    } else {
        1.0
    };
    let restart_color = move |theme: &Theme| {
        if highlighted {
            theme.palette().text
        } else {
            MUTED
        }
    };

    let restart_icon = svg(svg::Handle::from_memory(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/icons/restart.svg"
    ))))
    .width(ICON_SIZE)
    .height(ICON_SIZE)
    .opacity(opacity)
    .style(move |theme: &Theme, _| svg::Style {
        color: Some(restart_color(theme)),
    });
    let restart_button = mouse_area(
        button(restart_icon)
            .padding(ICON_PADDING)
            .style(move |theme: &Theme, _| {
                let color = restart_color(theme).scale_alpha(opacity);
                button::Style {
                    background: None,
                    text_color: color,
                    border: Border {
                        color,
                        width: 1.0,
                        radius: CORNER_RADIUS.into(),
                    },
                    ..button::Style::default()
                }
            })
            .on_press(Message::Restart),
    )
    .on_enter(Message::RestartHovered(true))
    .on_exit(Message::RestartHovered(false));

    center(
        column![header, typing_text, center_x(restart_button)]
            .spacing(16)
            .max_width(TEXT_WIDTH),
    )
    .padding(40)
    .into()
}
