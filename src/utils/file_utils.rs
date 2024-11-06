use std::path::Path;
use anyhow::Result;

pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn read_file_to_string(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

pub fn write_file(path: &Path, content: &str) -> Result<()> {
    Ok(std::fs::write(path, content)?)
}