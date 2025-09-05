//! Zobrist Hashing implementation for Dominoes
//!
//! This module provides a Zobrist hashing implementation specifically designed for domino games. Zobrist hashing is a technique
//! used to create an uncorrelated and uniformly distributed value that uniquely represents a game state. The values are useful
//! for indexing game states and detecting duplicates.
//!
//! # Overview
//!
//! The value combines three key components of domino game state:
//! 1. **Tiles in the layout** - Which domino tiles are currently placed
//! 2. **Open end counts** - How many times each value appears as an open end
//! 3. **Player turn** - Which player's turn it is
//!
//! # Collision Probability
//!
//! With 64-bit uniformly distributed and uncorrelated values, collision probability is low:
//! - 1 million states: ~2.71×10⁻⁸ chance of collision
//! - 1 billion states: ~1 in 40 chance of collision
//!
//! # Example
//!
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
use rand::{RngCore, SeedableRng};

use crate::layout::*;

/// Type alias for Zobrist hash values
/// 
/// This type represents a 64-bit unsigned integer used for Zobrist hashing.
pub type Z = u64;

/// Zobrist Hashing Calculator for Dominoes.
///
/// # Properties
///
/// - **Order Independent**: The same game state produces the same value regardless
///   of the order in which moves were made to reach that state
/// - **Incremental**: Zobrist values can be updated incrementally as moves are made
/// - **Collision Resistant**: 64-bit values provide strong collision resistance
///
/// # Thread Safety
///
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
    pub const UNDEFINED: Z = !0;

    /// Creates a new ZHash with the specified value
    ///
    /// # Arguments
    ///
    /// * `value` - The 64-bit value to store
    ///
    /// # Example
    ///
    /// ```rust
    /// # use dominoes_state::ZHash;
    /// 
    /// let hash = ZHash::from(12345);
    /// let value: u64 = hash.into();
    /// assert_eq!(value, 12345);
    /// ```
    pub fn new(value: Z) -> Self {
        Self { value }
    }

    /// Constructs a value from the current game state
    ///
    /// # Arguments
    ///
    /// * `layout` - Reference to the current board layout
    /// * `turn` - The player who moves next (0 or 1, only two-player games are supported)
    ///
    /// # Panics
    ///
    /// This function will panic if `turn` is not 0 or 1.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use dominoes_state::{ZHash, Layout};
    /// # use rules::{Configuration, Variation};
    /// 
    /// let config = Configuration::new(4, Variation::Traditional, 6, 6);
    /// let layout = Layout::new(&config);
    /// let hash = ZHash::from_state(&layout, 0);
    /// ```
    pub fn from_state(layout: &Layout, turn: u8) -> Self {
        let mut value = Self::START;

        // Include tiles in the layout
        for node in layout.nodes.iter() {
            value ^= Z_VALUE_TABLE.tile_value(u8::from(node.tile) as usize);
        }

        // Include open end counts
        for (v, count) in layout.end_counts.iter().enumerate() {
            value ^= Z_VALUE_TABLE.end_value(v, *count as usize);
        }

        // Include turn
        assert!(turn < 2, "Only valid for two-player games");
        if turn != 0 {
            value ^= Z_VALUE_TABLE.turn_value();
        }

        Self { value }
    }

    /// Updates the value for a tile added to the layout.
    ///
    /// # Arguments
    ///
    /// * `tile` - The ordinal of the tile added to the layout
    ///
    /// # Returns
    ///
    /// Mutable reference to self for chaining operations
    ///
    /// # Example
    ///
    /// ```rust
    /// # use dominoes_state::ZHash;
    /// 
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
    ///
    /// * `value` - The open end value
    /// * `old_count` - The old count before the change
    /// * `new_count` - The new count after the change
    ///
    /// # Returns
    ///
    /// Mutable reference to self for method chaining
    ///
    /// # Important
    ///
    /// The method cannot determine if the end value and counts match the state. Garbage in, garbage out.
    ///
    /// # Panics
    ///
    /// Panics if `old_count` is the same as `new_count`.
    ///
    /// # Example
    ///
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
        assert!(old_count != new_count); // Sanity check
        // Undo the previous count
        self.value ^= Z_VALUE_TABLE.end_value(value as usize, old_count as usize);
        // Update with the new count
        self.value ^= Z_VALUE_TABLE.end_value(value as usize, new_count as usize);
        self
    }

    /// Updates the value to reflect a turn change
    ///
    /// This method updates the value to reflect that it now the next player's turn.
    /// Returns a mutable reference to self for method chaining.
    ///
    /// # Returns
    ///
    /// Mutable reference to self for chaining operations
    ///
    /// # Example
    ///
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
    ///
    /// `true` if the value is undefined, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// # use dominoes_state::ZHash;
    /// 
    /// let undefined = ZHash::from(ZHash::UNDEFINED);
    /// assert!(undefined.is_undefined());
    ///
    /// let normal = ZHash::default();
    /// assert!(!normal.is_undefined());
    /// ```
    pub fn is_undefined(&self) -> bool {
        self.value == Self::UNDEFINED
    }
}

/// Creates a ZHash representing the start of a game
///
/// Equivalent to `ZHash::from(ZHash::START)`.
///
/// # Example
///
/// ```rust
/// # use dominoes_state::ZHash;
/// 
/// let hash = ZHash::default();
/// let value: u64 = hash.into();
/// assert_eq!(value, ZHash::START);
/// ```
impl Default for ZHash {
    fn default() -> Self {
        Self::from(Self::START)
    }
}

/// Enables ZHash::from(u64) -> ZHash and u64::into() -> ZHash
impl From<u64> for ZHash {
    fn from(value: u64) -> Self {
        Self { value }
    }
}

/// Enables u64::from(ZHash) -> u64 and ZHash::into() -> u64
impl From<ZHash> for u64 {
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

use rand_chacha::ChaCha8Rng;

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
        for i in 0..256 {
            table.tile_hashes[i] = rng.next_u64();
        }

        // Initialize end value count hashes
        for value in 0..22 {
            table.end_hashes[value][0] = 0; // Hash for count 0 is 0 because it represents the starting state.
            for count in 1..22 {
                table.end_hashes[value][count] = rng.next_u64();
            }
        }

        // Initialize turn hash
        table.turn_hash = rng.next_u64();

        table
    }

    // Returns the hash for a specific tile
    //
    // # Arguments
    //
    // * `tile` - Tile ordinal (0-255)
    //
    // # Returns
    //
    // Pre-computed hash for the specified tile
    //
    // # Panics
    //
    // Panics if `tile >= 256`
    fn tile_value(&self, tile: usize) -> Z {
        self.tile_hashes[tile]
    }

    // Returns the hash for an end value count
    //
    // # Arguments
    //
    // * `which` - The end value (0-21, representing pip counts)
    // * `count` - How many times this end value appears (0-21)
    //
    // # Returns
    //
    // Pre-computed hash for the specified end value/count pair
    //
    // # Panics
    //
    // Panics if `which >= 22` or `count >= 22`
    fn end_value(&self, which: usize, count: usize) -> Z {
        assert!(which < 22, "which must be < 22, got {which}");
        assert!(count < 22, "count must be < 22, got {count}");
        self.end_hashes[which][count]
    }

    // Returns the hash for player turn changes
    //
    // # Returns
    //
    // Hash to use when the turn changes to the next player
    //
    // # Note
    //
    // For two-player games, only one turn hash is needed since XORing the same hash twice returns to the original state.
    fn turn_value(&self) -> Z {
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
static Z_VALUE_TABLE: std::sync::LazyLock<ZTable> = std::sync::LazyLock::new(ZTable::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zhash_constants() {
        assert_eq!(ZHash::START, 0);
        assert_eq!(ZHash::UNDEFINED, !0);
        assert_ne!(ZHash::START, ZHash::UNDEFINED);
    }

    #[test]
    fn test_zhash_new() {
        let hash = ZHash::from(42);
        assert_eq!(u64::from(hash), 42);
        
        let hash_zero = ZHash::from(0);
        assert_eq!(u64::from(hash_zero), 0);
        
        let hash_max = ZHash::from(u64::MAX);
        assert_eq!(u64::from(hash_max), u64::MAX);
    }

    #[test]
    fn test_zhash_default() {
        let hash = ZHash::default();
        assert_eq!(hash, ZHash::from(ZHash::START));
        assert_eq!(u64::from(hash), 0);
        assert!(!hash.is_undefined());
    }

    #[test]
    fn test_zhash_value() {
        let test_values = [0, 1, 42, 12345, u64::MAX, ZHash::UNDEFINED];
        
        for &val in &test_values {
            let hash = ZHash::from(val);
            assert_eq!(u64::from(hash), val);
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

    // Removed test_zhash_from_state_turn_2 as Layout doesn't have a default() method

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
    fn test_zhash_end_different_values() {
        let mut hash1 = ZHash::default();
        let mut hash2 = ZHash::default();
        
        // Same count, different end values should produce different hashes
        hash1.change_end_count(5, 0, 1);
        hash2.change_end_count(6, 0, 1);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_zhash_end_different_counts() {
        let mut hash1 = ZHash::default();
        let mut hash2 = ZHash::default();
        
        // Same end value, different counts should produce different hashes
        hash1.change_end_count(5, 0, 1);
        hash2.change_end_count(5, 0, 2);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_zhash_end_order_independence() {
        let mut hash1 = ZHash::default();
        let mut hash2 = ZHash::default();
        
        // Different order of building up end counts should produce same result
        hash1.change_end_count(5, 0, 1).change_end_count(6, 0, 1);
        hash2.change_end_count(6, 0, 1).change_end_count(5, 0, 1);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_zhash_end_max_values() {
        let mut hash = ZHash::default();
        
        // Test maximum end value and count
        hash.change_end_count(21, 0, 21);
        assert_ne!(hash, ZHash::default());
    }

    #[test]
    fn test_zhash_end_chaining_with_other_methods() {
        let mut hash1 = ZHash::default();
        let mut hash2 = ZHash::default();
        
        // Test chaining end methods with tile and turn methods
        hash1.add_tile(42).change_end_count(5, 0, 1).turn();
        hash2.turn().change_end_count(5, 0, 1).add_tile(42);
        
        assert_eq!(hash1, hash2);
    }

    // Removed test_zhash_from_state_invalid_turn as Layout doesn't have a default() method

    #[test]
    #[should_panic]
    fn test_zhash_change_end_count_same_counts() {
        let mut hash = ZHash::default();
        
        // Test valid boundary values
        hash.change_end_count(0, 0, 0);    // same counts
    }

    #[test]
    fn test_zhash_end_incremental_vs_direct() {
        // Test that incremental updates match what from_state would compute
        let mut incremental = ZHash::default();
        incremental.change_end_count(5, 0, 2);  // End value 5 appears 2 times
        incremental.change_end_count(6, 0, 1);  // End value 6 appears 1 time
        
        // Simulate direct computation
        let mut direct = ZHash::from(ZHash::START);
        direct.value ^= Z_VALUE_TABLE.end_value(5, 2);
        direct.value ^= Z_VALUE_TABLE.end_value(6, 1);
        
        assert_eq!(incremental, direct);
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
    fn test_zhash_turn_chaining() {
        let mut hash1 = ZHash::default();
        let mut hash2 = ZHash::default();
        
        // Test method chaining
        hash1.turn().add_tile(42);
        hash2.add_tile(42).turn();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_zhash_equality() {
        let hash1 = ZHash::from(42);
        let hash2 = ZHash::from(42);
        let hash3 = ZHash::from(43);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
    }

    #[test]
    fn test_zhash_copy_clone() {
        let hash1 = ZHash::from(42);
        let hash2 = hash1; // Copy
        let hash3 = hash1.clone(); // Clone
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1, hash3);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn test_zhash_debug() {
        let hash = ZHash::from(42);
        let debug_str = format!("{:?}", hash);
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_ztable_tile_value() {
        let table = &*Z_VALUE_TABLE;
        
        // Test that different tiles have different hashes
        let hash1 = table.tile_value(0);
        let hash2 = table.tile_value(1);
        let hash255 = table.tile_value(255);
        
        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash255);
        assert_ne!(hash2, hash255);
        
        // Test that same tile always returns same hash
        assert_eq!(table.tile_value(42), table.tile_value(42));
    }

    #[test]
    fn test_ztable_end_value() {
        let table = &*Z_VALUE_TABLE;
        
        // Test that different end values/counts have different hashes
        let hash_0_0 = table.end_value(0, 0);
        let hash_0_1 = table.end_value(0, 1);
        let hash_2_1 = table.end_value(2, 1);
        let hash_21_21 = table.end_value(21, 21);
        
        assert_ne!(hash_0_0, hash_0_1);
        assert_ne!(hash_0_0, hash_2_1);
        assert_ne!(hash_0_0, hash_21_21);
        assert_ne!(hash_0_1, hash_2_1);
        assert_ne!(hash_0_1, hash_21_21);
        assert_ne!(hash_2_1, hash_21_21);
        
        // Test that same parameters always return same hash
        assert_eq!(table.end_value(5, 3), table.end_value(5, 3));
    }

    #[test]
    fn test_ztable_turn_value() {
        let table = &*Z_VALUE_TABLE;
        
        // Turn hash should be consistent
        let turn_hash1 = table.turn_value();
        let turn_hash2 = table.turn_value();
        assert_eq!(turn_hash1, turn_hash2);
        
        // Turn hash should be non-zero (very unlikely to be zero with good RNG)
        assert_ne!(turn_hash1, 0);
    }

    #[test]
    fn test_ztable_singleton() {
        // Test that Z_VALUE_TABLE is a singleton
        let table1 = &*Z_VALUE_TABLE;
        let table2 = &*Z_VALUE_TABLE;
        
        // Should be the same instance
        assert!(std::ptr::eq(table1, table2));
        
        // Should have same values
        assert_eq!(table1.tile_value(42), table2.tile_value(42));
        assert_eq!(table1.end_value(5, 3), table2.end_value(5, 3));
        assert_eq!(table1.turn_value(), table2.turn_value());
    }

    #[test]
    fn test_zhash_order_independence() {
        // Test that order of operations doesn't matter (fundamental Zobrist property)
        let mut hash1 = ZHash::default();
        hash1.add_tile(10).add_tile(20).turn();
        
        let mut hash2 = ZHash::default();
        hash2.turn().add_tile(20).add_tile(10);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_zhash_incremental_property() {
        // Test that incremental updates produce same result as from_scratch
        let mut incremental = ZHash::default();
        incremental.add_tile(5).add_tile(10).turn();
        
        // Simulate what from_state would do with these operations
        let mut from_scratch = ZHash::from(ZHash::START);
        from_scratch.value ^= Z_VALUE_TABLE.tile_value(5);
        from_scratch.value ^= Z_VALUE_TABLE.tile_value(10);
        from_scratch.value ^= Z_VALUE_TABLE.turn_value();
        
        assert_eq!(incremental, from_scratch);
    }

    #[test]
    fn test_type_alias() {
        // Test that Z is indeed u64
        let _: Z = 42u64;
        let hash = ZHash::from(42u64);
        let value: Z = hash.into();
        assert_eq!(value, 42u64);
    }

    #[test]
    #[should_panic]
    fn test_ztable_tile_value_bounds() {
        let table = &*Z_VALUE_TABLE;
        table.tile_value(256); // Should panic
    }

    #[test]
    #[should_panic]
    fn test_ztable_end_value_bounds_value() {
        let table = &*Z_VALUE_TABLE;
        table.end_value(22, 0); // Should panic
    }

    #[test]
    #[should_panic]
    fn test_ztable_end_value_bounds_count() {
        let table = &*Z_VALUE_TABLE;
        table.end_value(0, 22); // Should panic
    }

    #[test]
    fn test_zhash_hash_trait() {
        // Test that ZHash implements Hash trait properly
        use std::collections::HashMap;
        
        let mut map = HashMap::new();
        let hash1 = ZHash::from(42);
        let hash2 = ZHash::from(43);
        
        map.insert(hash1, "value1");
        map.insert(hash2, "value2");
        
        assert_eq!(map.get(&hash1), Some(&"value1"));
        assert_eq!(map.get(&hash2), Some(&"value2"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_zhash_different_tiles_different_hashes() {
        // Verify that adding different tiles produces different results
        let mut hash1 = ZHash::default();
        let mut hash2 = ZHash::default();
        
        hash1.add_tile(1);
        hash2.add_tile(2);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_zhash_reproducibility() {
        // Test that the same operations always produce the same result
        // (thanks to seeded RNG in ZTable)
        let mut hash1 = ZHash::default();
        hash1.add_tile(42).turn();
        
        let mut hash2 = ZHash::default();
        hash2.add_tile(42).turn();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_zhash_new_method() {
        // Test the new() method specifically
        let hash = ZHash::new(12345);
        assert_eq!(u64::from(hash), 12345);
        
        let hash_zero = ZHash::new(0);
        assert_eq!(u64::from(hash_zero), 0);
        
        let hash_max = ZHash::new(u64::MAX);
        assert_eq!(u64::from(hash_max), u64::MAX);
        
        let hash_undefined = ZHash::new(ZHash::UNDEFINED);
        assert!(hash_undefined.is_undefined());
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
    fn test_zhash_from_state_with_tiles() {
        use rules::{Configuration, Tile};
        
        let config = Configuration::default();
        let mut layout = crate::Layout::new(&config);
        
        // Add a tile to the layout
        let double_six = Tile::from((6, 6));
        layout.attach(double_six, None);
        
        let hash_with_tile = ZHash::from_state(&layout, 0);
        let hash_empty = ZHash::from_state(&crate::Layout::new(&config), 0);
        
        assert_ne!(hash_with_tile, hash_empty);
        assert_ne!(hash_with_tile, ZHash::default());
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
    fn test_zhash_from_state_reproducible() {
        use rules::{Configuration, Tile};
        
        let config = Configuration::default();
        let mut layout = crate::Layout::new(&config);
        
        // Add tiles to create a specific state
        let double_six = Tile::from((6, 6));
        layout.attach(double_six, None);
        
        let six_three = Tile::from((3, 6));
        layout.attach(six_three, Some(0));
        
        // Multiple calls should produce same result
        let hash1 = ZHash::from_state(&layout, 0);
        let hash2 = ZHash::from_state(&layout, 0);
        assert_eq!(hash1, hash2);
        
        // Different turn should produce different result
        let hash_turn1 = ZHash::from_state(&layout, 1);
        assert_ne!(hash1, hash_turn1);
    }

    #[test]
    fn test_zhash_from_state_vs_incremental() {
        use rules::{Configuration, Tile};
        
        let config = Configuration::default();
        let mut layout = crate::Layout::new(&config);
        
        // Build state incrementally
        let mut incremental = ZHash::default();
        
        // Add double-six
        let double_six = Tile::from((6, 6));
        layout.attach(double_six, None);
        incremental.add_tile(u8::from(double_six));
        incremental.change_end_count(6, 0, 2); // Double adds 2 open ends
        
        // Compare with from_state
        let from_state = ZHash::from_state(&layout, 0);
        assert_eq!(incremental, from_state);
    }

    #[test]
    fn test_zhash_add_tile_various_values() {
        // Test add_tile with boundary values
        let mut hash = ZHash::default();
        let original = hash;
        
        // Test with 0
        hash.add_tile(0);
        assert_ne!(hash, original);
        let after_zero = hash;
        
        // Test with 255 (max u8)
        hash.add_tile(255);
        assert_ne!(hash, after_zero);
        assert_ne!(hash, original);
        
        // Test that adding same tile twice returns to original (XOR property)
        hash.add_tile(255);
        assert_eq!(hash, after_zero);
        
        hash.add_tile(0);
        assert_eq!(hash, original);
    }

    #[test]
    fn test_zhash_change_end_count_boundary_values() {
        let mut hash = ZHash::default();
        
        // Test with minimum values
        hash.change_end_count(0, 0, 1);
        assert_ne!(hash, ZHash::default());
        
        // Test with maximum values
        let mut hash2 = ZHash::default();
        hash2.change_end_count(21, 0, 21);
        assert_ne!(hash2, ZHash::default());
        assert_ne!(hash2, hash);
        
        // Test reversibility
        hash.change_end_count(0, 1, 0);
        assert_eq!(hash, ZHash::default());
        
        hash2.change_end_count(21, 21, 0);
        assert_eq!(hash2, ZHash::default());
    }

    #[test]
    fn test_zhash_change_end_count_all_values() {
        // Test that different end values produce different hashes
        let mut hashes = Vec::new();
        
        for end_value in 0..22 {
            let mut hash = ZHash::default();
            hash.change_end_count(end_value, 0, 1);
            hashes.push(hash);
        }
        
        // All hashes should be different
        for i in 0..hashes.len() {
            for j in i+1..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "Hash collision between end values {} and {}", i, j);
            }
        }
    }

    #[test]
    fn test_zhash_change_end_count_all_counts() {
        // Test that different counts for same end value produce different hashes
        let mut hashes = Vec::new();
        
        for count in 0..22 {
            let mut hash = ZHash::default();
            if count > 0 {
                hash.change_end_count(5, 0, count);
            }
            hashes.push(hash);
        }
        
        // All hashes should be different
        for i in 0..hashes.len() {
            for j in i+1..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "Hash collision between counts {} and {}", i, j);
            }
        }
    }

    #[test]
    fn test_zhash_turn_consistency() {
        // Test that turn() always produces the same change
        let original = ZHash::default();
        
        let mut hash1 = original;
        hash1.turn();
        
        let mut hash2 = original;
        hash2.turn();
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, original);
    }

    #[test]
    fn test_zhash_turn_with_different_states() {
        // Test turn with various starting states
        let states = vec![
            ZHash::default(),
            ZHash::from(42),
            ZHash::from(u64::MAX),
            ZHash::from(12345),
        ];
        
        for state in states {
            let mut turned = state;
            turned.turn();
            assert_ne!(turned, state);
            
            // Turn again should return to original
            turned.turn();
            assert_eq!(turned, state);
        }
    }

    #[test]
    fn test_zhash_method_combinations() {
        // Test various combinations of methods
        let mut hash1 = ZHash::default();
        let mut hash2 = ZHash::default();
        
        // Same operations in different order should produce same result
        hash1.add_tile(10).change_end_count(5, 0, 2).turn().add_tile(20);
        hash2.turn().add_tile(20).add_tile(10).change_end_count(5, 0, 2);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_zhash_xor_properties() {
        // Test fundamental XOR properties of Zobrist hashing
        let mut hash = ZHash::default();
        let original = hash;
        
        // Adding and removing same tile should return to original
        hash.add_tile(42);
        assert_ne!(hash, original);
        hash.add_tile(42); // XOR again
        assert_eq!(hash, original);
        
        // Same for end counts
        hash.change_end_count(6, 0, 3);
        assert_ne!(hash, original);
        hash.change_end_count(6, 3, 0);
        assert_eq!(hash, original);
        
        // Same for turns
        hash.turn();
        assert_ne!(hash, original);
        hash.turn();
        assert_eq!(hash, original);
    }

    #[test]
    fn test_type_z_alias() {
        // Test that Z type alias works correctly
        let value: Z = 12345;
        let hash = ZHash::from(value);
        let retrieved: Z = hash.into();
        assert_eq!(value, retrieved);
        
        // Test with constants
        let start: Z = ZHash::START;
        let undefined: Z = ZHash::UNDEFINED;
        assert_eq!(start, 0);
        assert_eq!(undefined, u64::MAX);
    }

    #[test]
    fn test_zhash_constants_properties() {
        // Test properties of the constants
        assert_eq!(ZHash::START, 0);
        assert_eq!(ZHash::UNDEFINED, !0);
        
        // START should not be undefined
        let start_hash = ZHash::from(ZHash::START);
        assert!(!start_hash.is_undefined());
        
        // UNDEFINED should be undefined
        let undefined_hash = ZHash::from(ZHash::UNDEFINED);
        assert!(undefined_hash.is_undefined());
        
        // They should be different
        assert_ne!(ZHash::START, ZHash::UNDEFINED);
    }

    #[test]
    fn test_zhash_ord_trait() {
        // Test that ZHash implements Ord correctly
        let hash1 = ZHash::from(100);
        let hash2 = ZHash::from(200);
        let hash3 = ZHash::from(100);
        
        assert!(hash1 < hash2);
        assert!(hash2 > hash1);
        assert!(hash1 <= hash2);
        assert!(hash2 >= hash1);
        assert!(hash1 <= hash3);
        assert!(hash1 >= hash3);
        
        // Test with constants
        let start = ZHash::from(ZHash::START);
        let undefined = ZHash::from(ZHash::UNDEFINED);
        assert!(start < undefined); // 0 < MAX
    }

    #[test]
    fn test_zhash_partialord_trait() {
        use std::cmp::Ordering;
        
        let hash1 = ZHash::from(100);
        let hash2 = ZHash::from(200);
        let hash3 = ZHash::from(100);
        
        assert_eq!(hash1.partial_cmp(&hash2), Some(Ordering::Less));
        assert_eq!(hash2.partial_cmp(&hash1), Some(Ordering::Greater));
        assert_eq!(hash1.partial_cmp(&hash3), Some(Ordering::Equal));
    }
}
