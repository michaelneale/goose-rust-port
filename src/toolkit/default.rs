use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

use crate::models::Message;
use super::{Tool, Toolkit};

/// Provides the default set of tools that are always available
#[derive(Debug)]
pub struct DefaultToolkit {
    tools: Vec<Tool>,
}

impl DefaultToolkit {
    pub fn new() -> Self {
        let tools = vec![
            Tool::new(
                "bash",
                "Run commands in a bash shell. Perform bash-related operations in a specific order: \
                1. Change the working directory (if provided) \
                2. Source a file (if provided) \
                3. Run a shell command (if provided) \
                At least one of the parameters must be provided.",
                json!({
                    "type": "object",
                    "properties": {
                        "working_dir": {
                            "type": "string",
                            "description": "The directory to change to.",
                            "default": null
                        },
                        "source_path": {
                            "type": "string",
                            "description": "The file to source before running the command.",
                            "default": null
                        },
                        "command": {
                            "type": "string",
                            "description": "The bash shell command to run.",
                            "default": null
                        }
                    }
                }),
                vec![], // No required params since they are all optional but at least one needed
            ),
            Tool::new(
                "text_editor",
                "Perform text editing operations on files. The `command` parameter specifies the operation to perform.",
                json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The commands to run.\nAllowed options are: `view`, `create`, `str_replace`, `insert`, `undo_edit`.",
                            "enum": ["view", "create", "str_replace", "insert", "undo_edit"]
                        },
                        "path": {
                            "type": "string",
                            "description": "Absolute path (or relative path against cwd) to file or directory."
                        },
                        "file_text": {
                            "type": "string",
                            "description": "Required parameter of `create` command, with the content\nof the file to be created.",
                            "default": null
                        },
                        "old_str": {
                            "type": "string",
                            "description": "Required parameter of `str_replace` command containing the\nstring in `path` to replace.",
                            "default": null
                        },
                        "new_str": {
                            "type": "string",
                            "description": "Optional parameter of `str_replace` command\ncontaining the new string (if not given, no string will be added).\nRequired parameter of `insert` command containing the string to insert.",
                            "default": null
                        },
                        "insert_line": {
                            "type": "integer",
                            "description": "Required parameter of `insert` command.\nThe `new_str` will be inserted AFTER the line `insert_line` of `path`.",
                            "default": null
                        },
                        "view_range": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "Optional parameter of `view` command when `path` points to a file.\nIf none is given, the full file is shown.",
                            "default": null
                        }
                    },
                    "required": ["command", "path"]
                }),
                vec!["command".to_string(), "path".to_string()],
            ),
            Tool::new(
                "fetch_web_content",
                "Fetches content from a web page and returns paths to files containing the content.",
                json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "url of the site to visit."
                        }
                    },
                    "required": ["url"]
                }),
                vec!["url".to_string()],
            ),
            Tool::new(
                "process_manager",
                "Manage background processes.",
                json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to run.\nAllowed options are: `start`, `list`, `view_output`, `cancel`.",
                            "enum": ["start", "list", "view_output", "cancel"]
                        },
                        "shell_command": {
                            "type": "string",
                            "description": "Required parameter for the `start` command, representing\nthe shell command to be executed in the background.",
                            "default": null
                        },
                        "process_id": {
                            "type": "integer",
                            "description": "Required parameter for `view_output` and `cancel` commands,\nrepresenting the process ID of the background process to manage.",
                            "default": null
                        }
                    },
                    "required": ["command"]
                }),
                vec!["command".to_string()],
            ),
        ];

        Self { tools }
    }
}

#[async_trait]
impl Toolkit for DefaultToolkit {
    fn system(&self) -> String {
        "Default toolkit providing core functionality for file operations, command execution, and web content fetching.".to_string()
    }

    fn tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    async fn process_tool(&self, tool_call: &Tool) -> Result<Message> {
        // The actual tool implementations will be handled by the Session
        // This is just a placeholder that should never be called
        Ok(Message::assistant(&format!(
            "Default toolkit received tool call for {}", 
            tool_call.name
        )))
    }
}

/// Returns a list of default toolkits that should be automatically registered
pub fn get_default_toolkits() -> Vec<Box<dyn Toolkit>> {
    vec![
        Box::new(DefaultToolkit::new())
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_toolkit_creation() {
        let toolkit = DefaultToolkit::new();
        assert!(!toolkit.tools().is_empty());
        
        // Verify each tool has required fields
        for tool in toolkit.tools() {
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
            assert!(tool.parameters.is_object());
        }
    }

    #[tokio::test]
    async fn test_default_toolkit_process() {
        let toolkit = DefaultToolkit::new();
        let tools = toolkit.tools();
        let tool = tools.first().unwrap();
        
        let result = toolkit.process_tool(tool).await.unwrap();
        assert!(result.text().contains(&tool.name));
    }
}