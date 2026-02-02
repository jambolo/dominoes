# Dominoes PvP Implementation Plan

## Overview

Refactor the dominoes game from Player vs. AI to Player vs. Player with an Iced-based GUI.

## Current State Issues

1. **Compilation errors** in `dominoes-state/src/dominoes_state.rs`:
   - `PlayerView` and `PublicView` reference `self.state.hands` but `hands` field is missing from `DominoesState`
   - `Boneyard` lacks `.len()` method (should use `.count()`)

2. **No GUI** - current game is console-based with stdin input

3. **No undo/redo** - `History` struct exists but not integrated

4. **No serialization** - serde dependencies exist but no derives on core types

---

## Phase 1: Fix Core State

### 1.1 Add `hands` field to DominoesState

**File:** `dominoes-state/src/dominoes_state.rs`

```rust
pub struct DominoesState {
    pub layout: Layout,
    pub boneyard: Boneyard,
    pub whose_turn: u8,
    pub fingerprint: ZHash,
    pub consecutive_passes: u8,
    pub game_is_over: bool,
    pub winner: Option<u8>,
    pub hands: Vec<Hand>,  // NEW: index = player_id
}
```

Update `DominoesState::new()` to:

- Accept `num_players` and `hand_size` parameters
- Initialize hands by drawing from boneyard

Fix `PlayerView` and `PublicView`:

- Change `boneyard.len()` to `boneyard.count()`

### 1.2 Add Serialization

Add `#[derive(Serialize, Deserialize)]` to:

- `DominoesState` in `dominoes-state/src/dominoes_state.rs`
- `Action` in `dominoes-state/src/action.rs`
- `History` in `dominoes-state/src/action.rs`
- `Hand` in `dominoes-state/src/hand.rs`

Verify `Tile`, `Layout`, `Boneyard` in `rules/` have serde derives.

### 1.3 Create Export Structures

**New file:** `dominoes-state/src/game_export.rs`

```rust
#[derive(Serialize, Deserialize)]
pub struct GameExport {
    pub version: String,
    pub actions: Vec<Action>,
    pub initial_state: DominoesState,
}

#[derive(Serialize, Deserialize)]
pub struct StateExport {
    pub version: String,
    pub state: DominoesState,
}
```

---

## Phase 2: Remove AI Code

### Files to Delete

- `player/src/dominoes_player.rs`
- `player/src/dominoes_response_generator.rs`
- `player/src/dominoes_rollout.rs`
- `player/src/dominoes_static_evaluator.rs`

### Update `player/src/lib.rs`

Remove AI module exports:

```rust
// REMOVE these lines:
pub mod dominoes_player;
pub mod dominoes_response_generator;
pub mod dominoes_rollout;
pub mod dominoes_static_evaluator;
pub use dominoes_player::*;
pub use dominoes_response_generator::*;
pub use dominoes_rollout::*;
pub use dominoes_static_evaluator::*;
```

### Simplify Player Trait

**File:** `player/src/player.rs`

Keep only:

```rust
pub trait Player {
    fn id(&self) -> u8;
    fn name(&self) -> &str;
}
```

Remove: `reset()`, `set_up()`, `my_turn()`, `has_playable_tile()`, `hand()`, `highest_double()`

These will be handled by the GUI and `DominoesState`.

### Simplify HumanPlayer

**File:** `player/src/human_player.rs`

Remove console I/O. Keep only:

```rust
pub struct HumanPlayer {
    id: u8,
    name: String,
}

impl HumanPlayer {
    pub fn new(id: u8, name: &str) -> Self {
        Self { id, name: name.to_string() }
    }
}

impl Player for HumanPlayer {
    fn id(&self) -> u8 { self.id }
    fn name(&self) -> &str { &self.name }
}
```

### Update Dependencies

**File:** `player/Cargo.toml`

Remove `game-player` dependency if no longer needed after AI removal.

---

## Phase 3: Implement Undo/Redo

### Create GameHistory

**New file:** `game/src/game_history.rs`

```rust
pub struct GameHistory {
    states: Vec<DominoesState>,
    actions: Vec<Action>,
    current_index: usize,
}

impl GameHistory {
    pub fn new(initial_state: DominoesState) -> Self;
    pub fn push(&mut self, state: DominoesState, action: Action);
    pub fn undo(&mut self) -> Option<&DominoesState>;
    pub fn redo(&mut self) -> Option<&DominoesState>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn current_state(&self) -> &DominoesState;
    pub fn actions(&self) -> &[Action];
}
```

**Strategy:** State snapshots (not action replay) for simplicity.

---

## Phase 4: Build Iced GUI

### Main Application

**New file:** `game/src/app.rs`

```rust
pub struct DominoesApp {
    // Game state
    state: DominoesState,
    history: GameHistory,
    players: [HumanPlayer; 2],

    // UI state
    turn_phase: TurnPhase,
    error_message: Option<String>,

    // Rendering
    scene_graph: SceneGraph,
    tile_images: HashMap<u8, Handle>,
}
```

### UI Layout

```
+--------------------------------------------------+
|  [Undo] [Redo]  |  [Export] [Import]             |
+--------------------------------------------------+
|                                                  |
|              Layout Canvas                       |
|    (shows open ends, highlights valid targets)   |
|                                                  |
+--------------------------------------------------+
|  Turn: Alice  |  Boneyard: 14  |  [Pass]         |
+--------------------------------------------------+
|                                                  |
|              Current Player's Hand               |
|     [ 1|2 ] [ 3|4 ] [ 5|5 ] [ 6|1 ] ...         |
|        ^highlighted if playable^                 |
+--------------------------------------------------+
```

### Turn Flow and UI State Machine

```rust
#[derive(Debug, Clone)]
pub enum TurnPhase {
    /// Player has playable tiles - waiting for tile selection
    SelectTile {
        playable_indices: Vec<usize>,  // Indices of playable tiles in hand
        valid_ends: Vec<u8>,           // All open ends that can be played on
    },
    /// Tile selected - waiting for end selection
    SelectEnd {
        tile_index: usize,
        valid_ends: Vec<u8>,           // Ends this tile can play on
    },
    /// Player has no playable tiles - must draw from boneyard
    MustDraw,
    /// Boneyard empty, no playable tiles - must pass
    MustPass,
}
```

**Turn Flow:**

1. **Start of Turn** - Compute `TurnPhase`:
   - Find all playable tiles in hand (tiles matching any open end)
   - If playable tiles exist → `SelectTile` phase
   - If no playable tiles and boneyard not empty → `MustDraw` phase
   - If no playable tiles and boneyard empty → `MustPass` phase

2. **SelectTile Phase:**
   - UI highlights all playable tiles in hand (green border)
   - UI highlights all valid open ends in layout (green glow)
   - Player clicks a tile → transition to `SelectEnd`
   - Non-playable tiles are dimmed/disabled

3. **SelectEnd Phase:**
   - UI highlights selected tile (blue border)
   - UI highlights only the ends this tile can play on (green glow)
   - Other ends are dimmed
   - Player clicks an end → tile is played, turn ends
   - Player can click another playable tile to change selection

4. **MustDraw Phase:**
   - All hand tiles dimmed (none playable)
   - Boneyard highlighted (pulsing glow)
   - Player clicks boneyard → draw tile, return to step 1
   - If drawn tile is playable → `SelectTile` phase
   - If drawn tile not playable and boneyard not empty → stay in `MustDraw`
   - If drawn tile not playable and boneyard empty → `MustPass` phase

5. **MustPass Phase:**
   - All hand tiles dimmed
   - Boneyard dimmed (empty)
   - Pass button highlighted
   - Player clicks Pass → turn ends

**Visual Indicators:**

| Element              | Normal       | Highlighted      | Dimmed           | Selected         |
|----------------------|--------------|------------------|------------------|------------------|
| Playable tile        | Normal       | Green border     | -                | Blue border      |
| Non-playable tile    | Gray overlay | -                | Gray overlay     | -                |
| Valid open end       | Normal       | Green glow       | -                | -                |
| Invalid open end     | Normal       | -                | Faded            | -                |
| Boneyard (has tiles) | Normal       | Pulsing glow     | -                | -                |
| Boneyard (empty)     | Faded        | -                | Faded            | -                |
| Pass button          | Normal       | Highlighted      | Disabled         | -                |

### Updated Message Types

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // Tile selection
    TileClicked(usize),        // Index in current player's hand
    EndClicked(u8),            // Open end value clicked in layout
    BoneyardClicked,           // Draw from boneyard

    // Actions
    Pass,

    // History
    Undo,
    Redo,

    // Import/Export
    Export,
    ExportState,
    Import,
    ImportState,

    // UI
    DismissMessage,
}
```

### Components

**New directory:** `game/src/components/`

- `mod.rs` - Module exports
- `layout_view.rs` - Canvas rendering (adapt from visualize.rs)
- `hand_view.rs` - Tile buttons for current player
- `status_bar.rs` - Turn info, boneyard count
- `toolbar.rs` - Undo/Redo/Import/Export buttons
- `end_selector.rs` - Modal for choosing which end to play on

### Reuse from visualize.rs

Copy rendering logic:

- `load_tile_images()` - Load tile PNG files
- `calculate_scale()` / `calculate_offset()` - Viewport calculations
- `render_tile()` - Individual tile drawing

Use existing:

- `game/src/scene_graph.rs` - Tile placement calculations
- `game/src/layout_parser.rs` - For import functionality

### Update main.rs

**File:** `game/src/main.rs`

Replace console game with:

```rust
fn main() -> iced::Result {
    iced::application("Dominoes", DominoesApp::update, DominoesApp::view)
        .run_with(|| {
            let config = Configuration::new(2, Variation::Traditional, 6, 7);
            (DominoesApp::new(config), Task::none())
        })
}
```

---

## Phase 5: Import/Export

### Export Functions

**New file:** `game/src/io.rs`

```rust
pub fn export_history(history: &GameHistory) -> Result<String, Error>;
pub fn export_state(state: &DominoesState) -> Result<String, Error>;
pub fn import_history(json: &str) -> Result<GameHistory, Error>;
pub fn import_state(json: &str) -> Result<DominoesState, Error>;
```

### File Dialogs

Add to `game/Cargo.toml`:

```toml
rfd = "0.14"  # Native file dialogs
```

---

## Phase 6: Comprehensive Testing

### 6.1 Unit Tests for DominoesState

**File:** `dominoes-state/src/dominoes_state.rs`

```rust
#[cfg(test)]
mod tests {
    // Test hands initialization
    fn test_new_initializes_hands_correctly();
    fn test_new_draws_correct_hand_size();
    fn test_new_reduces_boneyard_by_hand_sizes();

    // Test hand access
    fn test_player_view_returns_correct_hand();
    fn test_public_view_returns_hand_sizes();

    // Test play_tile updates hands
    fn test_play_tile_removes_from_hand();
    fn test_play_tile_panics_if_tile_not_in_hand();

    // Test draw_tile updates hands
    fn test_draw_tile_adds_to_current_player_hand();
    fn test_draw_tile_returns_none_when_boneyard_empty();

    // Test serialization round-trip
    fn test_serialize_deserialize_roundtrip();
    fn test_deserialize_invalid_json_returns_error();
}
```

### 6.2 Unit Tests for GameHistory

**File:** `game/src/game_history.rs`

```rust
#[cfg(test)]
mod tests {
    // Basic operations
    fn test_new_starts_at_index_zero();
    fn test_push_increments_index();
    fn test_current_state_returns_latest();

    // Undo
    fn test_undo_decrements_index();
    fn test_undo_returns_previous_state();
    fn test_undo_at_start_returns_none();
    fn test_can_undo_false_at_start();
    fn test_can_undo_true_after_push();

    // Redo
    fn test_redo_increments_index();
    fn test_redo_returns_next_state();
    fn test_redo_at_end_returns_none();
    fn test_can_redo_false_at_end();
    fn test_can_redo_true_after_undo();

    // Branch handling
    fn test_push_after_undo_truncates_future();
    fn test_redo_not_available_after_new_push();

    // Actions tracking
    fn test_actions_returns_all_actions_up_to_current();
    fn test_actions_empty_at_start();
}
```

### 6.3 Unit Tests for Import/Export

**File:** `game/src/io.rs`

```rust
#[cfg(test)]
mod tests {
    // Export
    fn test_export_history_produces_valid_json();
    fn test_export_state_produces_valid_json();
    fn test_export_includes_version();

    // Import
    fn test_import_history_restores_state();
    fn test_import_state_restores_state();
    fn test_import_invalid_json_returns_error();
    fn test_import_wrong_version_returns_error();
    fn test_import_missing_fields_returns_error();

    // Round-trip
    fn test_export_import_history_roundtrip();
    fn test_export_import_state_roundtrip();

    // Edge cases
    fn test_import_empty_history();
    fn test_import_history_with_many_actions();
    fn test_export_state_with_empty_boneyard();
}
```

### 6.4 Unit Tests for Player

**File:** `player/src/human_player.rs`

```rust
#[cfg(test)]
mod tests {
    fn test_new_sets_id();
    fn test_new_sets_name();
    fn test_id_returns_correct_value();
    fn test_name_returns_correct_value();
}
```

### 6.5 Integration Tests

**New file:** `game/tests/integration_tests.rs`

```rust
// Full game flow tests
fn test_complete_game_to_win();
fn test_complete_game_to_draw();
fn test_game_with_all_passes();

// Undo/redo integration
fn test_undo_redo_full_game();
fn test_undo_to_start_redo_to_end();

// Import/export integration
fn test_export_game_import_continue_play();
fn test_import_midgame_state_and_complete();

// Traditional variation rules
fn test_traditional_highest_double_starts();
fn test_traditional_first_move_must_be_double();
fn test_traditional_game_ends_when_hand_empty();
fn test_traditional_game_ends_when_all_pass();
fn test_traditional_winner_lowest_pip_count();
```

### 6.6 Test Utilities

**New file:** `dominoes-state/src/test_utils.rs`

```rust
/// Create a deterministic state for testing
pub fn create_test_state() -> DominoesState;

/// Create a state with specific hands
pub fn create_state_with_hands(hands: Vec<Vec<Tile>>) -> DominoesState;

/// Create a state with specific layout
pub fn create_state_with_layout(layout_str: &str) -> DominoesState;

/// Create a sequence of actions for testing
pub fn create_action_sequence(moves: &[(u8, Tile, Option<u8>)]) -> Vec<Action>;
```

**New file:** `game/src/test_utils.rs`

```rust
/// Create a GameHistory with predefined states
pub fn create_test_history(num_moves: usize) -> GameHistory;

/// Create a DominoesApp for testing (without GUI)
pub fn create_test_app() -> DominoesApp;
```

### 6.7 Property-Based Tests (Optional)

**File:** `dominoes-state/tests/property_tests.rs`

Using `proptest` crate:

```rust
proptest! {
    // State invariants
    fn test_hand_sizes_always_match_config(seed: u64);
    fn test_tile_count_invariant(actions: Vec<Action>);
    fn test_fingerprint_unique_for_different_states(s1: State, s2: State);

    // Serialization
    fn test_any_state_serializes_deserializes(state: DominoesState);

    // Undo/redo
    fn test_undo_redo_returns_to_same_state(actions: Vec<Action>);
}
```

### 6.8 Test Coverage Targets

| Module                     | Target Coverage |
|----------------------------|-----------------|
| `dominoes-state`           | 90%             |
| `game/game_history`        | 95%             |
| `game/io`                  | 90%             |
| `player`                   | 80%             |
| `game/app` (non-UI logic)  | 70%             |

### 6.9 Test Commands

```bash
# Run all tests
cargo test

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Run specific test module
cargo test --package dominoes-state
cargo test --package game game_history

# Run integration tests only
cargo test --test integration_tests

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

---

## Phase 7: Documentation Updates

### README.md

- Update to describe PvP gameplay
- Document UI controls
- Mark AI as "Future Plans"
- Add import/export format documentation

### CLAUDE.md

- Update build commands
- Mark AI features as future plans
- Document new file structure

---

## Files Summary

### New Files

|                  File                 | Purpose |
|---------------------------------------|----------------------------|
| `game/src/app.rs`                     | Main Iced application      |
| `game/src/game_history.rs`            | Undo/redo state management |
| `game/src/io.rs`                      | Import/export functions    |
| `game/src/components/mod.rs`          | Component module           |
| `game/src/components/layout_view.rs`  | Layout canvas              |
| `game/src/components/hand_view.rs`    | Player hand display        |
| `game/src/components/status_bar.rs`   | Game status                |
| `game/src/components/toolbar.rs`      | Action buttons             |
| `game/src/components/end_selector.rs` | End selection UI           |
| `dominoes-state/src/game_export.rs`   | Serialization structs      |

### Modified Files

|                  File                  |                   Changes                   |
|----------------------------------------|---------------------------------------------|
| `dominoes-state/src/dominoes_state.rs` | Add `hands` field, serde derives, fix views |
| `dominoes-state/src/action.rs`         | Add serde derives                           |
| `dominoes-state/src/hand.rs`           | Add serde derives                           |
| `dominoes-state/src/lib.rs`            | Export game_export module                   |
| `player/src/lib.rs`                    | Remove AI exports                           |
| `player/src/player.rs`                 | Simplify trait                              |
| `player/src/human_player.rs`           | Remove console I/O                          |
| `game/src/main.rs`                     | Launch Iced app                             |
| `game/src/lib.rs`                      | Export new modules                          |
| `game/Cargo.toml`                      | Add rfd dependency                          |
| `README.md`                            | Update documentation                        |
| `CLAUDE.md`                            | Update documentation                        |

### Deleted Files

|                     File                    |             Reason             |
|---------------------------------------------|--------------------------------|
| `player/src/dominoes_player.rs`             | AI implementation              |
| `player/src/dominoes_response_generator.rs` | AI support                     |
| `player/src/dominoes_rollout.rs`            | AI support                     |
| `player/src/dominoes_static_evaluator.rs`   | AI support                     |
| `game/src/dominoes_game.rs`                 | Console game (replaced by GUI) |

---

## Verification

### Build Verification

```bash
cargo build
cargo clippy
cargo fmt --check
```

### Test Verification

```bash
cargo test
```

### Manual Testing

1. Start new game
2. Play tiles by clicking hand then layout
3. Draw tiles when no playable moves
4. Pass turn
5. Test undo/redo
6. Export game, close, reimport
7. Verify game state preserved

---

## Import/Export Format

### History Export (game replay)

```json
{
  "version": "1.0",
  "initial_state": { ... },
  "actions": [
    {"player_id": 0, "tile_drawn": null, "tile_played": [[6,6], null]},
    {"player_id": 1, "tile_drawn": null, "tile_played": [[3,6], 6]}
  ]
}
```

### State Export (debugging)

```json
{
  "version": "1.0",
  "state": {
    "layout": "6|6=(3|6)",
    "boneyard_count": 14,
    "whose_turn": 0,
    "hands": [[...], [...]]
  }
}
```
