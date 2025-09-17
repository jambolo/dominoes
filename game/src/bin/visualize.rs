//! A visualizer for a Dominoes game layout
//!
//! This executable provides functionality to visualize a layout in text format.
//!
//! # Command Line Syntax
//!
//! ```bash
//! visualize [OPTIONS] <LAYOUT>
//! ```
//!
//! ## Arguments
//! * `<LAYOUT>` - The layout string to visualize (see syntax below)
//!
//! ## Options
//! * `-j, --json` - Print the layout as JSON to stdout instead of displaying graphically
//! * `-h, --help` - Print help information
//! * `-V, --version` - Print version information
//!
//! # Example Usage
//!
//! Display a simple domino layout graphically:
//! ```bash
//! visualize "3|3=(3|4-4|5,3|6)"
//! ```
//!
//! Export the same layout as JSON:
//! ```bash
//! visualize --json "3|3=(3|4-4|5,3|6)"
//! ```

use clap::{Arg, Command as ClapCommand};
use ego_tree::{Tree, NodeId};
use iced::{
    widget::canvas::{self, Canvas, Frame, Geometry},
    Element, Point, Rectangle, Size, Task, Vector,
};
use std::collections::HashMap;
use std::path::Path;

use game::layout_parser::parse;
use rules::{self, is_double_tuple, Tile};

fn main() -> iced::Result {
    let matches = ClapCommand::new("Dominoes Visualizer")
        .version("1.0")
        .author("Jambolo <jambolo@users.noreply.github.com>")
        .arg(
            Arg::new("layout")
                .value_name("LAYOUT")
                .help("Specifies the layout to visualize")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .short('j')
                .help("Print the layout as JSON to stdout instead of displaying graphically")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let layout = matches.get_one::<String>("layout").unwrap();
    let json_output = matches.get_flag("json");

    let tree = parse(layout).expect("Failed to parse layout");

    if json_output {
        println!("{}", serde_json::to_string(&tree).expect("Failed to serialize to JSON"));
        Ok(())
    } else {
        iced::application("Dominoes Layout Visualizer", VisualizerApp::update, VisualizerApp::view)
            .run_with(move || (VisualizerApp::new(&tree), Task::none()))
    }
}

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
    ///
    /// # Example
    /// ```
    /// let side = TileSide::Right;
    /// let rotated = side.rotate(TileSide::Bottom); // 90° clockwise
    /// assert_eq!(rotated, TileSide::Bottom);
    /// ```
    fn rotate(self, rotation: TileSide) -> Self {
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

/// Placement information for a domino.
#[derive(Debug, Clone)]
struct Placement {
    /// The center position of the tile in world coordinates
    position: Vector,
    /// The rotation of the tile from its default orientation
    rotation: TileSide,
    /// Available attachment points for connecting child tiles
    attachments: Vec<TileSide>,
}

impl Placement {
    /// Creates a placement for the root tile at the origin.
    ///
    /// The root tile has no parent to attach to and is placed at (0,0) with top/bottom attachments available.
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
    /// * `parent_placement` - Mutable reference to parent's placement (attachments will be modified)
    /// * `size` - The size of tile
    /// * `spacing` - The spacing between tiles
    ///
    /// # Returns
    /// A new placement for the child tile
    fn new_child(child_tile: &Tile,parent_tile: &Tile,parent_placement: &mut Placement,size: Size,spacing: f32,) -> Self {
        let parent_tuple = parent_tile.as_tuple();
        let (attach_value, _) = child_tile
            .matches(parent_tile)
            .expect("Child tile must match parent tile");

        let parent_attachment =
            get_parent_attachment(&parent_tuple, attach_value, &mut parent_placement.attachments);
        let parent_attachment_offset = attachment_offset(parent_attachment, size, spacing);

        // Get the rotated child's position relative to the unrotated parent and then rotate by the parent's rotation
        // and then translate from the parent's position.
        let child_tuple = child_tile.as_tuple();
        let mut child_attachments = vec![TileSide::Top, TileSide::Bottom]; // Order is always Top then Bottom
        let child_attachment = get_child_attachment(&child_tuple, attach_value, &mut child_attachments);
        let child_rotation = child_attachment_to_parent_rotation(child_attachment, parent_attachment);
        let child_attachment_offset = attachment_offset(child_attachment, size, spacing);

        let child_position = rotated_vector(
            &(parent_attachment_offset - rotated_vector(&child_attachment_offset, child_rotation)),
            parent_placement.rotation,
        ) + parent_placement.position;

        Self {
            position: child_position,
            rotation: child_rotation.rotate(parent_placement.rotation),
            attachments: child_attachments,
        }
    }
}

/// The main visualizer application that renders domino layouts.
///
/// Manages the domino tree, tile images, placements, and rendering state.
struct VisualizerApp {
    tree: Tree<Tile>,
    handles: HashMap<u8, iced::widget::image::Handle>,
    placements: HashMap<NodeId, Placement>,
    tile_size: Size,
    bounds: Rectangle,
}

impl VisualizerApp {
    const TILE_WIDTH: f32 = 640.0;
    const TILE_HEIGHT: f32 = 1280.0;
    const SPACING: f32 = 4.0;
    const MARGIN_FACTOR: f32 = 0.8;

    /// Creates a new VisualizerApp for the given domino tree.
    ///
    /// # Arguments
    /// * `tree` - The domino tree to visualize
    ///
    /// # Returns
    /// A new VisualizerApp instance
    fn new(tree: &Tree<Tile>) -> Self {
        let tile_size = Size::new(Self::TILE_WIDTH, Self::TILE_HEIGHT);
        let handles = Self::load_images();
        let placements = Self::compute_placements(tree, tile_size, Self::SPACING);
        let bounds = Self::compute_bounds(&placements, tile_size);

        Self {
            tree: tree.clone(),
            handles,
            placements,
            tile_size,
            bounds,
        }
    }

    /// Updates the visualizer state (no-op for static visualization).
    fn update(&mut self, _message: ()) {
        // No updates needed for static visualization
    }

    /// Creates the view element for the visualizer.
    fn view(&self) -> Element<()> {
        Canvas::new(self).width(iced::Length::Fill).height(iced::Length::Fill).into()
    }

    /// Loads domino images for all tiles in a standard double-six set.
    ///
    /// Images are expected to be in the `assets/` directory with names like `domino-01.png`.
    ///
    /// # Returns
    /// A hashmap mapping tile ordinals to image handles
    fn load_images() -> HashMap<u8, iced::widget::image::Handle> {
        (0..rules::set_size(6))
            .filter_map(|i| {
                let (a, b) = rules::ordinal_to_tuple(i as u8);
                let image_path = format!("assets/domino-{a}{b}.png");

                if Path::new(&image_path).exists() {
                    Some((i as u8, iced::widget::image::Handle::from_path(image_path)))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Computes the placement information for all tiles in the tree.
    ///
    /// # Arguments
    /// * `tree` - The domino tree
    /// * `tile_size` - The size of each tile
    /// * `spacing` - The spacing between tiles
    ///
    /// # Returns
    /// A hashmap mapping node IDs to their placements
    fn compute_placements(
        tree: &Tree<Tile>,
        tile_size: Size,
        spacing: f32,
    ) -> HashMap<NodeId, Placement> {
        tree.root()
            .descendants()
            .fold(HashMap::new(), |mut placements, node| {
                let placement = match node.parent() {
                    Some(parent) => {
                        let parent_placement = placements
                            .get_mut(&parent.id())
                            .expect("Parent placement must exist");
                        Placement::new_child(
                            node.value(),
                            parent.value(),
                            parent_placement,
                            tile_size,
                            spacing,
                        )
                    }
                    None => Placement::new_root(),
                };
                placements.insert(node.id(), placement);
                placements
            })
    }

    /// Computes the bounding rectangle that contains all tiles.
    ///
    /// # Arguments
    /// * `placements` - The tile placements
    /// * `tile_size` - The size of each tile
    ///
    /// # Returns
    /// A rectangle that bounds all tiles
    fn compute_bounds(placements: &HashMap<NodeId, Placement>, tile_size: Size) -> Rectangle {
        if placements.is_empty() {
            return Rectangle::new(Point::ORIGIN, tile_size);
        }

        let (min_x, max_x, min_y, max_y) = placements.values().fold(
            (f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY, f32::NEG_INFINITY),
            |(min_x, max_x, min_y, max_y), placement| {
                let (half_w, half_h) = match placement.rotation {
                    TileSide::Right | TileSide::Left => (tile_size.width / 2.0, tile_size.height / 2.0),
                    TileSide::Top | TileSide::Bottom => (tile_size.height / 2.0, tile_size.width / 2.0),
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

    /// Draws a single tile.
    ///
    /// # Arguments
    /// * `frame` - The canvas frame to draw on
    /// * `tile` - The tile to draw
    /// * `placement` - The placement information for the tile
    fn draw_tile(&self, frame: &mut Frame, tile: &Tile, placement: &Placement) {
        if let Some(handle) = self.handles.get(&tile.ordinal) {
            let ul_offset = Vector::new(-self.tile_size.width / 2.0, -self.tile_size.height / 2.0);
            let rotation = std::f32::consts::FRAC_PI_2 * i8::from(placement.rotation) as f32;
            frame.with_save(|frame| {
                frame.translate(placement.position);
                frame.rotate(rotation);
                frame.translate(ul_offset);

                frame.draw_image(Rectangle::new(Point::ORIGIN, self.tile_size), handle);
            });
        }
    }
}

impl canvas::Program<()> for VisualizerApp {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let scale = {
            let scale_x = (bounds.width * Self::MARGIN_FACTOR) / self.bounds.width;
            let scale_y = (bounds.height * Self::MARGIN_FACTOR) / self.bounds.height;
            scale_x.min(scale_y)
        };

        let window_center = Vector::new(bounds.width / 2.0, bounds.height / 2.0);
        let content_center = Vector::new(
            (self.bounds.x + self.bounds.width / 2.0) * scale,
            (self.bounds.y + self.bounds.height / 2.0) * scale,
        );
        let offset = window_center - content_center;

        frame.with_save(|frame| {
            frame.translate(offset);
            frame.scale(scale);

            for node in self.tree.root().descendants() {
                if let Some(placement) = self.placements.get(&node.id()) {
                    self.draw_tile(frame, node.value(), placement);
                }
            }
        });

        vec![frame.into_geometry()]
    }
}

impl Default for VisualizerApp {
    fn default() -> Self {
        let tree = Tree::new(Tile::from((0, 0)));   // There is always at least one tile
        Self::new(&tree)
    }
}

// Helper functions
/// Returns the offset to an attachment point from the center of a tile.
///
/// # Arguments
/// * `side` - The side of the tile
/// * `tile_size` - The size of the tile
/// * `spacing` - The spacing between tiles
///
/// # Returns
/// The offset as a vector
fn attachment_offset(side: TileSide, tile_size: Size, spacing: f32) -> Vector {
    let offset_x = tile_size.width / 2.0 + spacing;
    let offset_y = tile_size.height / 2.0 + spacing;

    match side {
        TileSide::Right => Vector::new(offset_x, 0.0),
        TileSide::Top => Vector::new(0.0, -offset_y),
        TileSide::Left => Vector::new(-offset_x, 0.0),
        TileSide::Bottom => Vector::new(0.0, offset_y),
    }
}

/// Determines which attachment point to use on the parent tile.
///
/// # Arguments
/// * `parent_tuple` - The parent tile's pip values
/// * `value` - The matching pip value
/// * `attachments` - Mutable reference to available attachments (will be modified)
///
/// # Important
/// This function removes the attachment point from the `attachments` vector.
///
/// # Returns
/// The attachment point to use on the parent
fn get_parent_attachment(parent_tuple: &(u8, u8), value: u8, attachments: &mut Vec<TileSide>) -> TileSide {
    if rules::is_double_tuple(*parent_tuple) {
        // Any open attachment can be used
        attachments.pop().expect("Expected an open end.")
    } else if value == parent_tuple.0 {
        // Attach to Top
        assert!(attachments.first() == Some(&TileSide::Top));
        attachments.swap_remove(0) // Top will always be the first element
    } else if value == parent_tuple.1 {
        // Attach to Bottom
        assert!(attachments.last() == Some(&TileSide::Bottom));
        attachments.pop().expect("Expected an open end.") // Bottom will always be the last element
    } else {
        panic!("Value {value} does not match any side of parent {parent_tuple:?}");
    }
}

/// Determines which attachment point to use on the child tile.
///
/// # Arguments
/// * `child_tuple` - The child tile's pip values
/// * `value` - The matching pip value
/// * `attachments` - Mutable reference to available attachments (will be modified)
///
/// # Important
/// This function removes the attachment point from the `attachments` vector, except for double tiles.
///
/// # Returns
/// The attachment point to use on the child
fn get_child_attachment(child_tuple: &(u8, u8), value: u8, attachments: &mut Vec<TileSide>) -> TileSide {
    if is_double_tuple(*child_tuple) {
        // Double tile always attaches to the Left side and doesn't use any pre-existing attachments
        TileSide::Left
    } else if child_tuple.0 == value {
        assert!(attachments.first() == Some(&TileSide::Top));
        attachments.swap_remove(0) // Top will always be the first element
    } else if child_tuple.1 == value {
        assert!(attachments.last() == Some(&TileSide::Bottom));
        attachments.pop().expect("Expected an open end.") // Bottom will always be the last element
    } else {
        panic!("Tile {child_tuple:?} does not match any side of parent {value}");
    }
}

/// Calculates the rotation needed to align a child's attachment with a parent's attachment.
///
/// # Arguments
/// * `child` - The child's attachment side
/// * `parent` - The parent's attachment side
///
/// # Returns
/// The rotation needed for the child (as a TileSide)
fn child_attachment_to_parent_rotation(child: TileSide, parent: TileSide) -> TileSide {
    // To connect, child's attachment must face opposite direction of parent's attachment
    // Formula: (parent + 2 - child) % 4 gives the rotation needed
    TileSide::from((i8::from(parent) + 2 - i8::from(child)) % 4)
}

/// Rotates a vector by the specified rotation.
///
/// # Arguments
/// * `vector` - The vector to rotate
/// * `side` - The rotation amount (as a TileSide)
///
/// # Returns
/// The rotated vector
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
    fn test_placement_new_root() {
        let placement = Placement::new_root();
        assert_eq!(placement.position, Vector::ZERO);
        assert_eq!(placement.rotation, TileSide::Top);
        assert_eq!(placement.attachments, vec![TileSide::Top, TileSide::Bottom]);
    }

    #[test]
    fn test_attachment_offset() {
        let tile_size = Size::new(100.0, 200.0);
        let spacing = 10.0;

        assert_eq!(attachment_offset(TileSide::Right, tile_size, spacing), Vector::new(60.0, 0.0));
        assert_eq!(attachment_offset(TileSide::Top, tile_size, spacing), Vector::new(0.0, -110.0));
        assert_eq!(attachment_offset(TileSide::Left, tile_size, spacing), Vector::new(-60.0, 0.0));
        assert_eq!(attachment_offset(TileSide::Bottom, tile_size, spacing), Vector::new(0.0, 110.0));
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
    fn test_visualizer_compute_bounds_empty() {
        let placements = HashMap::new();
        let tile_size = Size::new(100.0, 200.0);

        let bounds = VisualizerApp::compute_bounds(&placements, tile_size);
        assert_eq!(bounds, Rectangle::new(Point::ORIGIN, tile_size));
    }

    #[test]
    fn test_visualizer_compute_bounds_single_tile() {
        let mut placements = HashMap::new();

        // Create a dummy tree to get a valid NodeId
        let tree = Tree::new(Tile::from((0, 0)));
        let node_id = tree.root().id();

        let placement = Placement {
            position: Vector::new(0.0, 0.0),
            rotation: TileSide::Right,
            attachments: vec![],
        };
        placements.insert(node_id, placement);

        let tile_size = Size::new(100.0, 200.0);
        let bounds = VisualizerApp::compute_bounds(&placements, tile_size);

        assert_eq!(bounds.x, -50.0);
        assert_eq!(bounds.y, -100.0);
        assert_eq!(bounds.width, 100.0);
        assert_eq!(bounds.height, 200.0);
    }
}
