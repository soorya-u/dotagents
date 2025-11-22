use anyhow::{Context, Result, anyhow};
use std::env;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use crate::constants::dir::ROOT_DIR;

fn get_dir_or_die(path: PathBuf) -> Result<PathBuf> {
    if path.is_dir() {
        Ok(path)
    } else {
        Err(anyhow!(format!(
            "{} is not a directory or needs permission",
            path.to_str().unwrap()
        )))
    }
}

pub fn get_workspace_dir() -> Result<PathBuf> {
    let mut current = env::current_dir().context("failed to get current directory")?;

    loop {
        let marker = current.join(ROOT_DIR);

        if marker.is_dir() {
            return Ok(current);
        }

        if !current.pop() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("No `{}` directory found in any parent directory", ROOT_DIR),
            )
            .into());
        }
    }
}

pub fn get_home_dir() -> Result<PathBuf> {
    home::home_dir().ok_or_else(|| anyhow!("failed to get user home directory"))
}

// TODO: Valid only for Unix as of Now. Make Win Compatible
pub fn get_config_dir() -> Result<PathBuf> {
    let home_dir = get_home_dir()?;
    let config_dir = home_dir.join(".config");
    get_dir_or_die(config_dir)
}

pub fn get_application_dir() -> Result<PathBuf> {
    let app_dir = get_workspace_dir()?.join(ROOT_DIR);
    get_dir_or_die(app_dir)
}

pub fn get_commands_dir() -> Result<PathBuf> {
    let commands_dir = get_application_dir()?.join("commands");
    get_dir_or_die(commands_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_dir_or_die_valid_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = get_dir_or_die(temp_dir.path().to_path_buf());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_get_dir_or_die_file_not_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "content").unwrap();

        let result = get_dir_or_die(file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_dir_or_die_nonexistent() {
        let nonexistent = PathBuf::from("/nonexistent/directory");
        let result = get_dir_or_die(nonexistent);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_workspace_dir_with_marker() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path();
        let root_marker = workspace.join(ROOT_DIR);
        fs::create_dir(&root_marker).unwrap();

        // Change to a nested directory
        let nested = workspace.join("nested").join("deep");
        fs::create_dir_all(&nested).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&nested).unwrap();

        let result = get_workspace_dir();

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), workspace);
    }

    #[test]
    fn test_get_workspace_dir_no_marker() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path().join("workspace");
        fs::create_dir(&workspace).unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&workspace).unwrap();

        let result = get_workspace_dir();

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();

        assert!(result.is_err());
    }

    #[test]
    fn test_get_home_dir() {
        let result = get_home_dir();
        // Home directory should exist on all systems
        assert!(result.is_ok());
        let home = result.unwrap();
        assert!(home.exists());
        assert!(home.is_dir());
    }

    #[test]
    #[cfg(unix)]
    fn test_get_config_dir() {
        let result = get_config_dir();
        // Config dir should exist on Unix systems
        if result.is_ok() {
            let config = result.unwrap();
            assert!(config.ends_with(".config"));
            assert!(config.is_dir());
        }
    }
}
