//use crate::{Tile, Variation, all_tiles_as_tiles, default_starting_hand_size};
use crate::*;

/// Configuration for a dominoes game session.
///
/// This struct encapsulates all the settings needed to set up and run a domino game,
/// including the game variation, number of players, domino set size, hand sizes,
/// and the complete set of tiles available.
///
/// # Examples
/// ```rust
/// # use rules::{Configuration, Variation};
/// 
/// // Create a standard 4-player double-six game
/// let config = Configuration::new(4, Variation::Traditional, 6, 6);
/// assert_eq!(config.set_size(), 28);
/// assert_eq!(config.num_players, 4);
/// 
/// // Create a 2-player double-nine All-Fives game  
/// let config = Configuration::new(2, Variation::AllFives, 9, 7);
/// assert_eq!(config.set_size(), 55);
/// assert_eq!(config.variation, Variation::AllFives);
/// 
/// // Use default configuration (2-player Traditional double-six)
/// let default_config = Configuration::default();
/// assert_eq!(default_config.starting_hand_size, 7);
/// ```
#[derive(Debug, Clone)]
pub struct Configuration {
    /// The game variation being played
    pub variation: Variation,
    /// The ID of the can_attach_tuples. Same as the highest tile value.
    pub set_id: u8,
    /// Number of tiles each player starts with
    pub starting_hand_size: usize,
    /// Number of players in the game
    pub num_players: usize,
    /// Complete set of all tiles available for this game
    pub tiles: Vec<Tile>,
}

impl Configuration {
    /// Creates a new configuration
    ///
    /// # Arguments
    /// * `num_players` - Number of players (must be ≥ 2)
    /// * `variation` - The game variation being played
    /// * `set_id` - ID of the set. Same as the highest tile value (must be ≤ 21 for u8 compatibility)
    /// * `starting_hand_size` - Number of tiles each player starts with
    ///
    /// # Panics
    /// * If `num_players < 2` (need at least 2 players for a game)
    /// * If `set_id > 21` (would exceed u8 ordinal capacity)
    ///
    /// # Examples
    /// ```
    /// # use rules::{Configuration, Variation, default_starting_hand_size};
    /// 
    /// // Standard double-six game for 4 players
    /// let config = Configuration::new(
    ///     4, 
    ///     Variation::Traditional, 
    ///     6, 
    ///     default_starting_hand_size(4, Variation::Traditional)
    /// );
    /// 
    /// // Double-twelve All-Fives game for 2 players
    /// let big_game = Configuration::new(2, Variation::AllFives, 12, 10);
    /// assert_eq!(big_game.set_size(), 91);
    /// assert_eq!(big_game.variation, Variation::AllFives);
    /// ```
    pub fn new(num_players: usize, variation: Variation, set_id: u8, starting_hand_size: usize) -> Self {
        assert!(num_players > 1, "Must have at least 2 players");
        assert!(set_id <= 21, "set_id must be <= 21 (u8 ordinal limit)");
        
        let tiles = all_tiles_as_tiles(set_id);
        
        Self {
            variation,
            set_id,
            starting_hand_size,
            num_players,
            tiles,
        }
    }

    /// Returns the total number of tiles in this game's domino set.
    ///
    /// # Examples
    /// ```rust
    /// # use rules::{Configuration, Variation};
    /// 
    /// let config = Configuration::new(2, Variation::Traditional, 6, 7);
    /// assert_eq!(config.set_size(), 28); // Double-six has 28 tiles
    /// 
    /// let config = Configuration::new(2, Variation::Traditional, 9, 7);
    /// assert_eq!(config.set_size(), 55); // Double-nine has 55 tiles
    /// ```
    pub fn set_size(&self) -> usize {
        self.tiles.len()
    }

    /// Returns a slice containing all tiles in this game's domino set.
    ///
    /// The tiles are in canonical order (by ordinal). This can be used to
    /// initialize boneyards, validate tiles, or enumerate all possible tiles.
    ///
    /// # Examples
    /// ```
    /// # use rules::{Configuration, Variation, Tile};
    /// 
    /// let config = Configuration::new(2, Variation::Traditional, 2, 6);
    /// let all_tiles = config.all_tiles();
    /// 
    /// assert_eq!(all_tiles.len(), 6);
    /// assert_eq!(all_tiles[0], Tile::from((0, 0))); // First tile
    /// assert_eq!(all_tiles[5], Tile::from((2, 2))); // Last tile
    /// ```
    pub fn all_tiles(&self) -> &[Tile] {
        &self.tiles
    }
}

/// Provides a typical configuration.
///
/// Creates a standard 2-player Traditional double-six domino game with appropriate starting hand sizes.
///
/// # Default Values
/// * Players: 2
/// * Variation: Traditional
/// * Set: Double-six (28 tiles)
/// * Hand size: 7 tiles per player
///
/// # Examples
/// ```rust
/// # use rules::{Configuration, Variation};
/// 
/// let config = Configuration::default();
/// assert_eq!(config.num_players, 2);
/// assert_eq!(config.variation, Variation::Traditional);
/// assert_eq!(config.set_id, 6);
/// assert_eq!(config.starting_hand_size, 7);
/// assert_eq!(config.set_size(), 28);
/// ```
impl Default for Configuration {
    fn default() -> Self {
        const NUM_PLAYERS: usize = 2;
        const VARIATION: Variation = Variation::Traditional;
        const SET_ID: u8 = 6;
        
        Self::new(
            NUM_PLAYERS,
            VARIATION,
            SET_ID,
            default_starting_hand_size(NUM_PLAYERS, VARIATION),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_new_comprehensive() {
        // Test various configurations
        let config1 = Configuration::new(2, Variation::Traditional, 6, 7);
        assert_eq!(config1.num_players, 2);
        assert_eq!(config1.variation, Variation::Traditional);
        assert_eq!(config1.set_id, 6);
        assert_eq!(config1.starting_hand_size, 7);
        assert_eq!(config1.set_size(), 28);
        
        let config2 = Configuration::new(4, Variation::Bergen, 9, 6);
        assert_eq!(config2.num_players, 4);
        assert_eq!(config2.variation, Variation::Bergen);
        assert_eq!(config2.set_id, 9);
        assert_eq!(config2.starting_hand_size, 6);
        assert_eq!(config2.set_size(), 55);
        
        let config3 = Configuration::new(3, Variation::Blind, 12, 8);
        assert_eq!(config3.set_size(), 91);
        
        // Test default configuration
        let default_config = Configuration::default();
        assert_eq!(default_config.num_players, 2);
        assert_eq!(default_config.variation, Variation::Traditional);
        assert_eq!(default_config.set_id, 6);
        assert_eq!(default_config.starting_hand_size, 7);
        assert_eq!(default_config.set_size(), 28);
        
        // Test all_tiles functionality
        let config_small = Configuration::new(2, Variation::Traditional, 2, 6);
        let all_tiles = config_small.all_tiles();
        assert_eq!(all_tiles.len(), 6);
        assert_eq!(all_tiles[0], Tile::from((0, 0)));
        assert_eq!(all_tiles[1], Tile::from((0, 1)));
        assert_eq!(all_tiles[2], Tile::from((1, 1)));
        assert_eq!(all_tiles[3], Tile::from((0, 2)));
        assert_eq!(all_tiles[4], Tile::from((1, 2)));
        assert_eq!(all_tiles[5], Tile::from((2, 2)));
        
        // Test clone functionality
        let config_clone = config1.clone();
        assert_eq!(config1.num_players, config_clone.num_players);
        assert_eq!(config1.variation, config_clone.variation);
        assert_eq!(config1.set_id, config_clone.set_id);
        assert_eq!(config1.starting_hand_size, config_clone.starting_hand_size);
        assert_eq!(config1.tiles.len(), config_clone.tiles.len());
        
        // Test debug formatting
        let debug_str = format!("{:?}", config1);
        assert!(debug_str.contains("Configuration"));
        assert!(debug_str.contains("variation"));
        assert!(debug_str.contains("Traditional"));
    }

    #[test]
    #[should_panic(expected = "Must have at least 2 players")]
    fn test_configuration_new_too_few_players() {
        Configuration::new(1, Variation::Traditional, 6, 7);
    }

    #[test]
    #[should_panic(expected = "set_id must be <= 21")]
    fn test_configuration_new_set_id_too_large() {
        Configuration::new(2, Variation::Traditional, 22, 7);
    }
}
