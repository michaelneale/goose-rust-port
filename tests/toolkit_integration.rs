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
        "test_tool",
        "A test tool",
        json!({
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

    let toolkit = TestToolkit {
        tools: vec![tool.clone()],
    };

    // Test system prompt
    assert_eq!(toolkit.system(), "Test toolkit for integration testing");

    // Test tool validation
    assert!(!tool.validate_parameters(&json!({})));
    assert!(tool.validate_parameters(&json!({"test_param": "test"})));

    // Test tool processing
    let result = toolkit.process_tool(&tool).await?;
    assert!(!result.text().is_empty());

    Ok(())
}