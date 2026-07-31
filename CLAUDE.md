# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture

Rust workspace. The `game-player` submodule may be uninitialized on disk (`git submodule update --init`); its summary below is the only in-repo documentation when it is.

### game-player (submodule, v0.4.0)

Generic AI framework with minimax (alpha-beta + transposition table) and MCTS searches.

**Core Traits:**

- `State` (`state.rs`): Game state with fingerprinting, turn tracking, action application. Has associated `Action` type. `whose_turn()` returns `PlayerId` (`Alice = 0`, `Bob = 1`).
- `mcts::ResponseGenerator` (`mcts.rs`): Generates legal moves. Policy: must return no actions if and only if `state.is_terminal()` is true (validated by `debug_assert` in the search); a non-terminal state with no plays must yield an explicit pass action. A separate `minimax::ResponseGenerator` (with a `depth` parameter) exists for minimax.
- `mcts::ValueEstimator` (`mcts.rs`): Replaces the old `Rollout` trait. `estimate(&self, state, rg) -> f32` returns a value in `[0.0, 1.0]` from the perspective of `state.whose_turn()`; `0.0`/`1.0` are reserved for certain outcomes.
- `StaticEvaluator` (`static_evaluator.rs`): Associated-type based (`type State`), evaluates from Alice's perspective. Used by minimax.

**Search:**

- `mcts::search(s0, rg, estimator, exploration_constant, initial_value_weight, estimate_on_expansion, max_iterations)`: Returns `Option<Action>` - the action with most visits. Defaults: `DEFAULT_EXPLORATION_CONSTANT` (`√2`), `DEFAULT_INITIAL_VALUE_WEIGHT` (`0.0`).
- `minimax::search()`: Alpha-beta minimax entry point.
- `RandomPlayoutEstimator` (`random_playout.rs`, feature `mcts_random_playout`): Generic random-rollout `ValueEstimator`.

**Integration:** Implement `State`, `mcts::ResponseGenerator`, `mcts::ValueEstimator` traits, call `mcts::search()`.

## Branch Strategy

- `master` - Release branch
- `develop` - Main development branch
- `feature/**` - Feature branches
