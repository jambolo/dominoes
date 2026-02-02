//! Main Iced application for the Dominoes game

use crate::game_history::GameHistory;
use crate::io::{self, IoError};
use crate::scene_graph::{RenderListNode, SceneGraph};
use dominoes_state::{Action, DominoesState};
use iced::widget::canvas::{self, Canvas, Geometry};
use iced::widget::{button, column, container, row, text, Row};
use iced::{Element, Length, Point, Rectangle, Task, Vector};
use player::{HumanPlayer, Player};
use rules::{Configuration, Tile, Variation};
use std::collections::HashMap;
use std::path::Path;

/// Turn phase state machine
#[derive(Debug, Clone)]
pub enum TurnPhase {
    /// Player has playable tiles - waiting for tile selection
    SelectTile {
        /// Indices of playable tiles in hand
        playable_indices: Vec<usize>,
        /// All open ends that can be played on
        valid_ends: Vec<u8>,
    },
    /// Tile selected - waiting for end selection
    SelectEnd {
        /// Index of selected tile in hand
        tile_index: usize,
        /// Ends this tile can play on
        valid_ends: Vec<u8>,
    },
    /// Player has no playable tiles - must draw from boneyard
    MustDraw,
    /// Boneyard empty, no playable tiles - must pass
    MustPass,
    /// Game is over
    GameOver,
}

/// UI messages
#[derive(Debug, Clone)]
pub enum Message {
    /// Tile clicked in current player's hand
    TileClicked(usize),
    /// Open end clicked in layout
    EndClicked(u8),
    /// Draw from boneyard
    BoneyardClicked,
    /// Pass turn
    Pass,
    /// Undo last action
    Undo,
    /// Redo previously undone action
    Redo,
    /// Start a new game
    NewGame,
    /// Export game history
    Export,
    /// Export current state only
    ExportState,
    /// Import game history
    Import,
    /// Import state only
    ImportState,
    /// File dialog result for export
    ExportComplete(Result<(), String>),
    /// File dialog result for import
    ImportComplete(Result<String, String>),
}

/// Main Dominoes application
pub struct DominoesApp {
    /// Game configuration
    config: Configuration,
    /// Current game state
    state: DominoesState,
    /// Game history for undo/redo
    history: GameHistory,
    /// Players
    players: [HumanPlayer; 2],
    /// Current turn phase
    turn_phase: TurnPhase,
    /// Error/status message to display
    status_message: Option<String>,
    /// Tile images
    tile_images: HashMap<u8, iced::widget::image::Handle>,
}

impl DominoesApp {
    /// Creates a new DominoesApp with the given configuration
    pub fn new(config: Configuration) -> Self {
        let mut state = DominoesState::new(&config);

        // Draw starting hands for both players
        let hand_size = config.starting_hand_size();
        for player_id in 0..2 {
            for _ in 0..hand_size {
                if let Some(tile) = state.draw_tile() {
                    state.hands[player_id].add_tile(tile);
                }
            }
        }

        let history = GameHistory::new(state.clone());
        let players = [
            HumanPlayer::new(0, "Player 1"),
            HumanPlayer::new(1, "Player 2"),
        ];

        let turn_phase = Self::compute_turn_phase(&state);

        Self {
            config,
            state,
            history,
            players,
            turn_phase,
            status_message: None,
            tile_images: Self::load_tile_images(),
        }
    }

    /// Computes the turn phase based on current state
    fn compute_turn_phase(state: &DominoesState) -> TurnPhase {
        if state.game_is_over {
            return TurnPhase::GameOver;
        }

        let current_player = state.whose_turn as usize;
        let hand = &state.hands[current_player];

        // Find playable tiles
        let playable_indices: Vec<usize> = hand
            .tiles()
            .iter()
            .enumerate()
            .filter(|(_, tile)| state.can_play_tile(tile, None))
            .map(|(i, _)| i)
            .collect();

        if !playable_indices.is_empty() {
            // Collect valid ends
            let valid_ends: Vec<u8> = state
                .layout
                .end_counts
                .iter()
                .enumerate()
                .filter(|&(_, count)| *count > 0)
                .map(|(i, _)| i as u8)
                .collect();

            TurnPhase::SelectTile {
                playable_indices,
                valid_ends,
            }
        } else if state.boneyard.count() > 0 {
            TurnPhase::MustDraw
        } else {
            TurnPhase::MustPass
        }
    }

    /// Gets valid ends for a specific tile
    fn get_valid_ends_for_tile(state: &DominoesState, tile: &Tile) -> Vec<u8> {
        let (a, b) = tile.as_tuple();
        let mut ends = Vec::new();

        if state.layout.end_counts[a as usize] > 0 {
            ends.push(a);
        }
        if a != b && state.layout.end_counts[b as usize] > 0 {
            ends.push(b);
        }

        ends
    }

    /// Handles the update logic
    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.status_message = None;

        match message {
            Message::TileClicked(index) => {
                self.handle_tile_clicked(index);
            }
            Message::EndClicked(end) => {
                self.handle_end_clicked(end);
            }
            Message::BoneyardClicked => {
                self.handle_boneyard_clicked();
            }
            Message::Pass => {
                self.handle_pass();
            }
            Message::Undo => {
                self.handle_undo();
            }
            Message::Redo => {
                self.handle_redo();
            }
            Message::NewGame => {
                *self = Self::new(self.config.clone());
            }
            Message::Export => {
                return self.handle_export();
            }
            Message::ExportState => {
                return self.handle_export_state();
            }
            Message::Import => {
                return self.handle_import();
            }
            Message::ImportState => {
                return self.handle_import_state();
            }
            Message::ExportComplete(result) => {
                match result {
                    Ok(()) => self.status_message = Some("Exported successfully".to_string()),
                    Err(e) => self.status_message = Some(format!("Export failed: {}", e)),
                }
            }
            Message::ImportComplete(result) => {
                match result {
                    Ok(json) => {
                        if let Err(e) = self.load_from_json(&json) {
                            self.status_message = Some(format!("Import failed: {}", e));
                        } else {
                            self.status_message = Some("Imported successfully".to_string());
                        }
                    }
                    Err(e) => self.status_message = Some(format!("Import failed: {}", e)),
                }
            }
        }

        Task::none()
    }

    fn handle_tile_clicked(&mut self, index: usize) {
        let current_player = self.state.whose_turn as usize;
        let hand = &self.state.hands[current_player];

        if index >= hand.len() {
            return;
        }

        let tile = hand.tiles()[index];

        // Check if tile is playable
        if !self.state.can_play_tile(&tile, None) {
            self.status_message = Some("This tile cannot be played".to_string());
            return;
        }

        // If layout is empty and tile is a double, play it directly
        if self.state.layout.is_empty() && tile.is_double() {
            self.play_tile(index, None);
            return;
        }

        // Get valid ends for this tile
        let valid_ends = Self::get_valid_ends_for_tile(&self.state, &tile);

        if valid_ends.len() == 1 {
            // Only one valid end - play directly
            self.play_tile(index, Some(valid_ends[0]));
        } else if valid_ends.len() > 1 {
            // Multiple valid ends - go to SelectEnd phase
            self.turn_phase = TurnPhase::SelectEnd {
                tile_index: index,
                valid_ends,
            };
        }
    }

    fn handle_end_clicked(&mut self, end: u8) {
        if let TurnPhase::SelectEnd { tile_index, ref valid_ends } = self.turn_phase {
            if valid_ends.contains(&end) {
                self.play_tile(tile_index, Some(end));
            }
        }
    }

    fn handle_boneyard_clicked(&mut self) {
        if !matches!(self.turn_phase, TurnPhase::MustDraw) {
            return;
        }

        let current_player = self.state.whose_turn as usize;

        if let Some(tile) = self.state.draw_tile() {
            self.state.hands[current_player].add_tile(tile);
            let action = Action::draw(current_player as u8, tile);
            self.history.push(self.state.clone(), action);
            self.status_message = Some(format!("Drew {}", tile));

            // Recompute turn phase
            self.turn_phase = Self::compute_turn_phase(&self.state);
        }
    }

    fn handle_pass(&mut self) {
        if !matches!(self.turn_phase, TurnPhase::MustPass) {
            return;
        }

        let current_player = self.state.whose_turn;
        self.state.pass();
        self.advance_turn();

        let action = Action::pass(current_player);
        self.history.push(self.state.clone(), action);

        // Check for game end (all players passed)
        if self.state.consecutive_passes as usize >= self.config.num_players() {
            self.end_game();
        } else {
            self.turn_phase = Self::compute_turn_phase(&self.state);
        }
    }

    fn handle_undo(&mut self) {
        if let Some(state) = self.history.undo() {
            self.state = state.clone();
            self.turn_phase = Self::compute_turn_phase(&self.state);
            self.status_message = Some("Undone".to_string());
        }
    }

    fn handle_redo(&mut self) {
        if let Some(state) = self.history.redo() {
            self.state = state.clone();
            self.turn_phase = Self::compute_turn_phase(&self.state);
            self.status_message = Some("Redone".to_string());
        }
    }

    fn play_tile(&mut self, tile_index: usize, end: Option<u8>) {
        let current_player = self.state.whose_turn as usize;
        let tile = self.state.hands[current_player].tiles()[tile_index];

        self.state.hands[current_player].remove_tile(&tile);
        self.state.play_tile(tile, end);

        let action = Action::play(current_player as u8, tile, end);

        // Check for win (empty hand)
        if self.state.hands[current_player].is_empty() {
            self.state.mark_game_over(Some(current_player as u8));
            self.history.push(self.state.clone(), action);
            self.turn_phase = TurnPhase::GameOver;
            self.status_message = Some(format!("{} wins!", self.players[current_player].name()));
        } else {
            self.advance_turn();
            self.history.push(self.state.clone(), action);
            self.turn_phase = Self::compute_turn_phase(&self.state);
        }
    }

    fn advance_turn(&mut self) {
        self.state.whose_turn = (self.state.whose_turn + 1) % self.config.num_players() as u8;
    }

    fn end_game(&mut self) {
        // Find winner by lowest pip count
        let scores: Vec<u32> = self.state.hands.iter().map(|h| h.score()).collect();
        let min_score = *scores.iter().min().unwrap_or(&0);
        let winners: Vec<usize> = scores
            .iter()
            .enumerate()
            .filter(|&(_, s)| *s == min_score)
            .map(|(i, _)| i)
            .collect();

        if winners.len() == 1 {
            self.state.mark_game_over(Some(winners[0] as u8));
            self.status_message = Some(format!("{} wins!", self.players[winners[0]].name()));
        } else {
            self.state.mark_game_over(None);
            self.status_message = Some("Draw!".to_string());
        }

        self.turn_phase = TurnPhase::GameOver;
    }

    fn handle_export(&self) -> Task<Message> {
        let json = match io::export_history(&self.history) {
            Ok(j) => j,
            Err(e) => return Task::done(Message::ExportComplete(Err(e.to_string()))),
        };

        Task::perform(
            async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("dominoes_game.json")
                    .save_file()
                    .await;

                match file {
                    Some(handle) => {
                        match std::fs::write(handle.path(), &json) {
                            Ok(()) => Ok(()),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    None => Err("Cancelled".to_string()),
                }
            },
            Message::ExportComplete,
        )
    }

    fn handle_export_state(&self) -> Task<Message> {
        let json = match io::export_state(&self.state) {
            Ok(j) => j,
            Err(e) => return Task::done(Message::ExportComplete(Err(e.to_string()))),
        };

        Task::perform(
            async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("dominoes_state.json")
                    .save_file()
                    .await;

                match file {
                    Some(handle) => {
                        match std::fs::write(handle.path(), &json) {
                            Ok(()) => Ok(()),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    None => Err("Cancelled".to_string()),
                }
            },
            Message::ExportComplete,
        )
    }

    fn handle_import(&self) -> Task<Message> {
        Task::perform(
            async {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file()
                    .await;

                match file {
                    Some(handle) => {
                        match std::fs::read_to_string(handle.path()) {
                            Ok(json) => Ok(json),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    None => Err("Cancelled".to_string()),
                }
            },
            Message::ImportComplete,
        )
    }

    fn handle_import_state(&self) -> Task<Message> {
        self.handle_import()
    }

    fn load_from_json(&mut self, json: &str) -> Result<(), IoError> {
        // Try to load as history first, then as state
        if let Ok(history) = io::import_history(json) {
            self.history = history;
            self.state = self.history.current_state().clone();
            self.turn_phase = Self::compute_turn_phase(&self.state);
            return Ok(());
        }

        if let Ok(state) = io::import_state(json) {
            self.state = state;
            self.history = GameHistory::new(self.state.clone());
            self.turn_phase = Self::compute_turn_phase(&self.state);
            return Ok(());
        }

        Err(IoError::InvalidData("Could not parse as history or state".to_string()))
    }

    /// Creates the view
    pub fn view(&self) -> Element<'_, Message> {
        let current_player = self.state.whose_turn as usize;

        // Toolbar
        let toolbar = self.view_toolbar();

        // Status bar
        let status_bar = self.view_status_bar();

        // Layout canvas
        let layout_view = self.view_layout();

        // Current player's hand
        let hand_view = self.view_hand(current_player);

        // Assemble the UI
        let content = column![
            toolbar,
            layout_view,
            status_bar,
            hand_view,
        ]
        .spacing(10)
        .padding(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        let undo_btn = button("Undo")
            .on_press_maybe(self.history.can_undo().then_some(Message::Undo));
        let redo_btn = button("Redo")
            .on_press_maybe(self.history.can_redo().then_some(Message::Redo));
        let new_game_btn = button("New Game").on_press(Message::NewGame);
        let export_btn = button("Export").on_press(Message::Export);
        let import_btn = button("Import").on_press(Message::Import);

        row![undo_btn, redo_btn, new_game_btn, export_btn, import_btn]
            .spacing(10)
            .into()
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        let current_player = self.state.whose_turn as usize;
        let player_name = self.players[current_player].name();

        let turn_text = if self.state.game_is_over {
            "Game Over".to_string()
        } else {
            format!("Turn: {}", player_name)
        };

        let boneyard_text = format!("Boneyard: {}", self.state.boneyard.count());

        let phase_text = match &self.turn_phase {
            TurnPhase::SelectTile { .. } => "Select a tile to play",
            TurnPhase::SelectEnd { .. } => "Select which end to play on",
            TurnPhase::MustDraw => "Click boneyard to draw",
            TurnPhase::MustPass => "No moves - must pass",
            TurnPhase::GameOver => "",
        };

        let status_text = self.status_message.as_deref().unwrap_or(phase_text);

        let pass_btn = button("Pass")
            .on_press_maybe(matches!(self.turn_phase, TurnPhase::MustPass).then_some(Message::Pass));

        let draw_btn = button("Draw")
            .on_press_maybe(matches!(self.turn_phase, TurnPhase::MustDraw).then_some(Message::BoneyardClicked));

        row![
            text(turn_text),
            text(" | "),
            text(boneyard_text),
            text(" | "),
            text(status_text),
            draw_btn,
            pass_btn,
        ]
        .spacing(10)
        .into()
    }

    fn view_layout(&self) -> Element<'_, Message> {
        if self.state.layout.is_empty() {
            return container(text("Play a double tile to start"))
                .width(Length::Fill)
                .height(Length::FillPortion(2))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        // Create scene graph from layout
        if let Some(tree) = self.state.layout.to_tree() {
            let scene_graph = SceneGraph::new(&tree);
            let canvas_state = LayoutCanvas {
                scene_graph,
                tile_images: &self.tile_images,
            };

            Canvas::new(canvas_state)
                .width(Length::Fill)
                .height(Length::FillPortion(2))
                .into()
        } else {
            container(text("Layout error"))
                .width(Length::Fill)
                .height(Length::FillPortion(2))
                .into()
        }
    }

    fn view_hand(&self, player_id: usize) -> Element<'_, Message> {
        let hand = &self.state.hands[player_id];
        let is_current_player = player_id == self.state.whose_turn as usize;

        let playable_indices = match &self.turn_phase {
            TurnPhase::SelectTile { playable_indices, .. } if is_current_player => {
                playable_indices.clone()
            }
            _ => Vec::new(),
        };

        let selected_index = match &self.turn_phase {
            TurnPhase::SelectEnd { tile_index, .. } if is_current_player => Some(*tile_index),
            _ => None,
        };

        let tile_buttons: Vec<Element<Message>> = hand
            .tiles()
            .iter()
            .enumerate()
            .map(|(i, tile)| {
                let is_playable = playable_indices.contains(&i);
                let is_selected = selected_index == Some(i);

                let label = format!("{}|{}", tile.as_tuple().0, tile.as_tuple().1);

                let btn = if is_selected {
                    button(text(label).size(20))
                        .style(button::primary)
                        .on_press(Message::TileClicked(i))
                } else if is_playable && is_current_player {
                    button(text(label).size(20))
                        .style(button::success)
                        .on_press(Message::TileClicked(i))
                } else {
                    button(text(label).size(20))
                        .style(button::secondary)
                };

                btn.into()
            })
            .collect();

        let hand_row = Row::with_children(tile_buttons).spacing(5);

        let player_name = self.players[player_id].name();
        let header = text(format!("{}'s Hand ({} tiles)", player_name, hand.len()));

        column![header, hand_row]
            .spacing(5)
            .into()
    }

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
}

impl Default for DominoesApp {
    fn default() -> Self {
        let config = Configuration::new(2, Variation::Traditional, 6, 7);
        Self::new(config)
    }
}

/// Canvas state for rendering the layout
struct LayoutCanvas<'a> {
    scene_graph: SceneGraph,
    tile_images: &'a HashMap<u8, iced::widget::image::Handle>,
}

impl<'a> canvas::Program<Message> for LayoutCanvas<'a> {
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

impl<'a> LayoutCanvas<'a> {
    const MARGIN: f32 = 0.1;
    const MAX_TILE_SIZE: f32 = 0.125;

    fn calculate_scale(&self, bounds: Rectangle) -> f32 {
        let scene_bounds = self.scene_graph.bounds();

        let content_scale = {
            let scale_x = (bounds.width * (1.0 - Self::MARGIN)) / scene_bounds.width;
            let scale_y = (bounds.height * (1.0 - Self::MARGIN)) / scene_bounds.height;
            scale_x.min(scale_y)
        };

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

    fn calculate_offset(&self, bounds: Rectangle, scale: f32) -> Vector {
        let scene_bounds = self.scene_graph.bounds();
        let window_center = Vector::new(bounds.width / 2.0, bounds.height / 2.0);
        let content_center = Vector::new(
            (scene_bounds.x + scene_bounds.width / 2.0) * scale,
            (scene_bounds.y + scene_bounds.height / 2.0) * scale,
        );
        window_center - content_center
    }

    fn render_tile(&self, frame: &mut canvas::Frame, node: &RenderListNode) {
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
