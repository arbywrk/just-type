use iced::{
    Alignment, Element, Font, Subscription,
    keyboard::{self, Key, key::Named},
    widget::{button, center, column, row, text},
};

use crate::{
    style::{BODY_SIZE, MUTED, SMALL_SIZE},
    widget::focus_ring,
};

use just_type_core::score::{Records, Score};

const STAT_SIZE: f32 = 56.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selected {
    Restart,
    Menu,
}

/// The selection after pressing Tab, or Shift+Tab when `backwards`.
fn cycle(selected: Option<Selected>, backwards: bool) -> Selected {
    match (selected, backwards) {
        (None | Some(Selected::Menu), false) => Selected::Restart,
        (Some(Selected::Restart), false) => Selected::Menu,
        (None | Some(Selected::Restart), true) => Selected::Menu,
        (Some(Selected::Menu), true) => Selected::Restart,
    }
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
    BackToMenu,
    CycleButtonSelect { backwards: bool },
    ActivateSelected,
    Cancel,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Restart,
    BackToMenu,
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
            Message::BackToMenu => Some(Event::BackToMenu),
            Message::CycleButtonSelect { backwards } => {
                self.selected = Some(cycle(self.selected, backwards));
                None
            }
            // restart is the default action, like a form's submit button.
            Message::ActivateSelected => match self.selected {
                None | Some(Selected::Restart) => Some(Event::Restart),
                Some(Selected::Menu) => Some(Event::BackToMenu),
            },
            Message::Cancel => match self.selected.take() {
                Some(_) => None,
                None => Some(Event::BackToMenu),
            },
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
            .color(MUTED)
        });

        let restart = button(text("Restart").size(BODY_SIZE))
            .padding([10, 32])
            .on_press(Message::Restart);
        let menu = button(text("Menu").size(BODY_SIZE))
            .padding([10, 32])
            .style(button::secondary)
            .on_press(Message::BackToMenu);
        // rings reserve space around each button, so less spacing keeps the same gap.
        let buttons = row![
            focus_ring(restart, self.selected == Some(Selected::Restart)),
            focus_ring(menu, self.selected == Some(Selected::Menu)),
        ]
        .spacing(8);

        center(
            column![
                column![new_best, stats, test_info, history]
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

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(key_to_message)
    }
}

fn key_to_message(event: keyboard::Event) -> Option<Message> {
    let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
        return None;
    };

    match key {
        Key::Named(Named::Enter) => Some(Message::ActivateSelected),
        Key::Named(Named::Escape) => Some(Message::Cancel),
        Key::Named(Named::Tab) => Some(Message::CycleButtonSelect {
            backwards: modifiers.shift(),
        }),
        _ => None,
    }
}
