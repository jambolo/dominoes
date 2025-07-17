/// Dominoes Game Application
/// 
/// This is the main application that brings together all the dominoes game components:
/// - DominoesGameState for game state management
/// - DominoesPlayer for basic player implementation
/// - HumanPlayer for human player interaction

use dominoes_gamestate::DominoesGameState;
use dominoes_player::DominoesPlayer;
use human_player::HumanPlayer;
use player::Player;

fn main() {
    println!("Welcome to the Dominoes Game!");
    
    // Initialize the game
    let mut game = DominoesGame::new();
    game.setup();
    
    // Run the game loop
    game.run();
    
    println!("Thanks for playing!");
}

/// Main game controller that manages the entire dominoes game
pub struct DominoesGame {
    /// The game state
    game_state: DominoesGameState,
    /// List of players in the game
    players: Vec<Box<dyn Player>>,
    /// Current player index
    current_player_index: usize,
}

impl DominoesGame {
    /// Creates a new dominoes game
    pub fn new() -> Self {
        Self {
            game_state: DominoesGameState::new(),
            players: Vec::new(),
            current_player_index: 0,
        }
    }
    
    /// Sets up the game with default players and configuration
    pub fn setup(&mut self) {
        println!("Setting up the game...");
        
        // Initialize the game state
        self.game_state.initialize();
        self.game_state.setup_dominoes();
        
        // Add players
        self.add_human_player("Player 1".to_string());
        self.add_dominoes_player("Player 2".to_string());
        
        // Deal dominoes to players
        self.game_state.deal_dominoes(self.players.len());
        
        println!("Game setup complete! {} players ready.", self.players.len());
    }
    
    /// Adds a human player to the game
    pub fn add_human_player(&mut self, name: String) {
        let player = HumanPlayer::new(name.clone());
        self.players.push(Box::new(player));
        println!("Added human player: {}", name);
    }
    
    /// Adds a dominoes player to the game
    pub fn add_dominoes_player(&mut self, name: String) {
        let player = DominoesPlayer::new(name.clone());
        self.players.push(Box::new(player));
        println!("Added dominoes player: {}", name);
    }
    
    /// Runs the main game loop
    pub fn run(&mut self) {
        println!("Starting the game...");
        
        let mut turn_count = 0;
        let max_turns = 10; // Prevent infinite loop in stub implementation
        
        while !self.game_state.is_game_over() && turn_count < max_turns {
            self.play_turn();
            self.next_player();
            turn_count += 1;
        }
        
        self.end_game();
    }
    
    /// Plays a single turn for the current player
    fn play_turn(&mut self) {
        if let Some(current_player) = self.players.get_mut(self.current_player_index) {
            println!("It's {}'s turn", current_player.name());
            
            // Get the current game state
            let current_state = self.game_state.game_state().clone();
            
            // Let the player make their move
            let new_state = current_player.my_turn(&current_state);
            
            // Update the game state (in a real implementation, this would be more sophisticated)
            // For now, this is just a placeholder since we're working with stubs
            
            println!("{} completed their turn", current_player.name());
        }
    }
    
    /// Advances to the next player
    fn next_player(&mut self) {
        self.current_player_index = (self.current_player_index + 1) % self.players.len();
        self.game_state.set_current_player(self.current_player_index);
    }
    
    /// Handles end of game logic
    fn end_game(&self) {
        println!("Game Over!");
        
        if let Some(winner_id) = self.game_state.get_winner() {
            if let Some(winner) = self.players.get(winner_id) {
                println!("Winner: {}", winner.name());
            }
        } else {
            println!("No winner determined (stub implementation)");
        }
        
        // Display final game statistics
        self.display_game_summary();
    }
    
    /// Displays a summary of the game
    fn display_game_summary(&self) {
        println!("\n--- Game Summary ---");
        println!("Players:");
        for (i, player) in self.players.iter().enumerate() {
            println!("  {}: {}", i + 1, player.name());
        }
        println!("Game completed successfully!");
    }
}
