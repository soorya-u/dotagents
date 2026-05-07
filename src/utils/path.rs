use anyhow::{Result, anyhow};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::constants::dir::{CACHE_DIR, ROOT_DIR, TEMPLATE_CACHE_SUBDIR};

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

/// Pre-populate the workspace dir from an explicit path; must be called before get_workspace_dir().
pub fn override_workspace_dir(path: PathBuf) -> Result<()> {
    if !path.join(ROOT_DIR).is_dir() {
        anyhow::bail!("No `{}` directory found in `{}`", ROOT_DIR, path.display());
    }
    let _ = WORKSPACE_DIR.set(Ok(path));
    Ok(())
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

pub fn get_config_dir() -> Result<PathBuf> {
    let base =
        dirs::config_dir().ok_or_else(|| anyhow!("failed to locate user config directory"))?;
    let config_dir = base.join("dotagents");
    std::fs::create_dir_all(&config_dir).map_err(|e| {
        anyhow!(
            "failed to create config directory {}: {}",
            config_dir.display(),
            e
        )
    })?;
    Ok(config_dir)
}

pub fn get_application_dir() -> Result<PathBuf> {
    let app_dir = get_workspace_dir()?.join(ROOT_DIR);
    get_dir_or_die(app_dir)
}

pub fn get_commands_dir() -> Result<PathBuf> {
    let commands_dir = get_application_dir()?.join("commands");
    get_dir_or_die(commands_dir)
}

/// Returns the user-level template source cache directory, creating it if necessary.
pub fn get_global_template_cache_dir() -> Result<PathBuf> {
    let config_base = get_config_dir()?;
    let dir = config_base.join(CACHE_DIR).join(TEMPLATE_CACHE_SUBDIR);
    std::fs::create_dir_all(&dir).map_err(|e| {
        anyhow!(
            "failed to create template cache directory {}: {}",
            dir.display(),
            e
        )
    })?;
    Ok(dir)
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
    #[cfg(unix)]
    // returns a path ending in "dotagents" when the directory exists
    fn test_get_config_dir() {
        let base = dirs::config_dir().expect("should have config dir");
        std::fs::create_dir_all(base.join("dotagents")).ok();
        let config = get_config_dir().expect("get_config_dir() should succeed");
        assert!(config.ends_with("dotagents"));
        assert!(config.is_dir());
    }

    #[test]
    // returns a path ending in dotagents/cache/templates and creates the directory
    fn test_get_global_template_cache_dir_ends_with_expected_suffix() {
        let base = dirs::config_dir().expect("should have config dir");
        std::fs::create_dir_all(base.join("dotagents")).ok();
        let result = get_global_template_cache_dir();
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let path = result.unwrap();
        assert!(
            path.ends_with("dotagents/cache/templates"),
            "path should end with dotagents/cache/templates, got {}",
            path.display()
        );
        assert!(path.is_dir(), "directory should have been created");
    }

    #[test]
    // override_workspace_dir returns Ok when path contains ROOT_DIR
    fn override_workspace_dir_ok_when_root_dir_present() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join(ROOT_DIR)).unwrap();
        // Use a fresh OnceLock-independent check via the validation logic directly
        let result = override_workspace_dir(temp.path().to_path_buf());
        // Either Ok (lock wasn't set yet) or Ok regardless — validation passed
        assert!(
            result.is_ok(),
            "expected Ok for a valid workspace path, got {:?}",
            result
        );
    }

    #[test]
    // override_workspace_dir returns Err when path has no ROOT_DIR subdirectory
    fn override_workspace_dir_err_when_no_root_dir() {
        let temp = TempDir::new().unwrap();
        // No ROOT_DIR created — validation should fail
        let result = override_workspace_dir(temp.path().to_path_buf());
        assert!(
            result.is_err(),
            "expected Err when ROOT_DIR is absent, got Ok"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains(ROOT_DIR),
            "error message should mention ROOT_DIR, got: {}",
            msg
        );
    }
}
