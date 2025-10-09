use serde::{Serialize, Deserialize, Serializer, Deserializer, de};
use serde::de::{Visitor, SeqAccess};
use std::fmt;

use crate::*;

/// A domino tile represented by its ordinal value.
///
/// ## Examples
/// ```rust
/// # use rules::Tile;
///
/// // Create tiles in different ways
/// let tile1 = Tile::new(5);           // From ordinal
/// let tile2 = Tile::from((1, 2));     // From tuple
/// let tile3: Tile = 10u8.into();      // From u8
///
/// // Convert back to different formats
/// let (a, b) = tile2.as_tuple();      // (1, 2)
/// let ordinal: u8 = tile2.into();     // 4
///
/// // Check properties
/// assert!(!tile2.is_double());
/// assert!(Tile::from((3, 3)).is_double());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tile {
    /// The ordinal value of this tile.
    pub ordinal: u8,
}

impl Tile {
    /// Creates a new tile from an ordinal value.
    ///
    /// # Examples
    /// ```rust
    /// # use rules::Tile;
    ///
    /// let tile = Tile::new(5);
    /// assert_eq!(tile.as_tuple(), (2, 2)); // 5th tile is (2,2)
    /// ```
    pub const fn new(ordinal: u8) -> Self {
        Self { ordinal }
    }

    /// Returns `true` if this tile is a double.
    ///
    /// # Examples
    /// ```rust
    /// # use rules::Tile;
    ///
    /// assert!(Tile::from((0, 0)).is_double());
    /// assert!(Tile::from((6, 6)).is_double());
    /// assert!(!Tile::from((1, 2)).is_double());
    /// ```
    pub const fn is_double(self) -> bool {
        is_double_ordinal(self.ordinal)
    }

    /// Returns the tile as a tuple `(a, b)`
    ///
    /// # Examples
    /// ```rust
    /// # use rules::Tile;
    ///
    /// let tile = Tile::from((1, 3));
    /// assert_eq!(tile.as_tuple(), (1, 3));
    /// ```
    pub const fn as_tuple(&self) -> (u8, u8) {
        ordinal_to_tuple(self.ordinal)
    }

    /// Returns which value on this tile matches the given tile and which does not, or None if neither matches.
    ///
    /// # Arguments
    /// * `tile` - The tile to check
    ///
    /// # Returns
    /// (matching value, other value), or None if the tile doesn't match.
    ///
    /// # Notes
    /// - It is not possible in a standard domino set for the result to be ambiguous.
    ///
    /// # Examples
    /// ```rust
    /// # use rules::Tile;
    /// let tile = Tile::from((2, 5));
    /// assert_eq!(tile.matches(&Tile::from((5, 6))), Some((5, 2)));
    /// assert_eq!(tile.matches(&Tile::from((1, 2))), Some((2, 5)));
    /// assert_eq!(tile.matches(&Tile::from((0, 1))), None);
    /// ```
    pub const fn matches(&self, other: &Self) -> Option<(u8, u8)> {
        let a = self.as_tuple();
        let b: (u8, u8) = other.as_tuple();
        matches_tuples(a, b)
    }

    /// Returns the score of the tile by adding up the pips on both sides.
    ///
    /// # Returns
    /// The score of the tile as u8
    /// # Examples
    /// ```rust
    /// # use rules::Tile;
    /// let tile = Tile::from((3, 5));
    /// assert_eq!(tile.score(), 8);
    /// let double = Tile::from((6, 6));
    /// assert_eq!(double.score(), 12);
    /// ```
    pub const fn score(self) -> u8 {
        let (a, b) = self.as_tuple();
        a + b
    }
}

/// Enables Tile::from((u8, u8)) -> Tile and (u8, u8)::into() -> Tile
///
/// # Panics
/// * If the input tuple is not in canonical form (i.e., if the first element is greater than the second).
///
/// # Examples
/// ```rust
/// # use rules::Tile;
///
/// let tile1 = Tile::from((1, 3));
/// let tile2: Tile = (1, 3).into();
/// assert_eq!(tile1, tile2);
/// ```
impl From<(u8, u8)> for Tile {
    fn from(values: (u8, u8)) -> Self {
        Self::new(tuple_to_ordinal(values))
    }
}

/// Enables Tile::from(u8) -> Tile and u8::into() -> Tile
///
/// # Examples
/// ```rust
/// # use rules::Tile;
///
/// let tile1 = Tile::from(5u8);
/// let tile2: Tile = 5u8.into();
/// assert_eq!(tile1, tile2);
/// ```
impl From<u8> for Tile {
    fn from(ordinal: u8) -> Self {
        Self::new(ordinal)
    }
}

/// Enables (u8, u8)::from(Tile) -> (u8, u8) and Tile::into() -> (u8, u8)
///
/// # Examples
///
/// ```rust
/// # use rules::Tile;
///
/// let tile = Tile::new(5);
/// let tuple1: (u8, u8) = tile.into();
/// let tuple2 = <(u8, u8)>::from(tile);
/// assert_eq!(tuple1, tuple2);
/// ```
impl From<Tile> for (u8, u8) {
    fn from(tile: Tile) -> Self {
        tile.as_tuple()
    }
}

impl From<&Tile> for (u8, u8) {
    fn from(tile: &Tile) -> Self {
        tile.as_tuple()
    }
}

/// Enables u8::from(Tile) -> u8 and Tile::into() -> u8
///
/// # Examples
///
/// ```rust
/// # use rules::Tile;
///
/// let tile = Tile::new(5);
/// let ordinal1: u8 = tile.into();
/// let ordinal2 = u8::from(tile);
/// assert_eq!(ordinal1, 5);
/// assert_eq!(ordinal2, 5);
/// ```
impl From<Tile> for u8 {
    fn from(tile: Tile) -> Self {
        tile.ordinal
    }
}

impl From<&Tile> for u8 {
    fn from(tile: &Tile) -> Self {
        tile.ordinal
    }
}

/// Enables pretty-printing tiles in the format "a|b".
///
/// # Examples
/// ```rust
/// # use rules::Tile;
///
/// let tile = Tile::from((3, 5));
/// assert_eq!(format!("{}", tile), "3|5");
///
/// let double = Tile::from((6, 6));
/// assert_eq!(format!("{}", double), "6|6");
/// ```
impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (a, b) = self.as_tuple();
        write!(f, "{a}|{b}")
    }
}

impl Serialize for Tile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (a, b) = self.as_tuple();
        (a, b).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Tile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TileVisitor;

        impl<'de> Visitor<'de> for TileVisitor {
            type Value = Tile;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array of two u8 values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Tile, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let a: u8 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let b: u8 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                if seq.next_element::<u8>()?.is_some() {
                    return Err(de::Error::invalid_length(3, &self));
                }
                // Create the tile, supporting non-canonical form
                if a < b { Ok(Tile::from((a, b))) } else { Ok(Tile::from((b, a))) }
            }
        }

        deserializer.deserialize_tuple(2, TileVisitor)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_comprehensive() {
        // Test tile creation
        let tile = Tile::new(5);
        assert_eq!(tile.ordinal, 5);
        assert_eq!(tile.as_tuple(), (2, 2));

        // Test tile is_double
        assert!(Tile::from((0, 0)).is_double());
        assert!(Tile::from((3, 3)).is_double());
        assert!(Tile::from((6, 6)).is_double());
        assert!(!Tile::from((1, 2)).is_double());
        assert!(!Tile::from((0, 6)).is_double());

        // Test tile as_tuple
        assert_eq!(Tile::new(0).as_tuple(), (0, 0));
        assert_eq!(Tile::new(1).as_tuple(), (0, 1));
        assert_eq!(Tile::new(27).as_tuple(), (6, 6));
        assert_eq!(Tile::new(21).as_tuple(), (0, 6));

        // Test from tuple
        let tile1 = Tile::from((1, 3));
        let tile2: Tile = (1, 3).into();
        assert_eq!(tile1, tile2);
        assert_eq!(tile1.as_tuple(), (1, 3));

        // Test from u8
        let tile3 = Tile::from(5u8);
        let tile4: Tile = 5u8.into();
        assert_eq!(tile3, tile4);
        assert_eq!(tile3.ordinal, 5);

        // Test into tuple
        let tile5 = Tile::new(5);
        let tuple1: (u8, u8) = tile5.into();
        let tuple2 = <(u8, u8)>::from(tile5);
        assert_eq!(tuple1, tuple2);
        assert_eq!(tuple1, (2, 2));

        // Test into u8
        let tile6 = Tile::new(5);
        let ordinal1: u8 = tile6.into();
        let ordinal2 = u8::from(tile6);
        assert_eq!(ordinal1, 5);
        assert_eq!(ordinal2, 5);

        // Test display
        assert_eq!(format!("{}", Tile::from((3, 5))), "3|5");
        assert_eq!(format!("{}", Tile::from((6, 6))), "6|6");
        assert_eq!(format!("{}", Tile::from((0, 0))), "0|0");
        assert_eq!(format!("{}", Tile::from((0, 1))), "0|1");

        // Test debug
        let tile7 = Tile::from((2, 4));
        let debug_str = format!("{:?}", tile7);
        assert!(debug_str.contains("Tile"));
        assert!(debug_str.contains("ordinal"));

        // Test equality and ordering
        let tile8 = Tile::from((1, 2));
        let tile9 = Tile::from((1, 2));
        let tile10 = Tile::from((2, 3));

        assert_eq!(tile8, tile9);
        assert_ne!(tile8, tile10);
        assert!(tile8 < tile10);
    }

    #[test]
    #[should_panic(expected = "Tile must be in canonical form")]
    fn test_tile_from_invalid_tuple() {
        let _ = Tile::from((3, 1)); // Should panic - not in canonical form
    }

    #[test]
    fn test_tile_hash() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let tile1 = Tile::from((1, 2));
        let tile2 = Tile::from((3, 4));

        map.insert(tile1, "first");
        map.insert(tile2, "second");

        assert_eq!(map.get(&tile1), Some(&"first"));
        assert_eq!(map.get(&tile2), Some(&"second"));
    }

    #[test]
    fn test_tile_from_ref() {
        let tile = Tile::from((2, 5));

        // Test From<&Tile> for (u8, u8)
        let tuple1: (u8, u8) = (&tile).into();
        let tuple2 = <(u8, u8)>::from(&tile);
        assert_eq!(tuple1, (2, 5));
        assert_eq!(tuple2, (2, 5));
        assert_eq!(tuple1, tuple2);

        // Test From<&Tile> for u8
        let ordinal1: u8 = (&tile).into();
        let ordinal2 = u8::from(&tile);
        assert_eq!(ordinal1, tile.ordinal);
        assert_eq!(ordinal2, tile.ordinal);
        assert_eq!(ordinal1, ordinal2);
    }

    #[test]
    fn test_tile_serialize_deserialize() {
        let tile = Tile::from((3, 5));

        // Test serialization
        let json = serde_json::to_string(&tile).expect("Serialization failed");
        assert_eq!(json, "[3,5]");

        // Test deserialization
        let deserialized: Tile = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized, tile);

        // Test double tile
        let double_tile = Tile::from((4, 4));
        let double_json = serde_json::to_string(&double_tile).expect("Serialization failed");
        assert_eq!(double_json, "[4,4]");

        let deserialized_double: Tile = serde_json::from_str(&double_json).expect("Deserialization failed");
        assert_eq!(deserialized_double, double_tile);
    }

    #[test]
    fn test_tile_deserialize_errors() {
        // Test invalid JSON formats
        assert!(serde_json::from_str::<Tile>("[]").is_err()); // Empty array
        assert!(serde_json::from_str::<Tile>("[1]").is_err()); // Single element
        assert!(serde_json::from_str::<Tile>("[1,2,3]").is_err()); // Too many elements
        assert!(serde_json::from_str::<Tile>("1").is_err()); // Not an array
        assert!(serde_json::from_str::<Tile>("{\"a\":1,\"b\":2}").is_err()); // Object instead of array
    }

    #[test]
    fn test_tile_deserialize_non_canonical() {
        // Test that non-canonical form [b,a] where b > a is correctly converted to canonical form

        // Non-canonical [5,3] should deserialize to canonical (3,5)
        let tile1: Tile = serde_json::from_str("[5,3]").expect("Deserialization failed");
        assert_eq!(tile1.as_tuple(), (3, 5));
    }

    #[test]
    fn test_tile_score() {
        assert_eq!(Tile::from((0, 0)).score(), 0);
        assert_eq!(Tile::from((1, 2)).score(), 3);
        assert_eq!(Tile::from((3, 5)).score(), 8);
        assert_eq!(Tile::from((6, 6)).score(), 12);
        assert_eq!(Tile::from((0, 6)).score(), 6);
    }
}