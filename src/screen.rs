//! Each screen owns its state and messages. It handles its own messages in `update`
//! and returns an `Event` when something has to happen outside of it, like
//! switching to another screen.

pub mod menu;
pub mod results;
pub mod typing;

use iced::Color;

pub const MUTED: Color = Color::from_rgb(0.55, 0.55, 0.55);
pub const CORRECT: Color = Color::from_rgb(0.0, 0.8, 0.0);
pub const WRONG: Color = Color::from_rgb(0.9, 0.1, 0.1);
pub const CARET: Color = Color::from_rgb(0.95, 0.75, 0.2);

/// Hints and labels.
pub const SMALL_SIZE: f32 = 16.0;
/// Buttons and options.
pub const BODY_SIZE: f32 = 18.0;

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
