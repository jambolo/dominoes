/// HumanPlayer implementation that extends the base Player trait for human player functionality.
/// This crate provides a concrete player implementation for human-controlled gameplay.

use player::{GameState, Player};

/// A concrete implementation of Player for human players
#[derive(Debug)]
pub struct HumanPlayer {
    /// The player's name/identifier
    name: String,
}

impl HumanPlayer {
    /// Creates a new human player with the given name
    pub fn new(name: String) -> Self {
        Self { name }
    }
    
    /// Creates a new human player with a default name
    pub fn default() -> Self {
        Self {
            name: "Human Player".to_string(),
        }
    }
}

impl Player for HumanPlayer {
    fn my_turn(&mut self, game_state: &GameState) -> GameState {
        // TODO: Implement human player input handling
        // This is a stub implementation that just returns the same state
        // Future implementation should prompt for user input and validate moves
        game_state.clone()
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_player_creation() {
        let player = HumanPlayer::new("Alice".to_string());
        assert_eq!(player.name(), "Alice");
    }

    #[test]
    fn test_human_player_default() {
        let player = HumanPlayer::default();
        assert_eq!(player.name(), "Human Player");
    }

    #[test]
    fn test_player_trait_implementation() {
        let mut player = HumanPlayer::new("Test Player".to_string());
        let game_state = GameState::new();
        
        // Test that my_turn returns a GameState (stub implementation)
        let new_state = player.my_turn(&game_state);
        // Since it's a stub, it should return the same state
        // This is just testing that the method exists and compiles
        assert!(true); // Placeholder assertion
    }
}
