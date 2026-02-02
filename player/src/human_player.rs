//! Human player implementation

use crate::Player;

/// A concrete implementation of Player for human players
#[derive(Debug, Clone)]
pub struct HumanPlayer {
    /// Unique identifier for this player
    id: u8,
    /// Display name for this player
    name: String,
}

impl HumanPlayer {
    /// Creates a new human player
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this player
    /// * `name` - Display name for this player
    pub fn new(id: u8, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}

impl Player for HumanPlayer {
    fn id(&self) -> u8 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_player_new() {
        let player = HumanPlayer::new(0, "Alice");
        assert_eq!(player.id(), 0);
        assert_eq!(player.name(), "Alice");
    }

    #[test]
    fn test_human_player_different_ids() {
        let player0 = HumanPlayer::new(0, "Player 0");
        let player1 = HumanPlayer::new(1, "Player 1");

        assert_eq!(player0.id(), 0);
        assert_eq!(player1.id(), 1);
    }

    #[test]
    fn test_human_player_clone() {
        let player = HumanPlayer::new(0, "Bob");
        let cloned = player.clone();

        assert_eq!(cloned.id(), player.id());
        assert_eq!(cloned.name(), player.name());
    }

    #[test]
    fn test_human_player_debug() {
        let player = HumanPlayer::new(0, "Test");
        let debug_str = format!("{:?}", player);
        assert!(debug_str.contains("HumanPlayer"));
        assert!(debug_str.contains("Test"));
    }
}
