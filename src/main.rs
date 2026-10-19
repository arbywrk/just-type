mod game;
mod score;
mod view;
mod words;

use crate::game::Game;

fn main() -> iced::Result {
    iced::application(Game::boot, Game::update, Game::view)
        .subscription(Game::subscription)
        .run()
}
