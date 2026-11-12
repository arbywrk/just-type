use iced::{Color, color};
use just_type_core::settings::theme::Theme;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub text: Color,
    pub accent: Color,
    pub correct: Color,
    pub wrong: Color,
    pub muted: Color,
}

/// The colors are picked per theme, because text needs a darker shade on a light
/// background and a lighter one on a dark background to stay readable.
pub fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Light => Palette {
            background: color!(0xffffff),
            text: color!(0x000000),
            accent: color!(0x5865f2),
            correct: color!(0x2b8a3e),
            wrong: color!(0xc3423f),
            muted: color!(0xcecece),
        },
        Theme::Dark => Palette {
            background: color!(0x2b2d31),
            text: color!(0xe6e6e6),
            accent: color!(0x5865f2),
            correct: color!(0x69db7c),
            wrong: color!(0xc3423f),
            muted: color!(0x50545d),
        },
    }
}

impl From<Palette> for iced::Theme {
    fn from(palette: Palette) -> Self {
        iced::Theme::custom(
            // Name is not important because the
            // theme name is not taken from `iced`,
            // but from [`core::Theme`]
            "Just Type Iced Custom Theme",
            iced::theme::Palette {
                background: palette.background,
                text: palette.text,
                primary: palette.accent,
                success: palette.correct,
                warning: palette.muted,
                danger: palette.wrong,
            },
        )
    }
}
