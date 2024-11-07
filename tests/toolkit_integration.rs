use anyhow::Result;
use rust_goose::toolkit::{Tool, Toolkit, ToolkitResult};
use serde_json::json;

#[derive(Debug)]
struct TestToolkit {
    tools: Vec<Tool>,
}

#[async_trait::async_trait]
impl Toolkit for TestToolkit {
    fn system(&self) -> String {
        "Test toolkit for integration testing".to_string()
    }

    fn tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    async fn process_tool(&self, tool_call: &Tool) -> Result<rust_goose::models::Message> {
        // Simple echo implementation for testing
        let params = tool_call.parameters.as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid parameters"))?;
        
        let result = format!("Processed tool {} with params: {:?}", tool_call.name, params);
        Ok(rust_goose::models::Message::assistant(&result))
    }
}

#[tokio::test]
async fn test_toolkit_basic() -> Result<()> {
    let tool = Tool::new(
        "bash",
        "Execute a bash command",
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                }
            }
        }),
        vec!["command".to_string()],
    );

    let toolkit = TestToolkit {
        tools: vec![tool.clone()],
    };

    // Test system prompt
    assert_eq!(toolkit.system(), "Test toolkit for integration testing");

    // Test tool validation
    assert!(!tool.validate_parameters(&json!({})));
    assert!(tool.validate_parameters(&json!({"command": "echo test"})));

    // Test tool processing
    let result = toolkit.process_tool(&tool).await?;
    assert!(!result.text().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_toolkit_bash_execution() -> Result<()> {
    use rust_goose::toolkit::default::DefaultToolkit;

    let tool = Tool::new(
        "bash",
        "Execute a bash command",
        json!({
            "command": "echo 'Hello, World!'"
        }),
        vec!["command".to_string()],
    );

    let toolkit = DefaultToolkit::new();
    let result = toolkit.process_tool(&tool).await?;
    
    assert_eq!(result.text().trim(), "Hello, World!");

    Ok(())
}