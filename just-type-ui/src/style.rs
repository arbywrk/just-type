//! Fonts, colors and sizes shared by every screen.

use iced::{Color, Font};

/// The typing text, title and stats. Bundled so the text looks the same on every OS.
/// The no-ligatures variant, so typed characters never merge into one glyph.
pub const MONO: Font = Font::with_name("JetBrains Mono NL");
pub const MONO_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/fonts/JetBrainsMonoNL-Regular.ttf"
));

pub const MUTED: Color = Color::from_rgb(0.55, 0.55, 0.55);
pub const CORRECT: Color = Color::from_rgb(0.0, 0.8, 0.0);
pub const WRONG: Color = Color::from_rgb(0.9, 0.1, 0.1);
pub const CARET: Color = Color::from_rgb(0.95, 0.75, 0.2);

/// Hints and labels.
pub const SMALL_SIZE: f32 = 16.0;
/// Buttons and options.
pub const BODY_SIZE: f32 = 18.0;
