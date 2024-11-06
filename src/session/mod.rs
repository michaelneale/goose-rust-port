use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use colored::*;
use ctrlc;
use log::{info, error};

use crate::exchange::Message;
use crate::input::{create_default_input_handler, InputHandler};
use crate::cli::config::LOG_PATH;

pub struct SessionLoop {
    messages: Vec<Message>,
    interrupted: Arc<AtomicBool>,
    name: String,
    profile_name: Option<String>,
}

impl SessionLoop {
    pub fn new(name: String, profile_name: Option<String>) -> Self {
        let interrupted = Arc::new(AtomicBool::new(false));
        let int_handler = Arc::clone(&interrupted);
        
        ctrlc::set_handler(move || {
            int_handler.store(true, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");

        Self {
            messages: Vec::new(),
            interrupted,
            name,
            profile_name,
        }
    }

    pub fn process_message(&mut self, message: Message) -> Result<()> {
        // Validate the message
        message.validate()?;
        
        // Add message to history
        self.messages.push(message);

        // Check for interruption
        if self.interrupted.load(Ordering::SeqCst) {
            self.handle_interrupt()?;
            return Ok(());
        }
        
        // TODO: Process the message through the exchange
        // This will involve:
        // 1. Sending message to LLM
        // 2. Getting response
        // 3. Processing any tool uses
        
        Ok(())
    }

    pub fn run(&mut self, new_session: bool) -> Result<()> {
        let time_start = Utc::now();
        
        let profile = self.profile_name.as_deref().unwrap_or("default");
        println!("{}", format!("starting session | name: {} profile: {}", 
            self.name.cyan(), profile.cyan()).dimmed());

        // Main interaction loop
        loop {
            // Check for interruption
            if self.interrupted.load(Ordering::SeqCst) {
                self.handle_interrupt()?;
                break;
            }

            // Get user input using the input handler
            let mut input_handler = create_default_input_handler();
            let input = input_handler.get_user_input()?;
            if input.to_exit() {
                break;
            }

            // Process the message
            let message = Message::user(&input);
            self.process_message(message)?;
        }
        
        let time_end = Utc::now();
        self.log_session_stats(time_start, time_end)?;
        
        Ok(())
    }

    fn handle_interrupt(&mut self) -> Result<()> {
        // Default recovery message if no user message is pending
        let mut recovery = "We interrupted before the next processing started.";

        if let Some(last_message) = self.messages.last() {
            if last_message.is_user() {
                // If the last message is from the user, remove it
                self.messages.pop();
                recovery = "We interrupted before the model replied and removed the last message.";
            }

            if let Some(last_message) = self.messages.last() {
                if last_message.is_assistant() && last_message.has_tool_use() {
                    // TODO: Handle tool interruption properly
                    recovery = "We interrupted the existing tool call. How would you like to proceed?";
                }
            }
        }

        println!("{}", recovery.yellow());
        self.interrupted.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn log_session_stats(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<()> {
        // Ensure log directory exists
        std::fs::create_dir_all(LOG_PATH)
            .with_context(|| format!("Failed to create log directory at {}", LOG_PATH))?;

        // Calculate duration
        let duration = end_time.signed_duration_since(start_time);
        
        // Log statistics
        info!(
            "Session {} completed. Duration: {}s, Messages: {}", 
            self.name,
            duration.num_seconds(),
            self.messages.len()
        );

        // TODO: Implement token usage and cost tracking
        
        Ok(())
    }
}

impl Drop for SessionLoop {
    fn drop(&mut self) {
        if let Err(e) = self.handle_interrupt() {
            error!("Error handling interrupt during cleanup: {}", e);
        }
    }
}