//! Module defining the Player trait

/// Base trait for all players in the game
///
/// This trait defines the minimal interface that all players (human, AI, etc.) must implement
/// to participate in a dominoes game.
pub trait Player {
    /// Returns the player's unique identifier
    fn id(&self) -> u8;

    /// Returns the player's name
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlayer {
        id: u8,
        name: String,
    }

    impl Player for TestPlayer {
        fn id(&self) -> u8 {
            self.id
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_player_id() {
        let player = TestPlayer {
            id: 0,
            name: "Alice".to_string(),
        };
        assert_eq!(player.id(), 0);
    }

    #[test]
    fn test_player_name() {
        let player = TestPlayer {
            id: 1,
            name: "Bob".to_string(),
        };
        assert_eq!(player.name(), "Bob");
    }

    #[test]
    fn test_player_trait_object() {
        let player: Box<dyn Player> = Box::new(TestPlayer {
            id: 2,
            name: "Charlie".to_string(),
        });
        assert_eq!(player.id(), 2);
        assert_eq!(player.name(), "Charlie");
    }
}
