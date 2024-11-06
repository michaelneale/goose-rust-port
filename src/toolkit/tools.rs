use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub required: Vec<String>,
}

impl Tool {
    pub fn new(name: &str, description: &str, parameters: Value, required: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
            required,
        }
    }

    pub fn validate_parameters(&self, params: &Value) -> bool {
        // Check that all required parameters are present
        for req in &self.required {
            if !params.get(req).is_some() {
                return false;
            }
        }
        true
    }
}