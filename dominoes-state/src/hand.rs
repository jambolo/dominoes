//! A module defining a player's hand of domino tiles
//!
//! This module provides the Hand struct, which represents a player's collection of domino tiles during a game. It includes
//! methods for managing the hand, such as adding and removing tiles, checking for specific tiles, and calculating the hand's score.

use rules::Tile;

/// A player's hand
///
/// The Hand struct manages a collection of domino tiles that a player holds during the game. It provides methods for adding,
/// removing, and querying tiles in the hand.
///
/// # Examples
/// ```rust
/// # use player::Hand;
/// # use rules::Tile;
///
/// let mut hand = Hand::new();
/// hand.add_tile(Tile::from((1, 2)));
/// hand.add_tile(Tile::from((3, 4)));
/// hand.add_tile(Tile::from((5, 6)));
///
/// // Check hand size
/// assert_eq!(hand.len(), 3);
///
/// // Check if hand contains a specific tile
/// assert!(hand.contains(&Tile::from((1, 2))));
///
/// // Remove a tile
/// hand.remove_tile(&Tile::from((1, 2)));
/// assert_eq!(hand.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct Hand {
    /// Vector storing all tiles currently in the hand
    tiles: Vec<Tile>,
}

impl Hand {
    /// Creates a new empty hand
    ///
    /// # Returns
    /// A new Hand instance with no tiles
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use player::Hand;
    ///
    /// let hand = Hand::new();
    /// assert_eq!(hand.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { tiles: Vec::new() }
    }

    /// Returns a slice of all tiles in the hand
    ///
    /// Provides read-only access to all tiles currently held by the player. The tiles are not sorted.
    ///
    /// # Returns
    /// A slice containing references to all tiles in the hand
    ///
    /// # Examples
    /// ```rust
    /// # use player::Hand;
    /// # use rules::Tile;
    ///
    /// let mut hand = Hand::new();
    /// hand.add_tile(Tile::from((1, 2)));
    /// hand.add_tile(Tile::from((3, 4)));
    ///
    /// let tiles = hand.tiles();
    /// assert_eq!(tiles.len(), 2);
    /// ```
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    /// Returns a reference to a tile in the hand at the given index
    ///
    /// Provides safe indexed access to tiles in the hand. Returns None if the index is out of bounds.
    ///
    /// # Arguments
    /// * `index` - The zero-based index of the tile to retrieve
    ///
    /// # Returns
    /// An Option containing a reference to the tile, or None if index is invalid
    ///
    /// # Examples
    /// ```rust
    /// # use player::Hand;
    /// # use rules::Tile;
    ///
    /// let mut hand = Hand::new();
    /// hand.add_tile(Tile::from((1, 2)));
    ///
    /// assert_eq!(hand.get_tile(0), Some(&Tile::from((1, 2))));
    /// assert_eq!(hand.get_tile(1), None);
    /// ```
    ///
    /// # Deprecated
    /// This method is deprecated and will be removed in a future version.
    pub fn get_tile(&self, index: usize) -> Option<&Tile> {
        self.tiles.get(index)
    }

    /// Adds a tile to the hand
    ///
    /// Appends the specified tile to the end of the hand. The tile becomes available for play and increases the hand size by one.
    ///
    /// # Arguments
    /// * `tile` - The tile to add to the hand
    ///
    /// # Examples
    /// ```rust
    /// # use player::Hand;
    /// # use rules::Tile;
    ///
    /// let mut hand = Hand::new();
    /// let tile = Tile::from((2, 5));
    ///
    /// hand.add_tile(tile);
    /// assert_eq!(hand.len(), 1);
    /// assert!(hand.contains(&tile));
    /// ```
    pub fn add_tile(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }

    /// Removes a tile from the hand
    ///
    /// Removes the first occurrence of the specified tile from the hand. The hand size decreases by one.
    ///
    /// # Arguments
    /// * `tile` - The tile to remove from the hand
    ///
    /// # Panics
    /// Panics if the specified tile is not found in the hand
    ///
    /// # Examples
    /// ```rust
    /// # use player::Hand;
    /// # use rules::Tile;
    ///
    /// let mut hand = Hand::new();
    /// let tile = Tile::from((3, 6));
    ///
    /// hand.add_tile(tile);
    /// hand.remove_tile(&tile);
    /// assert_eq!(hand.len(), 0);
    /// assert!(!hand.contains(&tile));
    /// ```
    pub fn remove_tile(&mut self, tile: &Tile) {
        let pos = self.tiles.iter()
            .position(|&x| x == *tile)
            .unwrap_or_else(|| panic!("Tile {tile} not found in hand"));
        self.tiles.remove(pos);
    }

    /// Gets the number of tiles in the hand
    ///
    /// # Returns
    /// The total number of tiles currently in the hand
    ///
    /// # Examples
    /// ```rust
    /// # use player::Hand;
    /// # use rules::Tile;
    ///
    /// let mut hand = Hand::new();
    /// assert_eq!(hand.len(), 0);
    ///
    /// hand.add_tile(Tile::from((1, 1)));
    /// assert_eq!(hand.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// Checks if the hand is empty
    ///
    /// # Returns
    /// `true` if the hand contains no tiles, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// Returns true if the hand contains the specified tile
    ///
    /// Checks whether the given tile is present in the hand.
    ///
    /// # Arguments
    /// * `tile` - The tile to search for
    ///
    /// # Returns
    /// `true` if the tile is found in the hand, `false` otherwise
    ///
    /// # Examples
    /// ```rust
    /// # use player::Hand;
    /// # use rules::Tile;
    ///
    /// let mut hand = Hand::new();
    /// let tile = Tile::from((4, 4));
    ///
    /// hand.add_tile(tile);
    /// assert!(hand.contains(&tile));
    /// assert!(!hand.contains(&Tile::from((1, 1))));
    /// ```
    pub fn contains(&self, tile: &Tile) -> bool {
        self.tiles.contains(tile)
    }

    /// Returns the score of the hand by adding up the pips on all tiles
    ///
    /// # Returns
    /// The total score of the hand
    pub fn score(&self) -> u32 {
        self.tiles.iter().map(|tile| tile.score() as u32).sum()
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rules::Tile;

    #[test]
    fn test_hand_new() {
        let hand = Hand::new();
        assert_eq!(hand.len(), 0);
    }

    #[test]
    fn test_hand_add_tile() {
        let mut hand = Hand::new();
        let tile = Tile::from((1, 2));

        hand.add_tile(tile);
        assert_eq!(hand.len(), 1);
        assert!(hand.contains(&tile));
    }

    #[test]
    fn test_hand_remove_tile() {
        let mut hand = Hand::new();
        let tile = Tile::from((3, 4));

        hand.add_tile(tile);
        hand.remove_tile(&tile);
        assert_eq!(hand.len(), 0);
        assert!(!hand.contains(&tile));
    }

    #[test]
    fn test_hand_default() {
        let hand = Hand::default();
        assert_eq!(hand.len(), 0);
        assert!(hand.tiles().is_empty());
    }

    #[test]
    fn test_hand_tiles() {
        let mut hand = Hand::new();
        let tile1 = Tile::from((1, 2));
        let tile2 = Tile::from((3, 4));

        // Initially empty
        assert!(hand.tiles().is_empty());

        // Add tiles and verify tiles() returns correct slice
        hand.add_tile(tile1);
        hand.add_tile(tile2);

        let tiles = hand.tiles();
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], tile1);
        assert_eq!(tiles[1], tile2);
    }

    #[test]
    fn test_hand_get_tile() {
        let mut hand = Hand::new();
        let tile1 = Tile::from((1, 2));
        let tile2 = Tile::from((3, 4));

        hand.add_tile(tile1);
        hand.add_tile(tile2);

        // Valid indices
        assert_eq!(hand.get_tile(0), Some(&tile1));
        assert_eq!(hand.get_tile(1), Some(&tile2));

        // Invalid indices
        assert_eq!(hand.get_tile(2), None);
        assert_eq!(hand.get_tile(100), None);
    }

    #[test]
    fn test_hand_get_tile_empty_hand() {
        let hand = Hand::new();
        assert_eq!(hand.get_tile(0), None);
    }

    #[test]
    fn test_hand_contains() {
        let mut hand = Hand::new();
        let tile1 = Tile::from((1, 2));
        let tile2 = Tile::from((3, 4));
        let tile3 = Tile::from((5, 6));

        // Empty hand contains nothing
        assert!(!hand.contains(&tile1));

        // Add tiles
        hand.add_tile(tile1);
        hand.add_tile(tile2);

        // Check contains
        assert!(hand.contains(&tile1));
        assert!(hand.contains(&tile2));
        assert!(!hand.contains(&tile3));
    }

    #[test]
    fn test_hand_len() {
        let mut hand = Hand::new();

        // Initially empty
        assert_eq!(hand.len(), 0);

        // Add tiles one by one
        hand.add_tile(Tile::from((1, 1)));
        assert_eq!(hand.len(), 1);

        hand.add_tile(Tile::from((2, 2)));
        assert_eq!(hand.len(), 2);

        hand.add_tile(Tile::from((3, 3)));
        assert_eq!(hand.len(), 3);

        // Remove tile
        hand.remove_tile(&Tile::from((2, 2)));
        assert_eq!(hand.len(), 2);
    }

    #[test]
    fn test_hand_add_multiple_tiles() {
        let mut hand = Hand::new();
        let tiles = vec![
            Tile::from((0, 0)),
            Tile::from((1, 2)),
            Tile::from((3, 4)),
            Tile::from((5, 6)),
            Tile::from((6, 6)),
        ];

        for tile in &tiles {
            hand.add_tile(*tile);
        }

        assert_eq!(hand.len(), tiles.len());
        for tile in &tiles {
            assert!(hand.contains(tile));
        }
    }

    #[test]
    fn test_hand_remove_multiple_tiles() {
        let mut hand = Hand::new();
        let tiles = vec![
            Tile::from((1, 1)),
            Tile::from((2, 2)),
            Tile::from((3, 3)),
        ];

        // Add all tiles
        for tile in &tiles {
            hand.add_tile(*tile);
        }

        // Remove tiles in different order
        hand.remove_tile(&tiles[1]); // Remove middle tile
        assert_eq!(hand.len(), 2);
        assert!(!hand.contains(&tiles[1]));
        assert!(hand.contains(&tiles[0]));
        assert!(hand.contains(&tiles[2]));

        hand.remove_tile(&tiles[0]); // Remove first tile
        assert_eq!(hand.len(), 1);
        assert!(hand.contains(&tiles[2]));

        hand.remove_tile(&tiles[2]); // Remove last tile
        assert_eq!(hand.len(), 0);
    }

    #[test]
    #[should_panic(expected = "not found in hand")]
    fn test_hand_remove_nonexistent_tile() {
        let mut hand = Hand::new();
        hand.add_tile(Tile::from((1, 2)));

        // Try to remove tile that's not in hand
        hand.remove_tile(&Tile::from((3, 4)));
    }

    #[test]
    #[should_panic(expected = "not found in hand")]
    fn test_hand_remove_from_empty_hand() {
        let mut hand = Hand::new();
        hand.remove_tile(&Tile::from((1, 2)));
    }

    #[test]
    fn test_hand_clone() {
        let mut hand1 = Hand::new();
        hand1.add_tile(Tile::from((1, 2)));
        hand1.add_tile(Tile::from((3, 4)));

        let hand2 = hand1.clone();

        // Both hands should have same tiles
        assert_eq!(hand1.len(), hand2.len());
        assert_eq!(hand1.tiles(), hand2.tiles());

        // But they should be independent
        hand1.add_tile(Tile::from((5, 6)));
        assert_ne!(hand1.len(), hand2.len());
    }

    #[test]
    fn test_hand_debug() {
        let mut hand = Hand::new();
        hand.add_tile(Tile::from((2, 5)));

        let debug_string = format!("{:?}", hand);
        assert!(debug_string.contains("Hand"));
        assert!(debug_string.contains("tiles"));
    }

    #[test]
    fn test_hand_tiles_order_preservation() {
        let mut hand = Hand::new();
        let tiles = vec![
            Tile::from((6, 6)),
            Tile::from((1, 2)),
            Tile::from((0, 5)),
            Tile::from((3, 3)),
        ];

        // Add tiles in specific order
        for tile in &tiles {
            hand.add_tile(*tile);
        }

        // Verify tiles() returns them in same order
        let hand_tiles = hand.tiles();
        for (i, tile) in tiles.iter().enumerate() {
            assert_eq!(hand_tiles[i], *tile);
        }
    }

    #[test]
    fn test_hand_large_capacity() {
        let mut hand = Hand::new();

        // Add many tiles (more than typical game would have)
        for i in 0..100 {
            let first = (i % 7) as u8;
            let second = ((i + 1) % 7) as u8;
            let tile = if first <= second {
                Tile::from((first, second))
            } else {
                Tile::from((second, first))
            };
            hand.add_tile(tile);
        }

        assert_eq!(hand.len(), 100);

        // Remove half of them
        let tiles_to_remove: Vec<_> = hand.tiles()[0..50].to_vec();
        for tile in tiles_to_remove {
            hand.remove_tile(&tile);
        }

        assert_eq!(hand.len(), 50);
    }

    #[test]
    fn test_hand_is_empty() {
        let mut hand = Hand::new();

        // New hand should be empty
        assert!(hand.is_empty());

        // Add a tile - should no longer be empty
        hand.add_tile(Tile::from((1, 2)));
        assert!(!hand.is_empty());

        // Add another tile - still not empty
        hand.add_tile(Tile::from((3, 4)));
        assert!(!hand.is_empty());

        // Remove one tile - still not empty
        hand.remove_tile(&Tile::from((1, 2)));
        assert!(!hand.is_empty());

        // Remove last tile - should be empty again
        hand.remove_tile(&Tile::from((3, 4)));
        assert!(hand.is_empty());
    }

    #[test]
    fn test_hand_is_empty_consistency_with_len() {
        let mut hand = Hand::new();

        // Empty hand: is_empty() should match len() == 0
        assert_eq!(hand.is_empty(), hand.len() == 0);

        // Add tiles and verify consistency
        for i in 0..5 {
            hand.add_tile(Tile::from((i, i + 1)));
            assert_eq!(hand.is_empty(), hand.len() == 0);
        }

        // Remove tiles and verify consistency
        let tiles_to_remove: Vec<_> = hand.tiles().to_vec();
        for tile in tiles_to_remove {
            hand.remove_tile(&tile);
            assert_eq!(hand.is_empty(), hand.len() == 0);
        }
    }

    #[test]
    fn test_hand_is_empty_default() {
        let hand = Hand::default();
        assert!(hand.is_empty());
    }

    #[test]
    fn test_hand_score_empty() {
        let hand = Hand::new();
        assert_eq!(hand.score(), 0);
    }

    #[test]
    fn test_hand_score_single_tile() {
        let mut hand = Hand::new();
        hand.add_tile(Tile::from((2, 3)));

        // Score should be 2 + 3 = 5
        assert_eq!(hand.score(), 5);
    }

    #[test]
    fn test_hand_score_multiple_tiles() {
        let mut hand = Hand::new();
        hand.add_tile(Tile::from((1, 2))); // Score: 3
        hand.add_tile(Tile::from((4, 5))); // Score: 9
        hand.add_tile(Tile::from((0, 0))); // Score: 0
        hand.add_tile(Tile::from((6, 6))); // Score: 12

        // Total: 3 + 9 + 0 + 12 = 24
        assert_eq!(hand.score(), 24);
    }

    #[test]
    fn test_hand_score_double_tiles() {
        let mut hand = Hand::new();
        hand.add_tile(Tile::from((3, 3))); // Score: 6
        hand.add_tile(Tile::from((5, 5))); // Score: 10

        // Total: 6 + 10 = 16
        assert_eq!(hand.score(), 16);
    }

    #[test]
    fn test_hand_score_changes_with_tiles() {
        let mut hand = Hand::new();
        assert_eq!(hand.score(), 0);

        hand.add_tile(Tile::from((2, 4)));
        assert_eq!(hand.score(), 6);

        hand.add_tile(Tile::from((1, 1)));
        assert_eq!(hand.score(), 8); // 6 + 2

        hand.remove_tile(&Tile::from((2, 4)));
        assert_eq!(hand.score(), 2); // Only (1,1) left
    }

    #[test]
    fn test_hand_score_consistency() {
        let mut hand = Hand::new();
        hand.add_tile(Tile::from((3, 6)));

        // Multiple calls should return same result
        let score1 = hand.score();
        let score2 = hand.score();
        assert_eq!(score1, score2);
        assert_eq!(score1, 9);
    }
}