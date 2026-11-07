//! Game settings.

use std::time::Duration;

use serde::{Deserialize, Serialize};

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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSettings {
    pub duration: DurationChoice,
}
