//! Entry point for the iced ui game binary

// prevent Windows from opening a terminal when running the app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod component;
mod game;
mod overlay;
mod screen;
mod style;
mod theme;

use crate::game::Game;

/// Create, setup and run the game.
fn main() -> iced::Result {
    iced::application(Game::new, Game::update, Game::view)
        .subscription(Game::subscription)
        .scale_factor(Game::scale_factor)
        .title("Just Type")
        .font(style::MONO_BYTES)
        .theme(Game::theme)
        .run()
}
