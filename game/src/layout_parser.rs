//! Parser for domino layout strings.
//!
//! This module provides functionality to parse a layout specified in a string format into a tree structure.
//!
//! See [`parse`] for detailed documentation on the layout string syntax and rules.

use regex::Regex;
use ego_tree::{NodeMut, NodeRef, Tree};
use rules::{self, Tile};

/// Error type returned when parsing a domino layout string fails.
///
/// This error provides detailed information about what went wrong during parsing,
/// including a human-readable message and the character position where the error occurred.
///
/// # Examples
///
/// ```
/// use game::layout_parser::parse;
///
/// // This will create a ParseError due to invalid tile format
/// let result = parse("invalid");
/// assert!(result.is_err());
///
/// let error = result.unwrap_err();
/// println!("Error at position {}: {}", error.position, error.message);
/// ```
#[derive(Debug)]
pub struct ParseError {
    /// A human-readable description of what went wrong during parsing.
    pub message: String,
    /// The zero-based character position in the input string where the error occurred.
    pub position: usize,
}

impl std::fmt::Display for ParseError {
    /// Formats the error with position and message information.
    ///
    /// This provides a user-friendly error message that includes both the
    /// character position where the error occurred and a descriptive message.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

struct ParseState<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
    tile_regex: Regex,
}

impl<'a> ParseState<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
            tile_regex: Regex::new(r"(\d+)\s*\|\s*(\d+)").unwrap(),
        }
    }

    fn parse_chain(&mut self, parent_end: Option<u8>) -> Result<Tree<Tile>, ParseError> {
        self.skip_whitespace();

        // Parse the first tile in the chain
        let (tile, first_end) = self.parse_tile(parent_end)?;
        let mut chain = Tree::new(tile);

        self.skip_whitespace();

        if tile.is_double() {
            // Double cannot be followed by '-'
            if self.next_is('-') {
                return Err(self.error(&format!("{} followed by '-'. Doubles must be followed by =", tile)));
            }

            // Check for '=' indicating a group follows
            if self.consume('=') {
                let group = self.parse_group(Some(first_end))?;
                for g in group {
                    append_tree(&mut chain.root_mut(), g);
                }
            }

            Ok(chain)
        } else {
            // Normal tiles cannot be followed by '='
            if self.next_is('=') {
                return Err(self.error(&format!("{} followed by '='. Only doubles can be followed by =", tile)));
            }

            // Check for chain continuation with '-'
            if self.consume('-') {
                let child_chain = self.parse_chain(Some(first_end))?;
                append_tree(&mut chain.root_mut(), child_chain);
            }

            Ok(chain)
        }
    }

    fn parse_group(&mut self, parent_end: Option<u8>) -> Result<Vec<Tree<Tile>>, ParseError> {
        self.skip_whitespace();

        if !self.consume('(') {
            return Err(self.error("Expected '(' to start group"));
        }

        let mut chains = Vec::new();

        loop {
            self.skip_whitespace();

            let chain = self.parse_chain(parent_end)?;
            chains.push(chain);

            self.skip_whitespace();

            // If ',' is found, continue to next chain; if ')' found, end group; else error
            if self.consume(',') {
                continue;
            } else if self.consume(')') {
                break;
            } else {
                return Err(self.error("Expected ',' or ')' in group"));
            }
        }

        Ok(chains)
    }

    // Parses x|y into a Tile and also returns the open end y
    fn parse_tile(&mut self, parent_end: Option<u8>) -> Result<(Tile, u8), ParseError> {
        self.skip_whitespace();
        let remaining = self.remaining_input();

        if let Some(captures) = self.tile_regex.captures(remaining) {
            let full_match = captures.get(0).unwrap();
            let from_str = &captures[1];
            let to_str = &captures[2];

            let from = self.parse_number(from_str)?;
            let to = self.parse_number(to_str)?;

            self.validate_connection(parent_end, from, to)?;

            // Create the tile in canonical form
            let tile = Tile::from(if from <= to { (from, to) } else { (to, from) });

            self.advance_by(full_match.len());
            Ok((tile, to))
        } else {
            Err(self.error(&format!("Expected tile in format 'x|y' where x,y are 0-{}", rules::MAX_PIPS)))
        }
    }

    // Parses a string number
    fn parse_number(&self, str: &str) -> Result<u8, ParseError> {
        str.parse::<u8>()
            .ok()
            .filter(|&value| value <= rules::MAX_PIPS)
            .ok_or_else(|| self.error(&format!("Number '{}' is out of range (0-{})", str, rules::MAX_PIPS)))
    }

    // Validate connection if we have a parent
    fn validate_connection(&self, parent_end: Option<u8>, from: u8, to: u8) -> Result<(), ParseError> {
        if let Some(expected) = parent_end {
            if from != expected {
                return Err(self.error(&format!(
                        "Invalid connection: tile {}|{} first number ({}) must match the preceding end ({})",
                        from, to, from, expected
                    )));
            }
        }
        Ok(())
    }

    fn next_is(&self, c: char) -> bool {
        self.pos < self.chars.len() && self.chars[self.pos] == c
    }

    fn consume(&mut self, c: char) -> bool {
        if self.next_is(c) {
            self.advance_by(1);
            true
        } else {
            false
        }
    }

    fn remaining_input(&self) -> &str {
        &self.input[self.pos..]
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn advance_by(&mut self, count: usize) {
        self.pos = (self.pos + count).min(self.chars.len());
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            position: self.pos,
        }
    }
}

/// Parse a layout string into an ego-tree Tree structure.
///
/// This function parses a domino layout string according to the grammar defined in this module and returns a tree structure
/// representing the layout. Each node in the tree contains a `Tile`.
///
/// # Layout String Syntax (BNF)
/// The layout string syntax follows this grammar:
///
/// ```bnf
/// - <layout> ::= <chain>
/// - <chain> ::= <tile> | <double> | <tile> "-" <chain> | <double> "=" "(" <group> ")"
/// - <tile> ::= <number> "|" <number>
/// - <double> ::= <number> "|" <number>
/// - <group> ::= <chain> | <chain> "," <chain> | <chain> "," <chain> "," <chain>
/// - <number> ::= "0" | "1" | ... | "MAX_PIPS"
/// ```
///
/// where,
/// - The two numbers in a `<tile>` are different
/// - The two numbers in a `<double>` are the same
/// - A `<group>` contains 1 to 3 chains
/// - When tiles are connected with "-", the left number of the succeeding tile must match the right number of the preceding
///   tile (the open end).
/// - In the group following a double, the left number of each first tile of each chain must match the double's value.
/// - Whitespace is ignored.
///
/// ## Valid Examples
/// - Single tile: `5|6`
/// - Simple chain: `1|2-2|3`
/// - Double tile with branches: `3|3=(3|4-4|5,3|6)`
/// - Double tile with single branch: `4|4=(4|5-5|6)`
/// - Layout with multiple doubles: `1|5-0|5-3|5-5|5=(4|5-2|5-5|6-3|6-6|6=(0|6-1|6-4|6-2|6))`
///
/// # Arguments
/// * `input` - A string slice containing the layout to parse
///
/// # Returns
/// Returns `Ok(Tree<Tile>)` if parsing succeeds, or `Err(ParseError)` if the input is invalid.
///
/// # Errors
/// This function returns a `ParseError` in the following cases:
/// - Invalid tile format (not in x|y format)
/// - Numbers outside the range 0-MAX_PIPS
/// - Mismatched connections (tile ends don't connect properly)
/// - Invalid syntax (missing parentheses, commas, etc.)
/// - Non-double tiles followed by "="
/// - Double tiles followed by "-"
/// - Unexpected characters after the layout
///
/// # Examples
/// ```rust
/// # use game::layout_parser::parse;
///
/// // Simple chain: 1|2 connected to 2|3
/// let tree = parse("1|2-2|3").unwrap();
///
/// // Double tile with branches: 3|3 connected to two chains
/// let tree = parse("3|3=(3|4-4|5,3|6)").unwrap();
///
/// // Single tile
/// let tree = parse("5|6").unwrap();
/// ```
///




pub fn parse(input: &str) -> Result<Tree<Tile>, ParseError> {
    let mut state = ParseState::new(input);
    let layout = state.parse_chain(None)?;

    state.skip_whitespace();
    if state.pos < state.chars.len() {
        return Err(state.error("Unexpected characters after layout"));
    }

    Ok(layout)
}

// Helper function appends source tree as a child of the destination node. This is how tiles are prepended in a chain.
fn append_tree(dest: &mut NodeMut<Tile>, source: Tree<Tile>) {
    fn append_node_r(dest: &mut NodeMut<Tile>, source: NodeRef<Tile>) {
        let mut new_child = dest.append(*source.value());
        for child in source.children() {
            append_node_r(&mut new_child, child);
        }
    }
    let source_root = source.root();

    append_node_r(dest, source_root);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rules::Tile;

    /// Helper function to verify tree structure by walking it and collecting tiles in order
    fn collect_tiles_preorder(tree: &Tree<Tile>) -> Vec<Tile> {
        tree.root().descendants()
            .map(|node| *node.value())
            .collect()
    }

    /// Helper function to count the number of children at the root
    fn count_root_children(tree: &Tree<Tile>) -> usize {
        tree.root().children().count()
    }

    #[test]
    fn test_parse_single_tile() {
        let result = parse("1|2");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0], Tile::from((1, 2)));
    }

    #[test]
    fn test_parse_double_tile() {
        let result = parse("3|3");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0], Tile::from((3, 3)));
    }

    #[test]
    fn test_parse_simple_chain() {
        let result = parse("1|2-2|3");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], Tile::from((1, 2)));
        assert_eq!(tiles[1], Tile::from((2, 3)));
        assert_eq!(count_root_children(&tree), 1);
    }

    #[test]
    fn test_parse_longer_chain() {
        let result = parse("1|2-2|4-4|5-5|6");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0], Tile::from((1, 2)));
        assert_eq!(tiles[1], Tile::from((2, 4)));
        assert_eq!(tiles[2], Tile::from((4, 5)));
        assert_eq!(tiles[3], Tile::from((5, 6)));
    }

    #[test]
    fn test_parse_double_with_single_branch() {
        let result = parse("3|3=(3|4)");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], Tile::from((3, 3)));
        assert_eq!(tiles[1], Tile::from((3, 4)));
        assert_eq!(count_root_children(&tree), 1);
    }

    #[test]
    fn test_parse_double_with_multiple_branches() {
        let result = parse("2|2=(2|3,2|4,2|5)");
        if result.is_err() {
            println!("Error: {}", result.as_ref().unwrap_err());
        }
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0], Tile::from((2, 2)));
        assert_eq!(tiles[1], Tile::from((2, 3)));
        assert_eq!(tiles[2], Tile::from((2, 4)));
        assert_eq!(tiles[3], Tile::from((2, 5)));
        assert_eq!(count_root_children(&tree), 3);
    }

    #[test]
    fn test_parse_non_canonical_tile() {
        // Test that non-canonical tiles in layout string are automatically corrected
        let result = parse("5|1");
        assert!(result.is_ok(), "Parser should accept non-canonical tiles");

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 1);
        // The tile should be stored in canonical form (1, 5)
        assert_eq!(tiles[0], Tile::from((1, 5)));
    }

    #[test]
    fn test_parse_non_canonical_tiles_in_chain() {
        // Test valid connections where tiles can be non-canonical
        // "5|1-1|6-6|2": 5|1 has open end 1, 1|6 connects with 1 and has open end 6, 6|2 connects with 6
        let result = parse("5|1-1|6-6|2");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 3);
        // All tiles should be stored in canonical form
        assert_eq!(tiles[0], Tile::from((1, 5)));
        assert_eq!(tiles[1], Tile::from((1, 6)));
        assert_eq!(tiles[2], Tile::from((2, 6)));
    }

    #[test]
    fn test_parse_error_invalid_non_canonical_connection() {
        // Test that invalid non-canonical connections are rejected
        // "5|1-3|6": 5|1 has open end 1, but 3|6 starts with 3, not 1
        let result = parse("5|1-3|6");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid connection"));
        assert!(error.message.contains("first number (3) must match the preceding end (1)"));
    }

    #[test]
    fn test_parse_non_canonical_tiles_in_branches() {
        // Test that non-canonical tiles in double branches are rejected
        // "2|2=(4|2,5|2,3|2)" is invalid because 4, 5, 3 don't match the expected 2
        let result = parse("2|2=(4|2,5|2,3|2)");
        assert!(result.is_err(), "Should fail because 4, 5, 3 don't match expected connection 2");

        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid connection"));
        assert!(error.message.contains("must match the preceding end"));

        // Test with correct connections instead
        let result = parse("2|2=(2|4,2|5,2|3)");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0], Tile::from((2, 2)));
        assert_eq!(tiles[1], Tile::from((2, 4)));
        assert_eq!(tiles[2], Tile::from((2, 5)));
        assert_eq!(tiles[3], Tile::from((2, 3)));
        assert_eq!(count_root_children(&tree), 3);
    }

    #[test]
    fn test_parse_mixed_canonical_and_non_canonical() {
        // Test that invalid connections are rejected
        // "1|2-2|2=(6|2,2|3,5|2)" has invalid connections: 6 != 2 and 5 != 2
        let result = parse("1|2-2|2=(6|2,2|3,5|2)");
        assert!(result.is_err(), "Should fail because 6 != 2 and 5 != 2");

        // Test with valid connections
        let result = parse("1|2-2|2=(2|6,2|3,2|5)");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 5);
        assert_eq!(tiles[0], Tile::from((1, 2))); // Already canonical
        assert_eq!(tiles[1], Tile::from((2, 2))); // Already canonical
        assert_eq!(tiles[2], Tile::from((2, 6))); // Canonical form
        assert_eq!(tiles[3], Tile::from((2, 3))); // Already canonical
        assert_eq!(tiles[4], Tile::from((2, 5))); // Canonical form
    }

    #[test]
    fn test_parse_complex_layout() {
        let result = parse("1|2-2|2=(2|3-3|4,2|5)");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 5);
        assert_eq!(tiles[0], Tile::from((1, 2)));
        assert_eq!(tiles[1], Tile::from((2, 2)));
        assert_eq!(tiles[2], Tile::from((2, 3)));
        assert_eq!(tiles[3], Tile::from((3, 4)));
        assert_eq!(tiles[4], Tile::from((2, 5)));
    }

    #[test]
    fn test_parse_with_whitespace() {
        let result = parse("  1 | 2  -  2 | 3  ");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], Tile::from((1, 2)));
        assert_eq!(tiles[1], Tile::from((2, 3)));
    }

    #[test]
    fn test_parse_complex_layout_with_whitespace() {
        let result = parse(" 1 | 2 - 2 | 2 = ( 2 | 3 - 3 | 4 , 2 | 5 ) ");
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 5);
        assert_eq!(tiles[0], Tile::from((1, 2)));
        assert_eq!(tiles[1], Tile::from((2, 2)));
        assert_eq!(tiles[2], Tile::from((2, 3)));
        assert_eq!(tiles[3], Tile::from((3, 4)));
        assert_eq!(tiles[4], Tile::from((2, 5)));
    }

    #[test]
    fn test_parse_error_empty_group() {
        let result = parse("3|3=()");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_max_values() {
        // Test with values that are within the ordinal system limits
        // Use MAX_PIPS as the maximum valid domino value
        let result = parse(&format!("0|{}-{}|{}", rules::MAX_PIPS, rules::MAX_PIPS, rules::MAX_PIPS));
        assert!(result.is_ok());

        let tree = result.unwrap();
        let tiles = collect_tiles_preorder(&tree);
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], Tile::from((0, rules::MAX_PIPS)));
        assert_eq!(tiles[1], Tile::from((rules::MAX_PIPS, rules::MAX_PIPS)));
    }

    #[test]
    fn test_parse_error_ordinal_exceeds_limit() {
        // Test that numbers exceeding MAX_PIPS are rejected
        let invalid_value = rules::MAX_PIPS + 1;
        let result = parse(&format!("0|{}", invalid_value));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("ordinal must not exceed limits") ||
                error.message.contains("Ordinal exceeds") ||
                error.message.contains(&format!("out of range (0-{})", rules::MAX_PIPS)));
    }

    // Error tests
    #[test]
    fn test_parse_error_invalid_format() {
        let result = parse("invalid");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.position, 0);
        assert!(error.message.contains("Expected tile in format 'x|y'"));
    }

    #[test]
    fn test_parse_error_missing_pipe() {
        let result = parse("12");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.position, 0);
        assert!(error.message.contains("Expected tile in format 'x|y'"));
    }

    #[test]
    fn test_parse_error_negative_number() {
        let result = parse("1|-2");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_connection_mismatch() {
        let result = parse("1|2-3|4");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("Invalid connection"));
        assert!(error.message.contains("first number (3) must match the preceding end (2)"));
    }

    #[test]
    fn test_parse_error_double_followed_by_dash() {
        let result = parse("3|3-3|4");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("3|3 followed by '-'"));
        assert!(error.message.contains("Doubles must be followed by ="));
    }

    #[test]
    fn test_parse_error_non_double_followed_by_equals() {
        let result = parse("1|2=(2|3)");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("1|2 followed by '='"));
        assert!(error.message.contains("Only doubles can be followed by ="));
    }

    #[test]
    fn test_parse_error_missing_opening_paren() {
        let result = parse("3|3=3|4)");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("Expected '(' to start group"));
    }

    #[test]
    fn test_parse_error_missing_closing_paren() {
        let result = parse("3|3=(3|4");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("Expected ',' or ')' in group"));
    }

    #[test]
    fn test_parse_error_unexpected_characters() {
        let result = parse("1|2 extra");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("Unexpected characters after layout"));
    }

    #[test]
    fn test_parse_error_missing_comma_in_group() {
        let result = parse("3|3=(3|4 3|5)");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.message.contains("Expected ',' or ')' in group"));
    }

    // Tests for ParseError Display implementation
    #[test]
    fn test_parse_error_display() {
        let error = ParseError {
            message: "Test error".to_string(),
            position: 5,
        };

        let display_str = format!("{}", error);
        assert_eq!(display_str, "Parse error at position 5: Test error");
    }

    #[test]
    fn test_parse_error_debug() {
        let error = ParseError {
            message: "Test error".to_string(),
            position: 5,
        };

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ParseError"));
        assert!(debug_str.contains("Test error"));
        assert!(debug_str.contains("5"));
    }

    #[test]
    fn test_parse_error_is_error_trait() {
        let error = ParseError {
            message: "Test error".to_string(),
            position: 5,
        };

        // This should compile because ParseError implements std::error::Error
        let _: &dyn std::error::Error = &error;
    }
}
