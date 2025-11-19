use anyhow::Result;
use serde_json::{Value, json};

use crate::utils::path::{get_application_dir, get_config_dir, get_home_dir, get_workspace_dir};

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

// TODO: Add Env Variables from `.env`
pub(crate) fn get_env_variables() -> Result<Value> {
    todo!()
}

pub(crate) fn get_command_name_variable(val: &str) -> Result<Value> {
    Ok(json!({
        "command": {
            "name": val,
        }
    }))
}
