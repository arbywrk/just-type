// I want to build a typeing game
//
// The game (MVP):
// It will show an infinit text (beautifully formatted for fast reading, mutted grey collor).
// Once the player starts typeing the text a countdown for 15 seconds will start.
// When a letter is written it will be colored with:
// - white if the input was correct
// - red if the input was incorrect (it will not show the actually typped letter at the moment)
//
// Once the timer is done the game is over and it will change to a stats screen,
// the stats screen will show the avrage WPM
// and will have a button for playing again.
use std::time::Duration;

use iced::{
    widget::{column, text},
    Element, Subscription,
};

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

#[derive(Default)]
struct App {
    value: i64,
    game_over: bool,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.value += 1;

                if self.value == 15 {
                    self.game_over = true;
                }
            },

        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![text(self.value.to_string())].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.game_over {
            Subscription::none()
        } else {
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        }
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .run()
}
