#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod game;
mod score;
mod typing_text;
mod view;
mod words;

use crate::game::Game;

fn main() -> iced::Result {
    iced::application(Game::boot, Game::update, Game::view)
        .subscription(Game::subscription)
        .run()
}
