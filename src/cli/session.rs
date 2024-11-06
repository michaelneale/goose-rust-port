use std::path::PathBuf;
use anyhow::Result;
use chrono::DateTime;
use crate::utils::session_file::{Message, read_or_create_file, log_messages};
use crate::cli::config::{session_path, ensure_config};

pub struct Session {
    name: String,
    profile_name: Option<String>,
    tracing: bool,
    session_file_path: PathBuf,
    messages: Vec<Message>,
}

impl Session {
    pub fn new(
        name: Option<String>, 
        profile: Option<String>,
        plan: Option<serde_yaml::Value>,
        log_level: Option<String>,
        tracing: bool,
    ) -> Result<Self> {
        let name = name.unwrap_or_else(|| generate_name());
        let session_file_path = session_path(&name);
        
        let mut session = Session {
            name,
            profile_name: profile,
            tracing,
            session_file_path,
            messages: Vec::new(),
        };

        session.messages.extend(session.load_session()?);

        if session.messages.is_empty() && plan.is_some() {
            session.setup_plan(plan.unwrap())?;
        }

        Ok(session)
    }

    pub fn run(&mut self, new_session: bool) -> Result<()> {
        let time_start = chrono::Utc::now();
        
        let profile_name = self.profile_name.as_deref().unwrap_or("default");
        println!("starting session | name: {} profile: {}", self.name, profile_name);
        println!("saving to {}", self.session_file_path.display());

        // TODO: Implement main session loop
        
        let time_end = chrono::Utc::now();
        self.log_cost(time_start, time_end)?;
        
        Ok(())
    }

    pub fn single_pass(&mut self, initial_message: String) -> Result<()> {
        let profile = self.profile_name.as_deref().unwrap_or("default");
        println!("starting session | name: {} profile: {}", self.name, profile);
        println!("saving to {}", self.session_file_path.display());

        // TODO: Add message and process response
        
        println!("ended run | name: {} profile: {}", self.name, profile);
        println!("to resume: goose session resume {} --profile {}", self.name, profile);
        
        Ok(())
    }

    fn load_session(&self) -> Result<Vec<Message>> {
        read_or_create_file(&self.session_file_path)
    }

    fn setup_plan(&mut self, plan: serde_yaml::Value) -> Result<()> {
        if !self.messages.is_empty() {
            return Err(anyhow::anyhow!("The plan can only be set on an empty session."));
        }

        // TODO: Implement plan setup
        Ok(())
    }

    fn log_cost(&self, start_time: DateTime<chrono::Utc>, end_time: DateTime<chrono::Utc>) -> Result<()> {
        // TODO: Implement cost logging
        Ok(())
    }
}

fn generate_name() -> String {
    crate::utils::generate_name()
}