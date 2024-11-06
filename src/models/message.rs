use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
}

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
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub id: String,
    pub created: u64,
    pub content: Vec<Content>,
}

impl Message {
    pub fn new(role: Role, content: Vec<Content>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            role,
            id: format!("msg_{}", uuid::Uuid::new_v4()),
            created: timestamp,
            content,
        }
    }

    pub fn user(text: &str) -> Self {
        Self::new(
            Role::User,
            vec![Content::Text { text: text.to_string() }],
        )
    }

    pub fn assistant(text: &str) -> Self {
        Self::new(
            Role::Assistant,
            vec![Content::Text { text: text.to_string() }],
        )
    }

    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|content| {
                if let Content::Text { text } = content {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn tool_use(&self) -> Vec<&Content> {
        self.content
            .iter()
            .filter(|content| matches!(content, Content::ToolUse { .. }))
            .collect()
    }

    pub fn tool_result(&self) -> Vec<&Content> {
        self.content
            .iter()
            .filter(|content| matches!(content, Content::ToolResult { .. }))
            .collect()
    }

    pub fn is_user(&self) -> bool {
        matches!(self.role, Role::User)
    }

    pub fn is_assistant(&self) -> bool {
        matches!(self.role, Role::Assistant)
    }

    pub fn has_tool_use(&self) -> bool {
        self.content.iter().any(|c| matches!(c, Content::ToolUse { .. }))
    }

    pub fn validate(&self) -> Result<()> {
        match self.role {
            Role::User => {
                if !self.content.iter().any(|c| matches!(c, Content::Text { .. } | Content::ToolResult { .. })) {
                    anyhow::bail!("User message must include a Text or ToolResult");
                }
                if self.content.iter().any(|c| matches!(c, Content::ToolUse { .. })) {
                    anyhow::bail!("User message does not support ToolUse");
                }
            }
            Role::Assistant => {
                if !self.content.iter().any(|c| matches!(c, Content::Text { .. } | Content::ToolUse { .. })) {
                    anyhow::bail!("Assistant message must include a Text or ToolUse");
                }
                if self.content.iter().any(|c| matches!(c, Content::ToolResult { .. })) {
                    anyhow::bail!("Assistant message does not support ToolResult");
                }
            }
        }
        Ok(())
    }

    pub fn summary(&self) -> String {
        let role = match self.role {
            Role::User => "user",
            Role::Assistant => "assistant",
        };
        
        format!("message:{}\n{}", role, self.text())
    }
}