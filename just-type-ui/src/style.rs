//! Fonts, colors and sizes shared by every screen.

use iced::Font;

/// The typing text, title and stats. Bundled so the text looks the same on every OS.
/// The no-ligatures variant, so typed characters never merge into one glyph.
pub const MONO: Font = Font::with_name("JetBrains Mono NL");
pub const MONO_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/fonts/JetBrainsMonoNL-Regular.ttf"
));

/// Hints and labels.
pub const SMALL_SIZE: f32 = 16.0;
/// Buttons and options.
pub const BODY_SIZE: f32 = 18.0;
