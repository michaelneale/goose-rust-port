use std::fmt::{self, Debug};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use crate::models::Message;
use super::tools::Tool;

#[derive(Debug)]
pub struct ToolkitError {
    pub message: String,
    pub details: Option<String>,
}

impl fmt::Display for ToolkitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(details) = &self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for ToolkitError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolkitResult {
    pub output: String,
    pub is_error: bool,
    pub error_message: Option<String>,
}

impl ToolkitResult {
    pub fn success(output: String) -> Self {
        Self {
            output,
            is_error: false,
            error_message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            output: String::new(),
            is_error: true,
            error_message: Some(message),
        }
    }
}

#[async_trait]
pub trait Toolkit: Send + Sync + Debug {
    /// Get the system prompt for this toolkit
    fn system(&self) -> String {
        String::new()
    }

    /// Get the tools provided by this toolkit
    fn tools(&self) -> Vec<Tool>;

    /// Process a tool call
    async fn process_tool(&self, tool_call: &Tool) -> Result<Message>;
}

pub struct Requirements {
    toolkit: String,
    requirements: std::collections::HashMap<String, Box<dyn Toolkit>>,
}

impl Requirements {
    pub fn new(toolkit: String) -> Self {
        Self {
            toolkit,
            requirements: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, requirement: &str) -> Option<&Box<dyn Toolkit>> {
        self.requirements.get(requirement)
    }
}

impl fmt::Debug for Requirements {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Requirements")
            .field("toolkit", &self.toolkit)
            .field("requirements", &"<dyn Toolkit>")
            .finish()
    }
}