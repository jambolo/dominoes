/// DominoesPlayer implementation that extends the base Player trait for dominoes-specific functionality.
/// This crate provides concrete player implementations for dominoes games.

use player::{GameState, Player};

/// A concrete implementation of Player for dominoes games
#[derive(Debug)]
pub struct DominoesPlayer {
    /// The player's name/identifier
    name: String,
}

impl DominoesPlayer {
    /// Creates a new dominoes player with the given name
    pub fn new(name: String) -> Self {
        Self { name }
    }
    
    /// Creates a new dominoes player with a default name
    pub fn default() -> Self {
        Self {
            name: "Dominoes Player".to_string(),
        }
    }
}

impl Player for DominoesPlayer {
    fn my_turn(&mut self, game_state: &GameState) -> GameState {
        // TODO: Implement dominoes-specific game logic
        // This is a stub implementation that just returns the same state
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
    fn test_dominoes_player_creation() {
        let player = DominoesPlayer::new("Alice".to_string());
        assert_eq!(player.name(), "Alice");
    }

    #[test]
    fn test_dominoes_player_default() {
        let player = DominoesPlayer::default();
        assert_eq!(player.name(), "Dominoes Player");
    }

    #[test]
    fn test_player_trait_implementation() {
        let mut player = DominoesPlayer::new("Test Player".to_string());
        let game_state = GameState::new();
        
        // Test that my_turn returns a GameState (stub implementation)
        let new_state = player.my_turn(&game_state);
        // Since it's a stub, it should return the same state
        // This is just testing that the method exists and compiles
        assert!(true); // Placeholder assertion
    }
}
