use std::io::{self, Write};

use dominoes_state::{Action, DominoesState};
use player::{Hand, Player};
use rules::{Configuration, Tile};

/// A concrete implementation of Player for human players
///
/// This player implementation handles human interaction through console input/output, allowing users to play dominoes by selecting
/// tiles and placement positions through keyboard input.
///
/// # Examples
/// ```rust
/// # use human_player::HumanPlayer;
/// # use player::Player;
/// # use rules::Configuration;
///
/// let config = Configuration::default();
/// let player = HumanPlayer::new(0, &config, "Alice");
/// assert_eq!(player.name(), "Alice");
/// ```
#[derive(Debug)]
pub struct HumanPlayer<'a> {
    /// Unique identifier for this player in the game (0 or 1 for two-player games)
    player_id: u8,
    /// Reference to the game configuration
    configuration: &'a Configuration,
    /// The tiles currently held by this player
    hand: Hand,
    /// Display name for this player
    name: String,
}

impl<'a> HumanPlayer<'a> {
    /// Creates a new human player with the given configuration
    ///
    /// Initializes a new human player with an empty hand and the specified name.
    /// The player will use console input/output for interaction during gameplay.
    ///
    /// # Arguments
    ///
    /// * `player_id` - Unique identifier for this player (typically 0 or 1)
    /// * `configuration` - Game rules and settings reference
    /// * `name` - Display name for this player
    ///
    /// # Returns
    ///
    /// A new `HumanPlayer` instance ready for game setup
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use human_player::HumanPlayer;
    /// # use player::Player;
    /// # use rules::Configuration;
    ///
    /// let config = Configuration::default();
    /// let alice = HumanPlayer::new(0, &config, "Alice");
    /// let bob = HumanPlayer::new(1, &config, "Bob");
    ///
    /// assert_eq!(alice.name(), "Alice");
    /// assert_eq!(bob.name(), "Bob");
    /// ```
    pub fn new(player_id: u8, configuration: &'a Configuration, name: &str) -> Self {
        Self {
            player_id,
            configuration,
            hand: Hand::new(),
            name: name.to_string(),
        }
    }

    // Get the player's choice of tile to play after displaying their hand
    fn get_player_input(&self, state: &DominoesState) -> (Tile, Option<u8>) {
        loop {
            // Get tile selection
            print!("Choose a tile (enter index 0-{}): ", self.hand.len() - 1);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let tile_index: usize = match input.trim().parse() {
                Ok(index) if index < self.hand.tiles().len() => index,
                _ => {
                    println!("Invalid tile index. Please try again.");
                    continue;
                }
            };

            let selected_tile = self.hand.get_tile(tile_index).unwrap();

            if state.layout.is_empty() {
                return (*selected_tile, None);
            }

            // Get end selection
            print!("Choose an end (0-{}): ", self.configuration.set_id);
            io::stdout().flush().unwrap();

            let mut end_input = String::new();
            io::stdin()
                .read_line(&mut end_input)
                .expect("Failed to read input");

            let end: u8 = match end_input.trim().parse() {
                Ok(end_val) if end_val <= self.configuration.set_id => end_val,
                _ => {
                    println!("Invalid end value. Please try again.");
                    continue;
                }
            };

            return (*selected_tile, Some(end));
        }
    }

    // Display the player's hand
    fn display_hand(&self) {
        println!(
            "Your hand:  {}",
            self.hand
                .tiles()
                .iter()
                .enumerate()
                .map(|(i, tile)| format!("{i}: {tile}"))
                .collect::<Vec<_>>()
                .join("   ")
        );
    }

    // Display the open ends available for tile placement
    fn display_open_ends(&self, state: &DominoesState) {
        // Create a vector containing indexes of open ends whose count is greater than 0
        let open_ends: Vec<u8> = state
            .layout
            .end_counts
            .iter()
            .enumerate()
            .filter_map(|(end, &count)| if count > 0 { Some(end as u8) } else { None })
            .collect();
        println!(
            "Open ends: {}",
            open_ends
                .iter()
                .map(|&e| {
                    let count = state.layout.open_count(e);
                    if count > 1 {
                        format!("{e}x{count}")
                    } else {
                        format!("{e}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Draw the starting hand size number of tiles from the boneyard
    fn draw_starting_hand(&mut self, state: &mut DominoesState) {
        // Draw the starting hand size number of tiles from the boneyard
        let hand_size = self.configuration.starting_hand_size;
        for _ in 0..hand_size {
            let tile = state.draw_tile().expect("Failed to draw tile during setup");
            self.hand.add_tile(tile);
        }
    }
}

impl<'a> Player for HumanPlayer<'a> {
    fn reset(&mut self) {
        self.hand = Hand::new();
    }

    fn set_up(&mut self, state: &mut DominoesState) {
        // Draw the starting hand size number of tiles from the boneyard
        self.draw_starting_hand(state);
    }

    fn my_turn(&mut self, state: &DominoesState) -> (Action, DominoesState) {
        let mut new_state = state.clone();

        // If the player has no playable tiles, they must draw
        if !self.has_playable_tile(&new_state) {
            // Draw a tile from the boneyard, but if the boneyard is empty, the player must pass
            if let Some(tile) = new_state.draw_tile() {
                println!("You drew a tile: {tile}");
                self.hand.add_tile(tile);
                return (Action::draw(self.player_id, tile), new_state);
            } else {
                println!("No playable tiles and boneyard is empty. Passing turn.");
                new_state.pass();
                return (Action::pass(self.player_id), new_state);
            }
        }

        // Display the current layout
        println!("Current Layout:\n\n{}\n", new_state.layout);
        self.display_open_ends(&new_state);

        // Get the player's choice from the console input
        self.display_hand();
        let (mut tile, mut end) = self.get_player_input(state);
        while !state.can_play_tile(&tile, end) {
            println!("Please choose a playable tile and open end.");
            (tile, end) = self.get_player_input(state);
        }
        self.hand.remove_tile(&tile);
        new_state.play_tile(tile, end);

        (Action::play(self.player_id, tile, end), new_state)
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
        &self.name
    }

    fn id(&self) -> u8 {
        self.player_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_player_creation() {
        let configuration = Configuration::default();
        let player = HumanPlayer::new(0, &configuration, "Test Player");
        // Test HumanPlayer-specific behavior
        assert_eq!(player.name(), "Test Player");
    }

    #[test]
    fn test_human_player_implements_player_trait() {
        let configuration = Configuration::default();
        let mut player = HumanPlayer::new(0, &configuration, "Test Player");
        let state = DominoesState::new(&configuration);

        // Test that my_turn method exists and returns expected types
        // Focus on HumanPlayer's implementation, not external dependencies
        let (returned_action, _new_state) = player.my_turn(&state);

        // Test HumanPlayer's specific behavior: should return pass action in stub implementation
        assert_eq!(returned_action.player_id, 0);
        assert!(returned_action.tile_drawn.is_some() || returned_action.tile_played.is_none());
    }

    #[test]
    fn test_human_player_configuration_access() {
        let configuration = Configuration::default();
        let player = HumanPlayer::new(0, &configuration, "Test Player");

        // Test that HumanPlayer stores the configuration reference correctly
        // This verifies our constructor works and configuration is accessible
        assert_eq!(player.configuration.num_players, configuration.num_players);
        assert_eq!(player.configuration.variation, configuration.variation);
        assert_eq!(player.configuration.set_id, configuration.set_id);
        assert_eq!(
            player.configuration.starting_hand_size,
            configuration.starting_hand_size
        );
    }

    #[test]
    fn test_human_player_setup() {
        let configuration = Configuration::default(); // 2 players, Traditional, 7 tiles each
        let mut player = HumanPlayer::new(0, &configuration, "Test Player");
        let mut state = DominoesState::new(&configuration);

        // Initially player has no tiles
        assert_eq!(player.hand.len(), 0);

        // Setup player - should draw 7 tiles for 2-player Traditional game
        player.set_up(&mut state);

        // Player should now have 7 tiles in hand
        assert_eq!(player.hand.len(), 7);

        // Boneyard should have 21 tiles left (28 - 7)
        assert_eq!(state.boneyard.count(), 21);

        // Test hand management functions
        if let Some(&first_tile) = player.hand.tiles().get(0) {
            assert!(player.hand.contains(&first_tile));
            player.hand.remove_tile(&first_tile);
            assert!(!player.hand.contains(&first_tile));
            assert_eq!(player.hand.len(), 6);
        }

        // Test with different variation
        use rules::default_starting_hand_size;
        let configuration_blind = Configuration::new(
            2,
            rules::Variation::Blind,
            6,
            default_starting_hand_size(2, rules::Variation::Blind),
        );
        let mut player_blind = HumanPlayer::new(0, &configuration_blind, "Test Player");
        let mut state_blind = DominoesState::new(&configuration_blind);

        // Blind uses 8 tiles for 2 players
        player_blind.set_up(&mut state_blind);
        assert_eq!(player_blind.hand.len(), 8);
        assert_eq!(state_blind.boneyard.count(), 20); // 28 - 8 = 20
    }

    #[test]
    fn test_human_player_new_comprehensive() {
        let configuration = Configuration::default();
        let player = HumanPlayer::new(1, &configuration, "Bob");

        // Test all fields are properly initialized
        assert_eq!(player.player_id, 1);
        assert_eq!(player.name(), "Bob");
        assert_eq!(player.hand.len(), 0); // Hand should start empty
        assert!(player.hand.tiles().is_empty());

        // Test configuration reference is stored correctly
        assert_eq!(player.configuration.num_players, configuration.num_players);
        assert_eq!(player.configuration.variation, configuration.variation);
        assert_eq!(player.configuration.set_id, configuration.set_id);
        assert_eq!(
            player.configuration.starting_hand_size,
            configuration.starting_hand_size
        );
    }

    #[test]
    fn test_human_player_new_different_names() {
        let configuration = Configuration::default();

        // Test with various name formats
        let player1 = HumanPlayer::new(0, &configuration, "");
        assert_eq!(player1.name(), "");

        let player2 = HumanPlayer::new(0, &configuration, "A");
        assert_eq!(player2.name(), "A");

        let player3 = HumanPlayer::new(0, &configuration, "Very Long Player Name");
        assert_eq!(player3.name(), "Very Long Player Name");

        let player4 = HumanPlayer::new(0, &configuration, "Player123");
        assert_eq!(player4.name(), "Player123");

        let player5 = HumanPlayer::new(0, &configuration, "Player with spaces");
        assert_eq!(player5.name(), "Player with spaces");
    }

    #[test]
    fn test_human_player_new_different_player_ids() {
        let configuration = Configuration::default();

        // Test with different player IDs
        let player0 = HumanPlayer::new(0, &configuration, "Alice");
        assert_eq!(player0.player_id, 0);

        let player1 = HumanPlayer::new(1, &configuration, "Bob");
        assert_eq!(player1.player_id, 1);

        let player255 = HumanPlayer::new(255, &configuration, "Max Player");
        assert_eq!(player255.player_id, 255);
    }

    #[test]
    fn test_human_player_new_different_configurations() {
        // Test with various configurations
        let config_traditional = Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let player_traditional = HumanPlayer::new(0, &config_traditional, "Traditional Player");
        assert_eq!(
            player_traditional.configuration.variation,
            rules::Variation::Traditional
        );
        assert_eq!(player_traditional.configuration.starting_hand_size, 7);

        let config_blind = Configuration::new(4, rules::Variation::Blind, 9, 10);
        let player_blind = HumanPlayer::new(1, &config_blind, "Blind Player");
        assert_eq!(
            player_blind.configuration.variation,
            rules::Variation::Blind
        );
        assert_eq!(player_blind.configuration.starting_hand_size, 10);
        assert_eq!(player_blind.configuration.num_players, 4);
        assert_eq!(player_blind.configuration.set_id, 9);
    }

    #[test]
    fn test_human_player_new_hand_initialization() {
        let configuration = Configuration::default();
        let player = HumanPlayer::new(0, &configuration, "Test Player");

        // Hand should be properly initialized and empty
        assert_eq!(player.hand.len(), 0);
        assert!(player.hand.tiles().is_empty());
        assert!(!player.hand.contains(&rules::Tile::from((0, 0))));

        // Hand should be ready to receive tiles
        // (We can't test adding tiles here since hand.add_tile() isn't public,
        // but we can verify the hand exists and is properly initialized)
    }

    #[test]
    fn test_human_player_new_multiple_instances() {
        let configuration = Configuration::default();

        // Test creating multiple instances
        let player1 = HumanPlayer::new(0, &configuration, "Player 1");
        let player2 = HumanPlayer::new(1, &configuration, "Player 2");

        // Each should be independent
        assert_ne!(player1.player_id, player2.player_id);
        assert_ne!(player1.name(), player2.name());

        // But they should share the same configuration reference
        assert!(std::ptr::eq(player1.configuration, player2.configuration));
    }

    #[test]
    fn test_human_player_implements_debug() {
        let configuration = Configuration::default();
        let player = HumanPlayer::new(0, &configuration, "Debug Test");

        // Test that Debug trait is implemented
        let debug_string = format!("{:?}", player);
        assert!(debug_string.contains("HumanPlayer"));
        assert!(debug_string.contains("Debug Test"));
    }

    #[test]
    fn test_human_player_name_ownership() {
        let configuration = Configuration::default();
        let name_string = String::from("Owned Name");
        let player = HumanPlayer::new(0, &configuration, &name_string);

        // Player should own its own copy of the name
        assert_eq!(player.name(), "Owned Name");

        // Original string should still exist and be usable
        assert_eq!(name_string, "Owned Name");
    }

    #[test]
    fn test_human_player_has_playable_tile_empty_hand() {
        let configuration = Configuration::default();
        let player = HumanPlayer::new(0, &configuration, "Test Player");
        let state = DominoesState::new(&configuration);

        // Player with empty hand should have no playable tiles
        assert!(!player.has_playable_tile(&state));
    }

    #[test]
    fn test_human_player_setup_different_hand_sizes() {
        // Test setup with various hand sizes
        let config_small = Configuration::new(2, rules::Variation::Traditional, 6, 3);
        let mut player_small = HumanPlayer::new(0, &config_small, "Small Hand");
        let mut state_small = DominoesState::new(&config_small);

        player_small.set_up(&mut state_small);
        assert_eq!(player_small.hand.len(), 3);
        assert_eq!(state_small.boneyard.count(), 25); // 28 - 3

        let config_large = Configuration::new(2, rules::Variation::Traditional, 6, 15);
        let mut player_large = HumanPlayer::new(0, &config_large, "Large Hand");
        let mut state_large = DominoesState::new(&config_large);

        player_large.set_up(&mut state_large);
        assert_eq!(player_large.hand.len(), 15);
        assert_eq!(state_large.boneyard.count(), 13); // 28 - 15
    }

    #[test]
    fn test_human_player_my_turn_no_playable_tiles_empty_boneyard() {
        let configuration = Configuration::default();
        let player = HumanPlayer::new(0, &configuration, "Test Player");
        let mut state = DominoesState::new(&configuration);

        // Empty the boneyard
        while state.draw_tile().is_some() {}

        // Player has no tiles and no playable tiles
        assert_eq!(player.hand.len(), 0);
        assert_eq!(state.boneyard.count(), 0);

        // This would require user input, so we can't easily test the full method
        // But we can verify the preconditions
        assert!(!player.has_playable_tile(&state));
    }

    #[test]
    fn test_human_player_player_trait_consistency() {
        let configuration = Configuration::default();
        let mut player = HumanPlayer::new(1, &configuration, "Consistency Test");
        let mut state = DominoesState::new(&configuration);

        // Test that Player trait methods work consistently
        assert_eq!(player.name(), "Consistency Test");

        player.set_up(&mut state);
        assert_eq!(player.name(), "Consistency Test"); // Name unchanged
        assert!(player.hand.len() > 0); // Has tiles after setup

        // has_playable_tile should work after setup
        let _has_playable = player.has_playable_tile(&state);
        assert_eq!(player.name(), "Consistency Test"); // Still unchanged
    }

    #[test]
    fn test_human_player_setup_with_different_configurations() {
        // Test setup behavior with various game configurations

        // Traditional 2-player
        let config_2p = Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut player_2p = HumanPlayer::new(0, &config_2p, "2P Player");
        let mut state_2p = DominoesState::new(&config_2p);
        player_2p.set_up(&mut state_2p);
        assert_eq!(player_2p.hand.len(), 7);

        // 4-player game (different hand size)
        let config_4p = Configuration::new(4, rules::Variation::Traditional, 6, 6);
        let mut player_4p = HumanPlayer::new(0, &config_4p, "4P Player");
        let mut state_4p = DominoesState::new(&config_4p);
        player_4p.set_up(&mut state_4p);
        assert_eq!(player_4p.hand.len(), 6);

        // Different domino set
        let config_double9 = Configuration::new(2, rules::Variation::Traditional, 9, 10);
        let mut player_d9 = HumanPlayer::new(0, &config_double9, "D9 Player");
        let mut state_d9 = DominoesState::new(&config_double9);
        player_d9.set_up(&mut state_d9);
        assert_eq!(player_d9.hand.len(), 10);
        assert_eq!(state_d9.boneyard.count(), 45); // 55 - 10
    }
}
