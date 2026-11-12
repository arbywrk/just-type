use iced::{
    Alignment, Element,
    widget::{button, center, column, row, text},
};

use crate::{
    style::{self, BODY_SIZE, SMALL_SIZE},
    theme::Palette,
};

use just_type_core::score::{Records, Score};

const STAT_SIZE: f32 = 56.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selected {
    Restart,
}

pub struct State {
    score: Score,
    records: Option<Records>,
    /// Nothing is selected until Tab is pressed.
    selected: Option<Selected>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Restart,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Restart,
}

impl State {
    pub fn new(score: Score, records: Option<Records>) -> Self {
        Self {
            score,
            records,
            selected: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::Restart => Some(Event::Restart),
        }
    }

    pub fn view(&self, palette: Palette) -> Element<'_, Message> {
        let stat = |label, value: String| {
            column![
                text(label).size(SMALL_SIZE).color(palette.muted),
                text(value).size(STAT_SIZE).font(style::MONO),
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
            .color(palette.muted);
        let new_best = self
            .records
            .filter(|records| records.new_personal_best)
            .map(|_| text("new personal best").size(BODY_SIZE).style(text::primary));
        let history = self.records.map(|records| {
            text(format!(
                "best {:.0}  -  last {} avg {:.0}",
                records.personal_best, records.recent_count, records.recent_average
            ))
            .size(SMALL_SIZE)
            .color(palette.muted)
        });

        let restart = button(text("Restart").size(BODY_SIZE))
            .padding([10, 32])
            .on_press(Message::Restart);

        center(
            column![
                column![new_best, stats, test_info, history]
                    .spacing(16)
                    .align_x(Alignment::Center),
                restart,
            ]
            .spacing(48)
            .align_x(Alignment::Center),
        )
        .padding(40)
        .into()
    }
}
