use anyhow::{Context, Result, anyhow};
use std::env;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::constants::dir::ROOT_DIR;

static WORKSPACE_DIR: OnceLock<Result<PathBuf, String>> = OnceLock::new();

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

/// Walk up from `start` until a directory containing `ROOT_DIR` is found.
///
/// `boundary` is an optional upper limit for the walk — the search will not
/// ascend past that directory.  Pass `None` for an unbounded walk (production
/// use); pass `Some(start_path)` in tests to confine the walk to a controlled
/// temp directory and avoid flakiness from any real `ROOT_DIR` that might
/// exist in an ancestor of the temp dir.
///
/// Separated from the `OnceLock` wrapper so it can be unit-tested with
/// arbitrary starting paths without touching global state.
fn find_workspace_dir(
    start: PathBuf,
    boundary: Option<&std::path::Path>,
) -> Result<PathBuf, String> {
    let mut current = start;
    loop {
        let marker = current.join(ROOT_DIR);
        if marker.is_dir() {
            return Ok(current);
        }
        if let Some(b) = boundary
            && current == b
        {
            break;
        }
        if !current.pop() {
            break;
        }
    }
    Err(format!(
        "No `{}` directory found in any parent directory",
        ROOT_DIR
    ))
}

pub fn get_workspace_dir() -> Result<PathBuf> {
    WORKSPACE_DIR
        .get_or_init(|| {
            let current = match env::current_dir() {
                Ok(dir) => dir,
                Err(e) => return Err(format!("failed to get current directory: {}", e)),
            };
            find_workspace_dir(current, None)
        })
        .clone()
        .map_err(|e| anyhow!(e.clone()))
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

pub fn get_skills_dir() -> Result<PathBuf> {
    let skills_dir = get_application_dir()?.join("skills");
    get_dir_or_die(skills_dir)
}

/// Strip the workspace prefix from an absolute path, returning a workspace-relative string.
pub(crate) fn make_workspace_relative(path: &Path, workspace: &Path) -> Option<String> {
    path.strip_prefix(workspace)
        .ok()
        .and_then(|rel| rel.to_str().map(|s| s.to_string()))
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
    fn test_find_workspace_dir_finds_marker_at_start() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join(ROOT_DIR)).unwrap();

        let result = find_workspace_dir(temp.path().to_path_buf(), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp.path());
    }

    #[test]
    fn test_find_workspace_dir_finds_marker_in_parent() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join(ROOT_DIR)).unwrap();
        let child = temp.path().join("a").join("b");
        fs::create_dir_all(&child).unwrap();

        // No boundary — the walk must ascend past `child` to find the marker.
        let result = find_workspace_dir(child, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp.path());
    }

    #[test]
    fn test_find_workspace_dir_returns_err_when_no_marker() {
        let temp = TempDir::new().unwrap();
        // Bound the walk to the temp dir itself so the test never escapes into
        // real filesystem ancestors, which could contain a ROOT_DIR and cause
        // a flaky false-positive.
        let result = find_workspace_dir(temp.path().to_path_buf(), Some(temp.path()));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(ROOT_DIR));
    }

    #[test]
    fn test_make_workspace_relative_subdirectory() {
        // returns relative path for a file inside the workspace
        let workspace = PathBuf::from("/home/user/project");
        let path = PathBuf::from("/home/user/project/.claude/commands/hello.md");
        assert_eq!(
            make_workspace_relative(&path, &workspace),
            Some(".claude/commands/hello.md".to_string())
        );
    }

    #[test]
    fn test_make_workspace_relative_root_file() {
        // returns bare filename for a file at the workspace root
        let workspace = PathBuf::from("/home/user/project");
        let path = PathBuf::from("/home/user/project/CLAUDE.md");
        assert_eq!(
            make_workspace_relative(&path, &workspace),
            Some("CLAUDE.md".to_string())
        );
    }

    #[test]
    fn test_make_workspace_relative_outside_workspace() {
        // returns None when the path does not share the workspace prefix
        let workspace = PathBuf::from("/home/user/project");
        let path = PathBuf::from("/home/other/file.md");
        assert!(make_workspace_relative(&path, &workspace).is_none());
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
