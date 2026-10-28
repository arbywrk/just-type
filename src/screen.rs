//! Each screen owns its state and messages. It handles its own messages in `update`
//! and returns an `Event` when something has to happen outside of it, like
//! switching to another screen.

pub mod menu;
pub mod results;
pub mod typing;

use crate::game::DurationChoice;

pub enum Screen {
    Menu(menu::State),
    Typing(typing::State),
    Results(results::State),
}

impl Default for Screen {
    fn default() -> Self {
        Self::Menu(menu::State::new(DurationChoice::default()))
    }
}

#[cfg(test)]
pub mod test_helpers {
    use iced::keyboard::{
        self, Key, Location, Modifiers,
        key::{NativeCode, Physical},
    };

    pub fn press(key: Key, modifiers: Modifiers, text: Option<&str>) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Unidentified(NativeCode::Unidentified),
            location: Location::Standard,
            modifiers,
            text: text.map(Into::into),
            repeat: false,
        }
    }

    pub fn character(c: &str, modifiers: Modifiers) -> keyboard::Event {
        press(Key::Character(c.into()), modifiers, Some(c))
    }
}
