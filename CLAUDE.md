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
game (main app - Iced GUI)
├── player (player trait and HumanPlayer)
│   └── game-player (generic game AI framework, submodule - not currently used)
├── dominoes-state (game state, actions, hands)
└── rules (tiles, layout, boneyard, configuration, variations)
```

### rules

Core types: `Tile` (ordinal-based u8), `Layout` (tree via ego_tree), `Boneyard`, `Configuration`, `Variation` (Traditional, AllFives, AllSevens, Bergen, Blind, FiveUp).

### dominoes-state

`DominoesState` implements `game_player::State` trait. Tracks layout, boneyard, turn, fingerprint, passes, game status, and player hands. Uses Zobrist hashing.

### player

Simple `Player` trait with `id()` and `name()` methods. `HumanPlayer` implementation for human players. AI implementation is planned for the future.

### game

Main Iced GUI application with PvP gameplay:

- `app.rs`: Main application struct `DominoesApp`, turn phase state machine, message handling
- `game_history.rs`: Undo/redo via state snapshots
- `io.rs`: Import/export game history and state as JSON
- `layout_parser.rs`: Parse text-based layout format
- `scene_graph.rs`: Tile placement calculations for rendering

### game-player (submodule) - Future AI Support

Generic AI framework with MCTS (Monte Carlo Tree Search). Not currently integrated.

**Core Traits:**

- `State` (`state.rs`): Game state with fingerprinting, turn tracking, action application.
- `ResponseGenerator` (`mcts.rs`): Generates legal moves.
- `Rollout` (`mcts.rs`): Simulates game to terminal state.

**Search:**

- `mcts::search()`: Entry point for MCTS search.
- `InformationSetMCTS`: For hidden information games.

## Branch Strategy

- `master` - Release branch
- `develop` - Main development branch
- `feature/**` - Feature branches
