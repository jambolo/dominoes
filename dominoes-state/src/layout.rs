//! Dominoes layout implementation
//!
//! This module provides the Layout struct for managing the layout of domino tiles.

use std::fmt::{self, Display, Formatter};
use multimap::MultiMap;
use ego_tree;
use serde::{Serialize, Deserialize, Deserializer, Serializer};
use serde::de::{self, Visitor, MapAccess};

use rules::{Configuration, Tile};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutNode {
    /// The tile
    pub tile: Tile,
    /// Index of the node's parent, `None` indicates this is the root node
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl Serialize for Layout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("Layout", 2)?;
        state.serialize_field("nodes", &self.nodes)?;
        state.serialize_field("set_id", &(self.end_counts.len().saturating_sub(1)))?; // -1 because end_counts has length set_id+1
        state.end()
    }
}

impl<'de> Deserialize<'de> for Layout {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field { Nodes, SetId }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("`nodes` or `set_id`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "nodes" => Ok(Field::Nodes),
                            "set_id" => Ok(Field::SetId),
                            _ => Err(de::Error::unknown_field(value, &["nodes", "set_id"])),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct LayoutVisitor;

        impl<'de> Visitor<'de> for LayoutVisitor {
            type Value = Layout;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Layout")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Layout, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut nodes = None;
                let mut set_id = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Nodes => {
                            if nodes.is_some() {
                                return Err(de::Error::duplicate_field("nodes"));
                            }
                            nodes = Some(map.next_value()?);
                        }
                        Field::SetId => {
                            if set_id.is_some() {
                                return Err(de::Error::duplicate_field("set_id"));
                            }
                            set_id = Some(map.next_value()?);
                        }
                    }
                }

                let nodes: Vec<LayoutNode> = nodes.ok_or_else(|| de::Error::missing_field("nodes"))?;
                let set_id: usize = set_id.ok_or_else(|| de::Error::missing_field("set_id"))?;

                // Reconstruct open and end_counts from nodes
                let mut layout = Layout {
                    nodes,
                    open: MultiMap::new(),
                    end_counts: vec![0; set_id + 1], // +1 because values are 0..set_id inclusive
                };

                layout.rebuild_open_and_end_counts().map_err(de::Error::custom)?;
                Ok(layout)
            }
        }

        const FIELDS: &[&str] = &["nodes", "set_id"];
        deserializer.deserialize_struct("Layout", FIELDS, LayoutVisitor)
    }
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

        let (end_value, created_count) = match parent_index {
            Some(parent_index) => {
                assert!(parent_index < self.nodes.len() && self.open.contains_key(&parent_index));

                let parent = &self.nodes[parent_index].tile;

                // Determine the matched and open values
                assert!(parent.can_attach(tile));
                let (matched_value, open_value) = if parent.as_tuple().0 == a || parent.as_tuple().1 == a {
                    (a, b)
                } else {
                    (b, a)
                };

                // Add a new tile node to the layout
                let tile_index = self.nodes.len();
                self.nodes.push(LayoutNode {
                    tile,
                    parent: Some(parent_index),
                    children: Vec::new(),
                });

                // Add the open ends. If the tile is a double, add twice.
                let open_count = if tile.is_double() { 2 } else { 1 };
                for _ in 0..open_count {
                    self.open.insert(tile_index, open_value);
                }
                self.end_counts[open_value as usize] += open_count;

                // Remove the parent's open end from the open list
                self.remove_from_open(parent_index, matched_value);
                self.end_counts[matched_value as usize] -= 1;

                // Add the new tile node's index to the parent's list of children
                self.nodes[parent_index].children.push(tile_index);
                (open_value, open_count)
            }
            None => {
                // The first tile is a special case
                assert!(tile.is_double());
                assert!(self.nodes.is_empty() && self.open.is_empty());

                // Add the new tile node to the layout
                self.nodes.push(LayoutNode {
                    tile,
                    parent: None,
                    children: Vec::new(),
                });

                // Both ends are open for the first tile.
                self.open.insert(0, a);
                self.open.insert(0, a);
                self.end_counts[a as usize] += 2;
                (a, 2)
            }
        };

        (end_value, created_count)
    }

    /// Returns a vector of node indices that have an open end with the specified value.
    ///
    /// This function scans the layout and returns the indices of all nodes that currently have an open end matching the given
    /// value. If there are no such nodes, an empty vector is returned.
    ///
    /// # Arguments
    /// * `end_value` - The domino value to search for among open ends (e.g., 0-6 for double-six)
    ///
    /// # Returns
    /// A vector of node indices.
    ///
    /// # Panics
    /// Panics if `end_value` is not a valid end value.
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Layout;
    /// # use rules::{Configuration, Tile};
    /// let config = Configuration::default();
    /// let mut layout = Layout::new(&config);
    /// layout.attach(Tile::from((6, 6)), None);
    /// layout.attach(Tile::from((3, 6)), Some(0));
    /// // Node 0 has one open 6, node 1 has open 3
    /// let six_nodes = layout.get_nodes_with_open_end(6);
    /// assert!(six_nodes.contains(&0));
    /// let three_nodes = layout.get_nodes_with_open_end(3);
    /// assert!(three_nodes.contains(&1));
    /// let four_nodes = layout.get_nodes_with_open_end(4);
    /// assert!(four_nodes.is_empty());
    /// ```
    pub fn get_nodes_with_open_end(&self, end_value: u8) -> Vec<usize> {
        // Quick return if there are no open ends with that value
        assert!((end_value as usize) < self.end_counts.len());
        if self.end_counts[end_value as usize] == 0 {
            return Vec::new();
        }

        self.open
            .iter()
            .filter_map(|(node_index, value)| {
                (*value == end_value).then_some(*node_index)
            })
            .collect()
    }

    /// Creates an ego_tree representation of the layout
    ///
    /// # Returns
    /// - `Some(Tree<Tile>)` if the layout contains tiles
    /// - `None` if the layout is empty
    ///
    /// # Examples
    /// ```rust
    /// # use dominoes_state::Layout;
    /// # use rules::{Configuration, Tile};
    ///
    /// let config = Configuration::default();
    /// let mut layout = Layout::new(&config);
    ///
    /// // Empty layout returns None
    /// assert!(layout.to_tree().is_none());
    ///
    /// // Build a simple chain: 6|6-6|3-3|1
    /// let double_six = Tile::from((6, 6));
    /// layout.attach(double_six, None);
    /// let three_six = Tile::from((3, 6));
    /// layout.attach(three_six, Some(0));
    /// let one_three = Tile::from((1, 3));
    /// layout.attach(one_three, Some(1));
    ///
    /// let tree = layout.to_tree().unwrap();
    /// assert_eq!(tree.root().value(), &double_six);
    ///
    /// // Tree preserves the layout structure
    /// let root_children: Vec<_> = tree.root().children().collect();
    /// assert_eq!(root_children.len(), 1);
    /// assert_eq!(root_children[0].value(), &three_six);
    /// ```
    ///
    /// # Panics
    /// Panics if any non-root node (index > 0) has a `None` parent, as this violates the
    /// expected layout structure.
    pub fn to_tree(&self) -> Option<ego_tree::Tree<Tile>> {
        if self.nodes.is_empty() {
            return None;
        }

        // Map from layout node index to ego_tree NodeId
        let mut node_ids = Vec::new();

        let mut tree = ego_tree::Tree::new(self.nodes[0].tile);
        let root_id = tree.root().id();
        node_ids.push(root_id);

        // Build the tree by processing nodes in order
        for (index, node) in self.nodes.iter().enumerate() {
            if index == 0 {
                continue; // Skip root, already created
            }

            let parent_index = node.parent.expect("Non-root node must have parent");
            let parent_id = node_ids[parent_index];

            let mut parent = tree.get_mut(parent_id).expect("Parent node should exist");
            let child = parent.append(node.tile);

            node_ids.push(child.id());
        }

        Some(tree)
    }

    /// Rebuilds the `open` and `end_counts` fields from the `nodes` structure.
    ///
    /// This method is used during deserialization to reconstruct the derived state from the serialized nodes. It analyzes the tree
    /// structure to determine which ends are open and updates the counts accordingly.
    fn rebuild_open_and_end_counts(&mut self) -> Result<(), String>{
        self.open.clear();
        self.end_counts.fill(0);

        if self.nodes.is_empty() {
            return Ok(());
        }

        // For each node, determine its open ends based on its connectivity
        for (node_index, node) in self.nodes.iter().enumerate() {
            // Validate the children count
            // Double tiles can have up to 2 children.
            // Non-doubles can have up to 2 children if it is the root node (otherwise only 1).
            if node.children.len() > 2 {
                return Err(format!("Tile node {node_index} has more than 2 children"));
            }

            let (a, b) = node.tile.as_tuple();

            // Count connections for each value
            let mut connections_a = 0;
            let mut connections_b = 0;

            // Count parent connection
            if let Some(parent_index) = node.parent {
                let (parent_a, parent_b) = self.nodes[parent_index].tile.as_tuple();
                if a == parent_a || a == parent_b {
                    connections_a += 1;
                } else {
                    connections_b += 1;
                }
            } else {
                // Root node, a double has an implicit parent connection, a non-double does not
                if node.tile.is_double() {
                    connections_a += 1; // both ends are 'a'
                }
            }

            // Count child connections
            for &child_index in &node.children {
                let (child_a, child_b) = self.nodes[child_index].tile.as_tuple();
                if a == child_a || a == child_b {
                    connections_a += 1;
                } else {
                    connections_b += 1;
                }
            }

            // Determine open ends
            if node.tile.is_double() {
                // For double tiles, both ends have the same value
                // Total connections possible = 3, used = connections_a (since a == b)
                let total_connections = connections_a; // connections_a == connections_b for doubles
                let remaining_connections = 3 - total_connections;

                for _ in 0..remaining_connections {
                    self.open.insert(node_index, a);
                    self.end_counts[a as usize] += 1;
                }
            } else {
                if node.parent.is_some() && node.children.len() > 1 {
                    return Err(format!("Non-double tile node {node_index} has more than 1 child"));
                }
                // For regular tiles, each end can have at most 1 connection
                if connections_a == 0 {
                    self.open.insert(node_index, a);
                    self.end_counts[a as usize] += 1;
                }
                if connections_b == 0 {
                    self.open.insert(node_index, b);
                    self.end_counts[b as usize] += 1;
                }
            }
        }
        Ok(())
    }

    // Removes a tile from the open list. Note that a double tile will have two entries with the same key, and only one of the
    // entries is removed.
    fn remove_from_open(&mut self, parent: usize, value: u8) {
        let values = self.open.get_vec_mut(&parent)
            .unwrap_or_else(|| panic!("Parent {parent} not found in open list"));
        let pos = values.iter().position(|&x| x == value)
            .unwrap_or_else(|| panic!("Value {value} not found for parent {parent}"));
        values.remove(pos);
        // If the vector is now empty, we need to remove the key from the multimap
        if values.is_empty() {
            self.open.remove(&parent);
        }
    }

    /// Recursive helper for formatting the layout as a string.
    fn fmt_r(&self, node: &LayoutNode, open: u8) -> String {
        let (a, b) = node.tile.as_tuple();
        let (a, b) = if open == a { (a, b) } else { (b, a) }; // Swap if necessary (tiles are added left-to-right)

        let mut result = format!("{a}|{b}");

        // Add the children recursively
        match node.children.len() {
            0 => {}, // Open end
            1 => {
                let child_node = &self.nodes[node.children[0]];
                result.push('-');
                result.push_str(&self.fmt_r(child_node, b));
            },
            _ => {
                result.push_str("=(");
                for (i, &child) in node.children.iter().enumerate() {
                    let child_node = &self.nodes[child];
                    if i > 0 { result.push(','); }
                    result.push_str(&self.fmt_r(child_node, b));
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
            return Ok(());
        }

        let root = &self.nodes[0];
        let (a, b) = root.tile.as_tuple();
        assert_eq!(a, b, "First node must be a double");
        write!(f, "{}", self.fmt_r(root, b))
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

        // Test with no parent and no children
        let root_node = LayoutNode {
            tile: create_tile(2, 5),
            parent: None,
            children: vec![],
        };

        assert_eq!(root_node.tile, create_tile(2, 5));
        assert_eq!(root_node.parent, None);
        assert!(root_node.children.is_empty());
    }

    #[test]
    fn test_new_layout() {
        let configuration = rules::Configuration::default();
        let layout = Layout::new(&configuration);
        assert!(layout.nodes.is_empty());
        assert!(layout.open.is_empty());
        assert_eq!(layout.end_counts.len(), 7); // 0..6 inclusive = 7 elements
    }
        #[test]
    fn test_attach_first_tile() {
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
        let layout = Layout::new(&configuration);
        assert_eq!(layout.to_string(), "");
    }

    #[test]
    fn test_to_string_single_tile() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);

        layout.attach(double_six, None);

        assert_eq!(layout.to_string(), "6|6");
    }

    #[test]
    fn test_to_string_linear_chain() {
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let tile = create_tile(3, 6);
        layout.attach(tile, Some(5)); // Parent index 5 doesn't exist
    }

    #[test]
    #[should_panic]
    fn test_attach_first_tile_to_nonempty_layout() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        let three_six = create_tile(3, 6);

        layout.attach(double_six, None);
        layout.attach(three_six, None); // Should panic - trying to add first tile to non-empty layout
    }

    #[test]
    #[should_panic]
    fn test_attach_to_parent_when_empty() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let tile = create_tile(3, 6);
        layout.attach(tile, Some(0)); // Should panic - layout is empty
    }

    #[test]
    fn test_remove_from_open() {
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        layout.remove_from_open(5, 6);
    }

    #[test]
    #[should_panic(expected = "Value 9 not found for parent 0")]
    fn test_remove_from_open_invalid_value() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        layout.attach(double_six, None);
        layout.remove_from_open(0, 9); // Value 9 doesn't exist for this tile
    }

    #[test]
    fn test_is_empty() {
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
    fn test_layout_public_fields() {
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
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
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);

        let three_six = create_tile(3, 6);
        layout.attach(three_six, None); // Should panic - first tile must be double
    }

    #[test]
    fn test_to_tree_empty_layout() {
        let configuration = rules::Configuration::default();
        let layout = Layout::new(&configuration);

        assert!(layout.to_tree().is_none());
    }

    #[test]
    fn test_to_tree_single_tile() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);

        layout.attach(double_six, None);

        let tree = layout.to_tree().unwrap();
        assert_eq!(tree.root().value(), &double_six);
        assert_eq!(tree.root().children().count(), 0);
    }

    #[test]
    fn test_to_tree_linear_chain() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        let three_six = create_tile(3, 6);
        let one_three = create_tile(1, 3);

        layout.attach(double_six, None);
        layout.attach(three_six, Some(0));
        layout.attach(one_three, Some(1));

        let tree = layout.to_tree().unwrap();

        // Check root
        assert_eq!(tree.root().value(), &double_six);

        // Check first child
        let root_children: Vec<_> = tree.root().children().collect();
        assert_eq!(root_children.len(), 1);
        assert_eq!(root_children[0].value(), &three_six);

        // Check second child (grandchild of root)
        let second_level_children: Vec<_> = root_children[0].children().collect();
        assert_eq!(second_level_children.len(), 1);
        assert_eq!(second_level_children[0].value(), &one_three);
        assert_eq!(second_level_children[0].children().count(), 0);
    }

    #[test]
    fn test_to_tree_branching() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let double_three = create_tile(3, 3);
        let two_three = create_tile(2, 3);
        let three_five = create_tile(3, 5);

        layout.attach(double_three, None);
        layout.attach(two_three, Some(0));
        layout.attach(three_five, Some(0));

        let tree = layout.to_tree().unwrap();

        // Check root
        assert_eq!(tree.root().value(), &double_three);

        // Check children - should have two
        let root_children: Vec<_> = tree.root().children().collect();
        assert_eq!(root_children.len(), 2);

        // Children should be in order they were added
        assert_eq!(root_children[0].value(), &two_three);
        assert_eq!(root_children[1].value(), &three_five);

        // Both children should be leaf nodes
        assert_eq!(root_children[0].children().count(), 0);
        assert_eq!(root_children[1].children().count(), 0);
    }

    #[test]
    fn test_to_tree_complex_structure() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let double_six = create_tile(6, 6);
        let three_six = create_tile(3, 6);
        let double_three = create_tile(3, 3);
        let one_three = create_tile(1, 3);
        let three_five = create_tile(3, 5);
        let two_five = create_tile(2, 5);

        // Build: 6|6-6|3-3|3=(3|1,3|5-5|2)
        layout.attach(double_six, None);
        layout.attach(three_six, Some(0));
        layout.attach(double_three, Some(1));
        layout.attach(one_three, Some(2));
        layout.attach(three_five, Some(2));
        layout.attach(two_five, Some(4));

        let tree = layout.to_tree().unwrap();

        // Root should be 6|6
        assert_eq!(tree.root().value(), &double_six);

        // Root should have one child (3|6)
        let level1: Vec<_> = tree.root().children().collect();
        assert_eq!(level1.len(), 1);
        assert_eq!(level1[0].value(), &three_six);

        // 3|6 should have one child (3|3)
        let level2: Vec<_> = level1[0].children().collect();
        assert_eq!(level2.len(), 1);
        assert_eq!(level2[0].value(), &double_three);

        // 3|3 should have two children (1|3 and 3|5)
        let level3: Vec<_> = level2[0].children().collect();
        assert_eq!(level3.len(), 2);
        assert_eq!(level3[0].value(), &one_three);
        assert_eq!(level3[1].value(), &three_five);

        // 1|3 should be a leaf
        assert_eq!(level3[0].children().count(), 0);

        // 3|5 should have one child (2|5)
        let level4: Vec<_> = level3[1].children().collect();
        assert_eq!(level4.len(), 1);
        assert_eq!(level4[0].value(), &two_five);
        assert_eq!(level4[0].children().count(), 0);
    }

    #[test]
    fn test_to_tree_node_order_preservation() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let double_four = create_tile(4, 4);
        let four_one = create_tile(1, 4);
        let four_six = create_tile(4, 6);

        // Attach children in specific order
        layout.attach(double_four, None);
        layout.attach(four_one, Some(0));    // First child
        layout.attach(four_six, Some(0));    // Second child

        let tree = layout.to_tree().unwrap();

        // Children should be in the order they were attached
        let children: Vec<_> = tree.root().children().collect();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].value(), &four_one);
        assert_eq!(children[1].value(), &four_six);
    }

    #[test]
    fn test_to_tree_all_nodes_have_correct_parents() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let double_two = create_tile(2, 2);
        let two_five = create_tile(2, 5);
        let two_six = create_tile(2, 6);
        let double_five = create_tile(5, 5);

        layout.attach(double_two, None);
        layout.attach(two_five, Some(0));
        layout.attach(two_six, Some(0));
        layout.attach(double_five, Some(1));

        let tree = layout.to_tree().unwrap();

        // Verify the tree structure matches the layout structure
        let mut node_count = 0;
        for node in tree.root().descendants() {
            node_count += 1;

            // Each node should correspond to a tile in the layout
            let found = layout.nodes.iter().any(|layout_node| layout_node.tile == *node.value());
            assert!(found, "Tree node {:?} not found in layout", node.value());
        }

        assert_eq!(node_count, layout.nodes.len());
    }

    #[test]
    #[should_panic(expected = "Non-root node must have parent")]
    fn test_to_tree_panics_on_invalid_layout() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);

        // Manually create an invalid layout with a non-root node having no parent
        layout.nodes.push(LayoutNode {
            tile: create_tile(6, 6),
            parent: None,
            children: Vec::new(),
        });
        layout.nodes.push(LayoutNode {
            tile: create_tile(3, 6),
            parent: None, // This should cause a panic
            children: Vec::new(),
        });

        layout.to_tree(); // Should panic
    }

    #[test]
    fn test_to_tree_preserves_tile_data() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);
        let tiles = vec![
            create_tile(6, 6),
            create_tile(3, 6),
            create_tile(1, 3),
            create_tile(1, 2),
        ];

        layout.attach(tiles[0], None);
        layout.attach(tiles[1], Some(0));
        layout.attach(tiles[2], Some(1));

        let tree = layout.to_tree().unwrap();

        // Collect all tiles from the tree
        let tree_tiles: Vec<Tile> = tree.root().descendants().map(|node| *node.value()).collect();

        // Should have the first 3 tiles
        assert_eq!(tree_tiles.len(), 3);
        assert!(tree_tiles.contains(&tiles[0]));
        assert!(tree_tiles.contains(&tiles[1]));
        assert!(tree_tiles.contains(&tiles[2]));
    }

    #[test]
    fn test_get_nodes_with_open_end() {
        let configuration = rules::Configuration::default();

        // Test empty layout
        let layout = Layout::new(&configuration);
        assert!(layout.get_nodes_with_open_end(0).is_empty());
        assert!(layout.get_nodes_with_open_end(6).is_empty());

        // Test basic functionality
        let mut layout = Layout::new(&configuration);
        let double_six = Tile::from((6, 6));
        let three_six = Tile::from((3, 6));
        let one_three = Tile::from((1, 3));

        layout.attach(double_six, None); // node 0
        layout.attach(three_six, Some(0)); // node 1
        layout.attach(one_three, Some(1)); // node 2

        // Node 0: open 6, Node 1: no open ends (3 was consumed), Node 2: open 1
        let six_nodes = layout.get_nodes_with_open_end(6);
        assert_eq!(six_nodes, vec![0]);
        let three_nodes = layout.get_nodes_with_open_end(3);
        assert!(three_nodes.is_empty());
        let one_nodes = layout.get_nodes_with_open_end(1);
        assert_eq!(one_nodes, vec![2]);
        let four_nodes = layout.get_nodes_with_open_end(4);
        assert!(four_nodes.is_empty());

        // Test multiple nodes
        let mut layout2 = Layout::new(&configuration);
        let double_three = Tile::from((3, 3));
        let three_two = Tile::from((2, 3));
        let three_five = Tile::from((3, 5));

        layout2.attach(double_three, None); // node 0
        layout2.attach(three_two, Some(0)); // node 1
        layout2.attach(three_five, Some(0)); // node 2

        // Node 1: open 2, Node 2: open 5
        // After attaching, node 0 should have no open ends
        assert!(layout2.get_nodes_with_open_end(3).is_empty());
        assert_eq!(layout2.get_nodes_with_open_end(2), vec![1]);
        assert_eq!(layout2.get_nodes_with_open_end(5), vec![2]);
    }

    #[test]
    fn test_layout_node_serialization() {
        // Test serialization of LayoutNode with parent
        let node_with_parent = LayoutNode {
            tile: Tile::from((3, 6)),
            parent: Some(0),
            children: vec![2, 3],
        };

        let json = serde_json::to_string(&node_with_parent).expect("Serialization failed");
        let deserialized: LayoutNode = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.tile, node_with_parent.tile);
        assert_eq!(deserialized.parent, node_with_parent.parent);
        assert_eq!(deserialized.children, node_with_parent.children);

        // Test serialization of LayoutNode without parent (root node)
        let root_node = LayoutNode {
            tile: Tile::from((6, 6)),
            parent: None,
            children: vec![1],
        };

        let json = serde_json::to_string(&root_node).expect("Serialization failed");
        let deserialized: LayoutNode = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.tile, root_node.tile);
        assert_eq!(deserialized.parent, None);
        assert_eq!(deserialized.children, root_node.children);

        // Verify that parent field is not present in JSON for root node
        assert!(!json.contains("\"parent\""));
    }

    #[test]
    fn test_layout_serialization_simple() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);

        // Create a simple layout: 6|6-3|6
        let double_six = Tile::from((6, 6));
        layout.attach(double_six, None);

        let three_six = Tile::from((3, 6));
        layout.attach(three_six, Some(0));

        // Serialize and deserialize
        let json = serde_json::to_string(&layout).expect("Serialization failed");
        let deserialized: Layout = serde_json::from_str(&json).expect("Deserialization failed");

        // Verify nodes are the same
        assert_eq!(deserialized.nodes.len(), layout.nodes.len());
        for (i, (original, deserialized)) in layout.nodes.iter().zip(deserialized.nodes.iter()).enumerate() {
            assert_eq!(original.tile, deserialized.tile, "Tile mismatch at node {}", i);
            assert_eq!(original.parent, deserialized.parent, "Parent mismatch at node {}", i);
            assert_eq!(original.children, deserialized.children, "Children mismatch at node {}", i);
        }

        // Verify end_counts size is preserved
        assert_eq!(deserialized.end_counts.len(), layout.end_counts.len());

        // Verify open ends are reconstructed correctly
        assert_eq!(deserialized.open_count(6), layout.open_count(6));
        assert_eq!(deserialized.open_count(3), layout.open_count(3));
        for i in 0..7 {
            assert_eq!(deserialized.open_count(i), layout.open_count(i), "Open count mismatch for value {}", i);
        }

        // Verify specific open ends
        assert_eq!(deserialized.get_nodes_with_open_end(6), layout.get_nodes_with_open_end(6));
        assert_eq!(deserialized.get_nodes_with_open_end(3), layout.get_nodes_with_open_end(3));
    }

    #[test]
    fn test_layout_serialization_complex() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);

        // Create a more complex layout: 4|4=(2|4, 4|1-1|5)
        let double_four = Tile::from((4, 4));
        layout.attach(double_four, None);

        let two_four = Tile::from((2, 4));
        layout.attach(two_four, Some(0));

        let one_four = Tile::from((1, 4));
        layout.attach(one_four, Some(0));

        let one_five = Tile::from((1, 5));
        layout.attach(one_five, Some(2));

        // Serialize and deserialize
        let json = serde_json::to_string(&layout).expect("Serialization failed");
        let deserialized: Layout = serde_json::from_str(&json).expect("Deserialization failed");

        // Verify structure is preserved
        assert_eq!(deserialized.nodes.len(), 4);
        assert_eq!(deserialized.nodes[0].children, vec![1, 2]);
        assert_eq!(deserialized.nodes[2].children, vec![3]);

        // Verify open ends are reconstructed correctly
        for i in 0..7 {
            assert_eq!(deserialized.open_count(i), layout.open_count(i), "Open count mismatch for value {}", i);
        }

        // Specific checks for this layout
        assert_eq!(deserialized.open_count(2), 1); // from node 1
        assert_eq!(deserialized.open_count(5), 1); // from node 3
        assert_eq!(deserialized.open_count(4), 0); // all 4s are used
        assert_eq!(deserialized.open_count(1), 0); // all 1s are used
    }

    #[test]
    fn test_layout_serialization_empty() {
        let configuration = rules::Configuration::default();
        let layout = Layout::new(&configuration);

        // Serialize and deserialize empty layout
        let json = serde_json::to_string(&layout).expect("Serialization failed");
        let deserialized: Layout = serde_json::from_str(&json).expect("Deserialization failed");

        assert!(deserialized.nodes.is_empty());
        assert!(deserialized.open.is_empty());
        assert_eq!(deserialized.end_counts, vec![0; 7]);
        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_layout_serialization_double_tiles() {
        let configuration = rules::Configuration::default();
        let mut layout = Layout::new(&configuration);

        // Create layout step by step and debug each step
        let double_three = Tile::from((3, 3));
        layout.attach(double_three, None);
        println!("After attaching root 3|3: open count 3 = {}", layout.open_count(3));

        let another_double_three = Tile::from((3, 3));
        layout.attach(another_double_three, Some(0));
        println!("After attaching second 3|3: open count 3 = {}", layout.open_count(3));

        let three_five = Tile::from((3, 5));
        layout.attach(three_five, Some(0));
        println!("After attaching 3|5: open count 3 = {}, open count 5 = {}", layout.open_count(3), layout.open_count(5));

        // Check node structure
        println!("Node 0 children: {:?}", layout.nodes[0].children);
        println!("Node 1 parent: {:?}, children: {:?}", layout.nodes[1].parent, layout.nodes[1].children);
        println!("Node 2 parent: {:?}, children: {:?}", layout.nodes[2].parent, layout.nodes[2].children);

        // Check what each node has open
        for i in 0..3 {
            if let Some(open_values) = layout.open.get_vec(&i) {
                println!("Node {} open values: {:?}", i, open_values);
            } else {
                println!("Node {} has no open values", i);
            }
        }

        // Serialize and deserialize
        let json = serde_json::to_string(&layout).expect("Serialization failed");
        let deserialized: Layout = serde_json::from_str(&json).expect("Deserialization failed");

        // Debug: Check the deserialized layout state
        println!("Deserialized layout open count for 3: {}", deserialized.open_count(3));
        println!("Deserialized layout open count for 5: {}", deserialized.open_count(5));

        // Verify double tile handling
        assert_eq!(deserialized.open_count(3), layout.open_count(3));
        assert_eq!(deserialized.open_count(5), layout.open_count(5));
    }
}
