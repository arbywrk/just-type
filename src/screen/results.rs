use iced::{
    Alignment, Element, Font,
    widget::{button, center, column, row, text},
};

use crate::{
    score::Score,
    style::{BODY_SIZE, MUTED, SMALL_SIZE},
};

const STAT_SIZE: f32 = 56.0;

pub struct State {
    score: Score,
}

#[derive(Debug, Clone)]
pub enum Message {
    Restart,
    BackToMenu,
}

pub enum Event {
    Restart,
    BackToMenu,
}

impl State {
    pub fn new(score: Score) -> Self {
        Self { score }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::Restart => Some(Event::Restart),
            Message::BackToMenu => Some(Event::BackToMenu),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let stat = |label, value: String| {
            column![
                text(label).size(SMALL_SIZE).color(MUTED),
                text(value).size(STAT_SIZE).font(Font::MONOSPACE),
            ]
            .spacing(4)
            .align_x(Alignment::Center)
        };
        let stats = row![
            stat("wpm", format!("{:.0}", self.score.net_wpm)),
            stat("raw", format!("{:.0}", self.score.raw_wpm)),
            stat("acc", format!("{:.0}%", self.score.accuracy)),
        ]
        .spacing(64);

        let test_info = text(format!("{}s test", self.score.duration.as_secs()))
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
                column![stats, test_info].spacing(16).align_x(Alignment::Center),
                buttons,
            ]
            .spacing(48)
            .align_x(Alignment::Center),
        )
        .padding(40)
        .into()
    }
}
