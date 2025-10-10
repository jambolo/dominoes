//! AI player implementation.
//!
//! This module defines a computer player for dominoes games, implementing the Player trait.
//! It uses Monte Carlo Tree Search (MCTS) for decision making and maintains
//! knowledge of hidden tiles and opponent tile probabilities.

use std::collections::HashMap;

use dominoes_state::{Action, DominoesState};
use crate::{Hand, Player, DominoesResponseGenerator, DominoesRollout};
use rules::{Configuration, Tile};
use hidden_game_player::{mcts, State};

/// An AI implementation of Player for dominoes games
#[derive(Debug, Clone)]
pub struct DominoesPlayer<'a> {
    /// Player ID
    player_id: u8,
    /// Game configuration
    configuration: &'a Configuration,
    /// List of tiles that are still hidden/unknown to this player
    /// Initially contains all tiles, but tiles are removed as they are played or drawn by this player
    hidden: Vec<Tile>,
    /// List of tiles that the player currently holds in their hand
    hand: Hand,
    /// Probability of the other player having each possible tile
    /// Maps tile -> probability (0.0 to 1.0)
    opponent_tile_probabilities: HashMap<Tile, f64>,
}

impl<'a> DominoesPlayer<'a> {
    /// Creates a new dominoes player with the specified configuration
    pub fn new(player_id: u8, configuration: &'a Configuration) -> Self {
        // Initialize opponent tile probabilities - initially the opponent's hand is empty
        let mut opponent_tile_probabilities = HashMap::new();
        for tile in configuration.all_tiles() {
            opponent_tile_probabilities.insert(*tile, 0.0);
        }

        Self {
            player_id,
            configuration,
            hidden: configuration.all_tiles().to_vec().clone(),
            hand: Hand::new(),
            opponent_tile_probabilities,
        }
    }

    /// Gets the list of tiles still hidden from this player
    pub fn hidden_tiles(&self) -> &Vec<Tile> {
        &self.hidden
    }

    /// Gets the probability map for opponent having each tile
    pub fn opponent_tile_probabilities(&self) -> &HashMap<Tile, f64> {
        &self.opponent_tile_probabilities
    }

    /// Gets the probability that the opponent has a specific tile
    pub fn opponent_tile_probability(&self, tile: Tile) -> f64 {
        self.opponent_tile_probabilities
            .get(&tile)
            .copied()
            .unwrap_or(0.0)
    }

    /// Removes a tile from the hidden list (when played or drawn by this player)
    pub fn remove_hidden_tile(&mut self, tile: Tile) {
        if let Some(pos) = self.hidden.iter().position(|&t| t == tile) {
            self.hidden.remove(pos);
        }
        // Also set opponent probability to 0 since this tile is no longer available
        self.opponent_tile_probabilities.insert(tile, 0.0);
    }

    /// Removes multiple tiles from the hidden list
    pub fn remove_hidden_tiles(&mut self, tiles: &[Tile]) {
        for tile in tiles {
            self.remove_hidden_tile(*tile);
        }
    }

    /// Updates opponent tile probabilities based on current game state
    /// This method recalculates probabilities assuming uniform distribution
    /// of remaining tiles between opponent hand and boneyard
    pub fn update_opponent_probabilities(&mut self, _boneyard_count: usize) {
        let opponent_hand_size = self.configuration.starting_hand_size(); // Assume opponent still has starting hand size

        // For tiles still hidden, calculate probability they're in opponent's hand
        // vs. still in the boneyard
        let total_unknown_tiles = self.hidden.len();

        if total_unknown_tiles > 0 {
            // Probability a hidden tile is in opponent's hand rather than boneyard
            let prob_in_opponent_hand = if total_unknown_tiles <= opponent_hand_size {
                // If there are fewer unknown tiles than opponent has, then the opponent must have all of them
                1.0
            } else {
                // Otherwise, probability is proportional to opponent hand size
                opponent_hand_size as f64 / total_unknown_tiles as f64
            };

            // Update probabilities for all hidden tiles
            for tile in &self.hidden {
                self.opponent_tile_probabilities
                    .insert(*tile, prob_in_opponent_hand);
            }
        }
    }
}

impl<'a> Player for DominoesPlayer<'a> {
    fn reset(&mut self) {
        self.hand = Hand::new();
        self.hidden = self.configuration.all_tiles().to_vec().clone();
        // Reset opponent probabilities
        for tile in self.configuration.all_tiles() {
            self.opponent_tile_probabilities.insert(*tile, 0.0);
        }
    }

    fn set_up(&mut self, state: &mut DominoesState) {
        // Draw the starting hand size number of tiles from the boneyard
        let hand_size = self.configuration.starting_hand_size();
        for _ in 0..hand_size {
            if let Some(tile) = state.draw_tile() {
                self.hand.add_tile(tile);
                self.remove_hidden_tile(tile); // Remove drawn tile from hidden
            }
        }
    }

    fn my_turn(&mut self, state: &DominoesState) -> (Action, DominoesState) {
        // TODO: Implement dominoes-specific game logic
        // Rules is available as self.configuration: self.configuration.num_players, self.configuration.variation, etc.
        // Action history is available via state.get_actions()
        // This is a stub implementation that just returns a pass action and the same state

        let rg = DominoesResponseGenerator::new();
        let rollout = DominoesRollout::new();
        let action: Option<Action> = mcts::search(state, &rg, &rollout, 1.414f32, 1000);

        match action {
            Some(action) => {
                let new_state = state.apply(&action);
                (action, new_state)
            }
            None => {
                // No actions available, pass
                let pass_action = Action::pass(self.player_id);
                (pass_action, state.clone())
            }
        }
    }

    fn has_playable_tile(&self, state: &DominoesState) -> bool {
        self.hand
            .tiles()
            .iter()
            .any(|tile| state.can_play_tile(tile, None))
    }

    fn hand(&self) -> &Hand {
        &self.hand
    }

    fn name(&self) -> &str {
        "Computer Player"
    }

    fn id(&self) -> u8 {
        self.player_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dominoes_player_creation() {
        let configuration = Configuration::default();
        let player = DominoesPlayer::new(1, &configuration);
        // Test DominoesPlayer-specific behavior
        assert_eq!(player.name(), "Computer Player");
        // Test that hidden tiles are initialized with all tiles
        assert_eq!(player.hidden_tiles().len(), configuration.set_size());
        assert_eq!(player.hidden_tiles(), configuration.all_tiles());
        // Test that hand is initially empty
        assert_eq!(player.hand.len(), 0);
        assert_eq!(player.hand.tiles().len(), 0);
    }

    #[test]
    fn test_dominoes_player_name() {
        let configuration = Configuration::default();
        let player = DominoesPlayer::new(1, &configuration);
        // Test that DominoesPlayer returns the correct name
        assert_eq!(player.name(), "Computer Player");
    }

    #[test]
    fn test_dominoes_player_implements_player_trait() {
        let configuration = Configuration::default();
        let mut player = DominoesPlayer::new(1, &configuration);
        let state = DominoesState::new(&configuration);

        // Test that my_turn method exists and returns expected types
        // Focus on DominoesPlayer's implementation, not external dependencies
        let (returned_action, new_state) = player.my_turn(&state);

        // Test DominoesPlayer's specific behavior: should return pass action in stub implementation
        assert_eq!(returned_action.player_id, 1);
        assert!(returned_action.tile_drawn.is_none());
        assert!(returned_action.tile_played.is_none());

        // Note: Since the stub implementation returns the same state, we can't test for added actions
        // This test focuses on verifying the method exists and returns correct types
        assert_eq!(new_state.whose_turn, state.whose_turn);
    }

    #[test]
    fn test_dominoes_player_configuration_access() {
        let configuration = Configuration::default();
        let player = DominoesPlayer::new(1, &configuration);

        // Test that DominoesPlayer stores the configuration reference correctly
        // This verifies our constructor works by checking initialization based on configuration
        // Verify that hidden tiles are initialized to the full set
        assert_eq!(player.hidden_tiles().len(), configuration.set_size());
        assert_eq!(player.hidden_tiles(), configuration.all_tiles());
    }

    #[test]
    fn test_hidden_tiles_management() {
        let configuration = Configuration::default();
        let mut player = DominoesPlayer::new(1, &configuration);

        // Test initial state - all tiles should be hidden
        let initial_count = player.hidden_tiles().len();
        assert_eq!(initial_count, configuration.set_size());
        assert_eq!(player.hidden_tiles(), configuration.all_tiles());

        // Test removing a single tile
        let tile_to_remove = Tile::from((0, 0));
        player.remove_hidden_tile(tile_to_remove);
        assert_eq!(player.hidden_tiles().len(), initial_count - 1);
        assert!(!player.hidden_tiles().contains(&tile_to_remove));

        // Test removing multiple tiles
        let tiles_to_remove = vec![Tile::from((0, 1)), Tile::from((1, 1)), Tile::from((2, 3))];
        player.remove_hidden_tiles(&tiles_to_remove);
        assert_eq!(player.hidden_tiles().len(), initial_count - 4); // 1 + 3 removed
        for tile in &tiles_to_remove {
            assert!(!player.hidden_tiles().contains(tile));
        }

        // Test removing a tile that doesn't exist (should not crash)
        let non_existent_tile = Tile::from((9, 9)); // Assuming this doesn't exist in default configuration
        let count_before = player.hidden_tiles().len();
        player.remove_hidden_tile(non_existent_tile);
        assert_eq!(player.hidden_tiles().len(), count_before); // No change

        // Test removing a tile that was already removed
        player.remove_hidden_tile(tile_to_remove);
        assert_eq!(player.hidden_tiles().len(), initial_count - 4); // Still the same count
    }

    #[test]
    fn test_hidden_tiles_integration_example() {
        let configuration = Configuration::default();
        let mut player = DominoesPlayer::new(1, &configuration);

        // Simulate a game scenario where:
        // 1. Player observes tiles being played by others
        // 2. Player draws tiles from boneyard

        // Initially all 28 tiles are hidden (for double-six set)
        assert_eq!(player.hidden_tiles().len(), 28);

        // Opponent plays (0,1) - player can see this, so remove from hidden
        player.remove_hidden_tile(Tile::from((0, 1)));
        assert_eq!(player.hidden_tiles().len(), 27);
        assert!(!player.hidden_tiles().contains(&Tile::from((0, 1))));

        // Player draws (2,3) from boneyard - player knows this tile, so remove from hidden
        player.remove_hidden_tile(Tile::from((2, 3)));
        assert_eq!(player.hidden_tiles().len(), 26);
        assert!(!player.hidden_tiles().contains(&Tile::from((2, 3))));

        // Multiple tiles played in sequence by different players
        let played_tiles = vec![Tile::from((1, 2)), Tile::from((3, 4)), Tile::from((5, 6))];
        player.remove_hidden_tiles(&played_tiles);
        assert_eq!(player.hidden_tiles().len(), 23);

        // Verify specific tiles are no longer hidden
        for tile in &played_tiles {
            assert!(!player.hidden_tiles().contains(tile));
        }

        // Remaining hidden tiles can be used for AI decision making
        let remaining_hidden = player.hidden_tiles().len();
        assert_eq!(remaining_hidden, 23);

        // Player can use this information to estimate probability of opponents having certain tiles
        // or to decide which tiles are still available in the boneyard
    }

    #[test]
    fn test_hand_management() {
        let configuration = Configuration::default();
        let mut player = DominoesPlayer::new(1, &configuration);

        // Test initial state - hand should be empty
        assert_eq!(player.hand.len(), 0);
        assert_eq!(player.hand.tiles().len(), 0);

        // Test adding tiles to hand using Hand methods directly
        let tile1 = Tile::from((1, 2));
        let tile2 = Tile::from((3, 4));
        player.hand.add_tile(tile1);
        player.remove_hidden_tile(tile1); // Manually remove from hidden for consistency
        assert_eq!(player.hand.len(), 1);
        assert_eq!(player.hand.tiles().len(), 1);
        assert!(player.hand.contains(&tile1));
        assert!(!player.hand.contains(&tile2));

        // Test that adding to hand removed from hidden
        assert!(!player.hidden_tiles().contains(&tile1));

        // Test adding multiple tiles
        let more_tiles = vec![tile2, Tile::from((5, 6)), Tile::from((0, 0))];
        for tile in &more_tiles {
            player.hand.add_tile(*tile);
            player.remove_hidden_tile(*tile);
        }
        assert_eq!(player.hand.len(), 4);
        assert_eq!(player.hand.tiles().len(), 4);

        // Verify all tiles are in hand and removed from hidden
        for tile in &more_tiles {
            assert!(player.hand.contains(tile));
            assert!(!player.hidden_tiles().contains(tile));
        }

        // Test removing tile from hand
        player.hand.remove_tile(&tile1);
        assert_eq!(player.hand.len(), 3);
        assert!(!player.hand.contains(&tile1));

        // Test removing tile not in hand - this will panic since Hand::remove_tile expects the tile to exist
        // So we'll skip this test for now

        // Test removing non-existent tile - skip as it would panic

        // Verify remaining tiles are still in hand
        assert!(player.hand.contains(&tile2));
        assert!(player.hand.contains(&Tile::from((5, 6))));
        assert!(player.hand.contains(&Tile::from((0, 0))));
    }

    #[test]
    fn test_hand_and_hidden_interaction() {
        let configuration = Configuration::default();
        let mut player = DominoesPlayer::new(1, &configuration);

        let initial_hidden_count = player.hidden_tiles().len();

        // Add tiles to hand - should remove from hidden
        let hand_tiles = vec![Tile::from((1, 1)), Tile::from((2, 3)), Tile::from((4, 5))];
        for tile in &hand_tiles {
            player.hand.add_tile(*tile);
            player.remove_hidden_tile(*tile);
        }

        // Verify hidden count decreased
        assert_eq!(
            player.hidden_tiles().len(),
            initial_hidden_count - hand_tiles.len()
        );

        // Verify tiles are in hand but not in hidden
        for tile in &hand_tiles {
            assert!(player.hand.contains(tile));
            assert!(!player.hidden_tiles().contains(tile));
        }

        // Remove tiles when opponents play them (not from our hand)
        let opponent_played = vec![Tile::from((0, 2)), Tile::from((1, 6)), Tile::from((3, 3))];
        player.remove_hidden_tiles(&opponent_played);

        // These should be removed from hidden but not affect our hand
        for tile in &opponent_played {
            assert!(!player.hidden_tiles().contains(tile));
            assert!(!player.hand.contains(tile)); // Not in our hand
        }

        // Our hand should still contain our tiles
        for tile in &hand_tiles {
            assert!(player.hand.contains(tile));
        }

        // Play a tile from our hand
        let played_tile = hand_tiles[0];
        player.hand.remove_tile(&played_tile);
        assert!(!player.hand.contains(&played_tile));
        assert_eq!(player.hand.len(), hand_tiles.len() - 1);
    }

    #[test]
    fn test_dominoes_player_set_up() {
        let configuration = Configuration::default(); // 2 players, Traditional, 7 tiles each
        let mut player = DominoesPlayer::new(1, &configuration);
        let mut state = DominoesState::new(&configuration);

        // Initially player has no tiles
        assert_eq!(player.hand.len(), 0);
        assert_eq!(player.hidden_tiles().len(), 28); // All tiles are hidden initially

        // Setup player - should draw 7 tiles for 2-player Traditional game
        player.set_up(&mut state);

        // Player should now have 7 tiles in hand
        assert_eq!(player.hand.len(), 7);
        // Note: We don't automatically remove tiles from hidden during set_up
        // This would need to be done manually in a real game implementation

        // Boneyard should have 21 tiles left (28 - 7)
        assert_eq!(state.boneyard.count(), 21);

        // Test with different variation
        let configuration_bergen = Configuration::new(4, rules::Variation::Bergen, 6, 6);
        let mut player_bergen = DominoesPlayer::new(1, &configuration_bergen);
        let mut state_bergen = DominoesState::new(&configuration_bergen);

        // Bergen uses 6 tiles per player regardless of player count
        player_bergen.set_up(&mut state_bergen);
        assert_eq!(player_bergen.hand.len(), 6);
        assert_eq!(state_bergen.boneyard.count(), 22); // 28 - 6 = 22
    }

    #[test]
    fn test_opponent_tile_probabilities_initialization() {
        let configuration = Configuration::default();
        let player = DominoesPlayer::new(1, &configuration);

        // Initially, all tiles should have probability 0.0 (opponent starts with empty hand)
        assert_eq!(player.opponent_tile_probabilities().len(), 28); // Double-six set

        for tile in configuration.all_tiles() {
            assert_eq!(player.opponent_tile_probability(*tile), 0.0);
        }
    }

    #[test]
    fn test_opponent_tile_probabilities_after_removal() {
        let configuration = Configuration::default();
        let mut player = DominoesPlayer::new(1, &configuration);

        // Remove a tile from hidden (e.g., when we draw it or see it played)
        let test_tile = Tile::from((1, 2));
        player.remove_hidden_tile(test_tile);

        // This tile should now have 0 probability for opponent
        assert_eq!(player.opponent_tile_probability(test_tile), 0.0);

        // Other tiles should still have probability 0.0 initially
        let other_tile = Tile::from((3, 4));
        assert_eq!(player.opponent_tile_probability(other_tile), 0.0);

        // Remove multiple tiles
        let more_tiles = vec![Tile::from((0, 0)), Tile::from((5, 6)), Tile::from((2, 3))];
        player.remove_hidden_tiles(&more_tiles);

        // All removed tiles should have 0 probability
        for tile in &more_tiles {
            assert_eq!(player.opponent_tile_probability(*tile), 0.0);
        }
    }

    #[test]
    fn test_opponent_probability_updates() {
        let configuration = Configuration::default();
        let mut player = DominoesPlayer::new(1, &configuration);

        // Simulate drawing tiles to hand (removes from hidden and sets probability to 0)
        let hand_tiles = vec![Tile::from((1, 1)), Tile::from((2, 3)), Tile::from((4, 5))];
        for tile in &hand_tiles {
            player.hand.add_tile(*tile);
            player.remove_hidden_tile(*tile);
        }

        // These tiles should have 0 probability since we have them
        for tile in &hand_tiles {
            assert_eq!(player.opponent_tile_probability(*tile), 0.0);
        }

        // Update probabilities based on game state
        let boneyard_count = 21; // Assume 21 tiles remain in boneyard
        player.update_opponent_probabilities(boneyard_count);

        // Hidden tiles should have updated probabilities
        for tile in player.hidden_tiles() {
            let prob = player.opponent_tile_probability(*tile);
            assert!(prob >= 0.0 && prob <= 1.0); // Valid probability range
        }

        // Our hand tiles should still have 0 probability
        for tile in &hand_tiles {
            assert_eq!(player.opponent_tile_probability(*tile), 0.0);
        }
    }

    #[test]
    fn test_probability_calculation_logic() {
        let configuration = Configuration::default();
        let mut player = DominoesPlayer::new(1, &configuration);

        // Setup: opponent should have 7 tiles, we have 7, boneyard has 14
        let mut state = DominoesState::new(&configuration);
        player.set_up(&mut state);

        // Manually remove the tiles in our hand from the hidden list
        // to simulate the real game behavior where we track known tiles
        let hand_tiles: Vec<Tile> = player.hand.tiles().to_vec();
        for tile in &hand_tiles {
            player.remove_hidden_tile(*tile);
        }

        // After setup and hiding our tiles, our tiles should have 0 probability
        for tile in &hand_tiles {
            assert_eq!(player.opponent_tile_probability(*tile), 0.0);
        }

        // Update probabilities
        player.update_opponent_probabilities(state.boneyard.count());

        // Hidden tiles (21 remaining) should have probability = 7/21 = 1/3
        // (7 tiles in opponent hand out of 21 unknown tiles)
        let expected_prob = 7.0 / 21.0;
        for tile in player.hidden_tiles() {
            let actual_prob = player.opponent_tile_probability(*tile);
            assert!((actual_prob - expected_prob).abs() < 0.001); // Float comparison with tolerance
        }
    }

    #[test]
    fn test_probability_integration_scenario() {
        let configuration = Configuration::default();
        let mut player = DominoesPlayer::new(1, &configuration);
        let mut state = DominoesState::new(&configuration);

        // Initial setup
        player.set_up(&mut state);

        // Simulate opponent playing a tile (we observe it)
        let opponent_played = Tile::from((0, 1));
        player.remove_hidden_tile(opponent_played);

        // This tile should have 0 probability now
        assert_eq!(player.opponent_tile_probability(opponent_played), 0.0);

        // Update probabilities based on new state
        player.update_opponent_probabilities(state.boneyard.count());

        // Verify that removed tile still has 0 probability
        assert_eq!(player.opponent_tile_probability(opponent_played), 0.0);

        // Other hidden tiles should have updated probabilities
        let remaining_hidden = player.hidden_tiles().len();
        let opponent_hand_size = 7; // Started with 7, played 1, so still 7 conceptually
        let expected_prob = opponent_hand_size as f64 / remaining_hidden as f64;

        for tile in player.hidden_tiles() {
            let actual_prob = player.opponent_tile_probability(*tile);
            assert!((actual_prob - expected_prob).abs() < 0.001);
        }
    }
}
