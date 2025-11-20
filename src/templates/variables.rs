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
            "config": get_config_dir()?,
            "home": get_home_dir()?,
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

pub(crate) fn get_user_defined_variables(var: Option<Value>) -> Result<Value> {
    Ok(json!({ "var": var }))
}
