use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

/// Computes SHA-256 hash of the given string content and returns hex string.
pub(crate) fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Computes SHA-256 hash of a file's contents; returns None if file doesn't exist.
pub(crate) fn hash_file(path: &PathBuf) -> Result<Option<String>> {
    match fs::read(path) {
        Ok(content) => {
            let mut hasher = Sha256::new();
            hasher.update(&content);
            Ok(Some(hex::encode(hasher.finalize())))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {

    use super::super::fs::write_file;
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hash_content() {
        let content1 = "Hello, World!";
        let content2 = "Hello, World!";
        let content3 = "Different";

        let hash1 = hash_content(content1);
        let hash2 = hash_content(content2);
        let hash3 = hash_content(content3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA-256 hex is 64 chars
    }

    #[test]
    fn test_hash_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("hash_test.txt");
        let content = "Test content for hashing";

        write_file(&file_path, content).unwrap();

        let hash_result = hash_file(&file_path).unwrap();
        assert!(hash_result.is_some());
        let hash = hash_result.unwrap();
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, hash_content(content));
    }

    #[test]
    fn test_hash_file_not_found() {
        let file_path = PathBuf::from("/nonexistent/path/file.txt");
        let result = hash_file(&file_path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_hash_file_empty() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty_hash.txt");

        write_file(&file_path, "").unwrap();

        let hash_result = hash_file(&file_path).unwrap();
        assert!(hash_result.is_some());
        let hash = hash_result.unwrap();
        assert_eq!(hash, hash_content(""));
    }
}
