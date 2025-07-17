# Dominoes Game Application

This is the main application crate that brings together all the dominoes game components to create a playable dominoes game.

## Overview

The game crate serves as the main application that orchestrates:

- **DominoesGameState**: For managing the game state and rules
- **DominoesPlayer**: For basic player implementations
- **HumanPlayer**: For human player interaction

## Features

- **Game Controller**: Central `DominoesGame` struct that manages the entire game flow
- **Player Management**: Support for multiple player types (human and dominoes players)
- **Game Loop**: Turn-based gameplay with proper game state management
- **Modular Design**: Clean separation between game logic and player implementations

## Usage

To run the dominoes game:

```bash
cargo run
```

The application will:

1. Initialize a new game
2. Set up default players (one human player, one dominoes player)
3. Run the game loop with turn-based gameplay
4. Display the winner and game summary when finished

## Architecture

```text
DominoesGame
├── DominoesGameState (game state management)
├── Vec<Box<dyn Player>> (player implementations)
│   ├── HumanPlayer
│   └── DominoesPlayer
└── Game loop and turn management
```

## Current Implementation Status

The game application currently provides a working framework with the following components:

### Implemented

- [x] Game initialization and setup
- [x] Player management and registration
- [x] Turn-based game loop
- [x] Game end detection and summary
- [x] Modular player system

### Stub Implementation

- [ ] Actual dominoes game rules and logic
- [ ] Real player input handling
- [ ] Move validation and board updates
- [ ] Scoring and winner determination
- [ ] Game persistence and save/load

## Future Development

1. **Game Logic Integration**: Implement actual dominoes rules and gameplay
2. **User Interface**: Add better console interface or GUI
3. **Game Configuration**: Allow customizable game settings and rules
4. **Save/Load**: Ability to save and resume games
5. **Multiplayer Support**: Network multiplayer functionality
6. **AI Players**: More sophisticated AI player implementations
7. **Tournament Mode**: Support for tournament-style gameplay

## Dependencies

- `dominoes-gamestate`: Game state management
- `dominoes-player`: Basic player implementation
- `human-player`: Human player interface
- `player`: Base player trait (via dependencies)
