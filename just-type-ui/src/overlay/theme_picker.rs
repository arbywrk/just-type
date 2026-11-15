use iced::{
    Background, Border, Color, Element, Event,
    Length::Fill,
    Subscription, Task,
    keyboard::{self, Key, key::Named},
    widget::{
        Id, button, center, column, container, mouse_area, opaque, operation, row, scrollable, space, text, text_input,
    },
};

use just_type_core::settings::theme::Theme;

use crate::{
    style::{self, BODY_SIZE},
    theme::{Palette, palette},
};

const WIDTH: f32 = 400.0;
/// The list scrolls once there are more themes than it can fit.
const MAX_HEIGHT: f32 = 360.0;
const CORNER_RADIUS: f32 = 12.0;
const DOT_SIZE: f32 = 14.0;

const SEARCH_ID: &str = "theme-search";
const LIST_ID: &str = "theme-list";

#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
    Hover(usize),
    Up,
    Down,
    Confirm,
    Pick(Theme),
    Close,
}

pub enum Action {
    None,
    Run(Task<Message>),
    Pick(Theme),
    Close,
}

pub struct State {
    query: String,
    /// Index of the highlighted theme among the matches.
    selected: usize,
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            query: String::new(),
            selected: 0,
        };
        // The search bar starts focused, so typing goes straight into it.
        (state, operation::focus(Id::new(SEARCH_ID)))
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Search(query) => {
                self.query = query;
                self.selected = 0;
                Action::Run(self.scroll_to_selected())
            }
            Message::Hover(index) => {
                self.selected = index;
                Action::None
            }
            Message::Up | Message::Down => {
                let count = self.matches().len();
                if count == 0 {
                    return Action::None;
                }
                self.selected = match message {
                    Message::Up => (self.selected + count - 1) % count,
                    _ => (self.selected + 1) % count,
                };
                Action::Run(self.scroll_to_selected())
            }
            Message::Confirm => match self.matches().get(self.selected) {
                Some(&theme) => Action::Pick(theme),
                None => Action::None,
            },
            Message::Pick(theme) => Action::Pick(theme),
            Message::Close => Action::Close,
        }
    }

    pub fn view(&self, current: Palette) -> Element<'_, Message> {
        let search = text_input("search", &self.query)
            .id(Id::new(SEARCH_ID))
            .on_input(Message::Search)
            .font(style::MONO)
            .size(BODY_SIZE)
            .padding(16)
            .style(move |_, _| text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                icon: current.muted,
                placeholder: current.muted,
                value: current.text,
                selection: current.accent.scale_alpha(0.3),
            });

        let rows = self
            .matches()
            .into_iter()
            .enumerate()
            .map(|(index, theme)| theme_row(theme, index == self.selected, index));
        let list = scrollable(column(rows).spacing(4).padding(8)).id(Id::new(LIST_ID));

        let card = container(column![search, list])
            .width(WIDTH)
            .max_height(MAX_HEIGHT)
            .style(move |_| container::Style {
                background: Some(current.background.into()),
                border: Border {
                    color: current.muted,
                    width: 1.0,
                    radius: CORNER_RADIUS.into(),
                },
                ..container::Style::default()
            });

        // The outer `opaque` blocks everything underneath. The inner one keeps clicks on
        // the card from reaching the backdrop, which closes the popup.
        opaque(
            mouse_area(center(opaque(card)).style(|_| container::background(Color::BLACK.scale_alpha(0.5))))
                .on_press(Message::Close),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Listens to every key, even the ones the search bar used, because it takes Esc.
        iced::event::listen_with(|event, _, _| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key), ..
            }) => match key {
                Named::Escape => Some(Message::Close),
                Named::ArrowUp => Some(Message::Up),
                Named::ArrowDown => Some(Message::Down),
                Named::Enter => Some(Message::Confirm),
                _ => None,
            },
            _ => None,
        })
    }

    fn matches(&self) -> Vec<Theme> {
        let query = self.query.trim().to_lowercase();
        Theme::ALL
            .into_iter()
            .filter(|theme| theme.name().contains(&query))
            .collect()
    }

    /// Keeps the highlighted theme in view.
    fn scroll_to_selected(&self) -> Task<Message> {
        let count = self.matches().len();
        let y = if count > 1 {
            self.selected as f32 / (count - 1) as f32
        } else {
            0.0
        };
        operation::snap_to(Id::new(LIST_ID), scrollable::RelativeOffset { x: 0.0, y })
    }
}

/// A line drawn in the theme's own colors, with its accent, muted and text colors as dots.
fn theme_row<'a>(theme: Theme, highlighted: bool, index: usize) -> Element<'a, Message> {
    let colors = palette(theme);

    let dot = |color: Color| {
        container(space().width(DOT_SIZE).height(DOT_SIZE)).style(move |_| container::Style {
            background: Some(color.into()),
            border: Border::default().rounded(DOT_SIZE / 2.0),
            ..container::Style::default()
        })
    };
    let dots = row![dot(colors.accent), dot(colors.muted), dot(colors.text)].spacing(6);

    let content = row![
        text(theme.name()).font(style::MONO).size(BODY_SIZE),
        space().width(Fill),
        dots
    ]
    .align_y(iced::Alignment::Center);

    let line = button(content)
        .width(Fill)
        .padding([10, 14])
        .on_press(Message::Pick(theme))
        .style(move |_, _| button::Style {
            background: Some(colors.background.into()),
            text_color: colors.text,
            border: Border {
                color: colors.text,
                width: if highlighted { 1.5 } else { 0.0 },
                radius: 8.0.into(),
            },
            ..button::Style::default()
        });

    // Hovering moves the highlight, so the mouse and the arrow keys share one selection.
    mouse_area(line).on_enter(Message::Hover(index)).into()
}
