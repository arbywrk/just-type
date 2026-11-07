//! UI specific settings.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    /// Scale factor applied to the whole UI, on top of the OS one.
    pub zoom: f32,
    /// Whether the caret glides between characters instead of jumping.
    pub smooth_caret: bool,
    /// How long the caret takes to glide to its next position. Lower is faster.
    pub caret_glide_ms: u64,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            smooth_caret: true,
            caret_glide_ms: 100,
        }
    }
}

impl UiSettings {
    /// How long the caret takes to glide, or `None` when it jumps.
    pub fn caret_glide(&self) -> Option<Duration> {
        self.smooth_caret.then(|| Duration::from_millis(self.caret_glide_ms))
    }
}
