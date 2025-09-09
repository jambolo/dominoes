//! Zobrist Hashing implementation for Dominoes
//!
//! This module provides a Zobrist hashing implementation specifically designed for domino games. Zobrist hashing is a technique
//! used to create an uncorrelated and uniformly distributed value that uniquely represents a game state. The values are useful
//! for indexing game states and detecting duplicates.
//!
//! # Overview
//! The value combines three key components of domino game state:
//! 1. **Tiles in the layout** - Which domino tiles are currently placed
//! 2. **Open end counts** - How many times each value appears as an open end
//! 3. **Player turn** - Which player's turn it is
//!
//! # Collision Probability
//! With 64-bit uniformly distributed and uncorrelated values, collision probability is low:
//! - 1 million states: ~2.71×10⁻⁸ chance of collision
//! - 1 billion states: ~1 in 40 chance of collision
//!
//! # Example
//! ```rust
//! # use dominoes_state::ZHash;
//!
//! // Create an empty hash
//! let mut hash = ZHash::default();
//!
//! // Create from specific value
//! let hash2 = ZHash::from(12345);
//!
//! // Check if undefined
//! let undefined_hash = ZHash::from(ZHash::UNDEFINED);
//! assert!(undefined_hash.is_undefined());
//! ```

use std::sync::LazyLock;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use crate::layout::Layout;

/// Type alias for Zobrist hash values
///
/// This type represents a 64-bit unsigned integer used for Zobrist hashing.
pub type Z = u64;

/// Zobrist Hashing Calculator for Dominoes.
///
/// # Properties
/// - **Order Independent**: The same game state produces the same value regardless
///   of the order in which moves were made to reach that state
/// - **Incremental**: Zobrist values can be updated incrementally as moves are made
/// - **Collision Resistant**: 64-bit values provide strong collision resistance
///
/// # Thread Safety
/// `ZHash` implements `Copy` and contains no mutable state, making it thread-safe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ZHash {
    /// The current Zobrist hash value
    value: Z,
}

impl ZHash {
    /// Value representing the starting state.
    ///
    /// This constant represents the hash value for the starting game state.
    pub const START: Z = 0;

    /// Special value indicating an undefined or invalid state
    ///
    /// This constant (all bits set to 1) is used to mark hash values that represent invalid or uninitialized game states.
    pub const UNDEFINED: Z = Z::MAX;

    /// Creates a new ZHash with the specified value
    ///
    /// # Arguments
    /// * `value` - The 64-bit value to store
    ///
    /// # Example
    /// ```rust
    /// # use dominoes_state::ZHash;
    ///
    /// let hash = ZHash::from(12345);
    /// let value: u64 = hash.into();
    /// assert_eq!(value, 12345);
    /// ```
    pub const fn new(value: Z) -> Self {
        Self { value }
    }

    /// Constructs a value from the current game state
    ///
    /// # Arguments
    /// * `layout` - Reference to the current board layout
    /// * `turn` - The player who moves next (0 or 1, only two-player games are supported)
    ///
    /// # Panics
    /// This function will panic if `turn` is not 0 or 1.
    ///
    /// # Example
    /// ```rust
    /// # use dominoes_state::{ZHash, Layout};
    /// # use rules::{Configuration, Variation};
    ///
    /// let config = Configuration::new(4, Variation::Traditional, 6, 6);
    /// let layout = Layout::new(&config);
    /// let hash = ZHash::from_state(&layout, 0);
    /// ```
    pub fn from_state(layout: &Layout, turn: u8) -> Self {
        assert!(turn < 2, "Only valid for two-player games");

        let mut value = Self::START;

        // Include tiles in the layout
        for node in &layout.nodes {
            value ^= Z_VALUE_TABLE.tile_value(u8::from(node.tile) as usize);
        }

        // Include open end counts
        for (end_value, &count) in layout.end_counts.iter().enumerate() {
            if count > 0 {
                value ^= Z_VALUE_TABLE.end_value(end_value, count as usize);
            }
        }

        // Include turn
        if turn != 0 {
            value ^= Z_VALUE_TABLE.turn_value();
        }

        Self { value }
    }

    /// Updates the value for a tile added to the layout.
    ///
    /// # Arguments
    /// * `tile` - The ordinal of the tile added to the layout
    ///
    /// # Returns
    /// Mutable reference to self for chaining operations
    ///
    /// # Example
    /// ```rust
    /// # use dominoes_state::ZHash;
    /// let mut hash = ZHash::default();
    /// hash.add_tile(42).add_tile(43);
    /// ```
    pub fn add_tile(&mut self, tile: u8) -> &mut Self {
        self.value ^= Z_VALUE_TABLE.tile_value(tile as usize);
        self
    }

    /// Updates the value for an open end added or removed from the layout
    ///
    /// # Arguments
    /// * `value` - The open end value
    /// * `old_count` - The old count before the change
    /// * `new_count` - The new count after the change
    ///
    /// # Returns
    /// Mutable reference to self for method chaining
    ///
    /// # Important
    /// The method cannot determine if the end value and counts match the state. Garbage in, garbage out.
    ///
    /// # Panics
    /// Panics if `old_count` is the same as `new_count`.
    ///
    /// # Example
    /// ```rust
    /// # use dominoes_state::ZHash;
    ///
    /// let mut hash = ZHash::default();
    /// // End value 6 now appears 2 times (0 -> 2), so ...
    /// hash.change_end_count(6, 0, 2);
    /// // End value 6 now appears 1 time (2 -> 1), so ...
    /// hash.change_end_count(6, 2, 1);
    /// ```
    pub fn change_end_count(&mut self, value: u8, old_count: u8, new_count: u8) -> &mut Self {
        assert_ne!(old_count, new_count, "Sanity check: Old and new counts should be different");

        // Undo the previous count and update with the new count
        self.value ^= Z_VALUE_TABLE.end_value(value as usize, old_count as usize);
        self.value ^= Z_VALUE_TABLE.end_value(value as usize, new_count as usize);

        self
    }

    /// Updates the value to reflect a turn change
    ///
    /// This method updates the value to reflect that it now the next player's turn.
    /// Returns a mutable reference to self for method chaining.
    ///
    /// # Returns
    /// Mutable reference to self for chaining operations
    ///
    /// # Example
    /// ```rust
    /// # use dominoes_state::ZHash;
    ///
    /// let mut hash = ZHash::default();
    /// hash.turn();  // Switch to player 1
    /// ```
    pub fn turn(&mut self) -> &mut Self {
        self.value ^= Z_VALUE_TABLE.turn_value();
        self
    }

    /// Checks if this value represents an undefined state
    ///
    /// # Returns
    /// `true` if the value is undefined, `false` otherwise
    ///
    /// # Example
    /// ```rust
    /// # use dominoes_state::ZHash;
    ///
    /// let undefined = ZHash::from(ZHash::UNDEFINED);
    /// assert!(undefined.is_undefined());
    ///
    /// let normal = ZHash::default();
    /// assert!(!normal.is_undefined());
    /// ```
    pub const fn is_undefined(self) -> bool {
        self.value == Self::UNDEFINED
    }
}

/// Creates a ZHash representing the start of a game
///
/// Equivalent to `ZHash::from(ZHash::START)`.
///
/// # Example
/// ```rust
/// # use dominoes_state::ZHash;
///
/// let hash = ZHash::default();
/// let value: u64 = hash.into();
/// assert_eq!(value, ZHash::START);
/// ```
impl Default for ZHash {
    fn default() -> Self {
        Self::new(Self::START)
    }
}

/// Enables ZHash::from(Z) -> ZHash and Z::into() -> ZHash
impl From<Z> for ZHash {
    fn from(value: Z) -> Self {
        Self::new(value)
    }
}

/// Enables u64::from(ZHash) -> u64 and ZHash::into() -> u64
impl From<ZHash> for Z {
    fn from(hash: ZHash) -> Self {
        hash.value
    }
}

// Lookup table for Zobrist hashes
//
// This structure contains pre-computed random hashes for all possible game state components. The hashes are generated once
// at program startup using a seeded random number generator to ensure reproducible results.
//
// Rather than requiring the set size to be specified, a maximum set size of 21 is assumed.
struct ZTable {
    // Hashes for domino tiles (indexed by tile ordinal)
    tile_hashes: [Z; 256],
    // Hashes for open end counts [end_value][count]
    end_hashes: [[Z; 22]; 22],
    // Hash for turn changes
    turn_hash: Z,
}

impl ZTable {
    // Creates a lookup table with randomly generated hashes
    fn new() -> Self {
        let mut table = Self {
            tile_hashes: [0; 256],
            end_hashes: [[0; 22]; 22],
            turn_hash: 0,
        };

        // Use a seeded RNG for reproducible results
        let mut rng = ChaCha8Rng::seed_from_u64(1);

        // Initialize tile hashes
        for hash in &mut table.tile_hashes {
            *hash = rng.next_u64();
        }

        // Initialize end value count hashes
        for end_values in &mut table.end_hashes {
            end_values[0] = 0; // Hash for count 0 is 0 because it represents the starting state.
            for count_hash in &mut end_values[1..] {
                *count_hash = rng.next_u64();
            }
        }

        // Initialize turn hash
        table.turn_hash = rng.next_u64();

        table
    }

    // Returns the hash for a specific tile
    //
    // # Arguments
    // * `tile` - Tile ordinal (0-255)
    //
    // # Returns
    // Pre-computed hash for the specified tile
    //
    // # Panics
    // Panics if `tile >= 256`
    const fn tile_value(&self, tile: usize) -> Z {
        assert!(tile < 256, "Tile index must be < 256");
        self.tile_hashes[tile]
    }

    // Returns the hash for an end value count
    //
    // # Arguments
    // * `which` - The end value (0-21, representing pip counts)
    // * `count` - How many times this end value appears (0-21)
    //
    // # Returns
    // Pre-computed hash for the specified end value/count pair
    //
    // # Panics
    // Panics if `which >= 22` or `count >= 22`
    const fn end_value(&self, which: usize, count: usize) -> Z {
        assert!(which < 22, "End value index must be < 22");
        assert!(count < 22, "Count must be < 22");
        self.end_hashes[which][count]
    }

    // Returns the hash for player turn changes
    //
    // # Returns
    // Hash to use when the turn changes to the next player
    //
    // # Note
    // For two-player games, only one turn hash is needed since XORing the same hash twice returns to the original state.
    const fn turn_value(&self) -> Z {
        // For two-player games, we only need one turn hash as XORing twice returns to original state
        self.turn_hash
    }
}

// Global singleton instance of the hash value lookup table
//
// This static variable provides thread-safe access to the pre-computed hashes. It's initialized lazily on first access using
// `LazyLock`.
//
// # Thread Safety
//
// `LazyLock` ensures that initialization happens exactly once, even in multi-threaded environments. The `ZTable` itself is
// immutable after initialization, making it safe to access from multiple threads.
static Z_VALUE_TABLE: LazyLock<ZTable> = LazyLock::new(ZTable::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zhash_constants() {
        assert_eq!(ZHash::START, 0);
        assert_eq!(ZHash::UNDEFINED, Z::MAX);
        assert_ne!(ZHash::START, ZHash::UNDEFINED);
    }

    #[test]
    fn test_zhash_new() {
        let hash = ZHash::from(42);
        assert_eq!(Z::from(hash), 42);

        let hash_zero = ZHash::from(0);
        assert_eq!(Z::from(hash_zero), 0);

        let hash_max = ZHash::from(Z::MAX);
        assert_eq!(Z::from(hash_max), Z::MAX);
    }

    #[test]
    fn test_zhash_default() {
        let hash = ZHash::default();
        assert_eq!(hash, ZHash::from(ZHash::START));
        assert_eq!(Z::from(hash), 0);
        assert!(!hash.is_undefined());
    }

    #[test]
    fn test_zhash_value() {
        let test_values = [0, 1, 42, 12345, Z::MAX, ZHash::UNDEFINED];

        for &val in &test_values {
            let hash = ZHash::from(val);
            assert_eq!(Z::from(hash), val);
        }
    }

    #[test]
    fn test_zhash_is_undefined() {
        let undefined = ZHash::from(ZHash::UNDEFINED);
        assert!(undefined.is_undefined());

        let normal = ZHash::from(42);
        assert!(!normal.is_undefined());

        let start = ZHash::default();
        assert!(!start.is_undefined());

        let zero = ZHash::from(0);
        assert!(!zero.is_undefined());
    }

    #[test]
    fn test_zhash_add_tile() {
        let mut hash = ZHash::default();
        let original_value = hash;

        // Add a tile and verify value changes
        hash.add_tile(42);
        assert_ne!(hash, original_value);

        // Store the new value
        let after_first_tile = hash;

        // Add another tile
        hash.add_tile(100);
        assert_ne!(hash, after_first_tile);
        assert_ne!(hash, original_value);

        // Test method chaining
        let mut hash2 = ZHash::default();
        hash2.add_tile(42).add_tile(100);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_zhash_change_end_count() {
        let mut hash = ZHash::default();
        let original = hash;

        // Add first occurrence of end value 6 (0 -> 1)
        hash.change_end_count(6, 0, 1);
        assert_ne!(hash, original);
        let after_first = hash;

        // Add second occurrence (1 -> 2)
        hash.change_end_count(6, 1, 2);
        assert_ne!(hash, after_first);
        assert_ne!(hash, original);
        let after_two = hash;

        // Remove one (2 -> 1)
        hash.change_end_count(6, 2, 1);
        assert_ne!(hash, after_two);
        assert_eq!(hash, after_first);
        assert_ne!(hash, original);

        // Remove last one (1 -> 0)
        hash.change_end_count(6, 1, 0);
        assert_ne!(hash, after_two);
        assert_ne!(hash, after_first);
        assert_eq!(hash, original);

        // Test method chaining
        let mut hash2 = ZHash::default();
        hash2.change_end_count(5, 0, 1).change_end_count(5, 1, 2);
        assert_ne!(hash2, ZHash::default());
    }

    #[test]
    fn test_zhash_change_end_count_multiple() {
        // Add 1 end value 6 three times (0 -> 1, 1 -> 2, 2 -> 3)
        let mut hash_1_thrice = ZHash::default();
        let original_hash_1_thrice_value = hash_1_thrice;
        hash_1_thrice.change_end_count(6, 0, 1).change_end_count(6, 1, 2).change_end_count(6, 2, 3);
        assert_ne!(hash_1_thrice, original_hash_1_thrice_value);

        // Add 3 end value 6s once (0 -> 3)
        let mut hash_3_once = ZHash::default();
        let original_hash_3_once_value = hash_3_once;
        hash_3_once.change_end_count(6, 0, 3);
        assert_ne!(hash_3_once, original_hash_3_once_value);

        // Both methods should yield the same final value
        assert_eq!(hash_1_thrice, hash_3_once);

        // Remove 1 end value 6 three times (3 -> 2, 2 -> 1, 1 -> 0)
        hash_1_thrice.change_end_count(6, 3, 2).change_end_count(6, 2, 1).change_end_count(6, 1, 0);
        assert_eq!(hash_1_thrice, original_hash_1_thrice_value);

        // Remove 3 end value 6s once (3 -> 0)
        hash_3_once.change_end_count(6, 3, 0);
        assert_eq!(hash_3_once, original_hash_3_once_value);
        assert_eq!(hash_1_thrice, hash_3_once);
    }

    #[test]
    fn test_zhash_turn() {
        let mut hash = ZHash::default();
        let original_value = hash;

        // Change turn and verify value changes
        hash.turn();
        assert_ne!(hash, original_value);

        let after_turn = hash;

        // Turn again should return to original value
        hash.turn();
        assert_eq!(hash, original_value);
        assert_ne!(hash, after_turn);
    }

    #[test]
    #[should_panic(expected = "Sanity check: Old and new counts should be different")]
    fn test_zhash_change_end_count_same_counts() {
        let mut hash = ZHash::default();
        hash.change_end_count(0, 0, 0); // Same counts should panic
    }

    #[test]
    fn test_zhash_from_state_basic() {
        use rules::Configuration;

        let config = Configuration::default();
        let layout = crate::Layout::new(&config);

        // Test with player 0's turn
        let hash0 = ZHash::from_state(&layout, 0);
        assert_eq!(hash0, ZHash::default()); // Empty layout, player 0 = START

        // Test with player 1's turn
        let hash1 = ZHash::from_state(&layout, 1);
        assert_ne!(hash1, hash0); // Different turn should produce different hash
        assert_ne!(hash1, ZHash::default());
    }

    #[test]
    #[should_panic(expected = "Only valid for two-player games")]
    fn test_zhash_from_state_invalid_turn() {
        use rules::Configuration;

        let config = Configuration::default();
        let layout = crate::Layout::new(&config);

        // Turn value 2 should panic
        ZHash::from_state(&layout, 2);
    }

    #[test]
    fn test_zhash_new_method() {
        // Test the new() method specifically
        let hash = ZHash::new(12345);
        assert_eq!(Z::from(hash), 12345);

        let hash_zero = ZHash::new(0);
        assert_eq!(Z::from(hash_zero), 0);

        let hash_max = ZHash::new(Z::MAX);
        assert_eq!(Z::from(hash_max), Z::MAX);

        let hash_undefined = ZHash::new(ZHash::UNDEFINED);
        assert!(hash_undefined.is_undefined());
    }

    #[test]
    fn test_ztable_methods() {
        let table = &*Z_VALUE_TABLE;

        // Test tile values are consistent
        assert_eq!(table.tile_value(42), table.tile_value(42));

        // Test different tiles have different hashes
        assert_ne!(table.tile_value(0), table.tile_value(1));

        // Test end values
        assert_eq!(table.end_value(5, 3), table.end_value(5, 3));
        assert_ne!(table.end_value(5, 1), table.end_value(5, 2));

        // Test turn value consistency
        assert_eq!(table.turn_value(), table.turn_value());
    }

    // Additional comprehensive tests would go here...
}
