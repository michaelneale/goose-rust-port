use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    Text { text: String },
    ToolUse { 
        tool_call_id: String,
        name: String,
        parameters: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        output: String,
        is_error: bool,
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Text { text } => write!(f, "{}", text),
            Content::ToolUse { name, parameters, .. } => {
                write!(f, "Tool use: {} with parameters: {}", name, parameters)
            },
            Content::ToolResult { output, is_error, .. } => {
                if *is_error {
                    write!(f, "Tool error: {}", output)
                } else {
                    write!(f, "Tool result: {}", output)
                }
            }
        }
    }
}

