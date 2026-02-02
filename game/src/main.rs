//! Dominoes Game Application

use game::app::DominoesApp;
use iced::Task;
use rules::{Configuration, Variation};

fn main() -> iced::Result {
    iced::application("Dominoes", DominoesApp::update, DominoesApp::view)
        .run_with(|| {
            let config = Configuration::new(2, Variation::Traditional, 6, 7);
            (DominoesApp::new(config), Task::none())
        })
}
