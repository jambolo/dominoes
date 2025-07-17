/// A GameState implementation using a dictionary-based approach for flexible game state management.
/// This allows storing arbitrary key-value pairs to represent any aspect of the game state.

use std::collections::HashMap;
use std::fmt;

/// Represents a value that can be stored in the game state
#[derive(Debug, Clone, PartialEq)]
pub enum GameValue {
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// List of values
    List(Vec<GameValue>),
    /// Nested dictionary
    Dictionary(HashMap<String, GameValue>),
}

impl fmt::Display for GameValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameValue::Integer(i) => write!(f, "{}", i),
            GameValue::Float(fl) => write!(f, "{}", fl),
            GameValue::String(s) => write!(f, "{}", s),
            GameValue::Boolean(b) => write!(f, "{}", b),
            GameValue::List(list) => {
                write!(f, "[")?;
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            GameValue::Dictionary(dict) => {
                write!(f, "{{")?;
                for (i, (key, value)) in dict.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Main GameState class that stores game state as a dictionary of key-value pairs
#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    /// Internal storage for game state data
    data: HashMap<String, GameValue>,
}

impl GameState {
    /// Creates a new empty game state
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Creates a game state with initial capacity for performance optimization
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }

    /// Sets a value in the game state
    pub fn set<K: Into<String>>(&mut self, key: K, value: GameValue) {
        self.data.insert(key.into(), value);
    }

    /// Gets a value from the game state
    pub fn get(&self, key: &str) -> Option<&GameValue> {
        self.data.get(key)
    }

    /// Gets a mutable reference to a value in the game state
    pub fn get_mut(&mut self, key: &str) -> Option<&mut GameValue> {
        self.data.get_mut(key)
    }

    /// Removes a value from the game state and returns it
    pub fn remove(&mut self, key: &str) -> Option<GameValue> {
        self.data.remove(key)
    }

    /// Checks if a key exists in the game state
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns the number of key-value pairs in the game state
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Checks if the game state is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clears all data from the game state
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns an iterator over the keys in the game state
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    /// Returns an iterator over the values in the game state
    pub fn values(&self) -> impl Iterator<Item = &GameValue> {
        self.data.values()
    }

    /// Returns an iterator over key-value pairs in the game state
    pub fn iter(&self) -> impl Iterator<Item = (&String, &GameValue)> {
        self.data.iter()
    }

    /// Merges another game state into this one, overwriting existing keys
    pub fn merge(&mut self, other: &GameState) {
        for (key, value) in &other.data {
            self.data.insert(key.clone(), value.clone());
        }
    }

    /// Creates a deep copy of the game state
    pub fn deep_copy(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GameState {{")?;
        for (i, (key, value)) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", key, value)?;
        }
        write!(f, "}}")
    }
}

// Convenience methods for common value types
impl GameState {
    /// Sets an integer value
    pub fn set_int<K: Into<String>>(&mut self, key: K, value: i64) {
        self.set(key, GameValue::Integer(value));
    }

    /// Sets a float value
    pub fn set_float<K: Into<String>>(&mut self, key: K, value: f64) {
        self.set(key, GameValue::Float(value));
    }

    /// Sets a string value
    pub fn set_string<K: Into<String>>(&mut self, key: K, value: String) {
        self.set(key, GameValue::String(value));
    }

    /// Sets a boolean value
    pub fn set_bool<K: Into<String>>(&mut self, key: K, value: bool) {
        self.set(key, GameValue::Boolean(value));
    }

    /// Gets an integer value
    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.get(key) {
            Some(GameValue::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    /// Gets a float value
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.get(key) {
            Some(GameValue::Float(f)) => Some(*f),
            _ => None,
        }
    }

    /// Gets a string value
    pub fn get_string(&self, key: &str) -> Option<&String> {
        match self.get(key) {
            Some(GameValue::String(s)) => Some(s),
            _ => None,
        }
    }

    /// Gets a boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key) {
            Some(GameValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gamestate() {
        let state = GameState::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut state = GameState::new();
        state.set("test_key", GameValue::String("test_value".to_string()));
        
        assert_eq!(state.len(), 1);
        assert!(state.contains_key("test_key"));
        assert_eq!(state.get("test_key"), Some(&GameValue::String("test_value".to_string())));
    }

    #[test]
    fn test_convenience_methods() {
        let mut state = GameState::new();
        
        state.set_int("score", 100);
        state.set_float("health", 95.5);
        state.set_string("name", "Player1".to_string());
        state.set_bool("active", true);
        
        assert_eq!(state.get_int("score"), Some(100));
        assert_eq!(state.get_float("health"), Some(95.5));
        assert_eq!(state.get_string("name"), Some(&"Player1".to_string()));
        assert_eq!(state.get_bool("active"), Some(true));
    }

    #[test]
    fn test_remove() {
        let mut state = GameState::new();
        state.set_int("temp", 42);
        
        assert!(state.contains_key("temp"));
        let removed = state.remove("temp");
        assert_eq!(removed, Some(GameValue::Integer(42)));
        assert!(!state.contains_key("temp"));
    }

    #[test]
    fn test_clear() {
        let mut state = GameState::new();
        state.set_int("a", 1);
        state.set_int("b", 2);
        
        assert_eq!(state.len(), 2);
        state.clear();
        assert!(state.is_empty());
    }

    #[test]
    fn test_merge() {
        let mut state1 = GameState::new();
        state1.set_int("a", 1);
        state1.set_int("b", 2);
        
        let mut state2 = GameState::new();
        state2.set_int("b", 3);  // This should overwrite
        state2.set_int("c", 4);
        
        state1.merge(&state2);
        
        assert_eq!(state1.get_int("a"), Some(1));
        assert_eq!(state1.get_int("b"), Some(3));  // Overwritten
        assert_eq!(state1.get_int("c"), Some(4));
    }

    #[test]
    fn test_game_value_display() {
        let int_val = GameValue::Integer(42);
        let str_val = GameValue::String("hello".to_string());
        let bool_val = GameValue::Boolean(true);
        let list_val = GameValue::List(vec![
            GameValue::Integer(1),
            GameValue::String("test".to_string())
        ]);
        
        assert_eq!(format!("{}", int_val), "42");
        assert_eq!(format!("{}", str_val), "hello");
        assert_eq!(format!("{}", bool_val), "true");
        assert_eq!(format!("{}", list_val), "[1, test]");
    }
}
