use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Default, Clone, Copy, Debug)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    /// Every theme, in the order they are listed.
    pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}
