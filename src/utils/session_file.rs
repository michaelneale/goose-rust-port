use std::path::Path;
use anyhow::Result;
use serde::{Serialize, Deserialize};

pub const SESSION_FILE_SUFFIX: &str = ".session.jsonl";

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    // TODO: Define message structure based on exchange.Message
}

pub fn is_existing_session(path: &Path) -> bool {
    path.is_file() && path.metadata().map(|m| m.len() > 0).unwrap_or(false)
}

pub fn is_empty_session(path: &Path) -> bool {
    path.is_file() && path.metadata().map(|m| m.len() == 0).unwrap_or(false)
}

pub fn read_or_create_file(file_path: &Path) -> Result<Vec<Message>> {
    if file_path.exists() {
        read_from_file(file_path)
    } else {
        std::fs::write(file_path, "")?;
        Ok(vec![])
    }
}

pub fn read_from_file(file_path: &Path) -> Result<Vec<Message>> {
    let content = std::fs::read_to_string(file_path)?;
    let messages = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(messages)
}

pub fn list_sorted_session_files(session_files_directory: &Path) -> Result<Vec<(String, std::path::PathBuf)>> {
    let mut sessions = list_session_files(session_files_directory)?
        .into_iter()
        .map(|path| (path.file_stem().unwrap().to_string_lossy().into_owned(), path))
        .collect::<Vec<_>>();
    
    sessions.sort_by(|a, b| {
        let a_modified = a.1.metadata().unwrap().modified().unwrap();
        let b_modified = b.1.metadata().unwrap().modified().unwrap();
        b_modified.cmp(&a_modified)
    });
    
    Ok(sessions)
}

pub fn list_session_files(session_files_directory: &Path) -> Result<Vec<std::path::PathBuf>> {
    Ok(std::fs::read_dir(session_files_directory)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "jsonl").unwrap_or(false))
        .collect())
}

pub fn session_file_exists(session_files_directory: &Path) -> bool {
    if !session_files_directory.exists() {
        return false;
    }
    list_session_files(session_files_directory)
        .map(|files| !files.is_empty())
        .unwrap_or(false)
}

use std::io::Write;

pub fn log_messages(file_path: &Path, messages: &[Message]) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;
    
    for message in messages {
        serde_json::to_writer(&mut file, message)?;
        writeln!(file)?;
    }
    Ok(())
}