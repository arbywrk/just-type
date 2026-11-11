use iced::Color;
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

pub fn palette(theme: Theme) -> Palette {
    let from_iced = |theme: iced::Theme| {
        let palette = theme.palette();
        let extended_palette = theme.extended_palette();

        Palette {
            background: palette.background,
            text: palette.text,
            accent: palette.primary,
            correct: palette.success,
            wrong: palette.danger,
            muted: extended_palette.background.strong.color,
        }
    };

    match theme {
        Theme::Light => from_iced(iced::Theme::Light),
        Theme::Dark => from_iced(iced::Theme::Dark),
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
