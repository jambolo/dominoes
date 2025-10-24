//! A comprehensive player crate for dominoes games.
//!
//! This crate provides the base `Player` trait, player hand management,
//! and concrete implementations for both human and AI players.

pub mod player;
pub mod human_player;
pub mod dominoes_player;
pub mod dominoes_response_generator;
pub mod dominoes_rollout;
pub mod dominoes_static_evaluator;

pub use player::*;
pub use human_player::*;
pub use dominoes_player::*;
pub use dominoes_response_generator::*;
pub use dominoes_rollout::*;
pub use dominoes_static_evaluator::*;

