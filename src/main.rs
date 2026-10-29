#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod game;
mod paths;
mod score;
mod screen;
mod settings;
mod style;
mod typing_text;
mod widget;
mod words;

use crate::game::Game;

fn main() -> iced::Result {
    iced::application(Game::new, Game::update, Game::view)
        .subscription(Game::subscription)
        .scale_factor(Game::scale_factor)
        .title("Just Type")
        .run()
}
