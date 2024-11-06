use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use anyhow::{Result, Context};
use chrono::DateTime;
use colored::*;
use log::{info, error};
use tokio::sync::Mutex;

use crate::exchange::{Exchange, Message, create_provider};
use crate::input::{create_default_input_handler, InputHandler};
use crate::stats::SessionStats;
use crate::cli::config::{session_path, ensure_config, LOG_PATH};
use crate::utils::session_file::{read_or_create_file, log_messages};

pub struct Session {
    pub name: String,
    pub profile_name: Option<String>,
    pub tracing: bool,
    pub session_file_path: PathBuf,
    pub messages: Vec<Message>,
    pub interrupted: Arc<AtomicBool>,
    pub exchange: Option<Exchange>,
    pub stats: SessionStats,
}

impl Session {
    pub async fn new(
        name: Option<String>, 
        profile: Option<String>,
        plan: Option<serde_yaml::Value>,
        log_level: Option<String>,
        tracing: bool,
    ) -> Result<Self> {
        let name = name.unwrap_or_else(|| generate_name());
        let session_file_path = session_path(&name);
        
        let interrupted = Arc::new(AtomicBool::new(false));
        let int_handler = Arc::clone(&interrupted);
        
        // Only set the handler in non-test environments
        // or if we're running specific tests that need it
        if !cfg!(test) || std::env::var("TEST_CTRL_C").is_ok() {
            if let Err(e) = ctrlc::set_handler(move || {
                int_handler.store(true, Ordering::SeqCst);
            }) {
                println!("Warning: Failed to set Ctrl-C handler: {}", e);
            }
        }

        let stats = SessionStats::new(name.clone());
        
        let mut session = Session {
            name,
            profile_name: profile,
            tracing,
            session_file_path,
            messages: Vec::new(),
            interrupted,
            exchange: None,
            stats,
        };

        session.messages.extend(session.load_session()?);

        // Initialize exchange with OpenAI provider
        let provider = create_provider("openai")?;
        session.exchange = Some(Exchange::new(provider).await?);

        if session.messages.is_empty() && plan.is_some() {
            session.setup_plan(plan.unwrap())?;
        }

        Ok(session)
    }

    pub async fn run(&mut self, new_session: bool) -> Result<()> {
        let time_start = chrono::Utc::now();
        
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
            let message = Message::user(&input.text);
            self.process_message(message).await?;
        }
        
        let time_end = chrono::Utc::now();
        self.log_session_stats(time_start, time_end)?;
        
        Ok(())
    }

    pub fn single_pass(&mut self, initial_message: String) -> Result<()> {
        let profile = self.profile_name.as_deref().unwrap_or("default");
        println!("starting session | name: {} profile: {}", self.name, profile);
        println!("saving to {}", self.session_file_path.display());

        // TODO: Add message and process response
        
        println!("ended run | name: {} profile: {}", self.name, profile);
        println!("to resume: goose session resume {} --profile {}", self.name, profile);
        
        Ok(())
    }

    fn load_session(&self) -> Result<Vec<Message>> {
        read_or_create_file(&self.session_file_path)
    }

    fn setup_plan(&mut self, plan: serde_yaml::Value) -> Result<()> {
        if !self.messages.is_empty() {
            return Err(anyhow::anyhow!("The plan can only be set on an empty session."));
        }

        // TODO: Implement plan setup
        Ok(())
    }

    pub async fn process_message(&mut self, message: Message) -> Result<()> {
        // Validate the message
        message.validate()?;
        
        // Add message to history
        self.messages.push(message);
        self.stats.add_message();

        // Check for interruption
        if self.interrupted.load(Ordering::SeqCst) {
            self.handle_interrupt()?;
            return Ok(());
        }
        
        // Process through exchange if available
        if let Some(exchange) = &self.exchange {
            let response = exchange.generate(&self.messages).await?;
            self.messages.push(response.clone());
            
            // Update token usage
            self.stats.add_tokens(exchange.get_token_usage().await);
            
            // Display response
            if !response.text().is_empty() {
                println!("{}", response.text());
            }
        }
        
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

    pub fn is_interrupted(&self) -> bool {
        self.interrupted.load(Ordering::SeqCst)
    }

    pub fn get_stats(&self) -> &SessionStats {
        &self.stats
    }

    pub fn interrupt(&self) {
        self.interrupted.store(true, Ordering::SeqCst);
    }

    fn log_session_stats(&self, start_time: DateTime<chrono::Utc>, end_time: DateTime<chrono::Utc>) -> Result<()> {
        // Ensure log directory exists
        std::fs::create_dir_all(LOG_PATH)
            .with_context(|| format!("Failed to create log directory at {}", LOG_PATH))?;

        // Calculate duration
        let duration = end_time.signed_duration_since(start_time);
        
        // Log statistics
        info!(
            "Session {} completed.\nDuration: {}s\nMessages: {}\nTokens: {}\nEstimated cost: ${:.4}", 
            self.name,
            duration.num_seconds(),
            self.messages.len(),
            self.stats.total_tokens,
            self.stats.total_cost
        );
        
        Ok(())
    }
}

fn generate_name() -> String {
    crate::utils::generate_name()
}