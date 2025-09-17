//! A crate defining the player trait and a base player implementation for a two-player game.
//! This module is designed to be extended for specific game implementations.

pub mod hand;
pub mod player;

pub use hand::*;
pub use player::*;

