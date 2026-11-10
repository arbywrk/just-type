//! The user's settings, saved as a single JSON file.
//!
//! The settings are grouped as follows:
//! - [`Gameplay`] -> settings that affect the gameplay
//! - [`Appearance`] -> visual settings.

use std::{fs, io, path::Path, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub gameplay: Gameplay,
    pub appearance: Appearance,
}

impl Settings {
    /// Reads the settings, falling back to the defaults when the file is missing or invalid.
    pub fn load(path: &Path) -> Self {
        let mut settings = match fs::read_to_string(path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_else(|error| {
                eprintln!("Ignoring invalid settings in {}: {error}", path.display());
                Self::default()
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(error) => {
                eprintln!("Couldn't read {}: {error}", path.display());
                Self::default()
            }
        };
        settings.normalize();
        settings
    }

    /// Writes a temporary file first and then replaces the old one, so a crash mid-write
    /// can't leave the settings half written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_string_pretty(self)?)?;
        fs::rename(temporary, path)
    }

    /// Brings every value back into its allowed range, e.g. after a hand edited file.
    pub fn normalize(&mut self) {
        self.appearance.zoom = self.appearance.zoom.clamp(Appearance::MIN_ZOOM, Appearance::MAX_ZOOM);
    }
}

/// Settings that change the rounds themselves.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Gameplay {
    pub duration: DurationChoice,
}

#[derive(Default, Clone, Debug, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum DurationChoice {
    #[default]
    #[serde(rename = "15s")]
    Secs15,
    #[serde(rename = "30s")]
    Secs30,
    #[serde(rename = "60s")]
    Secs60,
}

impl DurationChoice {
    pub fn as_duration(self) -> Duration {
        match self {
            DurationChoice::Secs15 => Duration::from_secs(15),
            DurationChoice::Secs30 => Duration::from_secs(30),
            DurationChoice::Secs60 => Duration::from_secs(60),
        }
    }

    /// The next longer choice, wrapping around to the shortest.
    pub fn next(self) -> Self {
        match self {
            DurationChoice::Secs15 => DurationChoice::Secs30,
            DurationChoice::Secs30 => DurationChoice::Secs60,
            DurationChoice::Secs60 => DurationChoice::Secs15,
        }
    }

    /// The next shorter choice, wrapping around to the longest.
    pub fn previous(self) -> Self {
        match self {
            DurationChoice::Secs15 => DurationChoice::Secs60,
            DurationChoice::Secs30 => DurationChoice::Secs15,
            DurationChoice::Secs60 => DurationChoice::Secs30,
        }
    }
}

/// Settings that only change how the app looks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    /// Scale factor applied to the whole interface, on top of the OS one.
    pub zoom: f32,
    /// Whether the caret glides between characters instead of jumping.
    pub smooth_caret: bool,
    /// How long the caret takes to glide to its next position. Lower is faster.
    pub caret_glide_ms: u64,
}

impl Appearance {
    pub const MIN_ZOOM: f32 = 0.5;
    pub const MAX_ZOOM: f32 = 2.0;

    /// How long the caret takes to glide, or `None` when it jumps.
    pub fn caret_glide(&self) -> Option<Duration> {
        self.smooth_caret.then(|| Duration::from_millis(self.caret_glide_ms))
    }
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            smooth_caret: true,
            caret_glide_ms: 100,
        }
    }
}
