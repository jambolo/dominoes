//! Module defining the Player trait and related functionality
//!

use crate::Hand;
use dominoes_state::{Action, DominoesState};
use rules::Tile;

/// Base trait for all players in the game
///
/// This trait defines the interface that all players (human, AI, etc.) must implement to participate in a dominoes game. It
/// provides methods for game setup, turn taking, and querying player capabilities.
///
/// # Examples
/// ```rust
/// # use player::{Player, Hand};
/// # use dominoes_state::{Action, DominoesState};
/// # use rules::Configuration;
///
/// struct MyPlayer {
///     hand: Hand,
///     name: String,
///     id: u8,
/// }
///
/// impl Player for MyPlayer {
///     fn reset(&mut self) {
///         self.hand = Hand::new();
///     }
///
///     fn set_up(&mut self, state: &mut DominoesState) {
///         // Draw starting tiles
///     }
///
///     fn my_turn(&mut self, state: &DominoesState) -> (Action, DominoesState) {
///         // Make a move
///         (Action::pass(0), state.clone())
///     }
///
///     fn has_playable_tile(&self, state: &DominoesState) -> bool {
///         // Check if player can make a move
///         true
///     }
///
///     fn hand(&self) -> &Hand {
///         &self.hand
///     }
///
///     fn name(&self) -> &str {
///         &self.name
///     }
///
///     fn id(&self) -> u8 {
///         self.id
///     }
/// }
/// ```
pub trait Player {
    /// Resets the player to the initial state.
    fn reset(&mut self);

    /// Called during game setup to set up the player's state before the game starts.
    ///
    /// # Arguments
    /// * `state` - The current state of the game
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::DominoesState;
    /// # use player::Hand;
    /// # struct MyPlayer { hand: Hand }
    /// # impl MyPlayer {
    /// fn set_up(&mut self, state: &mut DominoesState) {
    ///     for _ in 0..7 {  // Draw 7 tiles for standard game
    ///         if let Some(tile) = state.draw_tile() {
    ///             self.hand.add_tile(tile);
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    fn set_up(&mut self, state: &mut DominoesState);

    /// Called when it's this player's turn to make a move
    ///
    /// # Arguments
    /// * `state` - The current state of the game (including action history)
    ///
    /// # Returns
    /// A tuple containing:
    /// - The action taken by the player
    /// - The new game state after the player's move
    ///
    /// Note that this method may be called multiple times during a player's turn. For example, in the case of a player
    /// needing to draw tiles, this method would be called once for each tile drawn.
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::{DominoesState, Action};
    /// # use player::Hand;
    /// # struct MyPlayer { player_id: u8, hand: Hand }
    /// # impl MyPlayer {
    /// #   fn has_playable_tile(&self, state: &DominoesState) -> bool { false }
    /// #   fn choose_tile_to_play(&self, state: &DominoesState) -> rules::Tile { rules::Tile::from((1,1)) }
    /// fn my_turn(&mut self, state: &DominoesState) -> (Action, DominoesState) {
    ///     if self.has_playable_tile(state) {
    ///         // Play a tile
    ///         let tile = self.choose_tile_to_play(state);
    ///         let mut new_state = state.clone();
    ///         new_state.play_tile(tile, None);
    ///         (Action::play(self.player_id, tile, None), new_state)
    ///     } else {
    ///         // Pass turn
    ///         (Action::pass(self.player_id), state.clone())
    ///     }
    /// }
    /// # }
    /// ```
    fn my_turn(&mut self, state: &DominoesState) -> (Action, DominoesState);

    /// Returns true if the player has at least one tile that can be played
    ///
    /// This method checks whether the player has any tiles in their hand that can be legally placed on the current layout. It's
    /// used to determine whether the player must draw tiles or pass their turn.
    ///
    /// # Arguments
    /// * `state` - The current state of the game
    ///
    /// # Returns
    /// `true` if the player can make a legal move, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dominoes_state::DominoesState;
    /// # use player::Hand;
    /// # struct MyPlayer { hand: Hand }
    /// # impl MyPlayer {
    /// fn has_playable_tile(&self, state: &DominoesState) -> bool {
    ///     self.hand.tiles().iter()
    ///         .any(|tile| state.can_play_tile(tile, None))
    /// }
    /// # }
    /// ```
    fn has_playable_tile(&self, state: &DominoesState) -> bool;

    /// Returns the player's hand
    ///
    /// Provides access to the player's current hand. Must be implemented by all players.
    ///
    /// # Returns
    /// A reference to the player's Hand
    fn hand(&self) -> &Hand;

    /// Returns the highest double tile in the player's hand, if any
    fn highest_double(&self) -> Option<Tile> {
        self.hand()
            .tiles()
            .iter()
            .filter(|&&tile| tile.is_double())
            .max_by_key(|&&tile| tile.as_tuple().0)
            .copied()
    }

    /// Returns the player's name or identifier
    ///
    /// This method provides a human-readable name for the player, useful for display purposes and game logs.
    ///
    /// # Returns
    /// A string slice containing the player's name
    ///
    /// # Examples
    /// ```rust
    /// # struct MyPlayer;
    /// # impl MyPlayer {
    /// fn name(&self) -> &str {
    ///     "AI Player Level 1"
    /// }
    /// # }
    /// ```
    fn name(&self) -> &str;

    /// Returns the player's unique identifier
    ///
    /// This method provides a unique numeric ID for the player, used internally to distinguish between different players.
    ///
    /// # Returns
    /// A u8 representing the player's ID
    ///
    /// # Examples
    /// ```rust
    /// # use player::Player;
    /// # struct MyPlayer;
    /// # impl MyPlayer {
    /// fn id(&self) -> u8 {
    ///     1
    /// }
    /// # }
    /// ```
    fn id(&self) -> u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Hand;
    use dominoes_state::{Action, DominoesState};
    use rules::{Configuration, Tile};

    // Test implementation of Player trait
    struct TestPlayer {
        id: u8,
        name: String,
        hand: Hand,
    }

    impl TestPlayer {
        fn new(id: u8, name: &str) -> Self {
            Self {
                id,
                name: name.to_string(),
                hand: Hand::new(),
            }
        }
    }

    impl Player for TestPlayer {
        fn reset(&mut self) {
            self.hand = Hand::new();
        }

        fn set_up(&mut self, state: &mut DominoesState) {
            // Simple setup - just add a test tile
            if let Some(tile) = state.boneyard.draw() {
                self.hand.add_tile(tile);
            }
        }

        fn my_turn(&mut self, state: &DominoesState) -> (Action, DominoesState) {
            // Always pass for test
            (Action::pass(self.id), state.clone())
        }

        fn has_playable_tile(&self, _state: &DominoesState) -> bool {
            !self.hand.is_empty()
        }

        fn hand(&self) -> &Hand {
            &self.hand
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn id(&self) -> u8 {
            self.id
        }
    }

    #[test]
    fn test_player_reset() {
        let mut player = TestPlayer::new(0, "Test Player");
        player.hand.add_tile(Tile::from((1, 2)));
        assert!(!player.hand().is_empty());

        player.reset();
        assert!(player.hand().is_empty());
    }

    #[test]
    fn test_player_set_up() {
        let config = Configuration::default();
        let mut state = DominoesState::new(&config);
        let mut player = TestPlayer::new(0, "Test Player");

        let initial_hand_size = player.hand().len();
        player.set_up(&mut state);

        // Should have drawn at least one tile (if boneyard wasn't empty)
        assert!(player.hand().len() >= initial_hand_size);
    }

    #[test]
    fn test_player_my_turn() {
        let config = Configuration::default();
        let state = DominoesState::new(&config);
        let mut player = TestPlayer::new(1, "Test Player");

        let (action, _) = player.my_turn(&state);

        assert_eq!(action.player_id, 1);
        // Test implementation always passes
        assert!(action.tile_drawn.is_none() && action.tile_played.is_none());
    }

    #[test]
    fn test_player_has_playable_tile() {
        let config = Configuration::default();
        let state = DominoesState::new(&config);
        let mut player = TestPlayer::new(0, "Test Player");

        // Empty hand should return false
        assert!(!player.has_playable_tile(&state));

        // Add a tile
        player.hand.add_tile(Tile::from((1, 2)));
        assert!(player.has_playable_tile(&state));
    }

    #[test]
    fn test_player_hand() {
        let mut player = TestPlayer::new(0, "Test Player");
        let tile = Tile::from((3, 4));

        assert!(player.hand().is_empty());

        player.hand.add_tile(tile);
        assert!(player.hand().contains(&tile));
    }

    #[test]
    fn test_player_highest_double() {
        let mut player = TestPlayer::new(0, "Test Player");

        // No doubles in hand
        assert_eq!(player.highest_double(), None);

        // Add some tiles including doubles
        player.hand.add_tile(Tile::from((1, 2))); // Not a double
        player.hand.add_tile(Tile::from((3, 3))); // Double
        player.hand.add_tile(Tile::from((1, 1))); // Lower double
        player.hand.add_tile(Tile::from((5, 5))); // Higher double

        // Should return highest double (5,5)
        assert_eq!(player.highest_double(), Some(Tile::from((5, 5))));
    }

    #[test]
    fn test_player_highest_double_no_doubles() {
        let mut player = TestPlayer::new(0, "Test Player");

        // Add non-double tiles
        player.hand.add_tile(Tile::from((1, 2)));
        player.hand.add_tile(Tile::from((3, 4)));
        player.hand.add_tile(Tile::from((5, 6)));

        assert_eq!(player.highest_double(), None);
    }

    #[test]
    fn test_player_name() {
        let player = TestPlayer::new(0, "Alice");
        assert_eq!(player.name(), "Alice");

        let player2 = TestPlayer::new(1, "Bob");
        assert_eq!(player2.name(), "Bob");
    }

    #[test]
    fn test_player_id() {
        let player1 = TestPlayer::new(0, "Player 1");
        assert_eq!(player1.id(), 0);

        let player2 = TestPlayer::new(255, "Player 2");
        assert_eq!(player2.id(), 255);
    }

    #[test]
    fn test_player_trait_object() {
        let player: Box<dyn Player> = Box::new(TestPlayer::new(0, "Boxed Player"));

        assert_eq!(player.id(), 0);
        assert_eq!(player.name(), "Boxed Player");
        assert!(player.hand().is_empty());
    }
}
