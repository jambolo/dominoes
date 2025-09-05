//! Boneyard functionality
//!
//! This module provides the `Boneyard` struct which manages the boneyard, a collection of domino tiles that can be drawn from
//! during gameplay.
//!
//! # Example
//! ```rust
//! # use dominoes_state::Boneyard;
//! # use rules::{Configuration, Variation};
//!
//! // Create a standard double-six domino set
//! let config = Configuration::new(4, Variation::Traditional, 6, 6);
//! let mut boneyard = Boneyard::new(&config);
//!
//! // Players draw tiles when needed
//! while let Some(tile) = boneyard.draw() {
//!     println!("Player drew: {:?}", tile);
//!     if boneyard.count() < 2 {
//!         break; // Keep some tiles in boneyard
//!     }
//! }
//! ```

use rules::{Configuration, Tile};
use rand::{seq::SliceRandom, rng};

/// A boneyard implementation.
///
/// The boneyard holds the domino tiles that players can draw from during the game.
///
/// # Examples
/// ```rust
/// # use dominoes_state::Boneyard;
/// # use rules::{Configuration, Variation};
///
/// // Create a standard double-six domino set
/// let config = Configuration::new(4, Variation::Traditional, 6, 6);
/// let mut boneyard = Boneyard::new(&config);
///
/// // Draw some tiles
/// let first_tile = boneyard.draw();
/// let second_tile = boneyard.draw();
///
/// println!("Remaining tiles: {}", boneyard.count());
///
/// // Check what's next without drawing
/// if let Some(next) = boneyard.peek() {
///     println!("Next tile would be: {:?}", next);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Boneyard {
    /// All the tiles in the boneyard
    tiles: Vec<Tile>,
    /// Index of the next tile to draw
    next: usize,
}

impl Boneyard {
    /// Creates a new boneyard with the provided tiles
    ///
    /// The tiles are automatically shuffled on creation.
    ///
    /// # Arguments
    /// * `configuration` - The game configuration containing the rules and tile set
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Boneyard;
    /// # use rules::{Configuration, Variation};
    ///
    /// // Creates a standard double-six domino set (0-6)
    /// let config = Configuration::new(4, Variation::Traditional, 6, 6);
    /// let boneyard = Boneyard::new(&config);
    /// assert_eq!(boneyard.count(), 28); // 7*8/2 = 28 tiles
    /// ```
    pub fn new(configuration: &Configuration) -> Self {
        let mut tiles = configuration.all_tiles().to_vec();
        tiles.shuffle(&mut rng());
        Self { tiles, next: 0 }
    }

    /// Creates a new boneyard with a specific set of tiles without shuffling them
    ///
    /// This method creates a boneyard where tiles will be drawn in the exact order provided. It is primarily intended for testing
    /// purposes where you need predictable tile ordering, game replays, or debugging scenarios.
    ///
    /// # Arguments
    /// * `configuration` - The game configuration containing the rules and tile set
    /// * `tiles` - A vector of tiles in the order they should be drawn.
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Boneyard;
    /// # use rules::Tile;
    ///
    /// let tiles: Vec<rules::Tile> = vec![Tile::from((0, 0)), Tile::from((1, 1)), Tile::from((2, 2))];
    /// let mut boneyard = Boneyard::with(tiles);
    ///
    /// // Tiles will be drawn in the exact order provided
    /// assert_eq!(boneyard.draw(), Some(Tile::from((0, 0))));
    /// assert_eq!(boneyard.draw(), Some(Tile::from((1, 1))));
    /// assert_eq!(boneyard.draw(), Some(Tile::from((2, 2))));
    /// assert_eq!(boneyard.draw(), None);
    /// ```
    pub fn with(tiles: Vec<Tile>) -> Self {
        Self { tiles, next: 0 }
    }

    /// Shuffles the remaining tiles in the boneyard.
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Boneyard;
    /// # use rules::Tile;
    ///
    /// let tiles: Vec<rules::Tile> = vec![Tile::from((0, 0)), Tile::from((1, 1)), Tile::from((2, 2)), Tile::from((3, 3)), Tile::from((4, 4))];
    /// let mut boneyard = Boneyard::with(tiles);
    ///
    /// // Draw some tiles first
    /// let first = boneyard.draw(); // (0, 0)
    /// let second = boneyard.draw(); // (1, 1)
    /// assert_eq!(boneyard.count(), 3);
    ///
    /// // Shuffle remaining tiles - only affects (2,2), (3,3), (4,4)
    /// boneyard.shuffle();
    ///
    /// // The next tile drawn will be one of the remaining tiles in random order
    /// let next = boneyard.draw();
    /// assert!(next == Some(Tile::from((2, 2))) || next == Some(Tile::from((3, 3))) || next == Some(Tile::from((4, 4))));
    /// ```
    pub fn shuffle(&mut self) {
        self.tiles[self.next..].shuffle(&mut rng());
    }

    /// Draws a tile from the boneyard, removing and returning it if available
    ///
    /// This method returns a copy of the next tile from the boneyard. If the boneyard is empty, it returns `None`.
    ///
    /// # Returns
    /// * `Some(tile)` - The next tile from the boneyard if available
    /// * `None` - If there are no tiles remaining in the boneyard
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Boneyard;
    /// # use rules::{Configuration, Variation};
    ///
    /// let config = Configuration::new(4, Variation::Traditional, 1, 1);
    /// let mut boneyard = Boneyard::new(&config);
    ///
    /// // Draw tiles until empty
    /// let mut drawn_tiles = Vec::new();
    /// while let Some(tile) = boneyard.draw() {
    ///     drawn_tiles.push(tile);
    ///     println!("Drew tile: {:?}", tile);
    /// }
    ///
    /// // No more tiles available
    /// assert_eq!(boneyard.draw(), None);
    /// assert!(boneyard.is_empty());
    ///
    /// // All tiles from double-1 set should be drawn
    /// assert_eq!(drawn_tiles.len(), 3); // (0,0), (0,1), (1,1)
    /// ```
    pub fn draw(&mut self) -> Option<Tile> {
        let tile = self.tiles.get(self.next).copied();
        if tile.is_some() {
            self.next += 1;
        }
        tile
    }

    /// Returns the number of tiles remaining in the boneyard.
    ///
    /// # Returns
    /// The number of tiles remaining to be drawn
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Boneyard;
    /// # use rules::{Configuration, Variation};
    ///
    /// let config = Configuration::new(4, Variation::Traditional, 2, 2);
    /// let mut boneyard = Boneyard::new(&config);
    ///
    /// let initial_count = boneyard.count();
    /// println!("Initial tiles: {}", initial_count); // Should be 6 for double-2
    ///
    /// // Draw a tile and verify count decreases
    /// let tile = boneyard.draw();
    /// assert!(tile.is_some());
    /// assert_eq!(boneyard.count(), initial_count - 1);
    ///
    /// // Count remains accurate as we draw more tiles
    /// while !boneyard.is_empty() {
    ///     let remaining_before = boneyard.count();
    ///     boneyard.draw();
    ///     assert_eq!(boneyard.count(), remaining_before - 1);
    /// }
    ///
    /// assert_eq!(boneyard.count(), 0);
    /// ```
    pub fn count(&self) -> usize {
        self.tiles.len() - self.next
    }

    /// Checks if the boneyard is empty and there are no more tiles to be drawn.
    ///
    /// # Returns
    /// * `true` - If there are no tiles remaining to draw
    /// * `false` - If there are still tiles available to draw
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Boneyard;
    /// # use rules::Tile;
    ///
    /// let tiles: Vec<rules::Tile> = vec![Tile::from((0, 0))];
    /// let mut boneyard = Boneyard::with(tiles);
    ///
    /// // Initially not empty
    /// assert!(!boneyard.is_empty());
    /// assert_eq!(boneyard.count(), 1);
    ///
    /// // Draw the only tile
    /// let tile = boneyard.draw();
    /// assert_eq!(tile, Some(Tile::from((0, 0))));
    ///
    /// // Now empty
    /// assert!(boneyard.is_empty());
    /// assert_eq!(boneyard.count(), 0);
    ///
    /// // Further draws return None
    /// assert_eq!(boneyard.draw(), None);
    /// ```
    ///
    /// # Usage in Game Logic
    ///
    /// ```rust
    /// # use dominoes_state::Boneyard;
    /// # use rules::{Configuration, Variation};
    ///
    /// let config = Configuration::new(4, Variation::Traditional, 6, 6);
    /// let mut boneyard = Boneyard::new(&config);
    ///
    /// // Game loop - players try to draw when they can't play
    /// loop {
    ///     if boneyard.is_empty() {
    ///         println!("No more tiles to draw - game may end");
    ///         break;
    ///     }
    ///
    ///     if let Some(tile) = boneyard.draw() {
    ///         println!("Player drew: {:?}", tile);
    ///     }
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.next >= self.tiles.len()
    }

    /// Peeks at the next tile without removing it.
    ///
    /// This method allows you to see what the next tile would be without actually drawing it. The tile remains the next tile to be
    /// drawn from the boneyard and the count is not affected.
    ///
    /// # Returns
    /// * `Some(&tile)` - A reference to the next tile that would be drawn
    /// * `None` - If there are no tiles remaining in the boneyard
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Boneyard;
    /// # use rules::Tile;
    ///
    /// let tiles: Vec<rules::Tile> = vec![Tile::from((1, 2)), Tile::from((3, 4)), Tile::from((5, 6))];
    /// let mut boneyard = Boneyard::with(tiles);
    ///
    /// // Peek at next tile
    /// let next_tile = boneyard.peek();
    /// assert_eq!(next_tile, Some(&Tile::from((1, 2))));
    ///
    /// // Count unchanged after peeking
    /// assert_eq!(boneyard.count(), 3);
    ///
    /// // Multiple peeks return the same tile
    /// assert_eq!(boneyard.peek(), Some(&Tile::from((1, 2))));
    /// assert_eq!(boneyard.peek(), Some(&Tile::from((1, 2))));
    ///
    /// // Draw the tile and peek advances to next
    /// assert_eq!(boneyard.draw(), Some(Tile::from((1, 2))));
    /// assert_eq!(boneyard.peek(), Some(&Tile::from((3, 4))));
    ///
    /// // Peek on empty boneyard returns None
    /// boneyard.draw(); // (3, 4)
    /// boneyard.draw(); // (5, 6)
    /// assert_eq!(boneyard.peek(), None);
    /// ```
    /// # Note
    /// The returned reference is never invalidated as long as the boneyard exists, but will no longer reference the next tile to
    /// be drawn after a call to `draw()` or `shuffle()`.
    pub fn peek(&self) -> Option<&Tile> {
        self.tiles.get(self.next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rules;

    #[test]
    fn test_boneyard_creation() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let boneyard = Boneyard::new(&configuration);
        // Standard double-six domino set has 28 tiles: (n+1)*(n+2)/2
        assert_eq!(boneyard.count(), 28);
    }

    #[test]
    fn test_boneyard_small_set() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 1, 3);
        let boneyard = Boneyard::new(&configuration);
        // Should have tiles: (0,0), (0,1), (1,1) = 3 tiles
        assert_eq!(boneyard.count(), 3);
    }

    #[test]
    fn test_boneyard_draw() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 2, 6);
        let mut boneyard = Boneyard::new(&configuration);
        let initial_count = boneyard.count();

        // Draw a tile
        let tile = boneyard.draw();
        assert!(tile.is_some());
        assert_eq!(boneyard.count(), initial_count - 1);

        // Verify the tile values are within expected range
        if let Some(tile) = tile {
            let (a, b) = tile.as_tuple();
            assert!(a <= 2);
            assert!(b <= 2);
            assert!(a <= b); // Tiles should be in canonical form
        }
    }

    #[test]
    fn test_boneyard_empty() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 0, 3);
        let mut boneyard = Boneyard::new(&configuration);
        // Should only have (0,0)
        assert_eq!(boneyard.count(), 1);

        let tile = boneyard.draw();
        assert_eq!(tile, Some(rules::Tile::from((0, 0))));
        assert_eq!(boneyard.count(), 0);
        assert!(boneyard.is_empty());

        // Drawing from empty boneyard should return None
        let empty_draw = boneyard.draw();
        assert_eq!(empty_draw, None);
    }

    #[test]
    fn test_boneyard_peek() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 1, 3);
        let boneyard = Boneyard::new(&configuration);
        let initial_count = boneyard.count();

        // Peek should not change count
        let peeked = boneyard.peek();
        assert!(peeked.is_some());
        assert_eq!(boneyard.count(), initial_count);
    }

    #[test]
    fn test_boneyard_with_tiles() {
        let tiles = vec![rules::Tile::from((0, 0)), rules::Tile::from((1, 1)), rules::Tile::from((2, 2))];
        let mut boneyard = Boneyard::with(tiles);

        assert_eq!(boneyard.count(), 3);

        // Should draw tiles in order
        assert_eq!(boneyard.draw(), Some(rules::Tile::from((0, 0))));
        assert_eq!(boneyard.draw(), Some(rules::Tile::from((1, 1))));
        assert_eq!(boneyard.draw(), Some(rules::Tile::from((2, 2))));
        assert_eq!(boneyard.draw(), None);
    }

    #[test]
    fn test_boneyard_shuffle_remaining_tiles() {
        let tiles = vec![rules::Tile::from((0, 0)), rules::Tile::from((1, 1)), rules::Tile::from((2, 2)), rules::Tile::from((3, 3)), rules::Tile::from((4, 4))];
        let mut boneyard = Boneyard::with(tiles);

        // Draw some tiles first
        assert_eq!(boneyard.draw(), Some(rules::Tile::from((0, 0))));
        assert_eq!(boneyard.draw(), Some(rules::Tile::from((1, 1))));
        assert_eq!(boneyard.count(), 3);

        // Record remaining tiles before shuffle
        let remaining_before: Vec<_> = (0..3).map(|_| boneyard.draw().unwrap()).collect();

        // Reset boneyard to same state
        let tiles = vec![rules::Tile::from((0, 0)), rules::Tile::from((1, 1)), rules::Tile::from((2, 2)), rules::Tile::from((3, 3)), rules::Tile::from((4, 4))];
        let mut boneyard = Boneyard::with(tiles);
        boneyard.draw(); // (0, 0)
        boneyard.draw(); // (1, 1)

        // Shuffle and verify the remaining tiles are still the same set
        boneyard.shuffle();
        let remaining_after: Vec<_> = (0..3).map(|_| boneyard.draw().unwrap()).collect();

        // Should contain same tiles but potentially in different order
        assert_eq!(remaining_before.len(), remaining_after.len());
        for tile in &remaining_before {
            assert!(remaining_after.contains(tile));
        }
        for tile in &remaining_after {
            assert!(remaining_before.contains(tile));
        }
    }

    #[test]
    fn test_boneyard_shuffle_empty() {
        let tiles = vec![rules::Tile::from((0, 0)), rules::Tile::from((1, 1))];
        let mut boneyard = Boneyard::with(tiles);

        // Draw all tiles
        boneyard.draw();
        boneyard.draw();
        assert!(boneyard.is_empty());

        // Shuffling empty boneyard should not panic
        boneyard.shuffle();
        assert!(boneyard.is_empty());
        assert_eq!(boneyard.draw(), None);
    }

    #[test]
    fn test_boneyard_shuffle_single_tile() {
        let tiles = vec![rules::Tile::from((0, 0)), rules::Tile::from((1, 1))];
        let mut boneyard = Boneyard::with(tiles);

        // Draw one tile, leaving one remaining
        assert_eq!(boneyard.draw(), Some(rules::Tile::from((0, 0))));
        assert_eq!(boneyard.count(), 1);

        // Shuffle single remaining tile
        boneyard.shuffle();

        // Should still have the same tile
        assert_eq!(boneyard.count(), 1);
        assert_eq!(boneyard.peek(), Some(&rules::Tile::from((1, 1))));
        assert_eq!(boneyard.draw(), Some(rules::Tile::from((1, 1))));
    }

    #[test]
    fn test_boneyard_shuffle_full() {
        let tiles = vec![rules::Tile::from((0, 0)), rules::Tile::from((1, 1)), rules::Tile::from((2, 2)), rules::Tile::from((3, 3))];
        let mut boneyard = Boneyard::with(tiles.clone());

        // Shuffle without drawing any tiles
        boneyard.shuffle();

        // Should still have all tiles
        assert_eq!(boneyard.count(), 4);

        // Collect all tiles and verify they're the same set
        let mut drawn_tiles = Vec::new();
        while let Some(tile) = boneyard.draw() {
            drawn_tiles.push(tile);
        }

        assert_eq!(drawn_tiles.len(), tiles.len());
        for tile in &tiles {
            assert!(drawn_tiles.contains(tile));
        }
        for tile in &drawn_tiles {
            assert!(tiles.contains(tile));
        }
    }

    #[test]
    fn test_boneyard_shuffle_preserves_drawn_tiles() {
        let tiles = vec![rules::Tile::from((0, 0)), rules::Tile::from((1, 1)), rules::Tile::from((2, 2)), rules::Tile::from((3, 3)), rules::Tile::from((4, 4))];
        let mut boneyard = Boneyard::with(tiles);

        // Draw some tiles
        let first = boneyard.draw().unwrap();
        let second = boneyard.draw().unwrap();

        // Shuffle remaining tiles
        boneyard.shuffle();

        // Verify drawn tiles are still the same
        assert_eq!(first, rules::Tile::from((0, 0)));
        assert_eq!(second, rules::Tile::from((1, 1)));

        // Verify count is correct
        assert_eq!(boneyard.count(), 3);

        // Verify we can still draw the remaining tiles
        let remaining: Vec<_> = (0..3).map(|_| boneyard.draw().unwrap()).collect();
        assert_eq!(remaining.len(), 3);

        // Should contain the expected remaining tiles
        let expected_remaining = vec![rules::Tile::from((2, 2)), rules::Tile::from((3, 3)), rules::Tile::from((4, 4))];
        for tile in &expected_remaining {
            assert!(remaining.contains(tile));
        }
    }

    #[test]
    fn test_boneyard_shuffle_multiple_times() {
        let tiles = vec![rules::Tile::from((0, 0)), rules::Tile::from((1, 1)), rules::Tile::from((2, 2)), rules::Tile::from((3, 3))];
        let mut boneyard = Boneyard::with(tiles.clone());

        // Draw one tile
        let first = boneyard.draw().unwrap();
        assert_eq!(first, rules::Tile::from((0, 0)));

        // Shuffle multiple times
        boneyard.shuffle();
        boneyard.shuffle();
        boneyard.shuffle();

        // Should still have correct count and tiles
        assert_eq!(boneyard.count(), 3);

        let remaining: Vec<_> = (0..3).map(|_| boneyard.draw().unwrap()).collect();
        let expected_remaining = vec![rules::Tile::from((1, 1)), rules::Tile::from((2, 2)), rules::Tile::from((3, 3))];

        assert_eq!(remaining.len(), expected_remaining.len());
        for tile in &expected_remaining {
            assert!(remaining.contains(tile));
        }
    }

    #[test]
    fn test_boneyard_peek_empty() {
        let tiles = vec![rules::Tile::from((0, 0))];
        let mut boneyard = Boneyard::with(tiles);
        
        // Peek at the only tile
        assert_eq!(boneyard.peek(), Some(&rules::Tile::from((0, 0))));
        
        // Draw the tile
        boneyard.draw();
        
        // Peek on empty boneyard should return None
        assert_eq!(boneyard.peek(), None);
        assert!(boneyard.is_empty());
    }

    #[test]
    fn test_boneyard_peek_consistency() {
        let tiles = vec![rules::Tile::from((1, 2)), rules::Tile::from((3, 4)), rules::Tile::from((5, 6))];
        let mut boneyard = Boneyard::with(tiles);
        
        // Multiple peeks should return the same tile
        let first_peek = boneyard.peek();
        let second_peek = boneyard.peek();
        let third_peek = boneyard.peek();
        
        assert_eq!(first_peek, second_peek);
        assert_eq!(second_peek, third_peek);
        assert_eq!(first_peek, Some(&rules::Tile::from((1, 2))));
        
        // Draw should return the same tile that was peeked
        let drawn = boneyard.draw();
        assert_eq!(drawn, Some(rules::Tile::from((1, 2))));
        
        // Peek should now show the next tile
        assert_eq!(boneyard.peek(), Some(&rules::Tile::from((3, 4))));
    }

    #[test]
    fn test_boneyard_is_empty_states() {
        let tiles = vec![rules::Tile::from((0, 0)), rules::Tile::from((1, 1))];
        let mut boneyard = Boneyard::with(tiles);
        
        // Initially not empty
        assert!(!boneyard.is_empty());
        assert_eq!(boneyard.count(), 2);
        
        // Draw first tile - still not empty
        boneyard.draw();
        assert!(!boneyard.is_empty());
        assert_eq!(boneyard.count(), 1);
        
        // Draw second tile - now empty
        boneyard.draw();
        assert!(boneyard.is_empty());
        assert_eq!(boneyard.count(), 0);
        
        // Remains empty after additional draw attempts
        boneyard.draw();
        assert!(boneyard.is_empty());
        assert_eq!(boneyard.count(), 0);
    }

    #[test]
    fn test_boneyard_count_accuracy() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 3, 7);
        let mut boneyard = Boneyard::new(&configuration);
        
        // Initial count should match expected tile count for double-3 set
        let expected_count = 10; // (3+1)*(3+2)/2 = 4*5/2 = 10
        assert_eq!(boneyard.count(), expected_count);
        
        // Count should decrease accurately with each draw
        for i in 0..expected_count {
            assert_eq!(boneyard.count(), expected_count - i);
            let tile = boneyard.draw();
            assert!(tile.is_some(), "Failed to draw tile at iteration {}", i);
        }
        
        // Should be empty and count should be 0
        assert_eq!(boneyard.count(), 0);
        assert!(boneyard.is_empty());
        
        // Further draws shouldn't affect count
        boneyard.draw();
        assert_eq!(boneyard.count(), 0);
    }

    #[test]
    fn test_boneyard_with_empty_tiles() {
        let tiles: Vec<rules::Tile> = vec![];
        let mut boneyard = Boneyard::with(tiles);
        
        // Should be empty from the start
        assert!(boneyard.is_empty());
        assert_eq!(boneyard.count(), 0);
        assert_eq!(boneyard.peek(), None);
        assert_eq!(boneyard.draw(), None);
        
        // Shuffle on empty boneyard should work
        boneyard.shuffle();
        assert!(boneyard.is_empty());
    }

    #[test]
    fn test_boneyard_large_set() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 9, 10);
        let mut boneyard = Boneyard::new(&configuration);
        
        // Double-9 set should have 55 tiles: (9+1)*(9+2)/2 = 10*11/2 = 55
        assert_eq!(boneyard.count(), 55);
        
        // Draw multiple tiles and verify count consistency
        for _ in 0..20 {
            let initial_count = boneyard.count();
            let tile = boneyard.draw();
            assert!(tile.is_some());
            assert_eq!(boneyard.count(), initial_count - 1);
        }
        
        // Should still have tiles remaining
        assert_eq!(boneyard.count(), 35);
        assert!(!boneyard.is_empty());
    }

    #[test]
    fn test_boneyard_shuffle_after_peek() {
        let tiles = vec![
            rules::Tile::from((0, 0)), 
            rules::Tile::from((1, 1)), 
            rules::Tile::from((2, 2)), 
            rules::Tile::from((3, 3))
        ];
        let mut boneyard = Boneyard::with(tiles);
        
        // Peek at first tile
        let peeked_before = boneyard.peek();
        assert_eq!(peeked_before, Some(&rules::Tile::from((0, 0))));
        
        // Shuffle - peek result may change since we shuffle all remaining tiles
        boneyard.shuffle();
        
        // Should still have same count
        assert_eq!(boneyard.count(), 4);
        
        // Peek might now show different tile
        let peeked_after = boneyard.peek();
        assert!(peeked_after.is_some());
        
        // But should still contain all original tiles
        let mut drawn_tiles = Vec::new();
        while let Some(tile) = boneyard.draw() {
            drawn_tiles.push(tile);
        }
        
        let expected_tiles = vec![
            rules::Tile::from((0, 0)), 
            rules::Tile::from((1, 1)), 
            rules::Tile::from((2, 2)), 
            rules::Tile::from((3, 3))
        ];
        
        assert_eq!(drawn_tiles.len(), expected_tiles.len());
        for tile in &expected_tiles {
            assert!(drawn_tiles.contains(tile));
        }
    }

    #[test]
    fn test_boneyard_draw_all_tiles() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 2, 6);
        let mut boneyard = Boneyard::new(&configuration);
        
        let mut drawn_tiles = Vec::new();
        let initial_count = boneyard.count();
        
        // Draw all tiles
        while let Some(tile) = boneyard.draw() {
            drawn_tiles.push(tile);
        }
        
        // Should have drawn exactly the initial count
        assert_eq!(drawn_tiles.len(), initial_count);
        assert_eq!(boneyard.count(), 0);
        assert!(boneyard.is_empty());
        
        // All drawn tiles should be valid for double-2 set
        for tile in &drawn_tiles {
            let (a, b) = tile.as_tuple();
            assert!(a <= 2 && b <= 2 && a <= b);
        }
        
        // Should have no duplicates
        for i in 0..drawn_tiles.len() {
            for j in i+1..drawn_tiles.len() {
                assert_ne!(drawn_tiles[i], drawn_tiles[j], "Found duplicate tile: {:?}", drawn_tiles[i]);
            }
        }
    }
}
