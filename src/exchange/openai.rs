use std::env;
use std::sync::atomic::{AtomicU32, Ordering};
use anyhow::{Context, Result};
use async_openai::{
    config::{Config, OpenAIConfig},
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPart, 
        CreateChatCompletionRequest, Role,
        ChatCompletionRequestUserMessage, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionTool,
        ChatCompletionFunctions,
    },
    Client,
};
use log::debug;

use crate::exchange::Provider;
use crate::models::Message;
use crate::toolkit::Tool;

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
    
    async fn generate(&self, messages: &[Message], tools: Option<Vec<Tool>>) -> Result<Message> {
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

        let mut request = CreateChatCompletionRequest {
            model: self.options.model.clone(),
            messages: openai_messages,
            temperature: Some(self.options.temperature),
            max_tokens: Some(self.options.max_tokens),
            ..Default::default()
        };

        // Add tools if provided
        if let Some(tools) = tools {
            request.tools = Some(tools.into_iter().map(|tool| {
                ChatCompletionTool {
                    r#type: async_openai::types::ChatCompletionToolType::Function,
                    function: ChatCompletionFunctions {
                        name: tool.name,
                        description: Some(tool.description),
                        parameters: tool.parameters,
                    },
                }
            }).collect());
        }

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

        // Extract the response content or tool calls
        let message = &response.choices[0].message;
        
        if let Some(tool_calls) = &message.tool_calls {
            debug!("Received tool call response from OpenAI API");
            // Convert tool calls into our message format
            let mut content = String::new();
            for tool_call in tool_calls {
                content.push_str(&format!("Tool call: {}\nParameters: {}\n", 
                    tool_call.function.name,
                    tool_call.function.arguments));
            }
            Ok(Message::assistant(&content))
        } else if let Some(content) = &message.content {
            debug!("Received text response from OpenAI API");
            Ok(Message::assistant(content))
        } else {
            Err(anyhow::anyhow!("Response contained neither content nor tool calls"))
        }
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
        
        let response = provider.generate(&messages, None).await?;
        assert!(!response.text().is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_openai_tool_response() -> Result<()> {
        dotenv().ok();

        let options = OpenAIOptions {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            system_prompt: None,
        };
        let provider = OpenAIProvider::new(Some(options)).unwrap();

        // Create a test tool
        let tool = Tool::new(
            "test_tool",
            "A test tool",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "test_param": {
                        "type": "string",
                        "description": "A test parameter"
                    }
                }
            }),
            vec!["test_param".to_string()],
        );

        // Test conversation with tool
        let messages = vec![Message::user("Use the test tool")];
        let response = provider.generate(&messages, Some(vec![tool])).await?;
        
        // Response should contain either content or tool call info
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