//! Configuration module

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
/// assert_eq!(config.num_players(), 4);
///
/// // Create a 2-player double-nine All-Fives game
/// let config = Configuration::new(2, Variation::AllFives, 9, 7);
/// assert_eq!(config.set_size(), 55);
/// assert_eq!(config.variation(), Variation::AllFives);
///
/// // Use default configuration (2-player Traditional double-six)
/// let default_config = Configuration::default();
/// assert_eq!(default_config.starting_hand_size(), 7);
/// ```
#[derive(Debug, Clone)]
pub struct Configuration {
    /// The game variation being played
    variation: Variation,
    /// The ID of the dominoes set. Same as the highest pip value.
    set_id: u8,
    /// Number of tiles each player starts with
    starting_hand_size: usize,
    /// Number of players in the game
    num_players: usize,
    /// Complete set of all tiles available for this game
    tiles: Vec<Tile>,
}

impl Configuration {
    pub const DEFAULT_NUM_PLAYERS: usize = 2;
    pub const DEFAULT_VARIATION: Variation = Variation::Traditional;
    pub const DEFAULT_SET_ID: u8 = 6;

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
    /// # use rules::{Configuration, Variation};
    ///
    /// // Standard double-six game for 4 players
    /// let config = Configuration::new(
    ///     4,
    ///     Variation::Traditional,
    ///     6,
    ///     Configuration::default_starting_hand_size(4, Variation::Traditional)
    /// );
    ///
    /// // Double-twelve All-Fives game for 2 players
    /// let big_game = Configuration::new(2, Variation::AllFives, 12, 10);
    /// assert_eq!(big_game.set_size(), 91);
    /// assert_eq!(big_game.variation(), Variation::AllFives);
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

    /// Returns the game variation being played.
    pub fn variation(&self) -> Variation {
        self.variation
    }

    /// Returns the ID of the dominoes set.
    pub fn set_id(&self) -> u8 {
        self.set_id
    }

    /// Returns the number of tiles each player starts with.
    pub fn starting_hand_size(&self) -> usize {
        self.starting_hand_size
    }

    /// Returns the number of players in the game.
    pub fn num_players(&self) -> usize {
        self.num_players
    }

    /// Returns the complete set of all tiles available for this game.
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
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

    /// Returns the default starting hand size for a given number of players and variation.
    pub fn default_starting_hand_size(num_players: usize, variation: Variation) -> usize {
        match variation {
            Variation::Bergen => 6,
            Variation::Blind => match num_players {
                2 => 8,
                3 => 7,
                4..=8 => 6,
                _ => 5,
            },
            _ => match num_players {
                2 => 7,
                3..=4 => 6,
                5..=8 => 5,
                _ => 4,
            },
        }
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
/// assert_eq!(config.num_players(), 2);
/// assert_eq!(config.variation(), Variation::Traditional);
/// assert_eq!(config.set_id(), 6);
/// assert_eq!(config.starting_hand_size(), 7);
/// assert_eq!(config.set_size(), 28);
/// ```
impl Default for Configuration {
    fn default() -> Self {
        Self::new(
            Self::DEFAULT_NUM_PLAYERS,
            Self::DEFAULT_VARIATION,
            Self::DEFAULT_SET_ID,
            Configuration::default_starting_hand_size(Self::DEFAULT_NUM_PLAYERS, Self::DEFAULT_VARIATION),
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
