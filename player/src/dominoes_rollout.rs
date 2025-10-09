//! Rollout implementation for the Dominoes game
//!
//! This module contains the rollout algorithm for the game state analysis, which is used during the MCTS process to simulate
//! random games from a given state and evaluate the potential outcomes.
//!
use rand::Rng;

use hidden_game_player::mcts::Rollout;
use dominoes_state::DominoesState;

/// A rollout strategy for the Dominoes game that implements the `Rollout` trait.
///
/// This struct is responsible for simulating random game play from a given state to estimate the value of that state. It's a key
/// component in the Monte Carlo Tree Search (MCTS) algorithm, used during the simulation phase to quickly evaluate leaf nodes.
///
/// The rollout strategy currently returns values between 0.0 and 1.0 to simulate game outcomes, where higher values indicate
/// better positions for the current player.
///
/// # Examples
/// ```rust
/// use player::DominoesRollout;
/// use dominoes_state::DominoesState;
/// use rules::Configuration;
/// use hidden_game_player::mcts::Rollout;
///
/// let rollout = DominoesRollout::new();
/// let config = Configuration::default();
/// let state = DominoesState::new(&config);
///
/// let outcome = rollout.play(&state);
/// assert!(outcome >= 0.0 && outcome <= 1.0);
/// ```
pub struct DominoesRollout;

impl DominoesRollout {
    /// Creates a new `DominoesRollout` instance.
    ///
    /// This constructor creates a new rollout strategy for use with the MCTS algorithm.
    /// Since the rollout strategy is stateless, this simply returns a new instance
    /// of the struct.
    ///
    /// # Returns
    ///
    /// A new `DominoesRollout` instance ready for use in game simulations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use player::DominoesRollout;
    ///
    /// let rollout = DominoesRollout::new();
    /// ```
    pub fn new() -> Self {
        Self
    }
}

impl Default for DominoesRollout {
    /// Creates a default `DominoesRollout` instance.
    ///
    /// This implementation uses the `new()` method to create a default instance,
    /// providing a convenient way to create the rollout strategy using Rust's `Default` trait.
    /// This is particularly useful when the rollout strategy is used as part of larger
    /// configuration structures that implement `Default`.
    ///
    /// # Returns
    /// A new `DominoesRollout` instance.
    ///
    /// # Examples
    /// ```rust
    /// # use player::DominoesRollout;
    ///
    /// let rollout = DominoesRollout::default();
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl Rollout for DominoesRollout {
    type State = DominoesState;

    fn play(&self, _state: &DominoesState) -> f32 {
        // TODO: Unimplemented
        let mut rng = rand::rng();
        rng.random_range(0.0..=1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rules::Configuration;

    #[test]
    fn test_dominoes_rollout_new() {
        let rollout = DominoesRollout::new();
        // Test that the struct is created successfully
        // Since it's a unit struct, just verify it exists
        let _ = rollout;
    }

    #[test]
    fn test_dominoes_rollout_default() {
        let rollout = DominoesRollout::default();
        // Test that default implementation works
        let _ = rollout;
    }

    #[test]
    fn test_rollout_trait_implementation() {
        let rollout = DominoesRollout::new();
        let configuration = Configuration::default();
        let state = DominoesState::new(&configuration);

        // Test that play method returns a value in expected range
        let result = rollout.play(&state);
        assert!(result >= 0.0 && result <= 1.0, "Rollout result should be between 0.0 and 1.0");
    }

    #[test]
    fn test_rollout_with_different_states() {
        let rollout = DominoesRollout::new();
        let configuration = Configuration::default();
        let state1 = DominoesState::new(&configuration);
        let state2 = DominoesState::new(&configuration);

        // Test that rollout works with different state instances
        let result1 = rollout.play(&state1);
        let result2 = rollout.play(&state2);

        assert!(result1 >= 0.0 && result1 <= 1.0);
        assert!(result2 >= 0.0 && result2 <= 1.0);
    }
}
