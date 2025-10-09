//! A visualizer for a Dominoes game layout
//!
//! This executable provides functionality to visualize a text layout.
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
use ego_tree::Tree;
use iced::{
    widget::canvas::{self, Canvas, Geometry},
    Element, Point, Rectangle, Task, Vector,
};
use std::collections::HashMap;
use std::path::Path;

use game::layout_parser::parse;
use game::scene_graph::SceneGraph;
use rules::{self, Tile};

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

    let layout = matches.get_one::<String>("layout").expect("Layout is required");
    let json_output = matches.get_flag("json");

    let tree = parse(layout).expect("Failed to parse layout");

    if json_output {
        let json = serde_json::to_string(&tree).expect("Failed to serialize to JSON");
        println!("{json}");
        Ok(())
    } else {
        iced::application("Dominoes Layout Visualizer", VisualizerApp::update, VisualizerApp::view)
            .run_with(move || (VisualizerApp::new(&tree), Task::none()))
    }
}

/// The main visualizer application that renders domino layouts.
///
/// Manages the domino tree, tile images, scene graph, and rendering state.
struct VisualizerApp {
    scene_graph: SceneGraph,
    tile_images: HashMap<u8, iced::widget::image::Handle>,
}

impl VisualizerApp {
    /// Empty space around content as a fraction of total size
    const MARGIN: f32 = 0.1;
    /// Maximum size of a tile relative to window size
    const MAX_TILE_SIZE: f32 = 0.125;

    /// Creates a new VisualizerApp for the given domino tree.
    ///
    /// # Arguments
    /// * `tree` - The domino tree to visualize
    ///
    /// # Returns
    /// A new VisualizerApp instance
    fn new(tree: &Tree<Tile>) -> Self {
        Self {
            scene_graph: SceneGraph::new(tree),
            tile_images: Self::load_tile_images(),
        }
    }

    /// Updates the visualizer state (no-op for static visualization).
    fn update(&mut self, _message: ()) {
        // No updates needed for static visualization
    }

    /// Creates the view element for the visualizer.
    fn view(&self) -> Element<()> {
        Canvas::new(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    /// Loads domino images for all tiles in a standard double-six set.
    ///
    /// Images are expected to be in the `assets/` directory with names like `domino-01.png`.
    ///
    /// # Returns
    /// A hashmap mapping tile ordinals to image handles
    fn load_tile_images() -> HashMap<u8, iced::widget::image::Handle> {
        (0..rules::set_size(6))
            .filter_map(|i| {
                let (a, b) = rules::ordinal_to_tuple(i as u8);
                let image_path = format!("assets/domino-{a}{b}.png");

                Path::new(&image_path).exists().then(|| {
                    (i as u8, iced::widget::image::Handle::from_path(image_path))
                })
            })
            .collect()
    }

    /// Calculates the scale factor for rendering the scene.
    ///
    /// # Arguments
    /// * `bounds` - The available rendering bounds
    ///
    /// # Returns
    /// The scale factor to apply
    fn calculate_scale(&self, bounds: Rectangle) -> f32 {
        let scene_bounds = self.scene_graph.bounds();

        let content_scale = {
            let scale_x = (bounds.width * (1.0 - Self::MARGIN)) / scene_bounds.width;
            let scale_y = (bounds.height * (1.0 - Self::MARGIN)) / scene_bounds.height;
            scale_x.min(scale_y)
        };

        // Limit scale based on actual tile dimensions from the scene graph
        self.scene_graph
            .render_list()
            .first()
            .map(|node| {
                let max_scale_x = bounds.width * Self::MAX_TILE_SIZE / node.size.width;
                let max_scale_y = bounds.height * Self::MAX_TILE_SIZE / node.size.height;
                content_scale.min(max_scale_x.min(max_scale_y))
            })
            .unwrap_or(content_scale)
    }

    /// Calculates the offset to center the scene in the viewport.
    ///
    /// # Arguments
    /// * `bounds` - The available rendering bounds
    /// * `scale` - The scale factor being applied
    ///
    /// # Returns
    /// The offset vector
    fn calculate_offset(&self, bounds: Rectangle, scale: f32) -> Vector {
        let scene_bounds = self.scene_graph.bounds();
        let window_center = Vector::new(bounds.width / 2.0, bounds.height / 2.0);
        let content_center = Vector::new(
            (scene_bounds.x + scene_bounds.width / 2.0) * scale,
            (scene_bounds.y + scene_bounds.height / 2.0) * scale,
        );
        window_center - content_center
    }

    /// Renders a single tile to the frame.
    ///
    /// # Arguments
    /// * `frame` - The canvas frame to draw on
    /// * `node` - The render list node containing tile information
    fn render_tile(&self, frame: &mut canvas::Frame, node: &game::scene_graph::RenderListNode) {
        if let Some(handle) = self.tile_images.get(&node.tile.ordinal) {
            let ul_offset = Vector::new(-node.size.width / 2.0, -node.size.height / 2.0);

            frame.with_save(|frame| {
                frame.translate(node.position);
                frame.rotate(node.rotation);
                frame.translate(ul_offset);
                frame.draw_image(Rectangle::new(Point::ORIGIN, node.size), handle);
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

        let scale = self.calculate_scale(bounds);
        let offset = self.calculate_offset(bounds, scale);

        frame.with_save(|frame| {
            frame.translate(offset);
            frame.scale(scale);

            for node in self.scene_graph.render_list() {
                self.render_tile(frame, node);
            }
        });

        vec![frame.into_geometry()]
    }
}

impl Default for VisualizerApp {
    fn default() -> Self {
        // There must always be at least one tile
        let tree = Tree::new(Tile::from((0, 0)));
        Self::new(&tree)
    }
}
