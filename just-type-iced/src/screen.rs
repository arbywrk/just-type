pub mod menu;
pub mod results;
pub mod typing;

pub enum Screen {
    Menu(menu::State),
    // Only this state is put inside a pointer because
    // its size makes it resonable.
    Typing(Box<typing::State>),
    Results(results::State),
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
