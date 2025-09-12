use crate::{Action, Boneyard, Layout, ZHash};
use hidden_game_player::{PlayerId, State};
use rules::{Configuration, Tile};

/// A concrete implementation of hidden_game_player::State for dominoes games
#[derive(Debug, Clone)]
pub struct DominoesState {
    /// The layout
    pub layout: Layout,
    /// The boneyard
    pub boneyard: Boneyard,
    /// Whose turn is next (player ID)
    pub whose_turn: u8,
    /// State fingerprint
    pub fingerprint: ZHash,
    /// Number of consecutive passes (typically if consecutive_passes == self.configuration.num_players, everyone has passed)
    pub consecutive_passes: u8,
    /// Whether the game is over
    pub game_is_over: bool,
    /// Player ID of the winner, or None if the game is still ongoing
    pub winner: Option<u8>,
}

impl State<Action> for DominoesState {
    fn fingerprint(&self) -> u64 {
        self.fingerprint.into()
    }

    fn whose_turn(&self) -> u8 {
        self.whose_turn
    }

    fn is_terminal(&self) -> bool {
        self.game_is_over
    }

    fn apply(&self, action: &Action) -> Self {
        assert!(false, "apply() is not yet implemented for DominoesState");
        let mut new_state = self.clone();
        if action.tile_drawn.is_some() {
            let drawn_tile = new_state.draw_tile();
            assert_eq!(
                drawn_tile, action.tile_drawn,
                "Drawn tile does not match action's drawn tile"
            );
        }
        if let Some((tile, end)) = action.tile_played {
            new_state.play_tile(tile, end);
        }
        new_state
    }
}

impl DominoesState {
    /// Creates a new dominoes game state with the specified configuration
    ///
    /// # Arguments
    /// * `configuration` - Game configuration containing players, variation, and domino set
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::DominoesState;
    /// # use rules::Configuration;
    ///
    /// let config = Configuration::default();
    /// let state = DominoesState::new(&config);
    /// assert!(!state.game_is_over);
    /// ```
    pub fn new(configuration: &Configuration) -> Self {
        Self {
            layout: Layout::new(configuration),
            boneyard: Boneyard::new(configuration),
            whose_turn: PlayerId::ALICE as u8,
            fingerprint: ZHash::default(),
            consecutive_passes: 0,
            game_is_over: false,
            winner: None,
        }
    }

    /// Checks if a tile can be played on the current layout
    ///
    /// Validates whether the specified tile can be legally placed on the layout. For empty layouts, only doubles can be played.
    /// For non-empty layouts, the tile must match at least one open end.
    ///
    /// # Arguments
    /// * `tile` - The tile to check for playability
    /// * `end` - Optional specific end value to match. If None, any matching end is acceptable
    ///
    /// # Returns
    /// `true` if the tile can be legally played, `false` otherwise
    ///
    /// # Panics
    /// Panics if the layout is empty but a specific end is provided.
    ///
    /// # Examples
    /// ```
    /// # use dominoes_state::DominoesState;
    /// # use rules::{Tile, Configuration, Variation};
    ///
    /// let config = Configuration::new(4, Variation::Traditional, 6, 6);
    /// let mut state = DominoesState::new(&config);
    /// let tile = Tile::from((1, 2));
    /// if state.can_play_tile(&tile, None) {
    ///     // Tile can be played somewhere on the layout
    /// }
    /// ```
    pub fn can_play_tile(&self, tile: &Tile, end: Option<u8>) -> bool {
        if !self.layout.is_empty() {
            let (a, b) = tile.as_tuple();
            if let Some(end) = end {
                // If an end is specified, it must match that end and the end must be open somewhere
                (a == end || b == end) && self.layout.end_counts[end as usize] > 0
            } else {
                // If no end is specified, the tile must match any of the open ends
                self.layout.end_counts[a as usize] > 0 || self.layout.end_counts[b as usize] > 0
            }
        } else {
            // If the layout is empty, a double can be played
            // TODO: Handle different variations of the game here
            assert!(
                end.is_none(),
                "An end was specified for an empty layout. Something is wrong."
            );
            tile.is_double()
        }
    }

    /// Draws and returns a tile from the boneyard
    ///
    /// Removes and returns a random tile from the boneyard. Returns `None` if the boneyard is empty.
    ///
    /// # Returns
    /// `Some(Tile)` if a tile was successfully drawn, `None` if boneyard is empty
    pub fn draw_tile(&mut self) -> Option<Tile> {
        // Note: Unlike playing a tile, drawing does not reset the consecutive passes counter because a pass could still occur
        // afterward if the boneyard is empty.

        // FIXME: This currently does not change the fingerprint, but it probably should
        self.boneyard.draw()
    }

    /// Plays a tile on the layout
    ///
    /// Places the specified tile on the layout at the given open end. Updates the layout, fingerprint, and open ends accordingly.
    ///
    /// # Arguments
    /// * `tile` - The tile to place on the layout
    /// * `end` - The specific end to attach to. Must be `None` for empty layouts
    ///
    /// # Panics
    /// Panics if the tile cannot be legally played.
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::DominoesState;
    /// # use rules::{Tile, Configuration, Variation};
    ///
    /// let config = Configuration::new(4, Variation::Traditional, 6, 6);
    /// let mut state = DominoesState::new(&config);
    /// let tile = Tile::from((1, 2));
    /// if state.can_play_tile(&tile, None) {
    ///     state.play_tile(tile, None);
    /// }
    /// ```
    pub fn play_tile(&mut self, tile: Tile, end: Option<u8>) {
        assert!(
            self.can_play_tile(&tile, end),
            "Tile {tile} cannot be played on the layout"
        );

        if let Some(matched_end) = end {
            // Find the index of a matching open end
            let (parent_index, _) = self
                .layout
                .open
                .iter_all()
                .find(|(_, values)| values.contains(&matched_end))
                .expect("No matching open end found");

            // Place the tile in the layout
            let (new_end, new_end_change) = self.layout.attach(tile, Some(*parent_index));

            // Update the fingerprint for the new tile
            self.fingerprint.add_tile(tile.into());

            // Update the fingerprint for the matched end count already decremented by add_tile.
            let matched_count = self.layout.open_count(matched_end);
            self.fingerprint
                .change_end_count(matched_end, matched_count + 1, matched_count);

            // Update the fingerprint for the new end count already incremented by add_tile.
            let new_end_count = self.layout.open_count(new_end);
            assert!(new_end_count >= new_end_change);
            self.fingerprint.change_end_count(
                new_end,
                new_end_count - new_end_change,
                new_end_count,
            );
        } else {
            // If no end specified, the tile must be a double and the layout must be empty
            assert!(
                self.layout.is_empty(),
                "Layout is not empty; must specify an end to play on"
            );

            // Place the tile in the layout
            let (new_end, new_end_change) = self.layout.attach(tile, None);

            // Update the fingerprint for the new tile
            self.fingerprint.add_tile(tile.into());

            // Update the fingerprint for the new end count
            assert!(self.layout.open_count(new_end) == new_end_change);
            self.fingerprint
                .change_end_count(new_end, 0, new_end_change);
        }
        self.update_consecutive_passes(false); // Reset consecutive passes because a tile was played
    }

    /// Marks the game as over and optionally declares a winner (or a draw)
    ///
    /// This method sets the internal `done` flag to true and records the winner (if any). Once called, `game_is_over` will
    /// be true and `winner` will contain the ID of the winning player (or `None` if it is a draw).
    ///
    /// # Arguments
    /// * `winner` - Player ID of the winner, `None` if it is a draw.
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::DominoesState;
    /// # use rules::Configuration;
    ///
    /// let config = Configuration::default();
    /// let mut state = DominoesState::new(&config);
    ///
    /// // Initially game is not over
    /// assert!(!state.game_is_over);
    /// assert_eq!(state.winner, None);
    ///
    /// // End game with a winner (player ID 0)
    /// state.mark_game_over(Some(0));
    /// assert!(state.game_is_over);
    /// assert_eq!(state.winner, Some(0));
    ///
    /// // Can also end in a draw
    /// let mut state2 = DominoesState::new(&config);
    /// state2.mark_game_over(None);
    /// assert!(state2.game_is_over);
    /// assert_eq!(state2.winner, None);
    /// ```
    /// # Important Note
    /// This method does not automatically end the game. It only updates the game state. Game state update logic should call
    /// `mark_game_over()` when appropriate, and game control logic should check `game_is_over` and `winner` to determine if the game is
    /// over and who the winner is.
    pub fn mark_game_over(&mut self, winner: Option<u8>) {
        self.game_is_over = true;
        self.winner = winner;
    }

    /// Records a pass
    ///
    /// Increments the consecutive passes counter, which is used to track how players have passed in succession. When
    /// `consecutive_passes` equals the number of players in the game, it typically indicates that the game should end due to all
    /// players being unable to play.
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::DominoesState;
    /// # use rules::Configuration;
    ///
    /// let config = Configuration::default();
    /// let mut state = DominoesState::new(&config);
    ///
    /// // Initially no passes
    /// assert_eq!(state.consecutive_passes, 0);
    ///
    /// // Record a pass
    /// state.pass();
    /// assert_eq!(state.consecutive_passes, 1);
    ///
    /// // Record another pass
    /// state.pass();
    /// assert_eq!(state.consecutive_passes, 2);
    ///
    /// // Playing a tile resets the counter
    /// let tile = rules::Tile::from((6, 6));
    /// if state.can_play_tile(&tile, None) {
    ///     state.play_tile(tile, None);
    ///     assert_eq!(state.consecutive_passes, 0);
    /// }
    /// ```
    pub fn pass(&mut self) {
        self.update_consecutive_passes(true);
    }

    // Increments the consecutive passes counter, or resets it
    fn update_consecutive_passes(&mut self, increment: bool) {
        // FIXME: This currently does not change the fingerprint, but it probably should
        self.consecutive_passes = if increment {
            self.consecutive_passes + 1
        } else {
            0
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dominoes_dominoes_state_initialization() {
        let configuration = Configuration::default();
        let state = DominoesState::new(&configuration);
        assert!(!state.game_is_over);
        assert_eq!(state.winner, None);
        assert_eq!(state.whose_turn(), 0); // PlayerId::ALICE as u8
        assert_eq!(state.consecutive_passes, 0);
        assert_eq!(state.fingerprint(), 0);
    }

    #[test]
    fn test_state_access() {
        let configuration = Configuration::default();
        let state = DominoesState::new(&configuration);
        // Test DominoesState functionality
        let _boneyard = &state.boneyard;
        assert_eq!(state.boneyard.count(), 28);

        // Test that boneyard count matches configuration tile count
        assert_eq!(state.boneyard.count(), configuration.set_size());
    }

    #[test]
    fn test_boneyard_integration() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Test boneyard is properly initialized
        assert_eq!(state.boneyard.count(), 28); // Standard double-six set
        assert!(state.boneyard.count() > 0); // Not empty

        // Test drawing from boneyard
        let tile = state.draw_tile();
        assert!(tile.is_some());
        assert_eq!(state.boneyard.count(), 27);

        // Test boneyard access
        let boneyard_ref = &state.boneyard;
        assert_eq!(boneyard_ref.count(), 27);
    }

    #[test]
    fn test_custom_set() {
        let configuration = Configuration::new(2, rules::Variation::Traditional, 3, 7);
        let state = DominoesState::new(&configuration);
        // Test DominoesState with custom configuration - focus on DominoesState behavior
        assert_eq!(state.boneyard.count(), 10); // For n=3: (n+1)*(n+2)/2 = 4*5/2 = 10 tiles
        assert_eq!(state.boneyard.count(), configuration.set_size());

        // Test with smaller set
        let small_configuration = Configuration::new(2, rules::Variation::Traditional, 1, 3);
        let small_state = DominoesState::new(&small_configuration);
        assert_eq!(small_state.boneyard.count(), 3); // For n=1: 3 tiles
        assert_eq!(small_state.boneyard.count(), small_configuration.set_size());
    }

    #[test]
    fn test_configuration() {
        // Test DominoesState works with different rules - focus on DominoesState behavior
        let configuration = Configuration::new(4, rules::Variation::AllFives, 9, 12);
        let state = DominoesState::new(&configuration);
        assert_eq!(state.boneyard.count(), 55); // Double-nine: 10*11/2 = 55
        assert_eq!(state.boneyard.count(), configuration.set_size());

        // Test default configuration creates proper DominoesState
        let default_configuration = Configuration::default();
        let default_state = DominoesState::new(&default_configuration);
        assert_eq!(default_state.boneyard.count(), 28);
        assert!(!default_state.game_is_over);
    }

    #[test]
    fn test_can_play_tile_empty_layout() {
        let configuration = Configuration::default();
        let state = DominoesState::new(&configuration);

        // Empty layout should only accept doubles
        let double_tile = Tile::from((3, 3));
        let non_double_tile = Tile::from((1, 2));

        assert!(state.can_play_tile(&double_tile, None));
        assert!(!state.can_play_tile(&non_double_tile, None));
    }

    #[test]
    #[should_panic(expected = "An end was specified for an empty layout")]
    fn test_can_play_tile_empty_layout_with_end_panics() {
        let configuration = Configuration::default();
        let state = DominoesState::new(&configuration);

        let tile = Tile::from((3, 3));
        // Should panic when specifying an end for empty layout
        state.can_play_tile(&tile, Some(3));
    }

    #[test]
    fn test_can_play_tile_non_empty_layout() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Place initial double
        let initial_tile = Tile::from((3, 3));
        state.play_tile(initial_tile, None);

        // Now test tiles that can and cannot be played
        let matching_tile = Tile::from((3, 5));
        let non_matching_tile = Tile::from((1, 2));

        assert!(state.can_play_tile(&matching_tile, None));
        assert!(state.can_play_tile(&matching_tile, Some(3)));
        assert!(!state.can_play_tile(&non_matching_tile, None));
        assert!(!state.can_play_tile(&non_matching_tile, Some(1)));
    }

    #[test]
    fn test_draw_tile() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Initially should have 28 tiles
        assert_eq!(state.boneyard.count(), 28);

        // Draw a tile
        let drawn_tile = state.draw_tile();
        assert!(drawn_tile.is_some());
        assert_eq!(state.boneyard.count(), 27);

        // Draw multiple tiles
        for i in 0..26 {
            let tile = state.draw_tile();
            assert!(tile.is_some(), "Failed to draw tile at iteration {}", i);
            assert_eq!(state.boneyard.count(), 26 - i);
        }

        // Draw last tile
        let last_tile = state.draw_tile();
        assert!(last_tile.is_some());
        assert_eq!(state.boneyard.count(), 0);

        // Try to draw from empty boneyard
        let empty_draw = state.draw_tile();
        assert!(empty_draw.is_none());
        assert_eq!(state.boneyard.count(), 0);
    }

    #[test]
    fn test_play_tile_empty_layout() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        let double_tile = Tile::from((4, 4));

        // Should be able to play double on empty layout
        assert!(state.can_play_tile(&double_tile, None));
        state.play_tile(double_tile, None);

        // Layout should no longer be empty
        assert!(!state.layout.is_empty());

        // Should have open ends for value 4
        assert!(state.layout.end_counts[4] > 0);
    }

    #[test]
    fn test_play_tile_non_empty_layout() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Start with a double
        let initial_tile = Tile::from((2, 2));
        state.play_tile(initial_tile, None);

        // Play a matching tile
        let second_tile = Tile::from((2, 5));
        assert!(state.can_play_tile(&second_tile, Some(2)));
        state.play_tile(second_tile, Some(2));

        // Should now have open ends for 2 and 5
        assert!(state.layout.end_counts[2] > 0);
        assert!(state.layout.end_counts[5] > 0);

        // Play another tile
        let third_tile = Tile::from((1, 5));
        assert!(state.can_play_tile(&third_tile, Some(5)));
        state.play_tile(third_tile, Some(5));

        // Should now have open ends for 2 and 1
        assert!(state.layout.end_counts[2] > 0);
        assert!(state.layout.end_counts[1] > 0);
    }

    #[test]
    #[should_panic(expected = "cannot be played on the layout")]
    fn test_play_tile_invalid_move() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Try to play non-double on empty layout
        let invalid_tile = Tile::from((1, 2));
        state.play_tile(invalid_tile, None);
    }

    #[test]
    #[should_panic(expected = "Layout is not empty; must specify an end to play on")]
    fn test_play_tile_no_end_specified_on_non_empty_layout() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Place initial tile
        let initial_tile = Tile::from((3, 3));
        state.play_tile(initial_tile, None);

        // Try to play without specifying end on non-empty layout
        let second_tile = Tile::from((3, 4));
        state.play_tile(second_tile, None);
    }

    #[test]
    fn test_state_trait_implementation() {
        let configuration = Configuration::default();
        let state = DominoesState::new(&configuration);

        // Test State trait methods
        assert_eq!(state.whose_turn(), 0); // PlayerId::ALICE as u8

        // Test fingerprint (should be valid u64)
        let fingerprint = state.fingerprint();
        assert!(fingerprint == 0); // Starting state is always 0
    }

    #[test]
    fn test_fingerprint_updates_on_play_tile() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        let initial_fingerprint = state.fingerprint();

        // Play a tile
        let tile = Tile::from((3, 3));
        state.play_tile(tile, None);

        let after_play_fingerprint = state.fingerprint();

        // Fingerprint should change after playing a tile
        assert_ne!(initial_fingerprint, after_play_fingerprint);
    }

    #[test]
    fn test_complex_tile_sequence() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Play a sequence of tiles to test complex interactions
        let tiles = vec![
            (Tile::from((6, 6)), None),    // Start with double-6
            (Tile::from((3, 6)), Some(6)), // Play 6-3 on 6
            (Tile::from((1, 6)), Some(6)), // Play 6-1 on other 6
            (Tile::from((3, 4)), Some(3)), // Play 3-4 on 3
            (Tile::from((1, 2)), Some(1)), // Play 1-2 on 1
        ];

        for (tile, end) in tiles {
            assert!(
                state.can_play_tile(&tile, end),
                "Cannot play tile {:?} with end {:?}",
                tile,
                end
            );
            state.play_tile(tile, end);
        }

        // Should have open ends for 4 and 2
        assert!(state.layout.end_counts[4] > 0);
        assert!(state.layout.end_counts[2] > 0);

        // Test that non-matching tiles cannot be played
        let invalid_tile = Tile::from((0, 5));
        assert!(!state.can_play_tile(&invalid_tile, None));
        assert!(!state.can_play_tile(&invalid_tile, Some(4)));
        assert!(!state.can_play_tile(&invalid_tile, Some(2)));
    }

    #[test]
    fn test_end_game_with_winner() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Initially game should not be over
        assert!(!state.game_is_over);
        assert_eq!(state.winner, None);

        // End game with Alice as winner
        state.mark_game_over(Some(0)); // PlayerId::ALICE as u8

        assert!(state.game_is_over);
        assert_eq!(state.winner, Some(0));
    }

    #[test]
    fn test_end_game_without_winner() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // End game in a draw (no winner)
        state.mark_game_over(None);

        assert!(state.game_is_over);
        assert_eq!(state.winner, None);
    }

    #[test]
    fn test_end_game_multiple_calls() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // End game with Alice
        state.mark_game_over(Some(0)); // PlayerId::ALICE as u8
        assert!(state.game_is_over);
        assert_eq!(state.winner, Some(0));

        // Call mark_game_over again with different winner
        state.mark_game_over(Some(1)); // PlayerId::BOB as u8
        assert!(state.game_is_over);
        assert_eq!(state.winner, Some(1)); // Should update to new winner

        // Call mark_game_over with no winner
        state.mark_game_over(None);
        assert!(state.game_is_over);
        assert_eq!(state.winner, None); // Should update to no winner
    }

    #[test]
    fn test_end_game_during_active_game() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Play some tiles first
        let tile1 = Tile::from((3, 3));
        state.play_tile(tile1, None);

        let tile2 = Tile::from((3, 5));
        state.play_tile(tile2, Some(3));

        // Game should still be active
        assert!(!state.game_is_over);

        // End game abruptly
        state.mark_game_over(Some(0)); // PlayerId::ALICE as u8

        // Should be over regardless of game state
        assert!(state.game_is_over);
        assert_eq!(state.winner, Some(0));
    }

    #[test]
    fn test_pass_increments_counter() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Initially no passes
        assert_eq!(state.consecutive_passes, 0);

        // Record first pass
        state.pass();
        assert_eq!(state.consecutive_passes, 1);

        // Record second pass
        state.pass();
        assert_eq!(state.consecutive_passes, 2);

        // Record third pass
        state.pass();
        assert_eq!(state.consecutive_passes, 3);
    }

    #[test]
    fn test_pass_then_play_resets_counter() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Record some passes
        state.pass();
        state.pass();
        assert_eq!(state.consecutive_passes, 2);

        // Play a tile (should reset counter)
        let tile = Tile::from((4, 4));
        state.play_tile(tile, None);
        assert_eq!(state.consecutive_passes, 0);

        // Pass again after playing
        state.pass();
        assert_eq!(state.consecutive_passes, 1);
    }

    #[test]
    fn test_multiple_pass_play_cycles() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Cycle 1: Pass then play
        state.pass();
        state.pass();
        assert_eq!(state.consecutive_passes, 2);

        let tile1 = Tile::from((6, 6));
        state.play_tile(tile1, None);
        assert_eq!(state.consecutive_passes, 0);

        // Cycle 2: Pass then play again
        state.pass();
        assert_eq!(state.consecutive_passes, 1);

        let tile2 = Tile::from((5, 6));
        state.play_tile(tile2, Some(6));
        assert_eq!(state.consecutive_passes, 0);

        // Cycle 3: Multiple passes
        state.pass();
        state.pass();
        state.pass();
        assert_eq!(state.consecutive_passes, 3);
    }

    #[test]
    fn test_pass_after_draw_tile() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Draw a tile (doesn't reset pass counter)
        let _drawn_tile = state.draw_tile();

        // Pass (should increment normally)
        state.pass();
        assert_eq!(state.consecutive_passes, 1);

        // Draw another tile and pass again
        let _drawn_tile2 = state.draw_tile();
        state.pass();
        assert_eq!(state.consecutive_passes, 2);
    }

    #[test]
    fn test_pass_counter_with_game_ending() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Record passes equal to number of players
        state.pass(); // Player 1 passes
        state.pass(); // Player 2 passes

        // In a 2-player game, this might indicate game should end
        assert_eq!(state.consecutive_passes as usize, configuration.num_players);

        // But the pass method itself doesn't end the game
        assert!(!state.game_is_over);

        // Game logic would need to check this condition and call mark_game_over
        if state.consecutive_passes as usize >= configuration.num_players {
            state.mark_game_over(None); // End in draw due to all players passing
        }

        assert!(state.game_is_over);
        assert_eq!(state.winner, None);
    }

    #[test]
    fn test_pass_preserves_other_state() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Play a tile first to establish some game state
        let tile = Tile::from((2, 2));
        state.play_tile(tile, None);

        let initial_fingerprint = state.fingerprint();
        let initial_boneyard_count = state.boneyard.count();
        let initial_turn = state.whose_turn();

        // Pass should only affect consecutive_passes
        state.pass();

        assert_eq!(state.consecutive_passes, 1);
        assert_eq!(state.fingerprint(), initial_fingerprint); // Fingerprint unchanged
        assert_eq!(state.boneyard.count(), initial_boneyard_count); // Boneyard unchanged
        assert_eq!(state.whose_turn(), initial_turn); // Turn unchanged
        assert!(!state.game_is_over); // Game state unchanged
    }

    #[test]
    fn test_pass_and_end_game_interaction() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Record some passes
        state.pass();
        state.pass();
        assert_eq!(state.consecutive_passes, 2);

        // End the game
        state.mark_game_over(Some(0)); // PlayerId::ALICE as u8

        // Game should be over, but pass counter should remain
        assert!(state.game_is_over);
        assert_eq!(state.winner, Some(0));
        assert_eq!(state.consecutive_passes, 2); // Counter preserved

        // Additional passes after game ends (edge case)
        state.pass();
        assert_eq!(state.consecutive_passes, 3);
        assert!(state.game_is_over); // Still over
    }

    #[test]
    fn test_is_terminal_initial_state() {
        let configuration = Configuration::default();
        let state = DominoesState::new(&configuration);

        // New game should not be terminal
        assert!(!state.is_terminal());
    }

    #[test]
    fn test_is_terminal_after_game_over() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Initially not terminal
        assert!(!state.is_terminal());

        // Mark game as over with winner
        state.mark_game_over(Some(0));
        assert!(state.is_terminal());

        // Mark game as over without winner (draw)
        let mut state2 = DominoesState::new(&configuration);
        state2.mark_game_over(None);
        assert!(state2.is_terminal());
    }

    #[test]
    fn test_is_terminal_during_gameplay() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Play some tiles - game should remain non-terminal
        let tile1 = Tile::from((3, 3));
        state.play_tile(tile1, None);
        assert!(!state.is_terminal());

        let tile2 = Tile::from((3, 5));
        state.play_tile(tile2, Some(3));
        assert!(!state.is_terminal());

        // Draw tiles - game should remain non-terminal
        state.draw_tile();
        assert!(!state.is_terminal());

        // Record passes - game should remain non-terminal
        state.pass();
        state.pass();
        assert!(!state.is_terminal());
    }

    #[test]
    fn test_is_terminal_consistency() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Test consistency for non-terminal state
        assert!(!state.is_terminal());
        assert!(!state.is_terminal()); // Multiple calls should be consistent

        // Mark as terminal and test consistency
        state.mark_game_over(Some(1));
        assert!(state.is_terminal());
        assert!(state.is_terminal()); // Multiple calls should be consistent
        assert!(state.is_terminal()); // Still consistent
    }

    #[test]
    fn test_is_terminal_matches_game_is_over() {
        let configuration = Configuration::default();
        let mut state = DominoesState::new(&configuration);

        // Initially both should be false
        assert_eq!(state.is_terminal(), state.game_is_over);
        assert!(!state.is_terminal());

        // After marking game over, both should be true
        state.mark_game_over(Some(0));
        assert_eq!(state.is_terminal(), state.game_is_over);
        assert!(state.is_terminal());

        // Test with different winner scenarios
        let mut state2 = DominoesState::new(&configuration);
        state2.mark_game_over(None); // Draw
        assert_eq!(state2.is_terminal(), state2.game_is_over);
        assert!(state2.is_terminal());
    }
}
