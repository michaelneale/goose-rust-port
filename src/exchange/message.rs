use serde::{Serialize, Deserialize};
use crate::models::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    Text { text: String },
    ToolUse { 
        id: String,
        name: String,
        parameters: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        output: String,
        is_error: bool,
    }
}

pub use Content::*;