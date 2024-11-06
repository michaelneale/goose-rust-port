use std::env;
use anyhow::{Context, Result};
use async_openai::{
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequest, Role},
    Client,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::models::Message;
use super::Provider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u16,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

pub struct OpenAIProvider {
    client: Client,
    config: OpenAIConfig,
    last_token_usage: u32,
}

impl OpenAIProvider {
    pub fn new(config: Option<OpenAIConfig>) -> Result<Self> {
        // Check for API key
        let _api_key = env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable not set")?;

        Ok(Self {
            client: Client::new(),
            config: config.unwrap_or_default(),
            last_token_usage: 0,
        })
    }

    fn convert_message_to_openai(message: &Message) -> ChatCompletionRequestMessage {
        let role = match message.role {
            crate::models::message::Role::User => Role::User,
            crate::models::message::Role::Assistant => Role::Assistant,
        };

        ChatCompletionRequestMessage {
            role,
            content: Some(message.text()),
            name: None,
            function_call: None,
        }
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn initialize(&mut self) -> Result<()> {
        // No initialization needed for OpenAI
        Ok(())
    }

    async fn generate(&self, messages: &[Message]) -> Result<Message> {
        let request = CreateChatCompletionRequest {
            model: self.config.model.clone(),
            messages: messages
                .iter()
                .map(Self::convert_message_to_openai)
                .collect(),
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens as u16),
            ..Default::default()
        };

        let response = self.client
            .chat()
            .create(request)
            .await
            .context("Failed to get response from OpenAI")?;

        // Extract the response content
        let content = response.choices[0]
            .message
            .content
            .as_ref()
            .context("No content in response")?
            .clone();

        Ok(Message::assistant(&content))
    }

    fn get_token_usage(&self) -> u32 {
        self.last_token_usage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_openai_provider() {
        dotenv().ok(); // Load .env file if present

        let provider = OpenAIProvider::new(None).unwrap();
        let messages = vec![Message::user("Hello!")];
        
        let response = provider.generate(&messages).await.unwrap();
        assert!(!response.text().is_empty());
    }
}