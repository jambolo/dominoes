# Human Player Crate

This crate provides a concrete implementation of the `Player` trait specifically designed for human-controlled gameplay.

## Overview

The human-player crate defines:

- `HumanPlayer`: A concrete implementation of the Player trait for human players

## Features

- **Human Player Implementation**: Extends the base Player trait for interactive gameplay
- **Configurable Names**: Support for custom player names
- **Stub Implementation**: Ready for game-specific input handling implementation

## Usage

```rust
use human_player::HumanPlayer;
use player::{Player, GameState};

// Create a human player
let mut player = HumanPlayer::new("Alice".to_string());

// Or use default name
let mut default_player = HumanPlayer::default();

// Use player in a game
let game_state = GameState::new();
let new_state = player.my_turn(&game_state);
```

## Current Implementation Status

**Note**: This crate currently provides stub implementations only. The following functionality needs to be implemented:

### HumanPlayer

- [ ] User input handling for move selection
- [ ] Move validation and feedback
- [ ] Interactive game interface
- [ ] Error handling for invalid inputs
- [ ] User-friendly prompts and messages

## Future Development

1. **Input/Output Handling**: Implement proper user interaction for move selection
2. **Move Validation**: Ensure human players can only make legal moves
3. **User Interface**: Add clear prompts and feedback for player actions
4. **Error Handling**: Provide helpful error messages for invalid inputs
5. **Accessibility**: Consider different input methods and accessibility features
