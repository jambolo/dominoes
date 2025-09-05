//! Dominoes layout implementation
//! 
//! This module provides the Layout struct for managing the layout of domino tiles.

use std::fmt::{self, Display, Formatter};
use multimap::MultiMap;

use rules::Configuration;

type Tile = rules::Tile;

/// A node in the domino layout graph representing a single placed tile.
/// 
/// # Examples
/// ```rust
/// # use dominoes_state::LayoutNode;
/// # use rules::Tile;
/// 
/// let node = LayoutNode {
///     tile: Tile::from((3, 6)),
///     parent: Some(0),  // Connected to node at index 0
///     children: vec![2, 3],  // Has children at indices 2 and 3
/// };
/// ```
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// The tile
    pub tile: Tile,
    /// Index of the node's parent, `None` indicates this is the root node
    pub parent: Option<usize>,
    /// Indexes of the child nodes attached to this tile
    pub children: Vec<usize>,
}

/// Represents the layout of dominoes.
/// 
/// Manages the placement and connectivity of domino tiles, tracking which ends are available for new tile attachments.
/// 
/// # Examples
/// ```rust
/// # use dominoes_state::Layout;
/// # use rules::{Configuration, Tile};
/// 
/// let config = Configuration::default();
/// let mut layout = Layout::new(&config);
/// 
/// // Place the first tile (must be a double)
/// let double_six = Tile::from((6, 6));
/// layout.attach(double_six, None);
/// 
/// // Attach a second tile to the first
/// let three_six = Tile::from((3, 6));
/// layout.attach(three_six, Some(0));
/// 
/// // Get string representation
/// println!("{}", layout.to_string()); // "6|6-3|6"
/// ```
/// 
/// # Important Notes
/// - The layout contains *copies* of tiles, not references
/// - The first tile must be a double tile in order for serialization to work.
#[derive(Debug, Clone)]
pub struct Layout {
    /// Vector of all tiles in the layout with their connectivity information
    pub nodes: Vec<LayoutNode>,
    /// Map tracking open ends: node index -> open value
    /// 
    /// Each entry maps a node index to the values available for attachment at that node. Double tiles may have multiple entries
    /// with the same value.
    pub open: MultiMap<usize, u8>,
    /// Tracks the number of open ends for each value
    /// 
    /// Array where index corresponds to the domino value (0-6 for standard set) and the value at that index is the count of all
    /// open ends in the layout with that value.
    pub end_counts: Vec<u8>,
}

impl Layout {
    /// Creates a new empty layout.
    /// 
    /// # Returns
    /// A new `Layout` instance with no tiles placed and no open ends.
    /// 
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Layout;
    /// # use rules::{Configuration, Variation};
    /// 
    /// let config = Configuration::new(4, Variation::Traditional, 6, 6);
    /// let layout = Layout::new(&config);
    /// assert!(layout.nodes.is_empty());
    /// ```
    pub fn new(configuration: &Configuration) -> Self {
        Self {
            nodes: Vec::new(),
            open: MultiMap::new(),
            end_counts: vec![0; configuration.set_id as usize + 1], // +1 because values are 0..set_id inclusive
        }
    }

    /// Returns `true` if the layout is empty.
    /// 
    /// An empty layout has no tiles placed and no open ends available.
    /// 
    /// # Returns
    /// `true` if no tiles have been placed, `false` otherwise
    /// 
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Layout;
    /// # use rules::Configuration;
    /// 
    /// let config = Configuration::default();
    /// let layout = Layout::new(&config);
    /// assert!(layout.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of open ends in the layout for the specified end value.
    /// 
    /// This count represents how many attachment points are available for tiles that have the specified value on one of their
    /// ends.
    ///
    /// # Arguments
    /// * `end` - The domino value to check (e.g., 0-6 for standard double-six set)
    /// 
    /// # Returns
    /// The number of open attachment points for the specified value
    /// 
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Layout;
    /// # use rules::{Configuration, Tile};
    /// 
    /// let config = Configuration::default();
    /// let mut layout = Layout::new(&config);
    /// 
    /// // Initially no open ends
    /// assert_eq!(layout.open_count(6), 0);
    /// 
    /// // Place double-6 tile
    /// layout.attach(Tile::from((6, 6)), None);
    /// 
    /// // Now there are 2 open ends with value 6
    /// assert_eq!(layout.open_count(6), 2);
    /// ```
    pub fn open_count(&self, end: u8) -> u8 {
        self.end_counts[end as usize]
    }

    /// Attaches a domino tile to the layout.
    /// 
    /// Places a new tile either as the first tile (root) or attached to an existing tile at one of its open ends.
    /// 
    /// # Parameters
    /// - `tile`: The domino tile to place in the layout
    /// - `parent_index`: The index of the existing node to attach to, or `None` for the first tile
    /// 
    /// # Returns
    /// A tuple containing the new open end value and how many open ends were created.
    ///
    /// # Panics
    /// - If `parent_index` refers to a non-existent node
    /// - If the parent node has no open ends
    /// - If the tiles cannot be attached (no matching values)
    /// - If trying to place a first tile when layout is not empty
    /// - If trying to attach to a parent when layout is empty
    /// 
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Layout;
    /// # use rules::{Configuration, Variation, Tile};
    /// 
    /// let config = Configuration::new(4, Variation::Traditional, 6, 6);
    /// let mut layout = Layout::new(&config);
    /// 
    /// // Place first tile (e.g., 6|6 double)
    /// let double_six = Tile::from((6, 6));
    /// layout.attach(double_six, None);
    /// 
    /// // Attach second tile to first tile's node (index 0)
    /// let three_six = Tile::from((3, 6));
    /// layout.attach(three_six, Some(0));
    /// 
    /// // Attach third tile to second tile's node (index 1)  
    /// let one_three = Tile::from((1, 3));
    /// layout.attach(one_three, Some(1));
    /// ```
    pub fn attach(&mut self, tile: Tile, parent_index: Option<usize>) -> (u8, u8) {
        let (a, b) = tile.as_tuple();

        // Attach the tile and update the open list.
        let (end_value, created_count) = if let Some(parent_index) = parent_index {
            assert!(parent_index < self.nodes.len() && self.open.contains_key(&parent_index));

            let parent = &self.nodes[parent_index].tile;
            let (p_a, p_b) = parent.as_tuple();

            // Determine the matched and open values
            assert!(parent.can_attach(tile));
            let (matched_value, open_value) = if a == p_a || a == p_b { (a, b) } else { (b, a) };

            // Add a new tile node to the layout
            let tile_index = self.nodes.len();
            self.nodes.push(LayoutNode {
                tile,
                parent: Some(parent_index),
                children: Vec::new()
            });

            // Add the open ends. If the tile is a double, add twice.
            self.open.insert(tile_index, open_value);
            self.end_counts[open_value as usize] += 1;
            if tile.is_double() {
                self.open.insert(tile_index, open_value);
                self.end_counts[open_value as usize] += 1;
            }

            // Remove the parent's open end from the open list
            self.remove_from_open(parent_index, matched_value);
            assert!(self.end_counts[matched_value as usize] > 0);
            self.end_counts[matched_value as usize] -= 1;

            // Add the new tile node's index to the parent's list of children
            self.nodes[parent_index].children.push(tile_index);
            (open_value, if tile.is_double() { 2 } else { 1 })
        }
        // The first tile is a special case, though.
        else {
            assert!(tile.is_double());
            assert!(self.nodes.is_empty() && self.open.is_empty());

            // Add the new tile node to the layout
            self.nodes.push(LayoutNode {
                tile,
                parent: None,
                children: Vec::new()
            });

            // Both ends are open for the first tile.
            self.open.insert(0, a);
            self.open.insert(0, a);
            self.end_counts[a as usize] += 2;
            (a, 2)
        };

        //self.dump_open("open after attach");

        (end_value, created_count)
    }

    // Removes a tile from the open list. Note that a double tile will have two entries with the same key, and only one of the
    // entries is removed.
    fn remove_from_open(&mut self, parent: usize, value: u8) {
        let values = self.open.get_vec_mut(&parent)
            .unwrap_or_else(|| panic!("Parent {parent} not found in open list"));
        let pos = values.iter()
            .position(|&x| x == value)
            .unwrap_or_else(|| panic!("Value {value} not found for parent {parent}"));
        values.remove(pos);

        //self.dump_open("open after removal");
    }

    // Recursive helper function for generating the layout string
    fn to_string_r(&self, node: &LayoutNode, open: u8) -> String {
        let (a, b) = node.tile.as_tuple();
        let (a, b) = if open == a { (a, b) } else { (b, a) }; // Swap if necessary (tiles are added left-to-right)

        let mut result = format!("{a}|{b}");

        // Add the children recursively
        match node.children.len() {
            0 => {}, // Open end
            1 => {
                let child_node = &self.nodes[node.children[0]];
                result.push('-');
                result.push_str(&self.to_string_r(child_node, b));
            },
            _ => {
                result.push_str("=(");
                for (i, &child) in node.children.iter().enumerate() {
                    let child_node = &self.nodes[child];
                    if i > 0 { result.push(','); }
                    result.push_str(&self.to_string_r(child_node, b));
                }
                result.push(')');
            }
        }
        
        result
    }
}

/// Returns the layout as a human-readable string representation.
///
/// Generates a textual representation showing how domino tiles are connected in the layout. The format uses specific notation
/// to represent the tree structure:
/// - Single connections: `tile1-tile2`
/// - Multiple branches: `tile=(branch1,branch2,branch3)`
/// - Tiles shown as `a|b`
///
/// # Returns
/// A `String` representing the complete layout structure, or an empty string if no tiles have been placed.
///
/// # Format Examples
/// - **Linear chain**: `6|6-6|3-3|1` (each tile connected to the next)
/// - **Branching**: `3|3=(3|2,3|5,3|4)` (multiple tiles attached to one tile)
/// - **Complex tree**: `6|6-6|3=(3|1,3|5-5|2)`
///
/// # Panics
/// Panics if the root tile (first placed) is not a double tile, as the current implementation requires this for proper string
/// generation.
///
/// # Examples
/// ```rust
/// # use dominoes_state::Layout;
/// # use rules::{Configuration, Variation, Tile};
/// 
/// let config = Configuration::new(4, Variation::Traditional, 6, 6);
/// let mut layout = Layout::new(&config);
/// 
/// // Simple chain: 6|6-6|3-3|1
/// let double_six = Tile::from((6, 6));
/// layout.attach(double_six, None);
/// let three_six = Tile::from((3, 6));
/// layout.attach(three_six, Some(0));
/// let one_three = Tile::from((1, 3));
/// layout.attach(one_three, Some(1));
/// // Layout string representation will show tiles as connected
/// 
/// // Simple branching example
/// let mut layout2 = Layout::new(&config);
/// let double_six = Tile::from((6, 6));
/// layout2.attach(double_six, None);
/// let three_six = Tile::from((3, 6));
/// layout2.attach(three_six, Some(0));
/// // Layout will show branching pattern
/// ```
impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.nodes.is_empty() {
            return write!(f, "");
        }

        let root = &self.nodes[0];
        let (a, b) = root.tile.as_tuple();
        assert_eq!(a, b, "First node must be a double");
        write!(f, "{}", self.to_string_r(root, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_tile(a: u8, b: u8) -> Tile {
        Tile::from((a, b))
    }

    #[test]
    fn test_layout_node_construction() {
        let tile = create_tile(3, 6);
        let node = LayoutNode {
            tile,
            parent: Some(0),
            children: vec![2, 3],
        };
        
        assert_eq!(node.tile, tile);
        assert_eq!(node.parent, Some(0));
        assert_eq!(node.children, vec![2, 3]);
    }
    
    #[test]
    fn test_new_layout() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let layout = Layout::new(&configuration);
        assert!(layout.nodes.is_empty());
        assert!(layout.open.is_empty());
        assert_eq!(layout.end_counts.len(), 7); // 0..6 inclusive = 7 elements
    }

    #[test]
    fn test_default_layout() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let layout = Layout::new(&configuration);
        assert!(layout.nodes.is_empty());
        assert!(layout.open.is_empty());
    }

    #[test]
    fn test_new_layout_matches_docs() {
        let config = Configuration::default();
        let layout = Layout::new(&config);
        assert!(layout.nodes.is_empty());
    }
        #[test]
    fn test_attach_first_tile() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        
        layout.attach(double_six, None);
        
        assert_eq!(layout.nodes.len(), 1);
        assert_eq!(layout.nodes[0].tile, double_six);
        assert_eq!(layout.nodes[0].parent, None);
        assert!(layout.nodes[0].children.is_empty());
        
        // Both ends should be open
        let open_values: Vec<u8> = layout.open.get_vec(&0).unwrap().clone();
        assert_eq!(open_values.len(), 2);
        assert!(open_values.iter().all(|&x| x == 6));
    }

    #[test]
    fn test_attach_second_tile() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        let three_six = create_tile(3, 6);
        
        layout.attach(double_six, None);
        layout.attach(three_six, Some(0));
        
        assert_eq!(layout.nodes.len(), 2);
        
        // Check first node
        assert_eq!(layout.nodes[0].children, vec![1]);
        
        // Check second node
        assert_eq!(layout.nodes[1].tile, three_six);
        assert_eq!(layout.nodes[1].parent, Some(0));
        assert!(layout.nodes[1].children.is_empty());
        
        // Check open ends - first node should have one 6 removed, second node should have 3 open
        let first_open: Vec<u8> = layout.open.get_vec(&0).unwrap_or(&vec![]).clone();
        let second_open: Vec<u8> = layout.open.get_vec(&1).unwrap().clone();
        
        assert_eq!(first_open.len(), 1);
        assert!(first_open.contains(&6));
        assert_eq!(second_open.len(), 1);
        assert!(second_open.contains(&3));
    }

    #[test]
    fn test_attach_double_tile() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        let double_three = create_tile(3, 3);
        let three_six = create_tile(3, 6);
        
        layout.attach(double_six, None);
        layout.attach(three_six, Some(0));
        layout.attach(double_three, Some(1));
        
        // Double tile should have two open ends with the same value
        let double_open: Vec<u8> = layout.open.get_vec(&2).unwrap().clone();
        assert_eq!(double_open.len(), 2);
        assert!(double_open.iter().all(|&x| x == 3));
    }

    #[test]
    fn test_branching_layout() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_three = create_tile(3, 3);
        let two_three = create_tile(2, 3);
        let three_five = create_tile(3, 5);
        
        layout.attach(double_three, None);
        layout.attach(two_three, Some(0));
        layout.attach(three_five, Some(0));
        
        assert_eq!(layout.nodes.len(), 3);
        assert_eq!(layout.nodes[0].children, vec![1, 2]);
        
        // Root should have no open ends left
        assert!(layout.open.get_vec(&0).is_none() || layout.open.get_vec(&0).unwrap().is_empty());
    }

    #[test]
    fn test_to_string_empty() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let layout = Layout::new(&configuration);
        assert_eq!(layout.to_string(), "");
    }

    #[test]
    fn test_to_string_single_tile() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        
        layout.attach(double_six, None);
        
        assert_eq!(layout.to_string(), "6|6");
    }

    #[test]
    fn test_to_string_linear_chain() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        let three_six = create_tile(3, 6);
        let one_three = create_tile(1, 3);
        
        layout.attach(double_six, None);
        layout.attach(three_six, Some(0));
        layout.attach(one_three, Some(1));
        
        assert_eq!(layout.to_string(), "6|6-6|3-3|1");
    }

    #[test]
    fn test_to_string_branching() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_three = create_tile(3, 3);
        let two_three = create_tile(2, 3);
        let three_five = create_tile(3, 5);
        
        layout.attach(double_three, None);
        layout.attach(two_three, Some(0));
        layout.attach(three_five, Some(0));
        
        assert_eq!(layout.to_string(), "3|3=(3|2,3|5)");
    }

    #[test]
    fn test_to_string_complex_tree() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        let three_six = create_tile(3, 6);
        let double_three = create_tile(3, 3);
        let one_three = create_tile(1, 3);
        let three_five = create_tile(3, 5);
        let two_five = create_tile(2, 5);
        
        layout.attach(double_six, None);
        layout.attach(three_six, Some(0));
        layout.attach(double_three, Some(1));  // Attach double-3 to the 3|6 tile
        layout.attach(one_three, Some(2));     // Attach 1|3 to the double-3
        layout.attach(three_five, Some(2));    // Attach 3|5 to the double-3 (creates branching)
        layout.attach(two_five, Some(4));      // Attach 2|5 to the 3|5 tile
        
        assert_eq!(layout.to_string(), "6|6-6|3-3|3=(3|1,3|5-5|2)");
    }

    #[test]
    #[should_panic(expected = "First node must be a double")]
    fn test_to_string_non_double_root_panics() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        // Manually create invalid layout with non-double root
        layout.nodes.push(LayoutNode {
            tile: create_tile(3, 6),
            parent: None,
            children: Vec::new(),
        });
        layout.to_string();
    }

    #[test]
    #[should_panic]
    fn test_attach_to_nonexistent_parent() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let tile = create_tile(3, 6);
        layout.attach(tile, Some(5)); // Parent index 5 doesn't exist
    }

    #[test]
    #[should_panic]
    fn test_attach_first_tile_to_nonempty_layout() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        let three_six = create_tile(3, 6);
        
        layout.attach(double_six, None);
        layout.attach(three_six, None); // Should panic - trying to add first tile to non-empty layout
    }

    #[test]
    #[should_panic]
    fn test_attach_to_parent_when_empty() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let tile = create_tile(3, 6);
        layout.attach(tile, Some(0)); // Should panic - layout is empty
    }

    #[test]
    fn test_layout_node_creation() {
        let tile = create_tile(3, 6);
        let node = LayoutNode {
            tile,
            parent: Some(0),
            children: vec![1, 2],
        };
        
        assert_eq!(node.tile, tile);
        assert_eq!(node.parent, Some(0));
        assert_eq!(node.children, vec![1, 2]);
    }

    #[test]
    fn test_remove_from_open() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        
        layout.attach(double_six, None);
        
        // Manually test remove_from_open
        assert_eq!(layout.open.get_vec(&0).unwrap().len(), 2);
        layout.remove_from_open(0, 6);
        assert_eq!(layout.open.get_vec(&0).unwrap().len(), 1);
    }

    #[test]
    #[should_panic(expected = "Parent 5 not found in open list")]
    fn test_remove_from_open_invalid_parent() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        layout.remove_from_open(5, 6);
    }

    #[test]
    #[should_panic(expected = "Value 9 not found for parent 0")]
    fn test_remove_from_open_invalid_value() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        layout.attach(double_six, None);
        layout.remove_from_open(0, 9); // Value 9 doesn't exist for this tile
    }

    #[test]
    fn test_is_empty() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        
        // Initially empty
        assert!(layout.is_empty());
        
        // Add a tile - no longer empty
        let double_six = create_tile(6, 6);
        layout.attach(double_six, None);
        assert!(!layout.is_empty());
        
        // Add another tile - still not empty
        let three_six = create_tile(3, 6);
        layout.attach(three_six, Some(0));
        assert!(!layout.is_empty());
    }

    #[test]
    fn test_open_count() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        
        // Initially no open ends for any value
        for i in 0..=6 {
            assert_eq!(layout.open_count(i), 0);
        }
        
        // Place double-six
        let double_six = create_tile(6, 6);
        layout.attach(double_six, None);
        
        // Should have 2 open ends with value 6
        assert_eq!(layout.open_count(6), 2);
        for i in 0..=5 {
            assert_eq!(layout.open_count(i), 0);
        }
        
        // Attach 3-6 tile to consume one 6 end and add one 3 end
        let three_six = create_tile(3, 6);
        layout.attach(three_six, Some(0));
        
        assert_eq!(layout.open_count(6), 1);
        assert_eq!(layout.open_count(3), 1);
        for i in &[0, 1, 2, 4, 5] {
            assert_eq!(layout.open_count(*i), 0);
        }
        
        // Attach double-3 to consume 3 end and add two 3 ends
        let double_three = create_tile(3, 3);
        layout.attach(double_three, Some(1));
        
        assert_eq!(layout.open_count(6), 1);
        assert_eq!(layout.open_count(3), 2);
        for i in &[0, 1, 2, 4, 5] {
            assert_eq!(layout.open_count(*i), 0);
        }
    }

    #[test]
    fn test_open_count_complex_layout() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        
        // Build: 4|4=(4|2,4|1-1|5)
        let double_four = create_tile(4, 4);
        layout.attach(double_four, None);
        assert_eq!(layout.open_count(4), 2);
        
        let two_four = create_tile(2, 4);
        layout.attach(two_four, Some(0));
        assert_eq!(layout.open_count(4), 1);
        assert_eq!(layout.open_count(2), 1);
        
        let one_four = create_tile(1, 4);
        layout.attach(one_four, Some(0));
        assert_eq!(layout.open_count(4), 0);
        assert_eq!(layout.open_count(2), 1);
        assert_eq!(layout.open_count(1), 1);
        
        let one_five = create_tile(1, 5);
        layout.attach(one_five, Some(2));
        assert_eq!(layout.open_count(4), 0);
        assert_eq!(layout.open_count(2), 1);
        assert_eq!(layout.open_count(1), 0);
        assert_eq!(layout.open_count(5), 1);
    }

    #[test]
    fn test_attach_return_values() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        
        // First tile (double) should return (value, 2)
        let double_six = create_tile(6, 6);
        let (open_value, count) = layout.attach(double_six, None);
        assert_eq!(open_value, 6);
        assert_eq!(count, 2);
        
        // Regular tile should return (new_value, 1)
        let three_six = create_tile(3, 6);
        let (open_value, count) = layout.attach(three_six, Some(0));
        assert_eq!(open_value, 3);
        assert_eq!(count, 1);
        
        // Double tile attached should return (value, 2)
        let double_three = create_tile(3, 3);
        let (open_value, count) = layout.attach(double_three, Some(1));
        assert_eq!(open_value, 3);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_layout_node_fields() {
        let tile = create_tile(2, 5);
        let node = LayoutNode {
            tile,
            parent: None,
            children: vec![],
        };
        
        // Test all public fields are accessible
        assert_eq!(node.tile, tile);
        assert_eq!(node.parent, None);
        assert!(node.children.is_empty());
        
        // Test with parent and children
        let node2 = LayoutNode {
            tile: create_tile(3, 4),
            parent: Some(5),
            children: vec![1, 3, 7],
        };
        
        assert_eq!(node2.tile, create_tile(3, 4));
        assert_eq!(node2.parent, Some(5));
        assert_eq!(node2.children, vec![1, 3, 7]);
    }

    #[test]
    fn test_layout_public_fields() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        
        // Test nodes field access
        assert!(layout.nodes.is_empty());
        
        let double_three = create_tile(3, 3);
        layout.attach(double_three, None);
        
        // Test that nodes field is accessible and populated
        assert_eq!(layout.nodes.len(), 1);
        assert_eq!(layout.nodes[0].tile, double_three);
        assert_eq!(layout.nodes[0].parent, None);
        assert!(layout.nodes[0].children.is_empty());
        
        // Test open field access
        assert!(layout.open.contains_key(&0));
        let open_values = layout.open.get_vec(&0).unwrap();
        assert_eq!(open_values.len(), 2);
        assert!(open_values.iter().all(|&x| x == 3));
        
        // Test end_counts field access
        assert_eq!(layout.end_counts.len(), 7); // 0-6 inclusive
        assert_eq!(layout.end_counts[3], 2);
        for i in &[0, 1, 2, 4, 5, 6] {
            assert_eq!(layout.end_counts[*i], 0);
        }
    }

    #[test]
    fn test_attach_multiple_children_to_same_parent() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        
        // Start with double-4
        let double_four = create_tile(4, 4);
        layout.attach(double_four, None);
        
        // Attach multiple tiles to the same parent
        let four_one = create_tile(1, 4);
        layout.attach(four_one, Some(0));
        
        let four_six = create_tile(4, 6);
        layout.attach(four_six, Some(0));
        
        // Parent should have both children
        assert_eq!(layout.nodes[0].children, vec![1, 2]);
        
        // Parent should have no open ends left
        assert!(layout.open.get_vec(&0).is_none() || layout.open.get_vec(&0).unwrap().is_empty());
        assert_eq!(layout.open_count(4), 0);
        
        // Children should have their respective open ends
        assert_eq!(layout.open_count(1), 1);
        assert_eq!(layout.open_count(6), 1);
    }

    #[test]
    fn test_end_counts_consistency() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        
        // Build a complex layout and verify end_counts matches actual open entries
        let double_two = create_tile(2, 2);
        layout.attach(double_two, None);
        
        let two_five = create_tile(2, 5);
        layout.attach(two_five, Some(0));
        
        let two_six = create_tile(2, 6);
        layout.attach(two_six, Some(0));
        
        let double_five = create_tile(5, 5);
        layout.attach(double_five, Some(1));
        
        // Manually count open ends and compare with end_counts
        let mut actual_counts = vec![0u8; 7];
        for (_, values) in layout.open.iter_all() {
            for &value in values {
                actual_counts[value as usize] += 1;
            }
        }
        
        for i in 0..7 {
            assert_eq!(layout.end_counts[i], actual_counts[i], 
                      "Mismatch at index {}: end_counts={}, actual={}", 
                      i, layout.end_counts[i], actual_counts[i]);
        }
    }

    #[test]
    #[should_panic]
    fn test_attach_incompatible_tiles() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        
        let double_six = create_tile(6, 6);
        layout.attach(double_six, None);
        
        // Try to attach a tile that doesn't match any open end
        let one_two = create_tile(1, 2);
        layout.attach(one_two, Some(0)); // Should panic - no 1 or 2 open on node 0
    }

    #[test]
    #[should_panic]
    fn test_attach_non_double_as_first_tile() {
        let configuration = rules::Configuration::new(2, rules::Variation::Traditional, 6, 7);
        let mut layout = Layout::new(&configuration);
        
        let three_six = create_tile(3, 6);
        layout.attach(three_six, None); // Should panic - first tile must be double
    }
}
