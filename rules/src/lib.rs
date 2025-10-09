//! # Dominoes Rules and Types
//!
//! This crate provides the fundamental types, rules, and utilities for dominoes games.
//!
//! ## Examples
//! ```rust
//! # use rules::{Configuration, Variation, Tile};
//!
//! // Create a standard double-six game for 4 players
//! let config = Configuration::new(4, Variation::Traditional, 6, 6);
//!
//! // Work with individual tiles
//! let tile = Tile::from((3, 5));
//! println!("Tile: {}", tile); // Prints "3|5"
//! assert!(tile.is_double() == false);
//! ```

pub mod boneyard;
pub mod configuration;
pub mod layout;
pub mod tile;

pub use boneyard::*;
pub use configuration::*;
pub use layout::*;
pub use tile::*;

/// Domino game variations
///
/// # Examples
/// ```rust
/// # use rules::{Variation, default_starting_hand_size};
///
/// // Different variations have different hand sizes
/// assert_eq!(default_starting_hand_size(2, Variation::Traditional), 7);
/// assert_eq!(default_starting_hand_size(2, Variation::Bergen), 6);
/// assert_eq!(default_starting_hand_size(2, Variation::Blind), 8);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variation {
    Traditional,
    AllFives,
    AllSevens,
    Bergen,
    Blind,
    FiveUp,
}

impl Variation {
    /// Returns the name of the variation as a string.
    ///
    /// # Examples
    /// ```rust
    /// # use rules::Variation;
    ///
    /// assert_eq!(Variation::Traditional.name(), "Traditional");
    /// assert_eq!(Variation::AllFives.name(), "All Fives");
    /// assert_eq!(Variation::Bergen.name(), "Bergen");
    /// ```
    pub const fn name(self) -> &'static str {
        match self {
            Variation::Traditional => "Traditional",
            Variation::AllFives => "All Fives",
            Variation::AllSevens => "All Sevens",
            Variation::Bergen => "Bergen",
            Variation::Blind => "Blind",
            Variation::FiveUp => "Five Up",
        }
    }
}

/// Maximum number of pips on a domino tile supported by this library
pub const MAX_PIPS: u8 = 21;

// All domino tiles in order.
//
// This constant array contains all possible domino tile combinations in canonical form,
// indexed by their ordinal values. Each tuple (a, b) represents a domino tile where a ≤ b.
//
// # Note
// This array supports domino sets up to double-21, containing 253 total tiles.
const TUPLES: [(u8, u8); 253] = [
    (0, 0),
    (0, 1), (1, 1),
    (0, 2), (1, 2), (2, 2),
    (0, 3), (1, 3), (2, 3), (3, 3),
    (0, 4), (1, 4), (2, 4), (3, 4), (4, 4),
    (0, 5), (1, 5), (2, 5), (3, 5), (4, 5), (5, 5),
    (0, 6), (1, 6), (2, 6), (3, 6), (4, 6), (5, 6), (6, 6),
    (0, 7), (1, 7), (2, 7), (3, 7), (4, 7), (5, 7), (6, 7), (7, 7),
    (0, 8), (1, 8), (2, 8), (3, 8), (4, 8), (5, 8), (6, 8), (7, 8), (8, 8),
    (0, 9), (1, 9), (2, 9), (3, 9), (4, 9), (5, 9), (6, 9), (7, 9), (8, 9), (9, 9),
    (0, 10), (1, 10), (2, 10), (3, 10), (4, 10), (5, 10), (6, 10), (7, 10), (8, 10), (9, 10), (10, 10),
    (0, 11), (1, 11), (2, 11), (3, 11), (4, 11), (5, 11), (6, 11), (7, 11), (8, 11), (9, 11), (10, 11), (11, 11),
    (0, 12), (1, 12), (2, 12), (3, 12), (4, 12), (5, 12), (6, 12), (7, 12), (8, 12), (9, 12), (10, 12), (11, 12), (12, 12),
    (0, 13), (1, 13), (2, 13), (3, 13), (4, 13), (5, 13), (6, 13), (7, 13), (8, 13), (9, 13), (10, 13), (11, 13), (12, 13),
    (13, 13),
    (0, 14), (1, 14), (2, 14), (3, 14), (4, 14), (5, 14), (6, 14), (7, 14), (8, 14), (9, 14), (10, 14), (11, 14), (12, 14),
    (13, 14), (14, 14),
    (0, 15), (1, 15), (2, 15), (3, 15), (4, 15), (5, 15), (6, 15), (7, 15), (8, 15), (9, 15), (10, 15), (11, 15), (12, 15),
    (13, 15), (14, 15), (15, 15),
    (0, 16), (1, 16), (2, 16), (3, 16), (4, 16), (5, 16), (6, 16), (7, 16), (8, 16), (9, 16), (10, 16), (11, 16), (12, 16),
    (13, 16), (14, 16), (15, 16), (16, 16),
    (0, 17), (1, 17), (2, 17), (3, 17), (4, 17), (5, 17), (6, 17), (7, 17), (8, 17), (9, 17), (10, 17), (11, 17), (12, 17),
    (13, 17), (14, 17), (15, 17), (16, 17), (17, 17),
    (0, 18), (1, 18), (2, 18), (3, 18), (4, 18), (5, 18), (6, 18), (7, 18), (8, 18), (9, 18), (10, 18), (11, 18), (12, 18),
    (13, 18), (14, 18), (15, 18), (16, 18), (17, 18), (18, 18),
    (0, 19), (1, 19), (2, 19), (3, 19), (4, 19), (5, 19), (6, 19), (7, 19), (8, 19), (9, 19), (10, 19), (11, 19), (12, 19),
    (13, 19), (14, 19), (15, 19), (16, 19), (17, 19), (18, 19), (19, 19),
    (0, 20), (1, 20), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20), (7, 20), (8, 20), (9, 20), (10, 20), (11, 20), (12, 20),
    (13, 20), (14, 20), (15, 20), (16, 20), (17, 20), (18, 20), (19, 20), (20, 20),
    (0, 21), (1, 21), (2, 21), (3, 21), (4, 21), (5, 21), (6, 21), (7, 21), (8, 21), (9, 21), (10, 21), (11, 21), (12, 21),
    (13, 21), (14, 21), (15, 21), (16, 21), (17, 21), (18, 21), (19, 21), (20, 21), (21, 21)
];

// Ordinal values of double tiles for efficient lookup.
//
// This constant array contains the ordinal values corresponding to double tiles
// (tiles where both sides have the same value). Used for fast double tile detection.
//
// # Note
// Contains ordinals for doubles from (0,0) up to (21,21).
const DOUBLES: [u8; 22] = [
    0,   // (0,0)
    2,   // (1,1)
    5,   // (2,2)
    9,   // (3,3)
    14,  // (4,4)
    20,  // (5,5)
    27,  // (6,6)
    35,  // (7,7)
    44,  // (8,8)
    54,  // (9,9)
    65,  // (10,10)
    77,  // (11,11)
    90,  // (12,12)
    104, // (13,13)
    119, // (14,14)
    135, // (15,15)
    152, // (16,16)
    170, // (17,17)
    189, // (18,18)
    209, // (19,19)
    230, // (20,20)
    252, // (21,21)
];

/// Converts a domino tile tuple to its ordinal value.
///
/// This function maps each domino tile to a unique u8 ordinal value 0 to the number of tiles - 1.
///
/// # Arguments
/// * `tile` - A domino tile where the first value &le; second value (canonical form)
///
/// # Returns
/// The ordinal value for the tile as u8
///
/// # Panics
/// * If the input tuple is not in canonical form (i.e., if the first element is greater than the second).
/// * If the calculated ordinal exceeds `u8::MAX` (shouldn't happen for valid tiles)
///
/// # Examples
/// ```
/// # use rules::tuple_to_ordinal;
///
/// assert_eq!(tuple_to_ordinal((0, 0)), 0);
/// assert_eq!(tuple_to_ordinal((0, 1)), 1);
/// assert_eq!(tuple_to_ordinal((1, 1)), 2);
/// assert_eq!(tuple_to_ordinal((0, 6)), 21);
/// assert_eq!(tuple_to_ordinal((6, 6)), 27);
/// ```
pub const fn tuple_to_ordinal((a, b): (u8, u8)) -> u8 {
    assert!(a <= b, "Tile must be in canonical form (first <= second)");

    // Use usize to prevent overflow during calculation
    let ordinal = (b as usize) * (b as usize + 1) / 2 + a as usize;
    assert!(ordinal <= u8::MAX as usize, "Ordinal exceeds u8::MAX");

    ordinal as u8
}

/// Converts an ordinal to its corresponding tuple.
///
/// This is the inverse operation of `tuple_to_ordinal()`.
///
/// # Arguments
/// * `ordinal` - The unique ordinal (0-252)
///
/// # Returns
/// The corresponding domino tuple (a, b)
///
/// # Panics
/// If `ordinal` is outside the valid range (&ge; 253)
///
/// # Examples
/// ```rust
/// # use rules::ordinal_to_tuple;
///
/// assert_eq!(ordinal_to_tuple(0), (0, 0));
/// assert_eq!(ordinal_to_tuple(1), (0, 1));
/// assert_eq!(ordinal_to_tuple(2), (1, 1));
/// assert_eq!(ordinal_to_tuple(21), (0, 6));
/// assert_eq!(ordinal_to_tuple(27), (6, 6));
/// ```
///
/// # Performance
/// This is a `const fn` with O(1) performance.
pub const fn ordinal_to_tuple(ordinal: u8) -> (u8, u8) {
    TUPLES[ordinal as usize]
}

/// Returns `true` if the tile is a double (both sides equal).
///
/// Double tiles are important in many domino games as they often have
/// special placement rules, scoring bonuses, or are used to start games.
///
/// # Arguments
/// * `tile` - A domino tile represented as a tuple (u8, u8).
///
/// # Returns
/// `true` if both values of the tile are equal, `false` otherwise
///
/// # Examples
/// ```rust
/// # use rules::is_double_tuple;
///
/// assert!(is_double_tuple((0, 0)));   // Double-blank
/// assert!(is_double_tuple((6, 6)));   // Double-six
/// assert!(!is_double_tuple((0, 1)));  // Not a double
/// assert!(!is_double_tuple((2, 3)));  // Not a double
/// ```
///
/// # Performance
/// This is a `const fn` with O(1) performance.
pub const fn is_double_tuple((a, b): (u8, u8)) -> bool {
    a == b
}

/// Returns `true` if the ordinal corresponds to a double tile.
///
/// # Arguments
/// * `ordinal` - The unique ordinal of the domino tile
///
/// # Returns
/// `true` if the ordinal corresponds to a double tile, `false` otherwise
///
/// # Examples
/// ```
/// # use rules::is_double_ordinal;
///
/// assert!(is_double_ordinal(0));   // (0,0)
/// assert!(is_double_ordinal(2));   // (1,1)
/// assert!(is_double_ordinal(27));  // (6,6)
/// assert!(!is_double_ordinal(1));  // (0,1)
/// assert!(!is_double_ordinal(3));  // (0,2)
/// ```
///
/// # Performance
/// This is a `const fn` with O(log n) performance via binary search.
pub const fn is_double_ordinal(ordinal: u8) -> bool {
    // Manual binary search for const context
    let mut left = 0;
    let mut right = DOUBLES.len();

    while left < right {
        let mid = (left + right) / 2;
        if DOUBLES[mid] < ordinal {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    left < DOUBLES.len() && DOUBLES[left] == ordinal
}

/// Checks if two domino tuples match on either value.
///
/// A match occurs if either value of the first tuple equals either value of the second tuple.
///
/// # Arguments
/// * `a` - The first tuple
/// * `b` - The second tuple
///
/// # Returns
/// `Some((matching_a_value, other_a_value))` if there is a match, `None` otherwise
///
/// # Notes
/// - It is not possible in a standard domino set for the match to be ambiguous.
///
/// # Examples
/// ```rust
/// # use rules::matches_tuples;
/// assert_eq!(matches_tuples((2, 5), (5, 3)), Some((5, 2))); // Match on 5
/// assert_eq!(matches_tuples((1, 4), (4, 4)), Some((4, 1))); // Match on 4
/// assert_eq!(matches_tuples((0, 0), (0, 1)), Some((0, 0))); // Match on 0
/// assert_eq!(matches_tuples((2, 3), (4, 5)), None);      // No match
/// ```
pub const fn matches_tuples(a: (u8, u8), b: (u8, u8)) -> Option<(u8, u8)> {
    if a.0 == b.0 || a.0 == b.1 {
        Some((a.0, a.1))
    } else if a.1 == b.0 || a.1 == b.1 {
        Some((a.1, a.0))
    } else {
        None
    }
}

/// Returns the total number of tiles in a double-N domino set.
///
/// # Arguments
/// * `n` - The highest value in the domino set. Same as set ID
///
/// # Returns
/// The total number of tiles in the set
///
/// # Examples
/// ```
/// # use rules::set_size;
///
/// assert_eq!(set_size(0), 1);   // Only (0,0)
/// assert_eq!(set_size(1), 3);   // (0,0), (0,1), (1,1)
/// assert_eq!(set_size(6), 28);  // Standard double-six set
/// assert_eq!(set_size(9), 55);  // Double-nine set
/// assert_eq!(set_size(12), 91); // Double-twelve set
/// ```
///
/// # Performance
/// This is a `const fn` and can be evaluated at compile time.
pub const fn set_size(n: u8) -> usize {
    (n as usize + 1) * (n as usize + 2) / 2
}

/// Returns a sorted vector containing all domino tiles for a given set as tuples.
///
/// # Arguments
/// * `set_id` - ID of the set. Same as the highest value on the tiles.
///
/// # Returns
/// A vector containing all unique tile combinations as tuples in canonical order
///
/// # Examples
/// ```
/// # use rules::all_tiles_as_tuples;
///
/// let tiles = all_tiles_as_tuples(2);
/// assert_eq!(tiles, vec![
///     (0, 0), (0, 1), (1, 1), (0, 2), (1, 2), (2, 2)
/// ]);
///
/// let double_six = all_tiles_as_tuples(6);
/// assert_eq!(double_six.len(), 28); // Standard domino set
/// ```
pub fn all_tiles_as_tuples(set_id: u8) -> Vec<(u8, u8)> {
    (0..=set_id)
        .flat_map(|b| (0..=b).map(move |a| (a, b)))
        .collect()
}

/// Returns a sorted Vec containing all domino tiles for a given set
///
/// # Arguments
/// * `set_id` - ID of the set. Same as the highest value on the tiles.
///
/// # Returns
/// A sorted vector containing all tiles in the set
///
/// # Examples
/// ```
/// # use rules::{all_tiles_as_tiles, Tile};
///
/// let tiles = all_tiles_as_tiles(2);
/// assert_eq!(tiles.len(), 6);
///
/// // First few tiles in canonical order
/// assert_eq!(tiles[0], Tile::from((0, 0))); // First tile
/// assert_eq!(tiles[1], Tile::from((0, 1))); // Second tile
/// assert_eq!(tiles[2], Tile::from((1, 1))); // Third tile
///
/// let double_six = all_tiles_as_tiles(6);
/// assert_eq!(double_six.len(), 28); // Standard domino set
/// ```
pub fn all_tiles_as_tiles(set_id: u8) -> Vec<Tile> {
    (0..set_size(set_id) as u8).map(Tile::from).collect()
}

/// Returns a sorted vector containing all ordinal values for a given set as tuples.
///
/// # Arguments
/// * `set_id` - ID of the set. Same as the highest value on the tiles.
///
/// # Returns
/// A vector containing all ordinal values for the set
///
/// # Examples
/// ```
/// # use rules::all_tiles_as_ordinals;
///
/// let ordinals = all_tiles_as_ordinals(2);
/// assert_eq!(ordinals, vec![0, 1, 2, 3, 4, 5]);
///
/// let double_six = all_tiles_as_ordinals(6);
/// assert_eq!(double_six.len(), 28);
/// assert_eq!(double_six[0], 0);    // (0,0)
/// assert_eq!(double_six[27], 27);  // (6,6)
/// ```
pub fn all_tiles_as_ordinals(set_id: u8) -> Vec<u8> {
    (0..set_size(set_id) as u8).collect()
}

/// Returns the default starting hand size based on game variation and player count.
///
/// This function determines the initial hand size for each player based on the domino game variation and the number of players.
///
/// # Hand Size Rules
///
/// * **Traditional, All-Fives, All-Sevens, Five-Up**:
///   - 2 players: 7 tiles
///   - 3-4 players: 6 tiles
///   - 5+ players: 5 tiles
/// * **Bergen**: Always 6 tiles regardless of player count
/// * **Blind**:
///   - 2 players: 8 tiles
///   - 3 players: 7 tiles
///   - 4 players: 6 tiles
///   - 5+ players: 5 tiles
///
/// # Arguments
/// * `num_players` - The number of players in the game
/// * `variation` - The game variation being played
///
/// # Returns
/// The number of tiles each player should start with
///
/// # Examples
/// ```
/// # use rules::{default_starting_hand_size, Variation};
///
/// // Traditional game with different player counts
/// assert_eq!(default_starting_hand_size(2, Variation::Traditional), 7);
/// assert_eq!(default_starting_hand_size(4, Variation::Traditional), 6);
/// assert_eq!(default_starting_hand_size(6, Variation::Traditional), 5);
///
/// // Bergen always uses 6 tiles
/// assert_eq!(default_starting_hand_size(2, Variation::Bergen), 6);
/// assert_eq!(default_starting_hand_size(8, Variation::Bergen), 6);
///
/// // Blind uses more tiles initially
/// assert_eq!(default_starting_hand_size(2, Variation::Blind), 8);
/// assert_eq!(default_starting_hand_size(3, Variation::Blind), 7);
/// ```
pub const fn default_starting_hand_size(num_players: usize, variation: Variation) -> usize {
    use Variation::*;
    match variation {
        Traditional | AllFives | AllSevens | FiveUp => match num_players {
            2 => 7,
            3 | 4 => 6,
            _ => 5,
        },
        Bergen => 6,
        Blind => match num_players {
            2 => 8,
            3 => 7,
            4 => 6,
            _ => 5,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variation_debug() {
        let var = Variation::Traditional;
        let debug_str = format!("{:?}", var);
        assert!(debug_str.contains("Traditional"));
    }

    #[test]
    fn test_variation_equality() {
        assert_eq!(Variation::Traditional, Variation::Traditional);
        assert_ne!(Variation::Traditional, Variation::AllFives);
        assert_eq!(Variation::Bergen, Variation::Bergen);
    }

    #[test]
    fn test_set_size_comprehensive() {
        assert_eq!(set_size(0), 1);   // Only (0,0)
        assert_eq!(set_size(1), 3);   // (0,0), (0,1), (1,1)
        assert_eq!(set_size(2), 6);   // Double-two set
        assert_eq!(set_size(3), 10);  // Double-three set
        assert_eq!(set_size(6), 28);  // Standard double-six set
        assert_eq!(set_size(9), 55);  // Double-nine set
        assert_eq!(set_size(12), 91); // Double-twelve set
        assert_eq!(set_size(15), 136); // Double-fifteen set
        assert_eq!(set_size(21), 253); // Maximum supported set
    }

    #[test]
    fn test_all_tiles_as_tuples_comprehensive() {
        let tiles_0 = all_tiles_as_tuples(0);
        assert_eq!(tiles_0, vec![(0, 0)]);

        let tiles_1 = all_tiles_as_tuples(1);
        assert_eq!(tiles_1, vec![(0, 0), (0, 1), (1, 1)]);

        let tiles_2 = all_tiles_as_tuples(2);
        assert_eq!(tiles_2, vec![
            (0, 0), (0, 1), (1, 1), (0, 2), (1, 2), (2, 2)
        ]);

        let tiles_3 = all_tiles_as_tuples(3);
        assert_eq!(tiles_3.len(), 10);
        assert_eq!(tiles_3[0], (0, 0));
        assert_eq!(tiles_3[9], (3, 3));
    }

    #[test]
    fn test_all_tiles_as_tiles_comprehensive() {
        let tiles_0 = all_tiles_as_tiles(0);
        assert_eq!(tiles_0.len(), 1);
        assert_eq!(tiles_0[0], Tile::from((0, 0)));

        let tiles_2 = all_tiles_as_tiles(2);
        assert_eq!(tiles_2.len(), 6);
        assert_eq!(tiles_2[0], Tile::from((0, 0)));
        assert_eq!(tiles_2[1], Tile::from((0, 1)));
        assert_eq!(tiles_2[2], Tile::from((1, 1)));
        assert_eq!(tiles_2[5], Tile::from((2, 2)));

        let tiles_6 = all_tiles_as_tiles(6);
        assert_eq!(tiles_6.len(), 28);
        assert_eq!(tiles_6[0], Tile::from((0, 0)));
        assert_eq!(tiles_6[27], Tile::from((6, 6)));
    }

    #[test]
    fn test_all_tiles_as_ordinals_comprehensive() {
        let ordinals_0 = all_tiles_as_ordinals(0);
        assert_eq!(ordinals_0, vec![0]);

        let ordinals_2 = all_tiles_as_ordinals(2);
        assert_eq!(ordinals_2, vec![0, 1, 2, 3, 4, 5]);

        let ordinals_6 = all_tiles_as_ordinals(6);
        assert_eq!(ordinals_6.len(), 28);
        assert_eq!(ordinals_6[0], 0);
        assert_eq!(ordinals_6[27], 27);
    }

    #[test]
    fn test_default_starting_hand_size_comprehensive() {
        // Traditional variation
        assert_eq!(default_starting_hand_size(2, Variation::Traditional), 7);
        assert_eq!(default_starting_hand_size(3, Variation::Traditional), 6);
        assert_eq!(default_starting_hand_size(4, Variation::Traditional), 6);
        assert_eq!(default_starting_hand_size(5, Variation::Traditional), 5);
        assert_eq!(default_starting_hand_size(8, Variation::Traditional), 5);

        // AllFives variation (same as Traditional)
        assert_eq!(default_starting_hand_size(2, Variation::AllFives), 7);
        assert_eq!(default_starting_hand_size(4, Variation::AllFives), 6);
        assert_eq!(default_starting_hand_size(6, Variation::AllFives), 5);

        // AllSevens variation (same as Traditional)
        assert_eq!(default_starting_hand_size(2, Variation::AllSevens), 7);
        assert_eq!(default_starting_hand_size(4, Variation::AllSevens), 6);

        // FiveUp variation (same as Traditional)
        assert_eq!(default_starting_hand_size(2, Variation::FiveUp), 7);
        assert_eq!(default_starting_hand_size(4, Variation::FiveUp), 6);

        // Bergen variation (always 6)
        assert_eq!(default_starting_hand_size(2, Variation::Bergen), 6);
        assert_eq!(default_starting_hand_size(4, Variation::Bergen), 6);
        assert_eq!(default_starting_hand_size(8, Variation::Bergen), 6);
        assert_eq!(default_starting_hand_size(10, Variation::Bergen), 6);

        // Blind variation
        assert_eq!(default_starting_hand_size(2, Variation::Blind), 8);
        assert_eq!(default_starting_hand_size(3, Variation::Blind), 7);
        assert_eq!(default_starting_hand_size(4, Variation::Blind), 6);
        assert_eq!(default_starting_hand_size(5, Variation::Blind), 5);
        assert_eq!(default_starting_hand_size(8, Variation::Blind), 5);
    }
    #[test]
    fn test_tuple_ordinal_conversion_comprehensive() {
        // Test basic conversion cases
        let test_cases = [
            ((0, 0), 0), ((0, 1), 1), ((1, 1), 2), ((0, 2), 3),
            ((1, 2), 4), ((2, 2), 5), ((0, 6), 21), ((6, 6), 27),
        ];

        for ((a, b), expected_ordinal) in test_cases {
            assert_eq!(tuple_to_ordinal((a, b)), expected_ordinal);
            assert_eq!(ordinal_to_tuple(expected_ordinal), (a, b));
        }

        // Test boundary cases
        assert_eq!(tuple_to_ordinal((0, 0)), 0);
        assert_eq!(tuple_to_ordinal((0, 21)), 231);
        assert_eq!(tuple_to_ordinal((21, 21)), 252);
        assert_eq!(ordinal_to_tuple(0), (0, 0));
        assert_eq!(ordinal_to_tuple(252), (21, 21));

        // Test all ordinals are valid
        for ordinal in 0..=252 {
            let (a, b) = ordinal_to_tuple(ordinal);
            assert!(a <= b); // Should be in canonical form
            assert!(a <= 21 && b <= 21); // Should be within valid range
        }

        // Test doubles
        for i in 0..=21 {
            let ordinal = tuple_to_ordinal((i, i));
            assert!(is_double_ordinal(ordinal));
        }
    }

    #[test]
    fn test_double_detection_comprehensive() {
        // Test tuple double detection
        assert!(is_double_tuple((0, 0)));
        assert!(is_double_tuple((6, 6)));
        assert!(!is_double_tuple((0, 1)));

        // Test ordinal double detection
        assert!(is_double_ordinal(0));  // (0,0)
        assert!(is_double_ordinal(27)); // (6,6)
        assert!(!is_double_ordinal(1)); // (0,1)

        // Test all doubles
        for i in 0..=21 {
            let ordinal = tuple_to_ordinal((i, i));
            assert!(is_double_ordinal(ordinal));
        }

        // Test some non-doubles
        assert!(!is_double_ordinal(1));   // (0,1)
        assert!(!is_double_ordinal(3));   // (0,2)
        assert!(!is_double_ordinal(4));   // (1,2)
        assert!(!is_double_ordinal(6));   // (0,3)
        assert!(!is_double_ordinal(251)); // (20,21)
    }

    #[test]
    fn test_all_variations_defined() {
        // Ensure all variations can be used
        let variations = [
            Variation::Traditional,
            Variation::AllFives,
            Variation::AllSevens,
            Variation::Bergen,
            Variation::Blind,
            Variation::FiveUp,
        ];

        for variation in variations {
            let hand_size = default_starting_hand_size(2, variation);
            assert!(hand_size > 0);
            assert!(hand_size <= 10); // Reasonable upper bound
        }
    }
}
