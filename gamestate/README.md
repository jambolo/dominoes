# GameState Crate

This crate provides a flexible dictionary-based `GameState` implementation for storing and managing game state data.

## Overview

The gamestate crate defines:

- `GameValue` enum: Represents different types of values that can be stored (integers, floats, strings, booleans, lists, dictionaries)
- `GameState` struct: A dictionary-based container for storing key-value pairs representing game state

## Features

- **Flexible Storage**: Store any type of game data using the `GameValue` enum
- **Type-Safe Access**: Convenience methods for common data types
- **Nested Data**: Support for lists and nested dictionaries
- **Efficient Operations**: Built on `HashMap` for O(1) average case access
- **Serialization Ready**: All types implement necessary traits for serialization (when needed)

## Usage

```rust
use gamestate::{GameState, GameValue};

// Create a new game state
let mut state = GameState::new();

// Set various types of values
state.set_int("score", 1000);
state.set_string("player_name", "Alice".to_string());
state.set_bool("game_over", false);
state.set_float("health", 95.5);

// Set complex values
state.set("inventory", GameValue::List(vec![
    GameValue::String("sword".to_string()),
    GameValue::String("potion".to_string()),
]));

// Access values
if let Some(score) = state.get_int("score") {
    println!("Current score: {}", score);
}

// Check existence
if state.contains_key("player_name") {
    println!("Player name is set");
}

// Merge states
let mut other_state = GameState::new();
other_state.set_int("level", 5);
state.merge(&other_state);
```

## Design Philosophy

This implementation uses a dictionary approach to provide maximum flexibility without defining rigid interfaces. This allows:

- **Dynamic State Management**: Add or remove state properties at runtime
- **Game-Agnostic Design**: Works with any type of game or application
- **Easy Serialization**: Simple structure that can be easily saved/loaded
- **Extensible**: Can store any type of data through the `GameValue` enum

## Performance Considerations

- Uses `HashMap` internally for O(1) average case access
- Cloning operations create deep copies of all data
- Memory usage scales with the amount of data stored
- Consider using `with_capacity()` for known data sizes
