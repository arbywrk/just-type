//! Each screen owns its state and messages. It handles its own messages in `update`
//! and returns an `Event` when something has to happen outside of it, like
//! switching to another screen.

pub mod menu;
pub mod results;
pub mod typing;

pub enum Screen {
    Menu(menu::State),
    Typing(typing::State),
    Results(results::State),
}

impl Default for Screen {
    fn default() -> Self {
        Self::Menu(menu::State)
    }
}
