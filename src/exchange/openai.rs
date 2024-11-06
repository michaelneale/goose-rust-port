use std::env;
use std::sync::atomic::{AtomicU32, Ordering};
use anyhow::{Context, Result};
use async_openai::{
    config::{Config, OpenAIConfig},
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPart, 
        CreateChatCompletionRequest, Role,
        ChatCompletionRequestUserMessage, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestSystemMessage,
    },
    Client,
};
use log::{debug};

use crate::exchange::Provider;
use crate::models::Message;

// Configuration options for OpenAI provider
#[derive(Debug, Clone)]
pub struct OpenAIOptions {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u16,
    pub system_prompt: Option<String>,
}

impl Default for OpenAIOptions {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            system_prompt: None,
        }
    }
}

pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    options: OpenAIOptions,
    last_token_usage: AtomicU32,
}

impl OpenAIProvider {
    pub fn new(options: Option<OpenAIOptions>) -> Result<Self> {
        // Check for API key
        let api_key = env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable not set")?;

        let config = OpenAIConfig::new().with_api_key(api_key);
        
        Ok(Self {
            client: Client::with_config(config),
            options: options.unwrap_or_default(),
            last_token_usage: AtomicU32::new(0),
        })
    }

    fn convert_message_to_openai(message: &Message) -> ChatCompletionRequestMessage {
        match message.role {
            crate::models::message::Role::User => {
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: Some(vec![ChatCompletionRequestMessageContentPart::Text(message.text().into())].into()),
                        name: None,
                        role: Role::User,
                    }
                )
            }
            crate::models::message::Role::Assistant => {
                ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessage {
                        content: Some(message.text()),
                        name: None,
                        role: Role::Assistant,
                        function_call: None,
                        tool_calls: None,
                    }
                )
            }
        }
    }

    fn create_system_message(&self) -> Option<ChatCompletionRequestMessage> {
        self.options.system_prompt.as_ref().map(|prompt| {
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: Some(prompt.clone()),
                    name: None,
                    role: Role::System,
                }
            )
        })
    }
}

#[async_trait::async_trait]
impl Provider for OpenAIProvider {
    async fn initialize(&mut self) -> Result<()> {
        debug!("Initializing OpenAI provider with model: {}", self.options.model);
        Ok(())
    }
    
    async fn generate(&self, messages: &[Message]) -> Result<Message> {
        let mut openai_messages = Vec::new();
        
        // Add system message if configured
        if let Some(system_msg) = self.create_system_message() {
            openai_messages.push(system_msg);
        }

        // Add conversation history
        openai_messages.extend(
            messages.iter()
                .map(Self::convert_message_to_openai)
        );

        let request = CreateChatCompletionRequest {
            model: self.options.model.clone(),
            messages: openai_messages,
            temperature: Some(self.options.temperature),
            max_tokens: Some(self.options.max_tokens),
            ..Default::default()
        };

        debug!("Sending request to OpenAI API");
        let response = self.client
            .chat()
            .create(request)
            .await
            .context("Failed to get response from OpenAI")?;

        // Update token usage tracking
        if let Some(usage) = response.usage {
            self.last_token_usage.store(usage.total_tokens, Ordering::SeqCst);
            debug!("Token usage for request: {}", usage.total_tokens);
        }

        // Extract the response content
        let content = response.choices[0]
            .message
            .content
            .as_ref()
            .context("No content in response")?
            .clone();

        debug!("Received response from OpenAI API");
        Ok(Message::assistant(&content))
    }

    fn get_token_usage(&self) -> u32 {
        self.last_token_usage.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_openai_conversation() -> Result<()> {
        // Load environment variables
        dotenv().ok();

        // Create provider with test options
        let options = OpenAIOptions {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            system_prompt: Some("You are a helpful assistant.".to_string()),
        };
        let provider = OpenAIProvider::new(Some(options)).unwrap();
        
        // Test a simple conversation
        let messages = vec![Message::user("Hello!")];
        
        let response = provider.generate(&messages).await?;
        assert!(!response.text().is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_message_conversion() {
        let user_msg = Message::user("Hello");
        let assistant_msg = Message::assistant("Hi there");

        let openai_user = OpenAIProvider::convert_message_to_openai(&user_msg);
        let openai_assistant = OpenAIProvider::convert_message_to_openai(&assistant_msg);

        match openai_user {
            ChatCompletionRequestMessage::User(msg) => {
                assert_eq!(msg.role, Role::User);
                assert!(msg.content.is_some());
            }
            _ => panic!("Expected User message"),
        }

        match openai_assistant {
            ChatCompletionRequestMessage::Assistant(msg) => {
                assert_eq!(msg.role, Role::Assistant);
                assert_eq!(msg.content.unwrap(), "Hi there");
            }
            _ => panic!("Expected Assistant message"),
        }
    }
}