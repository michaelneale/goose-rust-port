mod prompt;

pub use prompt::{GoosePrompt, UserInput};

use anyhow::Result;

/// Trait for handling user input in a session
pub trait InputHandler {
    /// Get input from the user
    fn get_user_input(&mut self) -> Result<UserInput>;
    
    /// Display a message to the user
    fn display(&self, message: &str);
    
    /// Clear the display
    fn clear(&mut self);
}

/// Default implementation using rustyline for terminal input
pub fn create_default_input_handler() -> impl InputHandler {
    GoosePrompt::new()
}