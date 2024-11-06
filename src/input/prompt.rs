use std::io::{stdout, Write};
use anyhow::Result;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use super::InputHandler;

#[derive(Debug)]
pub struct UserInput {
    pub text: String,
}

impl UserInput {
    pub fn to_exit(&self) -> bool {
        self.text.trim().is_empty()
    }

    pub fn to_continue(&self) -> bool {
        !self.to_exit()
    }
}

pub struct GoosePrompt {
    editor: DefaultEditor,
}

impl GoosePrompt {
    pub fn new() -> Self {
        Self {
            editor: DefaultEditor::new().expect("Failed to create editor"),
        }
    }

    fn get_prompt(&self) -> String {
        format!("{} ", "❯".green().bold())
    }
}

impl InputHandler for GoosePrompt {
    fn get_user_input(&mut self) -> Result<UserInput> {
        let prompt = self.get_prompt();
        match self.editor.readline(&prompt) {
            Ok(line) => {
                self.editor.add_history_entry(&line)?;
                Ok(UserInput { text: line })
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                Ok(UserInput { text: String::new() })
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                Ok(UserInput { text: String::new() })
            }
            Err(err) => {
                Err(anyhow::anyhow!("Error reading line: {}", err))
            }
        }
    }

    fn display(&self, message: &str) {
        println!("{}", message);
        stdout().flush().expect("Failed to flush stdout");
    }

    fn clear(&mut self) {
        print!("\x1B[2J\x1B[1;1H");
        stdout().flush().expect("Failed to flush stdout");
    }
}

impl Default for GoosePrompt {
    fn default() -> Self {
        Self::new()
    }
}