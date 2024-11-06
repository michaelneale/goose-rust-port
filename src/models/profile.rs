use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolkitSpec {
    pub name: String,
    #[serde(default)]
    pub requires: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub provider: String,
    pub processor: String,
    pub accelerator: String,
    pub moderator: String,
    pub toolkits: Vec<ToolkitSpec>,
}

impl Profile {
    pub fn new(
        provider: String,
        processor: String,
        accelerator: String,
        moderator: String,
        toolkits: Vec<ToolkitSpec>,
    ) -> Self {
        Self {
            provider,
            processor,
            accelerator,
            moderator,
            toolkits,
        }
    }

    pub fn validate(&self) -> Result<()> {
        let installed_toolkits: std::collections::HashSet<_> = self.toolkits
            .iter()
            .map(|t| t.name.as_str())
            .collect();

        for toolkit in &self.toolkits {
            for (_, req) in &toolkit.requires {
                if !installed_toolkits.contains(req.as_str()) {
                    anyhow::bail!(
                        "Toolkit {} requires {} but it is not present",
                        toolkit.name,
                        req
                    );
                }
            }
        }
        Ok(())
    }

    pub fn profile_info(&self) -> String {
        let toolkit_names: Vec<_> = self.toolkits.iter()
            .map(|t| t.name.as_str())
            .collect();
        format!(
            "provider:{}, processor:{} toolkits: {}",
            self.provider,
            self.processor,
            toolkit_names.join(", ")
        )
    }
}

pub fn default_profile(
    provider: String,
    processor: String,
    accelerator: String,
) -> Profile {
    Profile::new(
        provider,
        processor,
        accelerator,
        "synopsis".to_string(),
        vec![ToolkitSpec {
            name: "synopsis".to_string(),
            requires: HashMap::new(),
        }],
    )
}