//! Widgets shared between screens.

use iced::{
    Border, Element, Theme,
    widget::{button, mouse_area, svg},
};

use crate::style::MUTED;

/// Space between the icon and the button's border, which also enlarges the hitbox.
const ICON_PADDING: f32 = 8.0;
const CORNER_RADIUS: f32 = 8.0;

/// A square icon button with a rounded border, muted unless `highlighted`.
///
/// When `visible` is false it is hidden, but still shows while highlighted.
/// iced doesn't give the icon the button's hover status, so the caller tracks
/// hover through `on_hover` and passes it back in as `highlighted`.
pub fn icon_button<'a, Message: Clone + 'a>(
    icon: svg::Handle,
    size: f32,
    visible: bool,
    highlighted: bool,
    on_press: Message,
    on_hover: impl Fn(bool) -> Message,
) -> Element<'a, Message> {
    let opacity = if visible || highlighted { 1.0 } else { 0.0 };
    let color = move |theme: &Theme| {
        if highlighted { theme.palette().text } else { MUTED }
    };

    let icon = svg(icon)
        .width(size)
        .height(size)
        // The svg color filter ignores alpha, so it has to be hidden with opacity.
        .opacity(opacity)
        .style(move |theme: &Theme, _| svg::Style {
            color: Some(color(theme)),
        });

    let button = button(icon)
        .padding(ICON_PADDING)
        .style(move |theme: &Theme, _| {
            let color = color(theme).scale_alpha(opacity);
            button::Style {
                background: None,
                text_color: color,
                border: Border {
                    color,
                    width: 1.0,
                    radius: CORNER_RADIUS.into(),
                },
                ..button::Style::default()
            }
        })
        .on_press(on_press);

    mouse_area(button)
        .on_enter(on_hover(true))
        .on_exit(on_hover(false))
        .into()
}
