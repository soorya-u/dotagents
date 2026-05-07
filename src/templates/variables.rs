use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};

use crate::{
    constants::file::ENV_FILE,
    utils::path::{get_application_dir, get_workspace_dir},
};

/// Custom env file paths supplied via `--env`; set before the templater initialises.
static ENV_PATHS: OnceLock<Vec<PathBuf>> = OnceLock::new();

/// Register custom env file paths to use instead of the default `.dotagents/.env`.
pub(crate) fn set_env_paths(paths: Vec<PathBuf>) {
    let _ = ENV_PATHS.set(paths);
}

/// Load and merge env vars from the given files left-to-right; later files win on duplicate keys.
fn load_env_from_paths(paths: &[PathBuf]) -> Result<serde_json::Map<String, Value>> {
    let mut env_vars = serde_json::Map::new();
    for path in paths {
        if !path.exists() {
            return Err(anyhow!(
                "load env file '{}': file not found",
                path.display()
            ));
        }
        let iter = dotenvy::from_path_iter(path)
            .with_context(|| format!("failed to read env file: {}", path.display()))?;
        for pair in iter {
            let (key, value) = pair?;
            env_vars.insert(key.to_lowercase(), value.into());
        }
    }
    Ok(env_vars)
}

pub(crate) fn get_dir_variables() -> Result<Value> {
    Ok(json!({
        "dir": {
            "workspace": get_workspace_dir()?,
            "application": get_application_dir()?,
        }
    }))
}

pub(crate) fn get_env_variables() -> Result<Value> {
    let env_vars = match ENV_PATHS.get() {
        Some(paths) if !paths.is_empty() => load_env_from_paths(paths)?,
        _ => {
            // Default: silently ignore missing .dotagents/.env
            let path = get_application_dir()?.join(ENV_FILE);
            let mut map = serde_json::Map::new();
            if let Ok(iter) = dotenvy::from_path_iter(&path) {
                for pair in iter {
                    let (key, value) = pair?;
                    map.insert(key.to_lowercase(), value.into());
                }
            }
            map
        }
    };
    Ok(json!({ "env": env_vars }))
}

pub(crate) fn get_command_name_variable(val: &str) -> Result<Value> {
    Ok(json!({
        "command": {
            "name": val,
        }
    }))
}

pub(crate) fn get_skill_name_variable(val: &str) -> Result<Value> {
    Ok(json!({
        "skill": {
            "name": val,
        }
    }))
}

pub(crate) fn get_user_defined_variables(var: Option<Value>) -> Result<Value> {
    Ok(json!({ "var": var }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_command_name_variable() {
        let result = get_command_name_variable("test_command").unwrap();
        assert_eq!(result, json!({"command": {"name": "test_command"}}));
    }

    #[test]
    fn test_get_command_name_variable_empty() {
        let result = get_command_name_variable("").unwrap();
        assert_eq!(result, json!({"command": {"name": ""}}));
    }

    #[test]
    fn test_get_skill_name_variable() {
        let result = get_skill_name_variable("my-skill").unwrap();
        assert_eq!(result, json!({"skill": {"name": "my-skill"}}));
    }

    #[test]
    fn test_get_skill_name_variable_empty() {
        let result = get_skill_name_variable("").unwrap();
        assert_eq!(result, json!({"skill": {"name": ""}}));
    }

    #[test]
    fn test_get_user_defined_variables_some() {
        let custom_vars = json!({"key1": "value1", "key2": 42});
        let result = get_user_defined_variables(Some(custom_vars.clone())).unwrap();
        assert_eq!(result, json!({"var": custom_vars}));
    }

    #[test]
    fn test_get_user_defined_variables_none() {
        let result = get_user_defined_variables(None).unwrap();
        assert_eq!(result, json!({"var": null}));
    }

    #[test]
    fn test_get_user_defined_variables_empty_object() {
        let empty = json!({});
        let result = get_user_defined_variables(Some(empty)).unwrap();
        assert_eq!(result, json!({"var": {}}));
    }

    #[test]
    fn test_get_user_defined_variables_nested() {
        let nested = json!({"level1": {"level2": {"level3": "deep value"}}});
        let result = get_user_defined_variables(Some(nested.clone())).unwrap();
        assert_eq!(result, json!({"var": nested}));
    }

    #[test]
    fn test_get_env_variables_structure() {
        // This test checks that get_env_variables returns the correct structure
        // even if no .env file exists
        let result = get_env_variables();
        // This may fail if not in a workspace, which is expected
        if result.is_ok() {
            let value = result.unwrap();
            assert!(value.get("env").is_some());
        }
    }

    #[test]
    // load_env_from_paths returns empty map for empty slice
    fn load_env_from_paths_empty_slice_returns_empty_map() {
        let result = load_env_from_paths(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    // load_env_from_paths populates env vars with lowercased keys from a single file
    fn load_env_from_paths_single_file_lowercases_keys() {
        let tmp = TempDir::new().unwrap();
        let env_file = tmp.path().join("test.env");
        fs::write(&env_file, "MY_KEY=hello\nANOTHER=world\n").unwrap();

        let result = load_env_from_paths(&[env_file]).unwrap();

        assert_eq!(result.get("my_key").unwrap(), "hello");
        assert_eq!(result.get("another").unwrap(), "world");
        assert!(result.get("MY_KEY").is_none());
    }

    #[test]
    // load_env_from_paths merges two files left-to-right with later file winning on duplicates
    fn load_env_from_paths_two_files_later_wins_on_duplicate() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("base.env");
        let override_file = tmp.path().join("override.env");
        fs::write(&base, "KEY=base_value\nBASE_ONLY=base\n").unwrap();
        fs::write(&override_file, "KEY=override_value\nOVERRIDE_ONLY=over\n").unwrap();

        let result = load_env_from_paths(&[base, override_file]).unwrap();

        assert_eq!(result.get("key").unwrap(), "override_value");
        assert_eq!(result.get("base_only").unwrap(), "base");
        assert_eq!(result.get("override_only").unwrap(), "over");
    }

    #[test]
    // load_env_from_paths returns an error when a specified file does not exist
    fn load_env_from_paths_missing_file_returns_error() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join("nonexistent.env");

        let result = load_env_from_paths(&[missing.clone()]);

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("nonexistent.env"),
            "error should name the missing file"
        );
    }
}
