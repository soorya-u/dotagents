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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello, World!";

        // Test write
        let write_result = write_file(&file_path, content);
        assert!(write_result.is_ok());

        // Test read
        let read_result = read_file(&file_path);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), content);
    }

    #[test]
    fn test_write_file_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nested").join("dirs").join("test.txt");
        let content = "Nested content";

        let write_result = write_file(&file_path, content);
        assert!(write_result.is_ok());
        assert!(file_path.exists());

        let read_result = read_file(&file_path);
        assert_eq!(read_result.unwrap(), content);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let file_path = PathBuf::from("/nonexistent/path/file.txt");
        let result = read_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_overwrite_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("overwrite.txt");

        write_file(&file_path, "First content").unwrap();
        write_file(&file_path, "Second content").unwrap();

        let content = read_file(&file_path).unwrap();
        assert_eq!(content, "Second content");
    }

    #[test]
    fn test_write_empty_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");

        write_file(&file_path, "").unwrap();
        let content = read_file(&file_path).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_write_multiline_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("multiline.txt");
        let content = "Line 1\nLine 2\nLine 3";

        write_file(&file_path, content).unwrap();
        let read_content = read_file(&file_path).unwrap();
        assert_eq!(read_content, content);
    }
}
