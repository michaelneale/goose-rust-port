mod message;
mod openai;

pub use message::{Content, Message, Role, Text, ToolResult, ToolUse};
pub use openai::{OpenAIConfig, OpenAIProvider};

use anyhow::{anyhow, Result};
use async_trait::async_trait;

/// Trait for LLM providers
#[async_trait]
pub trait Provider {
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
    provider: Box<dyn Provider>,
    messages: Vec<Message>,
}

impl Exchange {
    pub async fn new(provider: Box<dyn Provider>) -> Result<Self> {
        let mut provider = provider;
        provider.initialize().await?;
        
        Ok(Self {
            provider,
            messages: Vec::new(),
        })
    }
    
    pub async fn generate(&mut self) -> Result<Message> {
        let response = self.provider.generate(&self.messages).await?;
        self.messages.push(response.clone());
        Ok(response)
    }
    
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
    
    pub fn get_token_usage(&self) -> u32 {
        self.provider.get_token_usage()
    }
}