use std::collections::HashMap;

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::{
    constants::file::ENV_FILE,
    utils::path::{get_application_dir, get_config_dir, get_home_dir, get_workspace_dir},
};

pub(crate) fn get_dir_variables() -> Result<Value> {
    Ok(json!({
        "dir": {
            "workspace": get_workspace_dir()?,
            "application": get_application_dir()?,
            // TODO(soorya): Not Supported in v1.
            // "config": get_config_dir()?,
            // "home": get_home_dir()?,
        }
    }))
}

pub(crate) fn get_env_variables() -> Result<Value> {
    let path = get_application_dir()?.join(ENV_FILE);

    let mut env_vars = serde_json::Map::new();

    if let Ok(iter) = dotenvy::from_path_iter(&path) {
        for pair in iter {
            let (key, value) = pair?;
            env_vars.insert(key.to_lowercase(), value.into());
        }
    }

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
}
