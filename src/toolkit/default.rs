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
        match tool_call.name.as_str() {
            "bash" => {
                let params = tool_call.parameters.as_object()
                    .ok_or_else(|| anyhow::anyhow!("Invalid parameters for bash tool"))?;
                
                let working_dir = params.get("working_dir")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                let source_path = params.get("source_path")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                let command = params.get("command")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                // At least one parameter must be provided
                if working_dir.is_none() && source_path.is_none() && command.is_none() {
                    return Err(anyhow::anyhow!("At least one parameter must be provided for bash tool"));
                }
                
                let mut cmd = std::process::Command::new("bash");
                cmd.arg("-c");
                
                let mut script = String::new();
                
                if let Some(dir) = working_dir {
                    script.push_str(&format!("cd \"{}\" && ", dir));
                }
                
                if let Some(path) = source_path {
                    script.push_str(&format!("source \"{}\" && ", path));
                }
                
                if let Some(cmd_str) = command {
                    script.push_str(&cmd_str);
                }
                
                let output = cmd.arg(script)
                    .output()
                    .map_err(|e| anyhow::anyhow!("Failed to execute bash command: {}", e))?;
                
                let mut result = String::new();
                
                if !output.stdout.is_empty() {
                    result.push_str(&String::from_utf8_lossy(&output.stdout));
                }
                
                if !output.stderr.is_empty() {
                    if !result.is_empty() {
                        result.push_str("\n");
                    }
                    result.push_str(&String::from_utf8_lossy(&output.stderr));
                }
                
                Ok(Message::assistant(&result))
            },
            
            "text_editor" => {
                let params = tool_call.parameters.as_object()
                    .ok_or_else(|| anyhow::anyhow!("Invalid parameters for text_editor tool"))?;
                
                let command = params.get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing command parameter"))?;
                
                let path = params.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
                
                match command {
                    "view" => {
                        let content = std::fs::read_to_string(path)
                            .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
                        
                        if let Some(range) = params.get("view_range")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_i64())
                                .collect::<Vec<_>>()) 
                        {
                            if range.len() == 2 {
                                let lines: Vec<&str> = content.lines().collect();
                                let start = (range[0] - 1).max(0) as usize;
                                let end = range[1].min(lines.len() as i64) as usize;
                                
                                Ok(Message::assistant(&lines[start..end].join("\n")))
                            } else {
                                Err(anyhow::anyhow!("view_range must contain exactly 2 numbers"))
                            }
                        } else {
                            Ok(Message::assistant(&content))
                        }
                    },
                    
                    "create" => {
                        let content = params.get("file_text")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| anyhow::anyhow!("Missing file_text parameter"))?;
                        
                        std::fs::write(path, content)
                            .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
                        
                        Ok(Message::assistant(&format!("Created file {}", path)))
                    },
                    
                    "str_replace" => {
                        let old_str = params.get("old_str")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| anyhow::anyhow!("Missing old_str parameter"))?;
                        
                        let new_str = params.get("new_str")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        
                        let content = std::fs::read_to_string(path)
                            .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
                        
                        let new_content = content.replace(old_str, new_str);
                        
                        std::fs::write(path, new_content)
                            .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
                        
                        Ok(Message::assistant(&format!("Replaced '{}' with '{}' in {}", old_str, new_str, path)))
                    },
                    
                    "insert" => {
                        let new_str = params.get("new_str")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| anyhow::anyhow!("Missing new_str parameter"))?;
                        
                        let insert_line = params.get("insert_line")
                            .and_then(|v| v.as_i64())
                            .ok_or_else(|| anyhow::anyhow!("Missing or invalid insert_line parameter"))?;
                        
                        let content = std::fs::read_to_string(path)
                            .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
                        
                        let mut lines: Vec<String> = content.lines().map(String::from).collect();
                        if insert_line as usize > lines.len() {
                            return Err(anyhow::anyhow!("insert_line is beyond end of file"));
                        }
                        
                        lines.insert(insert_line as usize, new_str.to_string());
                        let new_content = lines.join("\n");
                        
                        std::fs::write(path, new_content)
                            .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
                        
                        Ok(Message::assistant(&format!("Inserted '{}' after line {} in {}", new_str, insert_line, path)))
                    },
                    
                    "undo_edit" => {
                        // TODO: Implement undo functionality
                        Err(anyhow::anyhow!("Undo functionality not yet implemented"))
                    },
                    
                    _ => Err(anyhow::anyhow!("Unknown text_editor command: {}", command))
                }
            },
            
            "fetch_web_content" => {
                // TODO: Implement web content fetching
                Err(anyhow::anyhow!("Web content fetching not yet implemented"))
            },
            
            "process_manager" => {
                // TODO: Implement process management
                Err(anyhow::anyhow!("Process management not yet implemented"))
            },
            
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_call.name))
        }
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
        
        // Test bash tool with echo command
        let tool = Tool::new(
            "bash",
            "Test bash command",
            serde_json::json!({
                "command": "echo 'test'"
            }),
            vec!["command".to_string()],
        );
        
        let result = toolkit.process_tool(&tool).await.unwrap();
        assert_eq!(result.text().trim(), "test");
    }
}