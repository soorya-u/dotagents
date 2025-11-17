use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

pub fn read_file(file_path: &PathBuf) -> Result<String> {
    match fs::read_to_string(file_path) {
        Ok(f) => Ok(f),
        Err(e) => Err(e.into()),
    }
}

pub fn write_file(file_path: &PathBuf, content: &str) -> Result<()> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::write(file_path, content) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
