# Player Crate

This crate provides the base `Player` trait and related types for the dominoes game.

## Overview

The player crate defines:

- `Color` enum: Represents which side/player (Player1 or Player2)
- `GameState` struct: Represents the current state of the game (placeholder for now)
- `Player` trait: The base interface that all players must implement
- `BasePlayer` struct: A concrete base implementation of the Player trait

## Usage

```rust
use player::{Player, BasePlayer, Color, GameState};

// Create a base player
let mut player = BasePlayer::new(Color::Player1);

// Or create with a custom name
let mut named_player = BasePlayer::with_name(Color::Player2, "Alice".to_string());

// Use the player in a game loop
let game_state = GameState::new();
let new_state = player.my_turn(&game_state);
```

## Implementing Custom Players

To create a custom player, implement the `Player` trait:

```rust
use player::{Player, Color, GameState};

struct MyCustomPlayer {
    color: Color,
}

impl Player for MyCustomPlayer {
    fn color(&self) -> Color {
        self.color
    }
    
    fn my_turn(&mut self, game_state: &GameState) -> GameState {
        // Your custom logic here
        game_state.clone() // placeholder
    }
}
```

## Future Development

- The `GameState` struct needs to be fully implemented with actual game state
- Additional player types (AI players, network players, etc.) can be built on this foundation
- Game-specific logic should be added to the `my_turn` method implementations
