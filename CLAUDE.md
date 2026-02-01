# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
cargo build                    # Build all crates
cargo test                     # Run all tests
cargo test <test_name>         # Run a single test
cargo clippy                   # Lint
cargo fmt                      # Format code
cargo run --bin dominoes       # Run the game
cargo run --bin visualize      # Run visualize utility
cargo run --bin generate       # Run generate utility
```

## Feature Flags (game-player crate)

- `analysis_game_tree` - JSON serialization for search tree analysis
- `analysis_game_state` - JSON serialization for game state analysis
- `debug_game_tree_node_info` - Debug info for tree nodes

## Architecture

Rust workspace with 5 crates:

```
game (main app)
├── player (AI implementation)
│   └── game-player (generic game ai framework, submodule)
├── dominoes-state (game state, actions, hands)
└── rules (tiles, layout, boneyard, configuration, variations)
```

### rules

Core types: `Tile` (ordinal-based u8), `Layout` (tree via ego_tree), `Boneyard`, `Configuration`, `Variation` (Traditional, AllFives, AllSevens, Bergen, Blind, FiveUp).

### dominoes-state

`DominoesState` implements `game_player::State` trait. Tracks layout, boneyard, turn, fingerprint, passes, game status. Uses Zobrist hashing.

### game-player (submodule)

Generic AI framework with MCTS (Monte Carlo Tree Search).

**Core Traits:**

- `State` (`state.rs`): Game state with fingerprinting, turn tracking, action application. Has associated `Action` type.
- `ResponseGenerator` (`mcts.rs`): Generates legal moves. Returns empty vec when none available.
- `Rollout` (`mcts.rs`): Simulates game to terminal state, returns score in `[-1.0, 1.0]`.

**Search:**

- `mcts::search()`: Entry point. Takes state, response generator, rollout, exploration constant (default `√2`), max iterations.
- Returns `Option<Action>` - the action with most visits.
- `InformationSetMCTS` (`information_set_mcts.rs`): For hidden information games.

**Integration:** Implement `State`, `ResponseGenerator`, `Rollout` traits, call `mcts::search()`.

### player

Dominoes-specific `DominoesPlayer`, `ResponseGenerator`, `Rollout`, `StaticEvaluator` implementations.

### game

Main app using Iced GUI. Contains `dominoes_game.rs` (game loop), `layout_parser.rs`, `scene_graph.rs`.

## Branch Strategy

- `master` - Release branch
- `develop` - Main development branch
- `feature/**` - Feature branches
