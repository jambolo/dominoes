//! Dominoes Game Application

mod dominoes_game;

use crate::dominoes_game::DominoesGame;
use rules::Configuration;

fn main() {
    println!("Welcome to the Dominoes Game!");

    // Create default configuration
    let configuration = Configuration::default();

    // Initialize the game with the configuration
    let mut game = DominoesGame::new(&configuration);

    // Run the game loop
    game.run();

    println!("Thanks for playing!");
}
