use std::sync::Arc;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use tokio::sync::Mutex;

pub use crate::models::Message;
mod message;
pub use message::Content;
mod openai;
pub use openai::{OpenAIOptions, OpenAIProvider};

/// Trait for LLM providers
#[async_trait]
pub trait Provider: Send + Sync {
    /// Initialize the provider with configuration
    async fn initialize(&mut self) -> Result<()>;
    
    /// Generate a response for the given messages
    async fn generate(&self, messages: &[Message]) -> Result<Message>;
    
    /// Get the token usage for the last request
    fn get_token_usage(&self) -> u32;
}

/// Create a new provider instance based on configuration
pub fn create_provider(provider_name: &str) -> Result<Box<dyn Provider>> {
    match provider_name {
        "openai" => Ok(Box::new(OpenAIProvider::new(None)?)),
        _ => Err(anyhow!("Unknown provider: {}", provider_name)),
    }
}

/// Exchange handles communication with the LLM provider
pub struct Exchange {
    provider: Arc<Box<dyn Provider>>,
    messages: Arc<Mutex<Vec<Message>>>,
    token_usage: Arc<Mutex<u32>>,
}

impl Exchange {
    pub async fn new(provider: Box<dyn Provider>) -> Result<Self> {
        let mut provider = provider;
        provider.initialize().await?;
        
        Ok(Self {
            provider: Arc::new(provider),
            messages: Arc::new(Mutex::new(Vec::new())),
            token_usage: Arc::new(Mutex::new(0)),
        })
    }
    
    /// Add a message to the conversation history
    pub async fn add_message(&self, message: Message) -> Result<()> {
        message.validate()?;
        let mut messages = self.messages.lock().await;
        messages.push(message);
        Ok(())
    }

    /// Generate a response using the provider
    pub async fn generate(&self, messages: &[Message]) -> Result<Message> {
        let response = self.provider.generate(messages).await?;
        
        // Update token usage
        let mut token_usage = self.token_usage.lock().await;
        *token_usage += self.provider.get_token_usage();
        
        // Add response to messages
        drop(token_usage); // Release token_usage lock before acquiring messages lock
        let mut messages = self.messages.lock().await;
        messages.push(response.clone());
        
        Ok(response)
    }

    /// Remove the last message from history
    pub async fn rewind(&self) -> Result<()> {
        let mut messages = self.messages.lock().await;
        messages.pop();
        Ok(())
    }
    
    /// Get the total token usage
    pub async fn get_token_usage(&self) -> u32 {
        *self.token_usage.lock().await
    }

    /// Get a reference to the messages
    pub async fn get_messages(&self) -> Vec<Message> {
        self.messages.lock().await.clone()
    }

    /// Process tool usage in a message
    pub async fn process_tool_use(&self, tool_use: &Content) -> Result<Content> {
        // TODO: Implement tool usage processing
        // For now return a placeholder error result
        Ok(Content::ToolResult {
            tool_use_id: "placeholder".to_string(),
            output: "Tool processing not implemented yet".to_string(),
            is_error: true,
        })
    }
}