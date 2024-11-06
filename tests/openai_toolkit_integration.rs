use anyhow::Result;
use dotenv::dotenv;
use rust_goose::{
    exchange::{OpenAIProvider, OpenAIOptions, Provider},
    models::Message,
    toolkit::{Tool, Toolkit},
};
use serde_json::json;

// Test toolkit implementation
#[derive(Debug)]
struct TestToolkit {
    tools: Vec<Tool>,
}

#[async_trait::async_trait]
impl Toolkit for TestToolkit {
    fn system(&self) -> String {
        "Test toolkit for OpenAI integration testing".to_string()
    }

    fn tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    async fn process_tool(&self, tool_call: &Tool) -> Result<Message> {
        // Echo implementation for testing
        let params = tool_call.parameters.as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid parameters"))?;
        
        let result = format!("Processed tool {} with params: {:?}", tool_call.name, params);
        Ok(Message::assistant(&result))
    }
}

// Helper to create test provider
async fn create_test_provider() -> Result<OpenAIProvider> {
    dotenv().ok();
    
    let options = OpenAIOptions {
        model: "gpt-4".to_string(),
        temperature: 0.7,
        max_tokens: 2048,
        system_prompt: Some("You are a helpful assistant that uses tools.".to_string()),
    };
    
    let provider = OpenAIProvider::new(Some(options))?;
    Ok(provider)
}

#[tokio::test]
async fn test_openai_toolkit_basic() -> Result<()> {
    // Create a test tool
    let tool = Tool::new(
        "echo",
        "A test tool that echoes input",
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo"
                }
            },
            "required": ["message"]
        }),
        vec!["message".to_string()],
    );

    let toolkit = TestToolkit {
        tools: vec![tool],
    };

    let provider = create_test_provider().await?;
    
    // Test conversation prompting tool use
    let messages = vec![
        Message::user("Please use the echo tool to say hello"),
    ];
    
    let response = provider.generate(&messages, Some(toolkit.tools())).await?;
    
    // Response should either be a tool call or contain content
    assert!(!response.text().is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_openai_toolkit_multiple_tools() -> Result<()> {
    // Create multiple test tools
    let tools = vec![
        Tool::new(
            "greet",
            "A greeting tool",
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name to greet"
                    }
                },
                "required": ["name"]
            }),
            vec!["name".to_string()],
        ),
        Tool::new(
            "calculate",
            "A simple calculator",
            json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "description": "Operation to perform",
                        "enum": ["add", "subtract", "multiply", "divide"]
                    },
                    "numbers": {
                        "type": "array",
                        "items": {"type": "number"},
                        "description": "Numbers to operate on"
                    }
                },
                "required": ["operation", "numbers"]
            }),
            vec!["operation".to_string(), "numbers".to_string()],
        ),
    ];

    let toolkit = TestToolkit { tools };
    let provider = create_test_provider().await?;
    
    // Test conversation requiring tool selection
    let messages = vec![
        Message::user("Please greet Alice and then calculate 2 + 2"),
    ];
    
    let response = provider.generate(&messages, Some(toolkit.tools())).await?;
    assert!(!response.text().is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_openai_toolkit_error_handling() -> Result<()> {
    // Create a tool with required parameters
    let tool = Tool::new(
        "validate",
        "A validation tool",
        json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input to validate"
                },
                "rules": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Validation rules to apply"
                }
            },
            "required": ["input", "rules"]
        }),
        vec!["input".to_string(), "rules".to_string()],
    );

    let toolkit = TestToolkit {
        tools: vec![tool],
    };

    let provider = create_test_provider().await?;
    
    // Test conversation with invalid tool usage
    let messages = vec![
        Message::user("Use the validate tool without any parameters"),
    ];
    
    let response = provider.generate(&messages, Some(toolkit.tools())).await?;
    
    // Response should indicate parameter validation or contain an error message
    assert!(!response.text().is_empty());
    
    Ok(())
}