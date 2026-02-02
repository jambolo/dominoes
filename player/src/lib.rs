//! Player crate for dominoes games.
//!
//! This crate provides the `Player` trait and `HumanPlayer` implementation.

pub mod player;
pub mod human_player;

pub use player::*;
pub use human_player::*;
