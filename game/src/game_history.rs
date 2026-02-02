//! Game history management for undo/redo functionality
//!
//! Uses state snapshots for simplicity rather than action replay.

use dominoes_state::{Action, DominoesState};

/// Manages game history with undo/redo support
#[derive(Debug, Clone)]
pub struct GameHistory {
    /// State snapshots at each point in history
    states: Vec<DominoesState>,
    /// Actions taken to reach each state (actions[i] transitions states[i] to states[i+1])
    actions: Vec<Action>,
    /// Current position in history (index into states)
    current_index: usize,
}

impl GameHistory {
    /// Creates a new history starting with the given initial state
    pub fn new(initial_state: DominoesState) -> Self {
        Self {
            states: vec![initial_state],
            actions: Vec::new(),
            current_index: 0,
        }
    }

    /// Pushes a new state and action to history
    ///
    /// If we're not at the end of history (after an undo), this truncates future states.
    pub fn push(&mut self, state: DominoesState, action: Action) {
        // Truncate any future states if we branched from a previous undo
        self.states.truncate(self.current_index + 1);
        self.actions.truncate(self.current_index);

        self.states.push(state);
        self.actions.push(action);
        self.current_index += 1;
    }

    /// Undoes the last action, returning the previous state
    ///
    /// Returns `None` if already at the beginning of history.
    pub fn undo(&mut self) -> Option<&DominoesState> {
        if self.can_undo() {
            self.current_index -= 1;
            Some(&self.states[self.current_index])
        } else {
            None
        }
    }

    /// Redoes the previously undone action, returning the next state
    ///
    /// Returns `None` if already at the end of history.
    pub fn redo(&mut self) -> Option<&DominoesState> {
        if self.can_redo() {
            self.current_index += 1;
            Some(&self.states[self.current_index])
        } else {
            None
        }
    }

    /// Returns true if undo is possible
    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    /// Returns true if redo is possible
    pub fn can_redo(&self) -> bool {
        self.current_index < self.states.len() - 1
    }

    /// Returns the current state
    pub fn current_state(&self) -> &DominoesState {
        &self.states[self.current_index]
    }

    /// Returns all actions up to the current position
    pub fn actions(&self) -> &[Action] {
        &self.actions[..self.current_index]
    }

    /// Returns the total number of states in history
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Returns true if history is empty (only initial state)
    pub fn is_empty(&self) -> bool {
        self.states.len() == 1
    }

    /// Returns the current index in history
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Returns the initial state
    pub fn initial_state(&self) -> &DominoesState {
        &self.states[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rules::{Configuration, Tile};

    fn create_test_state() -> DominoesState {
        let config = Configuration::default();
        DominoesState::new(&config)
    }

    #[test]
    fn test_new_starts_at_index_zero() {
        let state = create_test_state();
        let history = GameHistory::new(state);

        assert_eq!(history.current_index(), 0);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_push_increments_index() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        let action = Action::pass(0);
        history.push(state.clone(), action);

        assert_eq!(history.current_index(), 1);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_current_state_returns_latest() {
        let mut state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        // Play a tile to change state
        let tile = Tile::from((6, 6));
        state.play_tile(tile, None);
        let action = Action::play(0, tile, None);
        history.push(state.clone(), action);

        // Current state should have the tile played
        assert!(!history.current_state().layout.is_empty());
    }

    #[test]
    fn test_undo_decrements_index() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        let action = Action::pass(0);
        history.push(state.clone(), action);
        assert_eq!(history.current_index(), 1);

        history.undo();
        assert_eq!(history.current_index(), 0);
    }

    #[test]
    fn test_undo_returns_previous_state() {
        let initial_state = create_test_state();
        let mut history = GameHistory::new(initial_state.clone());

        let mut modified_state = initial_state.clone();
        modified_state.play_tile(Tile::from((6, 6)), None);
        history.push(modified_state, Action::play(0, Tile::from((6, 6)), None));

        let undone = history.undo();
        assert!(undone.is_some());
        assert!(undone.unwrap().layout.is_empty());
    }

    #[test]
    fn test_undo_at_start_returns_none() {
        let state = create_test_state();
        let mut history = GameHistory::new(state);

        assert!(history.undo().is_none());
    }

    #[test]
    fn test_can_undo_false_at_start() {
        let state = create_test_state();
        let history = GameHistory::new(state);

        assert!(!history.can_undo());
    }

    #[test]
    fn test_can_undo_true_after_push() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        history.push(state, Action::pass(0));
        assert!(history.can_undo());
    }

    #[test]
    fn test_redo_increments_index() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        history.push(state, Action::pass(0));
        history.undo();
        assert_eq!(history.current_index(), 0);

        history.redo();
        assert_eq!(history.current_index(), 1);
    }

    #[test]
    fn test_redo_returns_next_state() {
        let initial_state = create_test_state();
        let mut history = GameHistory::new(initial_state.clone());

        let mut modified_state = initial_state.clone();
        modified_state.play_tile(Tile::from((6, 6)), None);
        history.push(modified_state, Action::play(0, Tile::from((6, 6)), None));

        history.undo();
        let redone = history.redo();

        assert!(redone.is_some());
        assert!(!redone.unwrap().layout.is_empty());
    }

    #[test]
    fn test_redo_at_end_returns_none() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        history.push(state, Action::pass(0));
        assert!(history.redo().is_none());
    }

    #[test]
    fn test_can_redo_false_at_end() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        history.push(state, Action::pass(0));
        assert!(!history.can_redo());
    }

    #[test]
    fn test_can_redo_true_after_undo() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        history.push(state, Action::pass(0));
        history.undo();
        assert!(history.can_redo());
    }

    #[test]
    fn test_push_after_undo_truncates_future() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        // Push two states
        history.push(state.clone(), Action::pass(0));
        history.push(state.clone(), Action::pass(1));
        assert_eq!(history.len(), 3);

        // Undo once
        history.undo();
        assert_eq!(history.current_index(), 1);

        // Push a new state - should truncate the future
        history.push(state, Action::draw(0, Tile::from((1, 2))));
        assert_eq!(history.len(), 3); // Still 3, not 4
        assert_eq!(history.current_index(), 2);
    }

    #[test]
    fn test_redo_not_available_after_new_push() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        history.push(state.clone(), Action::pass(0));
        history.push(state.clone(), Action::pass(1));
        history.undo();
        history.push(state, Action::draw(0, Tile::from((1, 2))));

        assert!(!history.can_redo());
    }

    #[test]
    fn test_actions_returns_all_actions_up_to_current() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        let action1 = Action::pass(0);
        let action2 = Action::pass(1);
        history.push(state.clone(), action1.clone());
        history.push(state, action2.clone());

        let actions = history.actions();
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_actions_empty_at_start() {
        let state = create_test_state();
        let history = GameHistory::new(state);

        assert!(history.actions().is_empty());
    }

    #[test]
    fn test_is_empty() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        assert!(history.is_empty());

        history.push(state, Action::pass(0));
        assert!(!history.is_empty());
    }

    #[test]
    fn test_multiple_undo_redo_cycles() {
        let state = create_test_state();
        let mut history = GameHistory::new(state.clone());

        // Push 3 states
        history.push(state.clone(), Action::pass(0));
        history.push(state.clone(), Action::pass(1));
        history.push(state.clone(), Action::pass(0));

        // Undo all
        assert!(history.undo().is_some());
        assert!(history.undo().is_some());
        assert!(history.undo().is_some());
        assert!(history.undo().is_none());
        assert_eq!(history.current_index(), 0);

        // Redo all
        assert!(history.redo().is_some());
        assert!(history.redo().is_some());
        assert!(history.redo().is_some());
        assert!(history.redo().is_none());
        assert_eq!(history.current_index(), 3);
    }
}
