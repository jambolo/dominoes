//! Import/Export functionality for game state and history

use crate::game_history::GameHistory;
use dominoes_state::{Action, DominoesState};
use serde::{Deserialize, Serialize};
use std::io;

const VERSION: &str = "1.0";

/// Error type for import/export operations
#[derive(Debug)]
pub enum IoError {
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// File I/O error
    Io(io::Error),
    /// Version mismatch
    VersionMismatch { expected: String, found: String },
    /// Invalid data
    InvalidData(String),
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError::Json(e) => write!(f, "JSON error: {}", e),
            IoError::Io(e) => write!(f, "I/O error: {}", e),
            IoError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
            IoError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for IoError {}

impl From<serde_json::Error> for IoError {
    fn from(e: serde_json::Error) -> Self {
        IoError::Json(e)
    }
}

impl From<io::Error> for IoError {
    fn from(e: io::Error) -> Self {
        IoError::Io(e)
    }
}

/// Exported game history format
#[derive(Serialize, Deserialize)]
pub struct HistoryExport {
    /// Format version
    pub version: String,
    /// Initial game state
    pub initial_state: DominoesState,
    /// Actions taken during the game
    pub actions: Vec<Action>,
}

/// Exported state format
#[derive(Serialize, Deserialize)]
pub struct StateExport {
    /// Format version
    pub version: String,
    /// Current game state
    pub state: DominoesState,
}

/// Exports game history to JSON string
pub fn export_history(history: &GameHistory) -> Result<String, IoError> {
    let export = HistoryExport {
        version: VERSION.to_string(),
        initial_state: history.initial_state().clone(),
        actions: history.actions().to_vec(),
    };
    Ok(serde_json::to_string_pretty(&export)?)
}

/// Exports current state to JSON string
pub fn export_state(state: &DominoesState) -> Result<String, IoError> {
    let export = StateExport {
        version: VERSION.to_string(),
        state: state.clone(),
    };
    Ok(serde_json::to_string_pretty(&export)?)
}

/// Imports game history from JSON string
pub fn import_history(json: &str) -> Result<GameHistory, IoError> {
    let export: HistoryExport = serde_json::from_str(json)?;

    if export.version != VERSION {
        return Err(IoError::VersionMismatch {
            expected: VERSION.to_string(),
            found: export.version,
        });
    }

    let mut history = GameHistory::new(export.initial_state.clone());
    let mut current_state = export.initial_state;

    for action in export.actions {
        current_state = apply_action(&current_state, &action)?;
        history.push(current_state.clone(), action);
    }

    Ok(history)
}

/// Imports state from JSON string
pub fn import_state(json: &str) -> Result<DominoesState, IoError> {
    let export: StateExport = serde_json::from_str(json)?;

    if export.version != VERSION {
        return Err(IoError::VersionMismatch {
            expected: VERSION.to_string(),
            found: export.version,
        });
    }

    Ok(export.state)
}

/// Applies an action to a state, returning the new state
fn apply_action(state: &DominoesState, action: &Action) -> Result<DominoesState, IoError> {
    let mut new_state = state.clone();
    let player_id = action.player_id as usize;

    if let Some(tile) = action.tile_drawn {
        new_state.hands[player_id].add_tile(tile);
    }

    if let Some((tile, end)) = action.tile_played {
        new_state.hands[player_id].remove_tile(&tile);
        new_state.play_tile(tile, end);
    }

    if action.is_pass() {
        new_state.pass();
    }

    new_state.whose_turn = (new_state.whose_turn + 1) % new_state.hands.len() as u8;

    Ok(new_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rules::{Configuration, Tile, Variation};

    fn create_test_state() -> DominoesState {
        let config = Configuration::new(2, Variation::Traditional, 6, 7);
        DominoesState::new(&config)
    }

    #[test]
    fn test_export_state_produces_valid_json() {
        let state = create_test_state();
        let json = export_state(&state).unwrap();
        assert!(json.contains("\"version\""));
        assert!(json.contains("\"state\""));
    }

    #[test]
    fn test_export_includes_version() {
        let state = create_test_state();
        let json = export_state(&state).unwrap();
        assert!(json.contains(&format!("\"version\": \"{}\"", VERSION)));
    }

    #[test]
    fn test_import_state_restores_state() {
        let state = create_test_state();
        let json = export_state(&state).unwrap();
        let imported = import_state(&json).unwrap();
        assert_eq!(imported.whose_turn, state.whose_turn);
        assert_eq!(imported.game_is_over, state.game_is_over);
    }

    #[test]
    fn test_export_import_state_roundtrip() {
        let state = create_test_state();
        let json = export_state(&state).unwrap();
        let imported = import_state(&json).unwrap();

        let json2 = export_state(&imported).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn test_import_invalid_json_returns_error() {
        let result = import_state("not valid json");
        assert!(matches!(result, Err(IoError::Json(_))));
    }

    #[test]
    fn test_import_wrong_version_returns_error() {
        // Create a valid export, then modify version
        let state = create_test_state();
        let json = export_state(&state).unwrap();
        let json = json.replace(&format!("\"version\": \"{}\"", VERSION), "\"version\": \"99.0\"");
        let result = import_state(&json);
        assert!(matches!(result, Err(IoError::VersionMismatch { .. })));
    }

    #[test]
    fn test_export_history_produces_valid_json() {
        let state = create_test_state();
        let history = GameHistory::new(state);
        let json = export_history(&history).unwrap();
        assert!(json.contains("\"version\""));
        assert!(json.contains("\"initial_state\""));
        assert!(json.contains("\"actions\""));
    }

    #[test]
    fn test_import_empty_history() {
        let state = create_test_state();
        let history = GameHistory::new(state);
        let json = export_history(&history).unwrap();
        let imported = import_history(&json).unwrap();
        assert!(!imported.can_undo());
        assert!(!imported.can_redo());
    }

    #[test]
    fn test_export_import_history_roundtrip() {
        let mut state = create_test_state();
        state.hands[0].add_tile(Tile::from((6, 6)));
        let mut history = GameHistory::new(state.clone());

        let tile = Tile::from((6, 6));
        state.hands[0].remove_tile(&tile);
        state.play_tile(tile, None);
        let action = Action::play(0, tile, None);
        history.push(state.clone(), action);

        let json = export_history(&history).unwrap();
        let imported = import_history(&json).unwrap();

        assert!(imported.can_undo());
        assert_eq!(imported.actions().len(), 1);
    }

    #[test]
    fn test_io_error_display() {
        let err = IoError::InvalidData("test".to_string());
        assert!(err.to_string().contains("Invalid data"));

        let err = IoError::VersionMismatch {
            expected: "1.0".to_string(),
            found: "2.0".to_string(),
        };
        assert!(err.to_string().contains("Version mismatch"));
    }
}
