//! Entry point for the iced ui game binary

// prevent Windows from opening a terminal when running the app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod game;
mod paths;
mod screen;
mod settings;
mod style;
mod typing_text;
mod widget;

use crate::game::Game;

/// Create, setup and run the game.
fn main() -> iced::Result {
    iced::application(Game::new, Game::update, Game::view)
        .subscription(Game::subscription)
        .scale_factor(Game::scale_factor)
        .title("Just Type")
        .run()
}
