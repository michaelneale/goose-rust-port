use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Serialize, Deserialize};

pub const GOOSE_GLOBAL_PATH: &str = "~/.config/goose";
pub const PROFILES_CONFIG_PATH: &str = "~/.config/goose/profiles.yaml";
pub const SESSIONS_PATH: &str = "~/.config/goose/sessions";
pub const SESSION_FILE_SUFFIX: &str = ".jsonl";
pub const LOG_PATH: &str = "~/.config/goose/logs";
pub const RECOMMENDED_DEFAULT_PROVIDER: &str = "openai";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    // TODO: Define profile structure
}

pub fn session_path(name: &str) -> PathBuf {
    let mut path: PathBuf = shellexpand::tilde(SESSIONS_PATH).into_owned().into();
    std::fs::create_dir_all(&path).unwrap();
    path.push(format!("{}{}", name, SESSION_FILE_SUFFIX));
    path
}

pub fn write_config(profiles: &std::collections::HashMap<String, Profile>) -> Result<()> {
    let config_path = shellexpand::tilde(PROFILES_CONFIG_PATH).into_owned();
    let config_dir = Path::new(&config_path).parent().unwrap();
    std::fs::create_dir_all(config_dir)?;
    
    let yaml = serde_yaml::to_string(profiles)?;
    std::fs::write(config_path, yaml)?;
    Ok(())
}

pub fn ensure_config(name: Option<&str>) -> Result<(String, Profile)> {
    let default_profile_name = "default".to_string();
    let name = name.map(|s| s.to_string()).unwrap_or(default_profile_name.clone());
    
    // TODO: Load plugins and get default model configuration
    let provider = RECOMMENDED_DEFAULT_PROVIDER;
    let processor = "gpt-4";  // TODO: Get from provider
    let accelerator = "none";  // TODO: Get from provider
    
    let default_profile = Profile {
        // TODO: Create default profile
    };
    
    let config_path = shellexpand::tilde(PROFILES_CONFIG_PATH).into_owned();
    if !Path::new(&config_path).exists() {
        println!("No configuration present, we will create a profile '{}' at: {}\n\
                 You can add your own profile in this file to further configure goose!", 
                name, config_path);
        let mut profiles = std::collections::HashMap::new();
        profiles.insert(name.clone(), default_profile.clone());
        write_config(&profiles)?;
        return Ok((name, default_profile));
    }
    
    let mut profiles = read_config()?;
    if let Some(profile) = profiles.get(&name) {
        Ok((name, profile.clone()))
    } else {
        println!("Your configuration doesn't have a profile named '{}', adding one now", name);
        profiles.insert(name.clone(), default_profile.clone());
        write_config(&profiles)?;
        Ok((name, default_profile))
    }
}

pub fn read_config() -> Result<std::collections::HashMap<String, Profile>> {
    let config_path = shellexpand::tilde(PROFILES_CONFIG_PATH).into_owned();
    let content = std::fs::read_to_string(config_path)?;
    let profiles: std::collections::HashMap<String, Profile> = serde_yaml::from_str(&content)?;
    Ok(profiles)
}

pub fn default_model_configuration() -> (String, String, String) {
    // TODO: Load and check providers
    (
        RECOMMENDED_DEFAULT_PROVIDER.to_string(),
        "gpt-4".to_string(),  // TODO: Get from provider
        "none".to_string(),   // TODO: Get from provider
    )
}