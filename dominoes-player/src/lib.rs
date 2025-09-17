//! A computer player implementation of the base Player trait for dominoes.

pub mod dominoes_player;
pub mod dominoes_response_generator;
pub mod dominoes_rollout;
pub mod dominoes_static_evaluator;

pub use dominoes_player::*;
pub use dominoes_rollout::*;
pub use dominoes_response_generator::*;
pub use dominoes_static_evaluator::*;
