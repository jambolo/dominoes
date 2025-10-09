//! Manages the entire dominoes game, including player setup, turn management, and game state transitions.

use dominoes_state::{Action, DominoesState, History};
use hidden_game_player::{PlayerId, State};
use player::{HumanPlayer, Player};
use rules::Configuration;

/// An instance of a dominoes game
pub struct DominoesGame<'a> {
    /// The game configuration
    configuration: &'a Configuration,
    /// Human player representing Alice (Player 0)
    alice: HumanPlayer<'a>,
    /// Human player representing Bob (Player 1)
    bob: HumanPlayer<'a>,
    /// History of all actions taken during the game
    history: History,
}

impl<'a> DominoesGame<'a> {
    /// Creates a new dominoes game with the given configuration
    ///
    /// Initializes a fresh dominoes game by setting up the game state, creating two human players (Alice and Bob), and preparing
    /// an empty action history.
    ///
    /// # Arguments
    /// * `configuration` - Game rules and settings including hand size, set size, and game variation
    ///
    /// # Returns
    /// A new `DominoesGame` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use rules::Configuration;
    /// # use dominoes_game::DominoesGame;
    ///
    /// let config = Configuration::default();
    /// let game = DominoesGame::new(&config);
    ///
    /// // Game is initialized with two players
    /// ```
    pub fn new(configuration: &'a Configuration) -> Self {
        Self {
            configuration,
            alice: HumanPlayer::new(PlayerId::ALICE as u8, configuration, "Alice"),
            bob: HumanPlayer::new(PlayerId::BOB as u8, configuration, "Bob"),
            history: History::new(),
        }
    }

    /// Runs the main game loop
    ///
    /// This method handles the complete game flow:
    /// 1. Sets up both players by dealing initial hands
    /// 2. Runs the turn-based game loop until completion
    /// 3. Handles game end conditions and displays results
    ///
    /// The game continues until either a win condition is met or a maximum number of turns is reached (to prevent infinite loops
    /// in stub implementations).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rules::Configuration;
    /// # use dominoes_game::DominoesGame;
    ///
    /// let config = Configuration::default();
    /// let mut game = DominoesGame::new(&config);
    ///
    /// // This will start the interactive game loop
    /// // Marked as no_run because it requires user input
    /// game.run();
    /// ```
    pub fn run(&mut self) {
        println!("Setting up the game...\n");
        println!("Game Variation: {}", self.configuration.variation.name());
        println!("Number of Players: {}", self.configuration.num_players);
        println!("Domino Set: Double-{}", self.configuration.set_id);

        let mut state = DominoesState::new(self.configuration);

        // Setup players
        self.set_up_players_by_variation(&mut state);

        println!("Starting the game...");

        // Prevent infinite loop in stub implementation
        let mut turn_count = 0;
        let max_turns = self.configuration.set_size() * 2 + self.configuration.num_players; // draw+play for each tile plus passing

        while !state.game_is_over && turn_count < max_turns {
            let current_player_id = state.whose_turn();
            let player_name = self.player(current_player_id).name().to_string();
            println!("\nIt's {player_name}'s turn");
            loop {
                let (action, mut new_state) = self.player_mut(current_player_id).my_turn(&state);
                if !action.is_draw() {
                    println!("{player_name}'s action: {action}");
                }

                // Determine if the game should end according to the variation
                if let Some(winner) = self.game_is_over_by_variation(&new_state) {
                    new_state.mark_game_over(winner);
                }

                // Update the game state
                state = new_state;

                // Record the action in history
                self.history.add_action(action.clone());

                turn_count += 1;

                // The turn is over if the game is over
                if state.game_is_over {
                    break;
                }

                // Determine other criteria for the turn being over according to the variation
                if self.turn_is_over_by_variation(&action) {
                    break;
                }
            }

            // Next player's turn
            state.whose_turn = (state.whose_turn + 1) % 2;
        }

        self.wrap_up(&state);
    }

    // Helper to get player by ID
    fn player(&self, player_id: u8) -> &dyn Player {
        match player_id {
            0 => &self.alice,
            1 => &self.bob,
            _ => unreachable!("Only two players supported"),
        }
    }

    // Helper to get mutable player by ID
    fn player_mut(&mut self, player_id: u8) -> &mut dyn Player {
        match player_id {
            0 => &mut self.alice,
            1 => &mut self.bob,
            _ => unreachable!("Only two players supported"),
        }
    }

    // Sets up both players according to the game variation
    fn set_up_players_by_variation(&mut self, state: &mut DominoesState) {
        self.alice.set_up(state);
        self.bob.set_up(state);

        // Handle setup variations
        match self.configuration.variation {
            // In traditional dominoes, the player with the highest double starts. If nobody has a double, players must redraw.
            rules::Variation::Traditional => {
                // Determine starting player based on highest double
                println!("Determining starting player based on highest double...");
                let mut first_player = None;
                while first_player.is_none() {
                    first_player = match (self.alice.highest_double(), self.bob.highest_double()) {
                        (Some(a), Some(b)) => {
                            // Both players have doubles, highest starts
                            Some(if a > b {
                                PlayerId::ALICE as u8
                            } else {
                                PlayerId::BOB as u8
                            })
                        }
                        (Some(_), None) => {
                            // Alice has a double, Bob does not
                            Some(PlayerId::ALICE as u8)
                        }
                        (None, Some(_)) => {
                            // Bob has a double, Alice does not
                            Some(PlayerId::BOB as u8)
                        }
                        (None, None) => {
                            // Neither have doubles, must redraw
                            println!("No doubles found. Both players must redraw.");
                            self.alice.reset();
                            self.bob.reset();
                            *state = DominoesState::new(self.configuration);
                            self.alice.set_up(state);
                            self.bob.set_up(state);
                            None
                        }
                    };
                }

                // Now we know first_player is Some, but still use match to be safe
                state.whose_turn = first_player.expect("Should have a first player after the loop");
            }
            _ => {
                // For other variations, nothing special to do here
            }
        }
    }

    // Marks the game as over according to the variation
    fn game_is_over_by_variation(&self, state: &DominoesState) -> Option<Option<u8>> {
        match self.configuration.variation {
            rules::Variation::Traditional => {
                // In Traditional variation, game ends when a player empties their hand or both players pass. The winner is
                // the player with the lowest hand score.
                if self.alice.hand().is_empty() {
                    return Some(Some(self.alice.id()));
                } else if self.bob.hand().is_empty() {
                    return Some(Some(self.bob.id()));
                } else if state.consecutive_passes as usize >= self.configuration.num_players {
                    let alice_score = self.alice.hand().score();
                    let bob_score = self.bob.hand().score();
                    return Some(if alice_score < bob_score {
                        Some(PlayerId::ALICE as u8)
                    } else if bob_score < alice_score {
                        Some(PlayerId::BOB as u8)
                    } else {
                        None
                    });
                }
            }
            _ => {
                // FIXME: Add real game ending logic based on variation here.
                if state.consecutive_passes as usize >= self.configuration.num_players {
                    return Some(None); // Game ends in a draw
                }
            }
        }
        // Otherwise, the game is not over
        None
    }

    // Determines if the turn is over according to the variation
    fn turn_is_over_by_variation(&self, action: &Action) -> bool {
        match self.configuration.variation {
            rules::Variation::Traditional => {
                if action.tile_drawn.is_none() {
                    return true; // Tile was played or passed, end turn
                }
            }
            _ => {
                if action.tile_played.is_some() || action.tile_drawn.is_none() {
                    return true; // Tile was played or passed, end turn
                }
            }
        }
        false
    }

    // Handles end of game logic
    fn wrap_up(&self, state: &DominoesState) {
        println!("Game Over!");

        if let Some(winner_id) = state.winner {
            println!("Winner: {}", self.player(winner_id).name());
        } else {
            println!("It's a draw");
        }

        // Display final game statistics
        self.display_game_summary(state);
    }

    // Displays a summary of the game
    fn display_game_summary(&self, state: &DominoesState) {
        println!("\n--- Game Summary ---");
        println!("Players:");
        println!("  {}", self.alice.name());
        println!("  {}", self.bob.name());

        // Display the final layout
        let layout_string = state.layout.to_string();
        println!("Final Layout:\n{layout_string}");

        // Display action history
        let actions = self.history.get_actions();
        println!("\nAction History ({} actions):", actions.len());
        for (i, action) in actions.iter().enumerate() {
            println!(
                "  {}: {} - {action}",
                i + 1,
                self.player(action.player_id).name()
            );
        }

        println!("Game completed successfully!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rules::{Configuration, Variation};

    fn create_test_configuration() -> Configuration {
        Configuration {
            starting_hand_size: 7,
            set_id: 6,
            ..Configuration::default()
        }
    }

    #[test]
    fn test_dominoes_game_new_basic() {
        let config = create_test_configuration();
        let game = DominoesGame::new(&config);

        // Verify the game was created successfully
        // Check that the configuration values are properly stored
        assert_eq!(game.configuration.num_players, config.num_players);
        assert_eq!(
            game.configuration.starting_hand_size,
            config.starting_hand_size
        );
        assert_eq!(game.configuration.set_id, config.set_id);
    }

    #[test]
    fn test_dominoes_game_new_with_different_configurations() {
        // Test with default configuration
        let default_config = Configuration::default();
        let default_game = DominoesGame::new(&default_config);
        assert_eq!(
            default_game.configuration.num_players,
            default_config.num_players
        );
        assert_eq!(
            default_game.configuration.starting_hand_size,
            default_config.starting_hand_size
        );

        // Test with traditional variation
        let traditional_config = Configuration::new(2, Variation::Traditional, 7, 6);
        let traditional_game = DominoesGame::new(&traditional_config);
        assert_eq!(
            traditional_game.configuration.num_players,
            traditional_config.num_players
        );
        assert_eq!(
            traditional_game.configuration.set_id,
            traditional_config.set_id
        );

        // Test with different hand size
        let large_hand_config = Configuration {
            starting_hand_size: 10,
            ..Configuration::default()
        };
        let large_hand_game = DominoesGame::new(&large_hand_config);
        assert_eq!(
            large_hand_game.configuration.starting_hand_size,
            large_hand_config.starting_hand_size
        );
        assert_eq!(
            large_hand_game.configuration.num_players,
            large_hand_config.num_players
        );
    }

    #[test]
    fn test_dominoes_game_new_initializes_history() {
        let config = create_test_configuration();
        let game = DominoesGame::new(&config);

        // History should start empty
        assert!(game.history.get_actions().is_empty());
    }

    #[test]
    fn test_dominoes_game_new_with_minimal_configuration() {
        // Test with minimal valid configuration
        let minimal_config = Configuration::new(2, Variation::Traditional, 1, 1);
        let game = DominoesGame::new(&minimal_config);

        assert_eq!(game.configuration.num_players, minimal_config.num_players);
        assert_eq!(
            game.configuration.starting_hand_size,
            minimal_config.starting_hand_size
        );
        assert_eq!(game.configuration.set_id, minimal_config.set_id);
        assert!(game.history.get_actions().is_empty());
    }

    #[test]
    fn test_dominoes_game_new_with_large_configuration() {
        // Test with large configuration
        let large_config = Configuration::new(2, Variation::Traditional, 15, 12);
        let game = DominoesGame::new(&large_config);

        assert_eq!(game.configuration.num_players, large_config.num_players);
        assert_eq!(
            game.configuration.starting_hand_size,
            large_config.starting_hand_size
        );
        assert_eq!(game.configuration.set_id, large_config.set_id);
        assert!(game.history.get_actions().is_empty());
    }

    #[test]
    fn test_dominoes_game_configuration_reference() {
        let config = create_test_configuration();
        let game = DominoesGame::new(&config);

        // Verify that the game holds a reference to the same configuration
        assert_eq!(game.configuration.num_players, config.num_players);
        assert_eq!(
            game.configuration.starting_hand_size,
            config.starting_hand_size
        );
        assert_eq!(game.configuration.set_id, config.set_id);
    }

    #[test]
    fn test_dominoes_game_new_memory_safety() {
        // Test that the game properly handles the lifetime of the configuration reference
        let config = create_test_configuration();
        let game = DominoesGame::new(&config);

        // Game should be valid as long as config is in scope
        assert_eq!(game.configuration.num_players, 2);

        // Create another game with the same config
        let game2 = DominoesGame::new(&config);
        assert_eq!(game2.configuration.num_players, 2);
    }

    #[test]
    fn test_dominoes_game_history_tracking_initialized() {
        let config = create_test_configuration();
        let game = DominoesGame::new(&config);

        // History should be properly initialized
        let actions = game.history.get_actions();
        assert!(actions.is_empty());
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn test_dominoes_game_new_consistent_initialization() {
        let config = create_test_configuration();

        // Create multiple games with same config
        let game1 = DominoesGame::new(&config);
        let game2 = DominoesGame::new(&config);

        // Both should be initialized consistently
        assert!(game1.history.get_actions().is_empty());
        assert!(game2.history.get_actions().is_empty());

        assert_eq!(
            game1.configuration.num_players,
            game2.configuration.num_players
        );
    }

    #[test]
    fn test_dominoes_game_new_with_all_variations() {
        // Test game creation with different variations
        for &variation in &[Variation::Traditional] {
            let config = Configuration::new(2, variation, 7, 6);
            let game = DominoesGame::new(&config);

            assert_eq!(game.configuration.variation, variation);
            assert!(game.history.get_actions().is_empty());
        }
    }

    // Note: Testing run() method directly is challenging because it requires user input
    // from HumanPlayer. Integration tests would be needed to test the full game flow.

    #[test]
    fn test_dominoes_game_public_api_structure() {
        let config = create_test_configuration();
        let mut game = DominoesGame::new(&config);

        // Verify that the public API is accessible
        // new() method works
        assert!(game.history.get_actions().is_empty());

        // run() method exists and can be called (though we can't test it fully without mocking input)
        // We're not actually calling run() here to avoid requiring user input
        // Just verifying it exists and compiles
        let _can_call_run = || game.run();
    }

    #[test]
    fn test_dominoes_game_lifetime_parameters() {
        // Test that lifetime parameters work correctly
        fn create_and_return_game(config: &Configuration) -> DominoesGame<'_> {
            DominoesGame::new(config)
        }

        let config = create_test_configuration();
        let game = create_and_return_game(&config);

        assert!(game.history.get_actions().is_empty());
    }

    #[test]
    fn test_dominoes_game_documentation_examples() {
        // Test the examples from the documentation
        let config = Configuration::default();
        let game = DominoesGame::new(&config);

        // Game is initialized (as claimed in doctest)
        assert!(game.history.get_actions().is_empty());
    }
}
