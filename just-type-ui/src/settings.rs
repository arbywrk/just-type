//! Settings for both the game and the ui.
//!
//! `GameSettings` is held under the core library because it is ui agnostic.

pub mod ui;

use std::{fs, io, path::Path};

use just_type_core::settings::GameSettings;
use serde::{Deserialize, Serialize};

use ui::UiSettings;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub game: GameSettings,
    pub ui: UiSettings,
}

impl Settings {
    /// Reads the settings, falling back to the defaults when the file is missing or invalid.
    pub fn load(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_else(|error| {
                eprintln!("Ignoring invalid settings in {}: {error}", path.display());
                Self::default()
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(error) => {
                eprintln!("Couldn't read {}: {error}", path.display());
                Self::default()
            }
        }
    }

    /// Writes a temporary file first and then replaces the old one, so a crash mid-write
    /// can't leave the settings half written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_string_pretty(self)?)?;
        fs::rename(temporary, path)
    }
}
