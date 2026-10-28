use iced::{
    Alignment, Element, Font, Subscription,
    keyboard::{self, Key, key::Named},
    widget::{button, center, column, row, text},
};

use crate::{
    score::Score,
    style::{BODY_SIZE, MUTED, SMALL_SIZE},
    widget::focus_ring,
};

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
    pub fn new(score: Score) -> Self {
        Self { score, selected: None }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::Restart => Some(Event::Restart),
            Message::BackToMenu => Some(Event::BackToMenu),
            Message::CycleButtonSelect { backwards } => {
                self.selected = Some(cycle(self.selected, backwards));
                None
            }
            // Restart is the default action, like a form's submit button.
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

        let restart = button(text("Restart").size(BODY_SIZE))
            .padding([10, 32])
            .on_press(Message::Restart);
        let menu = button(text("Menu").size(BODY_SIZE))
            .padding([10, 32])
            .style(button::secondary)
            .on_press(Message::BackToMenu);
        // The rings reserve space around each button, so less spacing keeps the same gap.
        let buttons = row![
            focus_ring(restart, self.selected == Some(Selected::Restart)),
            focus_ring(menu, self.selected == Some(Selected::Menu)),
        ]
        .spacing(8);

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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use iced::keyboard::Modifiers;
    use rstest::rstest;

    use super::*;
    use crate::screen::test_helpers as helpers;

    mod key_to_message {
        use super::*;

        #[rstest]
        #[case::enter(Named::Enter, Modifiers::empty(), Message::ActivateSelected)]
        #[case::escape(Named::Escape, Modifiers::empty(), Message::Cancel)]
        #[case::tab(Named::Tab, Modifiers::empty(), Message::CycleButtonSelect { backwards: false })]
        #[case::shift_tab(Named::Tab, Modifiers::SHIFT, Message::CycleButtonSelect { backwards: true })]
        fn maps_key_to_its_message(#[case] key: Named, #[case] modifiers: Modifiers, #[case] expected: Message) {
            let event = helpers::press(Key::Named(key), modifiers, None);

            assert_eq!(key_to_message(event), Some(expected));
        }

        #[rstest]
        fn ignores_other_keys() {
            assert_eq!(key_to_message(helpers::character("a", Modifiers::empty())), None);
        }
    }

    mod update {
        use super::*;

        const TAB: Message = Message::CycleButtonSelect { backwards: false };
        const SHIFT_TAB: Message = Message::CycleButtonSelect { backwards: true };

        fn state() -> State {
            State::new(Score {
                raw_wpm: 0.0,
                net_wpm: 0.0,
                accuracy: 100.0,
                duration: Duration::from_secs(15),
            })
        }

        #[rstest]
        #[case::restart_without_a_selection(vec![], Event::Restart)]
        #[case::tab_selects_restart(vec![TAB], Event::Restart)]
        #[case::second_tab_selects_the_menu(vec![TAB, TAB], Event::BackToMenu)]
        #[case::tab_wraps_back_to_restart(vec![TAB, TAB, TAB], Event::Restart)]
        #[case::shift_tab_selects_the_menu(vec![SHIFT_TAB], Event::BackToMenu)]
        fn enter_presses_the_selected_button(#[case] tabs: Vec<Message>, #[case] expected: Event) {
            let mut state = state();
            for tab in tabs {
                assert_eq!(state.update(tab), None);
            }

            assert_eq!(state.update(Message::ActivateSelected), Some(expected));
        }

        #[rstest]
        fn escape_clears_the_selection_before_leaving() {
            let mut state = state();
            state.update(TAB);
            state.update(TAB);

            assert_eq!(state.update(Message::Cancel), None);
            assert_eq!(state.update(Message::ActivateSelected), Some(Event::Restart));
            assert_eq!(state.update(Message::Cancel), Some(Event::BackToMenu));
        }
    }
}
