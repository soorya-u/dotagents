use std::{fs::read_to_string, path::PathBuf};

use anyhow::Result;

pub fn read_file(file_path: PathBuf) -> Result<String> {
    match read_to_string(file_path) {
        Ok(f) => Ok(f),
        Err(e) => Err(e.into()),
    }
}
