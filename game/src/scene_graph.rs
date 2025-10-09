//! Scene graph module for computing tile placements and rendering information

use ego_tree::{NodeId, Tree};
use iced::{Point, Rectangle, Size, Vector};
use rules::Tile;
use std::collections::HashMap;

// Size and spacing constants for tiles
const TILE_SIZE: Size = Size::new(640.0, 1280.0);  // Size of tile in model space
const SPACING: f32 = 8.0;                           // Space between tiles in model space
const TILE_OFFSET_X: f32 = TILE_SIZE.width / 2.0 + SPACING;  // Distance from center to edge plus spacing
const TILE_OFFSET_Y: f32 = TILE_SIZE.height / 2.0 + SPACING; // Distance from center to edge plus spacing

/// Represents one of the four sides of a domino tile for attachment and rotation purposes.
///
/// The enum values correspond to cardinal directions when a tile is in its default orientation:
/// - `Right`: The right side of the tile (0° rotation)
/// - `Bottom`: The bottom side of the tile (90° clockwise rotation)
/// - `Left`: The left side of the tile (180° rotation)
/// - `Top`: The top side of the tile (270° clockwise rotation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TileSide {
    Right,
    Bottom,
    Left,
    Top,
}

impl TileSide {
    /// Rotates this side by the specified rotation amount.
    ///
    /// # Arguments
    /// * `rotation` - The rotation to apply
    ///
    /// # Returns
    /// The new side after rotation
    pub fn rotate(self, rotation: TileSide) -> Self {
        Self::from((i8::from(self) + i8::from(rotation)) % 4)
    }
}

impl From<TileSide> for i8 {
    fn from(side: TileSide) -> i8 {
        match side {
            TileSide::Right => 0,
            TileSide::Bottom => 1,
            TileSide::Left => 2,
            TileSide::Top => 3,
        }
    }
}

impl From<i8> for TileSide {
    fn from(value: i8) -> Self {
        match value {
            0 => TileSide::Right,
            1 => TileSide::Bottom,
            2 => TileSide::Left,
            3 => TileSide::Top,
            _ => panic!("Invalid value for TileSide: {value}"),
        }
    }
}

/// Placement information for a tile.
#[derive(Debug, Clone)]
struct Placement {
    /// The center position of the tile in world coordinates
    position: Vector,
    /// The rotation of the tile from its default orientation
    rotation: TileSide,
    /// Available attachment points for connecting child tiles
    attachments: Vec<TileSide>,
}

/// A map of node IDs to their placement information
type PlacementMap = HashMap<NodeId, Placement>;

impl Placement {
    /// Creates a placement for the first tile at the origin.
    ///
    /// The first tile has no parent to attach to and is placed sideways at (0,0) with top/bottom attachments available.
    fn new_root() -> Self {
        Self {
            position: Vector::ZERO, // Centered
            rotation: TileSide::Top, // Sideways
            attachments: vec![TileSide::Top, TileSide::Bottom], // Order is always Top then Bottom
        }
    }

    /// Creates a placement for a child tile attached to a parent.
    ///
    /// # Arguments
    /// * `child_tile` - The child tile to place
    /// * `parent_tile` - The parent tile to attach to
    /// * `parent_placement` - Mutable reference to parent's placement (attachments will be modified as tiles are attached)
    ///
    /// # Returns
    /// A new placement for the child tile
    fn new_child(
        child_tile: &Tile,
        parent_tile: &Tile,
        parent_placement: &mut Placement
    ) -> Self {
        let parent_tuple = parent_tile.as_tuple();
        let (attach_value, _) = child_tile
            .matches(parent_tile)
            .expect("Child tile must match parent tile");

        let parent_attachment =
            get_parent_attachment(&parent_tuple, attach_value, &mut parent_placement.attachments);
        let parent_attachment_offset = attachment_offset(parent_attachment);

        // Get the rotated child's position relative to the unrotated parent and then rotate by the parent's rotation
        // and then translate from the parent's position.
        let child_tuple = child_tile.as_tuple();
        let mut child_attachments = vec![TileSide::Top, TileSide::Bottom]; // Order is always Top then Bottom
        let child_attachment = get_child_attachment(&child_tuple, attach_value, &mut child_attachments);
        let child_rotation = child_attachment_to_parent_rotation(child_attachment, parent_attachment);

        let child_position = rotated_vector(
            &(parent_attachment_offset - rotated_vector(&attachment_offset(child_attachment), child_rotation)),
            parent_placement.rotation,
        ) + parent_placement.position;

        Self {
            position: child_position,
            rotation: child_rotation.rotate(parent_placement.rotation),
            attachments: child_attachments,
        }
    }
}

/// Information about a tile's rendering properties
#[derive(Debug, Clone)]
pub struct RenderListNode {
    /// The tile to render
    pub tile: Tile,
    /// The center position of the tile in world coordinates
    pub position: Vector,
    /// The rotation of the tile in radians
    pub rotation: f32,
    /// The size of the tile
    pub size: Size,
}

/// A list of tiles with their rendering information
pub type RenderList = Vec<RenderListNode>;

/// A scene graph that manages the layout and rendering information for domino tiles
#[derive(Debug)]
pub struct SceneGraph {
    bounds: Rectangle,
    render_list: RenderList,
}

impl SceneGraph {
    /// Creates a new SceneGraph from a tile tree.
    ///
    /// # Arguments
    /// * `tree` - The domino tree to visualize
    ///
    /// # Returns
    /// A new SceneGraph instance
    pub fn new(tree: &Tree<Tile>) -> Self {
        let placements = Self::compute_placements(tree);
        let bounds = Self::compute_bounds(&placements);
        let render_list = Self::build_render_list(tree, &placements);

        Self { bounds, render_list }
    }

    /// Returns the bounding rectangle that contains all tiles.
    ///
    /// # Returns
    /// A rectangle that bounds all tiles
    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }

    /// Returns a list of all tiles with their positions and orientations for rendering.
    ///
    /// # Returns
    /// A slice of RenderListNode containing rendering information for each tile
    pub fn render_list(&self) -> &[RenderListNode] {
        &self.render_list
    }

    // Computes the placement information for all tiles in the tree.
    fn compute_placements(tree: &Tree<Tile>) -> PlacementMap {
        tree.root()
            .descendants()
            .fold(PlacementMap::default(), |mut placements, node| {
                let placement = match node.parent() {
                    Some(parent) => {
                        let parent_placement = placements
                            .get_mut(&parent.id())
                            .expect("Parent placement must exist");
                        Placement::new_child(node.value(), parent.value(), parent_placement)
                    }
                    None => Placement::new_root(),
                };
                placements.insert(node.id(), placement);
                placements
            })
    }

    // Builds the render list from the tree and placements.
    fn build_render_list(tree: &Tree<Tile>, placements: &PlacementMap) -> RenderList {
        tree.root()
            .descendants()
            .filter_map(|node| {
                placements.get(&node.id()).map(|placement| RenderListNode {
                    tile: *node.value(),
                    position: placement.position,
                    rotation: std::f32::consts::FRAC_PI_2 * i8::from(placement.rotation) as f32,
                    size: TILE_SIZE,
                })
            })
            .collect()
    }

    // Computes the bounding rectangle that contains all tiles.
    fn compute_bounds(placements: &PlacementMap) -> Rectangle {
        if placements.is_empty() {
            return Rectangle::new(Point::ORIGIN, TILE_SIZE);
        }

        let (min_x, max_x, min_y, max_y) = placements.values().fold(
            (f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY, f32::NEG_INFINITY),
            |(min_x, max_x, min_y, max_y), placement| {
                let (half_w, half_h) = match placement.rotation {
                    TileSide::Right | TileSide::Left => (TILE_SIZE.width / 2.0, TILE_SIZE.height / 2.0),
                    TileSide::Top | TileSide::Bottom => (TILE_SIZE.height / 2.0, TILE_SIZE.width / 2.0),
                };

                (
                    min_x.min(placement.position.x - half_w),
                    max_x.max(placement.position.x + half_w),
                    min_y.min(placement.position.y - half_h),
                    max_y.max(placement.position.y + half_h),
                )
            },
        );

        Rectangle {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

// Helper functions

// Returns the offset to an attachment point from the center of a tile.
fn attachment_offset(side: TileSide) -> Vector {
    match side {
        TileSide::Right => Vector::new(TILE_OFFSET_X, 0.0),
        TileSide::Top => Vector::new(0.0, -TILE_OFFSET_Y),
        TileSide::Left => Vector::new(-TILE_OFFSET_X, 0.0),
        TileSide::Bottom => Vector::new(0.0, TILE_OFFSET_Y),
    }
}

// Determines which attachment point to use on the parent tile.
fn get_parent_attachment(parent_tuple: &(u8, u8), value: u8, attachments: &mut Vec<TileSide>) -> TileSide {
    if rules::is_double_tuple(*parent_tuple) {
        // Any open attachment can be used
        attachments.pop().expect("Expected an open end")
    } else if value == parent_tuple.0 {
        // Attach to Top
        debug_assert_eq!(attachments.first(), Some(&TileSide::Top));
        attachments.swap_remove(0) // Top will always be the first element
    } else if value == parent_tuple.1 {
        // Attach to Bottom
        debug_assert_eq!(attachments.last(), Some(&TileSide::Bottom));
        attachments.pop().expect("Expected an open end") // Bottom will always be the last element
    } else {
        panic!("Value {value} does not match any side of parent {parent_tuple:?}");
    }
}

// Determines which attachment point to use on the child tile.
fn get_child_attachment(child_tuple: &(u8, u8), value: u8, attachments: &mut Vec<TileSide>) -> TileSide {
    if rules::is_double_tuple(*child_tuple) {
        // Double tile always attaches to the Left side and doesn't use any pre-existing attachments
        TileSide::Left
    } else if child_tuple.0 == value {
        debug_assert_eq!(attachments.first(), Some(&TileSide::Top));
        attachments.swap_remove(0) // Top will always be the first element
    } else if child_tuple.1 == value {
        debug_assert_eq!(attachments.last(), Some(&TileSide::Bottom));
        attachments.pop().expect("Expected an open end") // Bottom will always be the last element
    } else {
        panic!("Tile {child_tuple:?} does not match any side of parent at {value}");
    }
}

// Calculates the rotation needed to align a child's attachment with a parent's attachment.
fn child_attachment_to_parent_rotation(child: TileSide, parent: TileSide) -> TileSide {
    // To connect, child's attachment must face opposite direction of parent's attachment
    // Formula: (parent + 2 - child) % 4 gives the rotation needed
    TileSide::from((i8::from(parent) + 2 - i8::from(child)) % 4)
}

// Rotates a vector by the specified rotation.
fn rotated_vector(vector: &Vector, side: TileSide) -> Vector {
    match side {
        TileSide::Right => *vector,
        TileSide::Bottom => Vector::new(-vector.y, vector.x),
        TileSide::Left => Vector::new(-vector.x, -vector.y),
        TileSide::Top => Vector::new(vector.y, -vector.x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_side_rotation() {
        assert_eq!(TileSide::Right.rotate(TileSide::Right), TileSide::Right);
        assert_eq!(TileSide::Right.rotate(TileSide::Bottom), TileSide::Bottom);
        assert_eq!(TileSide::Right.rotate(TileSide::Left), TileSide::Left);
        assert_eq!(TileSide::Right.rotate(TileSide::Top), TileSide::Top);

        assert_eq!(TileSide::Bottom.rotate(TileSide::Bottom), TileSide::Left);
        assert_eq!(TileSide::Left.rotate(TileSide::Bottom), TileSide::Top);
        assert_eq!(TileSide::Top.rotate(TileSide::Bottom), TileSide::Right);
    }

    #[test]
    fn test_tile_side_from_i8() {
        assert_eq!(TileSide::from(0), TileSide::Right);
        assert_eq!(TileSide::from(1), TileSide::Bottom);
        assert_eq!(TileSide::from(2), TileSide::Left);
        assert_eq!(TileSide::from(3), TileSide::Top);
    }

    #[test]
    fn test_tile_side_to_i8() {
        assert_eq!(i8::from(TileSide::Right), 0);
        assert_eq!(i8::from(TileSide::Bottom), 1);
        assert_eq!(i8::from(TileSide::Left), 2);
        assert_eq!(i8::from(TileSide::Top), 3);
    }

    #[test]
    #[should_panic(expected = "Invalid value for TileSide: 4")]
    fn test_tile_side_from_invalid_i8() {
        let _ = TileSide::from(4);
    }

    #[test]
    fn test_attachment_offset() {
        assert_eq!(attachment_offset(TileSide::Right), Vector::new(TILE_OFFSET_X, 0.0));
        assert_eq!(attachment_offset(TileSide::Top), Vector::new(0.0, -TILE_OFFSET_Y));
        assert_eq!(attachment_offset(TileSide::Left), Vector::new(-TILE_OFFSET_X, 0.0));
        assert_eq!(attachment_offset(TileSide::Bottom), Vector::new(0.0, TILE_OFFSET_Y));
    }

    #[test]
    fn test_rotated_vector() {
        let vector = Vector::new(10.0, 20.0);

        assert_eq!(rotated_vector(&vector, TileSide::Right), Vector::new(10.0, 20.0));
        assert_eq!(rotated_vector(&vector, TileSide::Bottom), Vector::new(-20.0, 10.0));
        assert_eq!(rotated_vector(&vector, TileSide::Left), Vector::new(-10.0, -20.0));
        assert_eq!(rotated_vector(&vector, TileSide::Top), Vector::new(20.0, -10.0));
    }

    #[test]
    fn test_child_attachment_to_parent_rotation() {
        assert_eq!(child_attachment_to_parent_rotation(TileSide::Left, TileSide::Right), TileSide::Right);
        assert_eq!(child_attachment_to_parent_rotation(TileSide::Right, TileSide::Right), TileSide::Left);
        assert_eq!(child_attachment_to_parent_rotation(TileSide::Top, TileSide::Bottom), TileSide::Right);
        assert_eq!(child_attachment_to_parent_rotation(TileSide::Bottom, TileSide::Top), TileSide::Right);
    }

    #[test]
    fn test_get_parent_attachment_non_double() {
        let mut attachments = vec![TileSide::Top, TileSide::Bottom];
        let parent_tuple = (1, 2);

        // Attach to value 1 (top)
        let result = get_parent_attachment(&parent_tuple, 1, &mut attachments);
        assert_eq!(result, TileSide::Top);
        assert_eq!(attachments, vec![TileSide::Bottom]);

        // Reset and attach to value 2 (bottom)
        attachments = vec![TileSide::Top, TileSide::Bottom];
        let result = get_parent_attachment(&parent_tuple, 2, &mut attachments);
        assert_eq!(result, TileSide::Bottom);
        assert_eq!(attachments, vec![TileSide::Top]);
    }

    #[test]
    fn test_get_child_attachment_double() {
        let mut attachments = vec![TileSide::Top, TileSide::Bottom];
        let child_tuple = (3, 3);

        let result = get_child_attachment(&child_tuple, 3, &mut attachments);
        assert_eq!(result, TileSide::Left);
        // Attachments should be unchanged for double tiles
        assert_eq!(attachments, vec![TileSide::Top, TileSide::Bottom]);
    }

    #[test]
    fn test_scene_graph_new() {
        let tree = Tree::new(Tile::from((0, 0)));
        let scene_graph = SceneGraph::new(&tree);

        // After construction, tiles should be laid out
        assert!(!scene_graph.bounds().width.is_nan());
        assert_eq!(scene_graph.render_list().len(), 1);
    }
}
