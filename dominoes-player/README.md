# Dominoes Player Crate

This crate provides concrete implementations of the `Player` trait specifically designed for dominoes games.

## Overview

The dominoes-player crate defines:

- `DominoesPlayer`: A basic concrete implementation of the Player trait
- `DominoesAIPlayer`: An AI player implementation with configurable difficulty
- `DominoesHumanPlayer`: A human player implementation for interactive gameplay

## Features

- **Multiple Player Types**: Support for different types of players (basic, AI, human)
- **Configurable AI**: AI players with adjustable difficulty levels
- **Extensible Design**: Built on the base Player trait for consistency
- **Stub Implementation**: Ready for game-specific logic implementation

## Usage

```rust
use dominoes_player::{DominoesPlayer, DominoesAIPlayer, DominoesHumanPlayer};
use player::{Player, GameState};

// Create different types of players
let mut basic_player = DominoesPlayer::new("Alice".to_string());
let mut ai_player = DominoesAIPlayer::new("AI Bob".to_string(), 3);
let mut human_player = DominoesHumanPlayer::new("Charlie".to_string());

// Use players in a game
let game_state = GameState::new();
let new_state = basic_player.my_turn(&game_state);

// Configure AI difficulty
ai_player.set_difficulty(5);
```

## Current Implementation Status

**Note**: This crate currently provides stub implementations only. The following functionality needs to be implemented:

### DominoesPlayer

- [ ] Basic dominoes game logic in `my_turn` method
- [ ] Domino hand management
- [ ] Valid move detection

### DominoesAIPlayer

- [ ] AI decision-making algorithms
- [ ] Difficulty-based strategy selection
- [ ] Move evaluation and scoring

### DominoesHumanPlayer

- [ ] User input handling for move selection
- [ ] Move validation and feedback
- [ ] Interactive game interface

## Future Development

1. **Game Logic Integration**: Implement actual dominoes rules and move validation
2. **AI Strategies**: Develop different AI algorithms for various difficulty levels
3. **Input/Output Handling**: Add proper user interaction for human players
4. **Game State Integration**: Work with actual dominoes game state representation
5. **Move Validation**: Ensure players can only make legal moves
6. **Strategy Patterns**: Implement different playing strategies and styles
