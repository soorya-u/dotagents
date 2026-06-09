use std::{fs, path::Path, path::PathBuf};

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

/// Deletes a file; returns an error if deletion fails for any reason other than not found.
pub fn delete_file(path: &Path) -> Result<()> {
    fs::remove_file(path).with_context(|| format!("failed to delete {}", path.display()))
}

/// Removes the immediate parent directory of `path` if it is empty after the file was deleted.
pub fn prune_empty_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && parent.is_dir()
    {
        let is_empty = fs::read_dir(parent)
            .map(|mut d| d.next().is_none())
            .unwrap_or(false);
        if is_empty {
            // Best-effort: ignore errors (race conditions, permissions, non-empty).
            let _ = fs::remove_dir(parent);
        }
    }
    Ok(())
}

/// Creates a symbolic link at `target` pointing to `source`.
/// Creates parent directories if needed and overwrites the target if it already exists.
#[cfg(unix)]
pub fn write_symlink(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("unable to create parent dirs for {}", target.display()))?;
    }
    if target.exists() || target.is_symlink() {
        fs::remove_file(target)
            .with_context(|| format!("unable to remove existing target {}", target.display()))?;
    }
    std::os::unix::fs::symlink(source, target).with_context(|| {
        format!(
            "unable to symlink {} -> {}",
            source.display(),
            target.display()
        )
    })
}

/// Creates a symbolic link at `target` pointing to `source`.
/// Creates parent directories if needed and overwrites the target if it already exists.
#[cfg(windows)]
pub fn write_symlink(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("unable to create parent dirs for {}", target.display()))?;
    }
    if target.exists() || target.is_symlink() {
        fs::remove_file(target)
            .with_context(|| format!("unable to remove existing target {}", target.display()))?;
    }
    if source.is_dir() {
        std::os::windows::fs::symlink_dir(source, target)
    } else {
        std::os::windows::fs::symlink_file(source, target)
    }
    .with_context(|| {
        format!(
            "unable to symlink {} -> {}",
            source.display(),
            target.display()
        )
    })
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

    #[test]
    fn test_delete_file_existing() {
        // deletes an existing file without error
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("to_delete.txt");
        fs::write(&file_path, "content").unwrap();
        assert!(file_path.exists());
        delete_file(&file_path).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_file_missing_returns_error() {
        // returns an error when file does not exist
        let path = PathBuf::from("/nonexistent/path/ghost.txt");
        assert!(delete_file(&path).is_err());
    }

    #[test]
    fn test_prune_empty_dir_removes_empty_parent() {
        // removes parent directory when it becomes empty
        let temp_dir = TempDir::new().unwrap();
        let sub = temp_dir.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();
        let file_path = sub.join("only_file.txt");
        fs::write(&file_path, "x").unwrap();
        fs::remove_file(&file_path).unwrap();
        prune_empty_dir(&file_path).unwrap();
        assert!(!sub.exists(), "empty parent should have been pruned");
    }

    #[test]
    fn test_prune_empty_dir_leaves_non_empty_parent() {
        // leaves parent directory when it still has other files
        let temp_dir = TempDir::new().unwrap();
        let sub = temp_dir.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();
        let file_a = sub.join("a.txt");
        let file_b = sub.join("b.txt");
        fs::write(&file_a, "a").unwrap();
        fs::write(&file_b, "b").unwrap();
        fs::remove_file(&file_a).unwrap();
        prune_empty_dir(&file_a).unwrap();
        assert!(sub.exists(), "non-empty parent should remain");
        assert!(file_b.exists(), "sibling file should remain");
    }
}
