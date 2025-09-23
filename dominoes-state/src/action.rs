//! Management of player actions
//!
//! This module defines the `Action` and `History` structs for tracking player actions and game history in a dominoes game.

use std::fmt::Display;

use rules::{self, Tile};

/// Represents an action taken by a player
///
/// An action captures what a player did during their turn, including any tiles drawn from the boneyard and any tiles played on the
/// layout.
///
/// # Examples
/// ```rust
/// # use dominoes_state::Action;
/// # use rules::Tile;
///
/// // Player draws a tile
/// let draw_action = Action::draw(0, Tile::from((1, 2)));
///
/// // Player plays a tile
/// let play_action = Action::play(1, Tile::from((3, 4)), Some(3));
///
/// // Player passes their turn
/// let pass_action = Action::pass(0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    /// The ID of the player who took this action
    pub player_id: u8,
    /// The tile drawn from the boneyard during this action, if any
    pub tile_drawn: Option<Tile>,
    /// The tile that was played on the layout during this action, if any
    pub tile_played: Option<(Tile, Option<u8>)>,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            player_id: 0,
            tile_drawn: None,
            tile_played: None,
        }
    }
}

impl Action {
    /// Creates a new action with the specified components
    ///
    /// This is the most general constructor that allows specifying all components of an action.
    ///
    /// # Arguments
    /// * `player_id` - The ID of the player taking this action
    /// * `tile_drawn` - Optional tile drawn from boneyard
    /// * `tile_played` - Optional tile played on layout
    ///
    /// # Returns
    /// A new Action instance
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Action;
    /// # use rules::Tile;
    ///
    /// let drawn_tile = Tile::from((1, 2));
    /// let played_tile = Tile::from((3, 4));
    ///
    /// // Action where player draws and plays in same turn
    /// let action = Action::new(0, Some(drawn_tile), Some((played_tile, Some(3))));
    ///
    /// // Action where player only draws
    /// let action = Action::new(0, Some(drawn_tile), None);
    /// ```
    pub fn new(player_id: u8, tile_drawn: Option<Tile>, tile_played: Option<(Tile, Option<u8>)>) -> Self {
        Self {
            player_id,
            tile_drawn,
            tile_played,
        }
    }

    /// Creates an action representing drawing a single tile without playing
    ///
    /// This is used when a player draws a tile from the boneyard.
    ///
    /// # Arguments
    /// * `player_id` - The ID of the player drawing the tile
    /// * `tile` - The tile drawn from the boneyard
    ///
    /// # Returns
    /// A new Action representing a draw-only turn
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Action;
    /// # use rules::Tile;
    ///
    /// let tile = Tile::from((2, 5));
    /// let action = Action::draw(1, tile);
    /// assert_eq!(action.tile_drawn, Some(tile));
    /// assert_eq!(action.tile_played, None);
    /// ```
    pub fn draw(player_id: u8, tile: Tile) -> Self {
        Self {
            player_id,
            tile_drawn: Some(tile),
            tile_played: None,
        }
    }

    /// Creates an action representing playing a tile without drawing
    ///
    /// This is used when a player plays a tile from their hand without needing to draw from the boneyard first.
    ///
    /// # Arguments
    /// * `player_id` - The ID of the player playing the tile
    /// * `tile` - The tile being played on the layout
    ///
    /// # Returns
    /// A new Action representing a play-only turn
    ///
    /// # Panics
    /// This function will panic if the tile is not a double and no end value is provided.
    /// This function will also panic if the provided end does not match either side of the tile.
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Action;
    /// # use rules::Tile;
    ///
    /// let tile = Tile::from((6, 6));
    /// let action = Action::play(0, tile, Some(6));
    /// assert_eq!(action.tile_drawn, None);
    /// assert_eq!(action.tile_played, Some((tile, Some(6))));
    /// ```
    pub fn play(player_id: u8, tile: Tile, end: Option<u8>) -> Self {
        assert!(end.is_none() || end == Some(tile.as_tuple().0) || end == Some(tile.as_tuple().1));
        assert!(end.is_some() || tile.is_double());
        Self {
            player_id,
            tile_drawn: None,
            tile_played: Some((tile, end)),
        }
    }

    /// Creates an action representing passing the turn
    ///
    /// This is used when a player passes their turn without taking any other actions.
    ///
    /// # Arguments
    /// * `player_id` - The ID of the player passing their turn
    ///
    /// # Returns
    /// A new Action representing a passed turn
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Action;
    ///
    /// let action = Action::pass(1);
    /// assert_eq!(action.tile_drawn, None);
    /// assert_eq!(action.tile_played, None);
    /// assert_eq!(action.player_id, 1);
    /// ```
    pub fn pass(player_id: u8) -> Self {
        Self {
            player_id,
            tile_drawn: None,
            tile_played: None,
        }
    }

    /// Checks if the action is a pass (no tiles drawn or played)
    ///
    /// # Returns
    /// `true` if the action is a pass, `false` otherwise
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Action;
    ///
    /// let action = Action::pass(1);
    /// assert!(action.is_pass());
    /// ```
    pub fn is_pass(&self) -> bool {
        self.tile_drawn.is_none() && self.tile_played.is_none()
    }

    /// Checks if the action involves drawing a tile
    ///
    /// # Returns
    /// `true` if a tile was drawn, `false` otherwise
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Action;
    /// # use rules::Tile;
    ///
    /// let tile = Tile::from((2, 5));
    /// let action = Action::draw(1, tile);
    /// assert!(action.is_draw());
    /// ```
    pub fn is_draw(&self) -> bool {
        self.tile_drawn.is_some()
    }

    /// Checks if the action involves playing a tile
    ///
    /// # Returns
    /// `true` if a tile was played, `false` otherwise
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Action;
    /// # use rules::Tile;
    /// let tile = Tile::from((6, 6));
    /// let action = Action::play(0, tile, Some(6));
    /// assert!(action.is_play());
    /// ```
    pub fn is_play(&self) -> bool {
        self.tile_played.is_some()
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player {}: ", self.player_id)?;
        if let Some(tile) = &self.tile_drawn {
            write!(f, "draw {tile}")?;
        }
        if let Some((tile, end)) = &self.tile_played {
            if let Some(n) = *end {
                write!(f, "play {tile} on {n}")?;
            } else {
                write!(f, "play {tile}")?;
            }
        }
        if self.tile_drawn.is_none() && self.tile_played.is_none() {
            write!(f, "pass")?;
        }
        Ok(())
    }
}

/// A history of actions
///
/// The History struct maintains a chronological record of all actions taken by all players during a game. This can be used for
/// game replay, analysis, or implementing undo functionality.
///
/// # Examples
/// ```rust
/// # use dominoes_state::{History, Action};
///
/// let mut history = History::new();
/// history.add_action(Action::pass(0));
/// history.add_action(Action::pass(1));
///
/// assert_eq!(history.get_actions().len(), 2);
/// ```
#[derive(Default)]
pub struct History {
    /// Vector storing all actions in chronological order
    actions: Vec<Action>,
}

impl History {
    /// Creates a new empty history
    ///
    /// # Returns
    /// A new History instance with no actions recorded
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::History;
    ///
    /// let history = History::new();
    /// assert!(history.get_actions().is_empty());
    /// ```
    pub fn new() -> Self {
        Self { actions: Vec::new() }
    }

    /// Adds an action to the game history
    ///
    /// Actions are added in chronological order and cannot be removed once added.
    ///
    /// # Arguments
    /// * `action` - The action to add to the history
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::{History, Action};
    ///
    /// let mut history = History::new();
    /// let action = Action::pass(0);
    /// history.add_action(action);
    /// assert_eq!(history.get_actions().len(), 1);
    /// ```
    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    /// Gets all actions taken during the game
    ///
    /// Returns a reference to the complete list of actions in chronological order.
    ///
    /// # Returns
    /// A reference to the vector of all actions
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::{History, Action};
    ///
    /// let mut history = History::new();
    /// history.add_action(Action::pass(0));
    /// history.add_action(Action::pass(1));
    ///
    /// let actions = history.get_actions();
    /// for (i, action) in actions.iter().enumerate() {
    ///     println!("Turn {}: Player {} {:?}", i + 1, action.player_id, action);
    /// }
    /// ```
    pub fn get_actions(&self) -> &Vec<Action> {
        &self.actions
    }

    /// Gets the last action taken (if any)
    ///
    /// Returns a reference to the most recent action, or None if no actions have been taken yet.
    ///
    /// # Returns
    /// An Option containing a reference to the last action, or None if history is empty
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::{History, Action};
    ///
    /// let mut history = History::new();
    /// history.add_action(Action::pass(0));
    ///
    /// if let Some(last_action) = history.get_last_action() {
    ///     println!("Last action was by player {}", last_action.player_id);
    /// }
    /// ```
    pub fn get_last_action(&self) -> Option<&Action> {
        self.actions.last()
    }

    /// Gets all actions taken by a specific player
    ///
    /// Returns a vector of references to all actions taken by the specified player, in chronological order.
    ///
    /// # Arguments
    /// * `player_id` - The ID of the player whose actions to retrieve
    ///
    /// # Returns
    /// A vector of references to the player's actions
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::{History, Action};
    ///
    /// let mut history = History::new();
    /// history.add_action(Action::pass(0));
    /// history.add_action(Action::pass(1));
    /// history.add_action(Action::pass(0));
    ///
    /// let player_actions = history.get_player_actions(0);
    /// println!("Player 0 took {} actions", player_actions.len());
    /// ```
    pub fn get_player_actions(&self, player_id: u8) -> Vec<&Action> {
        self.actions.iter().filter(|action| action.player_id == player_id).collect()
    }

    /// Gets all actions that follow the last action by the specified player
    ///
    /// This is useful for determining what has happened since a particular player's last turn. Returns an empty vector if the
    /// player has no actions or if the player's last action is the final action in the history.
    ///
    /// # Arguments
    /// * `player_id` - The ID of the player to check
    ///
    /// # Returns
    /// A vector of references to actions that occurred after the player's last action
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::{History, Action};
    ///
    /// let mut history = History::new();
    /// history.add_action(Action::pass(0));
    /// history.add_action(Action::pass(1));
    /// history.add_action(Action::pass(1));
    ///
    /// // Get what happened since player 0's last turn
    /// let actions_since = history.get_actions_after_player(0);
    /// for action in actions_since {
    ///     println!("Player {} acted after player 0", action.player_id);
    /// }
    /// ```
    pub fn get_actions_after_player(&self, player_id: u8) -> Vec<&Action> {
        // Find the index of the last action by the specified player
        if let Some(last_player_index) = self.actions.iter().rposition(|action| action.player_id == player_id) {
            // Return all actions after that index
            self.actions.iter().skip(last_player_index + 1).collect()
        } else {
            // Player has no actions, return empty vector
            Vec::new()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rules::Tile;

    // Tests for Action struct
    #[test]
    fn test_action_new() {
        let tile1 = Tile::from((1, 2));
        let tile2 = Tile::from((3, 4));
        let end2 = Some(3);

        let action = Action::new(1, Some(tile1), Some((tile2, end2)));

        assert_eq!(action.player_id, 1);
        assert_eq!(action.tile_drawn, Some(tile1));
        assert_eq!(action.tile_played, Some((tile2, end2)));

        // Test all combinations of Some/None for drawn and played tiles
        let action1 = Action::new(0, None, None);
        assert_eq!(action1.tile_drawn, None);
        assert_eq!(action1.tile_played, None);

        let action2 = Action::new(0, Some(tile1), None);
        assert_eq!(action2.tile_drawn, Some(tile1));
        assert_eq!(action2.tile_played, None);

        let action3 = Action::new(0, None, Some((tile2, end2)));
        assert_eq!(action3.tile_drawn, None);
        assert_eq!(action3.tile_played, Some((tile2, end2)));
    }

    #[test]
    fn test_action_draw_one() {
        let tile = Tile::from((5, 6));
        let action = Action::draw(0, tile);

        assert_eq!(action.player_id, 0);
        assert_eq!(action.tile_drawn, Some(tile));
        assert_eq!(action.tile_played, None);
    }

    #[test]
    fn test_action_play_tile() {
        let tile = Tile::from((2, 3));
        let end = Some(2);
        let action = Action::play(1, tile, end);

        assert_eq!(action.player_id, 1);
        assert_eq!(action.tile_drawn, None);
        assert_eq!(action.tile_played, Some((tile, end)));
    }

    #[test]
    fn test_action_pass() {
        let action = Action::pass(0);

        assert_eq!(action.player_id, 0);
        assert_eq!(action.tile_drawn, None);
        assert_eq!(action.tile_played, None);
    }

    #[test]
    fn test_action_clone() {
        let tile = Tile::from((1, 1));
        let end = Some(1);
        let action1 = Action::play(1, tile, end);
        let action2 = action1.clone();

        assert_eq!(action1.player_id, action2.player_id);
        assert_eq!(action1.tile_drawn, action2.tile_drawn);
        assert_eq!(action1.tile_played, action2.tile_played);
    }

    #[test]
    fn test_action_debug() {
        let tile = Tile::from((4, 5));
        let action = Action::draw(1, tile);
        let debug_string = format!("{:?}", action);

        assert!(debug_string.contains("Action"));
        assert!(debug_string.contains("player_id"));
        assert!(debug_string.contains("tile_drawn"));
        assert!(debug_string.contains("tile_played"));
    }

    // Tests for History struct
    #[test]
    fn test_history_default() {
        let history = History::default();

        assert!(history.get_actions().is_empty());
        assert_eq!(history.get_last_action(), None);

        // Test equivalence with History::new()
        let history_new = History::new();
        assert_eq!(history.get_actions().len(), history_new.get_actions().len());
        assert_eq!(history.get_last_action(), history_new.get_last_action());
    }

    #[test]
    fn test_history_add_action() {
        let mut history = History::default();
        let action = Action::pass(0);

        history.add_action(action);

        assert_eq!(history.get_actions().len(), 1);
        assert_eq!(history.get_last_action().unwrap().player_id, 0);
    }

    #[test]
    fn test_history_get_actions() {
        let mut history = History::default();
        let action1 = Action::pass(0);
        let action2 = Action::pass(1);

        history.add_action(action1);
        history.add_action(action2);

        let actions = history.get_actions();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].player_id, 0);
        assert_eq!(actions[1].player_id, 1);
    }

    #[test]
    fn test_history_get_last_action() {
        let mut history = History::default();

        // Empty history
        assert_eq!(history.get_last_action(), None);

        // Add one action
        let action1 = Action::pass(0);
        history.add_action(action1);
        assert_eq!(history.get_last_action().unwrap().player_id, 0);

        // Add another action
        let action2 = Action::pass(1);
        history.add_action(action2);
        assert_eq!(history.get_last_action().unwrap().player_id, 1);
    }

    #[test]
    fn test_history_get_player_actions() {
        let mut history = History::default();
        let tile1 = Tile::from((1, 2));
        let tile2 = Tile::from((3, 4));

        // Add actions for different players
        history.add_action(Action::play(0, tile1, Some(1)));
        history.add_action(Action::draw(1, tile2));
        history.add_action(Action::pass(0));
        history.add_action(Action::pass(1));

        // Get actions for player 0
        let player0_actions = history.get_player_actions(0);
        assert_eq!(player0_actions.len(), 2);
        assert_eq!(player0_actions[0].tile_played, Some((tile1, Some(1))));
        assert_eq!(player0_actions[1].tile_played, None);

        // Get actions for player 1
        let player1_actions = history.get_player_actions(1);
        assert_eq!(player1_actions.len(), 2);
        assert_eq!(player1_actions[0].tile_drawn, Some(tile2));
        assert_eq!(player1_actions[1].tile_drawn, None);
    }

    #[test]
    fn test_history_get_player_actions_no_matches() {
        let mut history = History::default();
        history.add_action(Action::pass(0));
        history.add_action(Action::pass(1));

        // Player 2 has no actions
        let player2_actions = history.get_player_actions(2);
        assert!(player2_actions.is_empty());
    }

    #[test]
    fn test_history_get_actions_after_player() {
        let mut history = History::default();
        let tile1 = Tile::from((1, 1));
        let tile2 = Tile::from((2, 2));
        let tile3 = Tile::from((3, 3));

        // Add sequence of actions: P0, P1, P0, P1, P0
        history.add_action(Action::play(0, tile1, None));  // index 0
        history.add_action(Action::draw(1, tile2));   // index 1
        history.add_action(Action::pass(0));              // index 2
        history.add_action(Action::play(1, tile3, None));  // index 3
        history.add_action(Action::pass(0));              // index 4

        // Get actions after player 0's last action (should be empty)
        let after_player0 = history.get_actions_after_player(0);
        assert!(after_player0.is_empty());

        // Get actions after player 1's last action (should have 1 action)
        let after_player1 = history.get_actions_after_player(1);
        assert_eq!(after_player1.len(), 1);
        assert_eq!(after_player1[0].player_id, 0);
    }

    #[test]
    fn test_history_get_actions_after_player_no_player_actions() {
        let mut history = History::default();
        history.add_action(Action::pass(0));
        history.add_action(Action::pass(1));

        // Player 2 has no actions
        let after_player2 = history.get_actions_after_player(2);
        assert!(after_player2.is_empty());
    }

    #[test]
    fn test_history_get_actions_after_player_middle_action() {
        let mut history = History::default();
        let tile1 = Tile::from((1, 2));
        let tile2 = Tile::from((3, 4));

        // Sequence: P0, P1, P0, P1
        history.add_action(Action::play(0, tile1, Some(1)));  // index 0
        history.add_action(Action::draw(1, tile2));   // index 1 - last P1 action
        history.add_action(Action::pass(0));              // index 2
        history.add_action(Action::pass(0));              // index 3

        // Get actions after player 1's last action (indices 2 and 3)
        let after_player1 = history.get_actions_after_player(1);
        assert_eq!(after_player1.len(), 2);
        assert_eq!(after_player1[0].player_id, 0);
        assert_eq!(after_player1[1].player_id, 0);
    }

    #[test]
    fn test_history_empty() {
        let history = History::default();

        assert!(history.get_actions().is_empty());
        assert_eq!(history.get_last_action(), None);
        assert!(history.get_player_actions(0).is_empty());
        assert!(history.get_actions_after_player(0).is_empty());
    }

    #[test]
    fn test_history_single_action() {
        let mut history = History::default();
        let tile = Tile::from((6, 6));
        let action = Action::play(1, tile, Some(6));

        history.add_action(action);

        assert_eq!(history.get_actions().len(), 1);
        assert_eq!(history.get_last_action().unwrap().player_id, 1);
        assert_eq!(history.get_player_actions(1).len(), 1);
        assert!(history.get_actions_after_player(1).is_empty());
    }

    #[test]
    fn test_action_equality() {
        let tile = Tile::from((2, 3));
        let action1 = Action::play(0, tile, Some(2));
        let action2 = Action::play(0, tile, Some(2));
        let action3 = Action::play(1, tile, Some(2));

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_action_different_player_ids() {
        let tile = Tile::from((0, 0));

        // Test actions with different player IDs
        for player_id in 0..=255u8 {
            let action = Action::draw(player_id, tile);
            assert_eq!(action.player_id, player_id);
        }
    }

    #[test]
    fn test_action_with_different_tiles() {
        // Test actions with various tile combinations
        let tiles = vec![
            Tile::from((0, 0)),
            Tile::from((6, 6)),
            Tile::from((1, 5)),
            Tile::from((2, 4)),
            Tile::from((3, 3)),
        ];

        for tile in tiles {
            let draw_action = Action::draw(0, tile);
            assert_eq!(draw_action.tile_drawn, Some(tile));

            // Only play with None end if it's a double, otherwise use a valid end
            let end = if tile.is_double() { None } else { Some(tile.as_tuple().0) };
            let play_action = Action::play(1, tile, end);
            assert_eq!(play_action.tile_played, Some((tile, end)));
        }
    }

    #[test]
    fn test_history_add_action_order() {
        let mut history = History::new();
        let tile1 = Tile::from((1, 1));
        let tile2 = Tile::from((2, 2));

        let action1 = Action::play(0, tile1, None);
        let action2 = Action::draw(1, tile2);
        let action3 = Action::pass(0);

        history.add_action(action1.clone());
        history.add_action(action2.clone());
        history.add_action(action3.clone());

        let actions = history.get_actions();
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0], action1);
        assert_eq!(actions[1], action2);
        assert_eq!(actions[2], action3);
    }

    #[test]
    fn test_history_get_player_actions_multiple_players() {
        let mut history = History::new();

        // Add actions for players 0, 1, and 2
        history.add_action(Action::pass(0));
        history.add_action(Action::pass(1));
        history.add_action(Action::pass(2));
        history.add_action(Action::pass(0));
        history.add_action(Action::pass(1));

        assert_eq!(history.get_player_actions(0).len(), 2);
        assert_eq!(history.get_player_actions(1).len(), 2);
        assert_eq!(history.get_player_actions(2).len(), 1);
        assert_eq!(history.get_player_actions(3).len(), 0);
    }

    #[test]
    fn test_history_get_actions_after_player_edge_cases() {
        let mut history = History::new();

        // Test with single action
        history.add_action(Action::pass(0));
        assert!(history.get_actions_after_player(0).is_empty());
        assert!(history.get_actions_after_player(1).is_empty());

        // Test with player who acted in middle
        history.add_action(Action::pass(1));
        history.add_action(Action::pass(2));

        // After player 0's action (which was first), should get 2 actions
        let after_p0 = history.get_actions_after_player(0);
        assert_eq!(after_p0.len(), 2);
        assert_eq!(after_p0[0].player_id, 1);
        assert_eq!(after_p0[1].player_id, 2);

        // After player 1's action, should get 1 action
        let after_p1 = history.get_actions_after_player(1);
        assert_eq!(after_p1.len(), 1);
        assert_eq!(after_p1[0].player_id, 2);

        // After player 2's action (which was last), should get 0 actions
        let after_p2 = history.get_actions_after_player(2);
        assert!(after_p2.is_empty());
    }

    #[test]
    fn test_history_large_sequence() {
        let mut history = History::new();

        // Add a large sequence of actions
        for i in 0..100 {
            let player_id = (i % 4) as u8; // 4 players alternating
            history.add_action(Action::pass(player_id));
        }

        assert_eq!(history.get_actions().len(), 100);

        // Each player should have 25 actions
        for player_id in 0..4 {
            assert_eq!(history.get_player_actions(player_id).len(), 25);
        }

        // Last action should be from player 3 (since 99 % 4 = 3)
        assert_eq!(history.get_last_action().unwrap().player_id, 3);
    }

    #[test]
    fn test_history_mixed_action_types() {
        let mut history = History::new();
        let tile = Tile::from((4, 5));

        // Add different types of actions
        history.add_action(Action::draw(0, tile));
        history.add_action(Action::play(1, tile, Some(4)));
        history.add_action(Action::pass(0));
        history.add_action(Action::new(1, Some(tile), Some((tile, Some(4)))));

        let actions = history.get_actions();
        assert_eq!(actions.len(), 4);

        // Verify each action type
        assert!(actions[0].tile_drawn.is_some() && actions[0].tile_played.is_none());
        assert!(actions[1].tile_drawn.is_none() && actions[1].tile_played.is_some());
        assert!(actions[2].tile_drawn.is_none() && actions[2].tile_played.is_none());
        assert!(actions[3].tile_drawn.is_some() && actions[3].tile_played.is_some());
    }

    #[test]
    fn test_history_get_last_action_persistence() {
        let mut history = History::new();
        let tile1 = Tile::from((1, 1));
        let tile2 = Tile::from((2, 2));

        // Test that get_last_action always returns the most recent
        history.add_action(Action::play(0, tile1, Some(1)));
        assert_eq!(history.get_last_action().unwrap().tile_played, Some((tile1, Some(1))));

        history.add_action(Action::draw(1, tile2));
        assert_eq!(history.get_last_action().unwrap().tile_drawn, Some(tile2));

        history.add_action(Action::pass(0));
        let last = history.get_last_action().unwrap();
        assert_eq!(last.player_id, 0);
        assert!(last.tile_drawn.is_none() && last.tile_played.is_none());
    }

    #[test]
    fn test_action_field_access() {
        let tile = Tile::from((5, 6));
        let action = Action::new(2, Some(tile), None);

        // Test direct field access
        assert_eq!(action.player_id, 2);
        assert_eq!(action.tile_drawn, Some(tile));
        assert_eq!(action.tile_played, None);

        // Fields should be public and directly accessible
        let _ = action.player_id;
        let _ = action.tile_drawn;
        let _ = action.tile_played;
    }
}