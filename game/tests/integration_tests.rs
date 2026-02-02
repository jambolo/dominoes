//! Integration tests for the dominoes game

use dominoes_state::{Action, DominoesState};
use game::game_history::GameHistory;
use game::io;
use rules::{Configuration, Tile, Variation};

fn create_test_state() -> DominoesState {
    let config = Configuration::new(2, Variation::Traditional, 6, 7);
    DominoesState::new(&config)
}

/// Helper to set up a state with specific hands
fn setup_game_with_hands(tiles_p0: &[(u8, u8)], tiles_p1: &[(u8, u8)]) -> DominoesState {
    let config = Configuration::new(2, Variation::Traditional, 6, 7);
    let mut state = DominoesState::new(&config);

    for &(a, b) in tiles_p0 {
        // Ensure canonical form (low, high)
        let tile = if a <= b { Tile::from((a, b)) } else { Tile::from((b, a)) };
        state.hands[0].add_tile(tile);
    }
    for &(a, b) in tiles_p1 {
        let tile = if a <= b { Tile::from((a, b)) } else { Tile::from((b, a)) };
        state.hands[1].add_tile(tile);
    }

    state
}

#[test]
fn test_complete_game_to_win() {
    // Set up a simple game where player 0 can win quickly
    // Using canonical form (5, 6) instead of (6, 5)
    let mut state = setup_game_with_hands(&[(6, 6), (5, 6)], &[(4, 5), (3, 4)]);

    // Player 0 plays 6|6
    let tile = Tile::from((6, 6));
    state.hands[0].remove_tile(&tile);
    state.play_tile(tile, None);

    // Advance turn
    state.whose_turn = 1;

    // Player 1 can't play (no 6)
    // For this test, skip player 1
    state.whose_turn = 0;

    // Player 0 plays 5|6
    let tile = Tile::from((5, 6));
    state.hands[0].remove_tile(&tile);
    state.play_tile(tile, Some(6));

    // Player 0's hand is empty - they win
    assert!(state.hands[0].is_empty());
}

#[test]
fn test_undo_redo_full_game() {
    let mut state = create_test_state();
    state.hands[0].add_tile(Tile::from((6, 6)));
    state.hands[0].add_tile(Tile::from((5, 6))); // Canonical form

    let mut history = GameHistory::new(state.clone());

    // Play first tile
    let tile = Tile::from((6, 6));
    state.hands[0].remove_tile(&tile);
    state.play_tile(tile, None);
    history.push(state.clone(), Action::play(0, tile, None));

    // Play second tile (5|6 in canonical form)
    let tile = Tile::from((5, 6));
    state.hands[0].remove_tile(&tile);
    state.play_tile(tile, Some(6));
    history.push(state.clone(), Action::play(0, tile, Some(6)));

    // Verify we can undo
    assert!(history.can_undo());

    // Undo both moves
    history.undo();
    history.undo();
    assert!(!history.can_undo());

    // Verify we're back at initial state
    assert!(history.current_state().layout.is_empty());

    // Redo to end
    history.redo();
    history.redo();
    assert!(!history.can_redo());

    // Verify final state
    assert!(!history.current_state().layout.is_empty());
}

#[test]
fn test_export_game_import_continue_play() {
    let mut state = create_test_state();
    state.hands[0].add_tile(Tile::from((6, 6)));

    let mut history = GameHistory::new(state.clone());

    // Play a tile
    let tile = Tile::from((6, 6));
    state.hands[0].remove_tile(&tile);
    state.play_tile(tile, None);
    history.push(state.clone(), Action::play(0, tile, None));

    // Export
    let json = io::export_history(&history).unwrap();

    // Import
    let imported = io::import_history(&json).unwrap();

    // Verify state is the same
    assert_eq!(imported.actions().len(), 1);
    assert!(!imported.current_state().layout.is_empty());
}

#[test]
fn test_state_roundtrip_preserves_hands() {
    let mut state = create_test_state();
    state.hands[0].add_tile(Tile::from((1, 2)));
    state.hands[0].add_tile(Tile::from((3, 4)));
    state.hands[1].add_tile(Tile::from((5, 6)));

    // Export and import
    let json = io::export_state(&state).unwrap();
    let imported = io::import_state(&json).unwrap();

    // Verify hands are preserved
    assert_eq!(imported.hands[0].len(), 2);
    assert_eq!(imported.hands[1].len(), 1);
    assert!(imported.hands[0].contains(&Tile::from((1, 2))));
    assert!(imported.hands[0].contains(&Tile::from((3, 4))));
    assert!(imported.hands[1].contains(&Tile::from((5, 6))));
}

#[test]
fn test_game_with_passes() {
    let config = Configuration::new(2, Variation::Traditional, 6, 7);
    let mut state = DominoesState::new(&config);

    // Play initial tile
    state.hands[0].add_tile(Tile::from((6, 6)));
    let tile = Tile::from((6, 6));
    state.hands[0].remove_tile(&tile);
    state.play_tile(tile, None);

    // Player 1 passes (simulating no valid moves and empty boneyard)
    state.whose_turn = 1;
    state.pass();

    assert_eq!(state.consecutive_passes, 1);

    // Player 0 passes
    state.whose_turn = 0;
    state.pass();

    assert_eq!(state.consecutive_passes, 2);
}

#[test]
fn test_traditional_first_move_must_be_double() {
    let config = Configuration::new(2, Variation::Traditional, 6, 7);
    let state = DominoesState::new(&config);

    // Non-double tiles cannot be played on empty layout
    let non_double = Tile::from((1, 2));
    assert!(!state.can_play_tile(&non_double, None));

    // Double tiles can be played on empty layout
    let double = Tile::from((6, 6));
    assert!(state.can_play_tile(&double, None));
}

#[test]
fn test_game_history_branch() {
    let mut state = create_test_state();
    state.hands[0].add_tile(Tile::from((6, 6)));
    state.hands[0].add_tile(Tile::from((5, 5)));

    let mut history = GameHistory::new(state.clone());

    // Play 6|6
    let tile1 = Tile::from((6, 6));
    state.hands[0].remove_tile(&tile1);
    state.play_tile(tile1, None);
    history.push(state.clone(), Action::play(0, tile1, None));

    // Undo
    history.undo();
    let state = history.current_state().clone();

    // Now play 5|5 instead (branch)
    let mut state = state;
    let tile2 = Tile::from((5, 5));
    state.hands[0].remove_tile(&tile2);
    state.play_tile(tile2, None);
    history.push(state.clone(), Action::play(0, tile2, None));

    // Verify history reflects the branch
    assert_eq!(history.actions().len(), 1);
    assert!(!history.can_redo());
}
