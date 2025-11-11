use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use std::{collections::HashMap, fs};

use crate::{constants::file::MCP_FILE, utils::path::get_application_dir};

#[derive(Serialize, Deserialize)]
pub(crate) struct McpConfig {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub servers: HashMap<String, ServerConfig>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonConfig {
    pub disabled: Option<bool>,
    #[serde(rename = "disabledTools")]
    pub disabled_tools: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ServerConfig {
    #[serde(rename = "http")]
    Http {
        #[serde(flatten)]
        common: Option<CommonConfig>,
        url: String,
        headers: Option<HashMap<String, String>>,
    },

    #[serde(rename = "stdio")]
    Stdio {
        #[serde(flatten)]
        common: Option<CommonConfig>,
        command: String,
        args: Vec<String>,
        cwd: Option<String>,
        env: Option<HashMap<String, String>>,
        env_file: Option<String>,
    },
}

impl McpConfig {
    pub fn from_json(json: &str) -> Result<Self> {
        let result = serde_json::from_str::<McpConfig>(json)
            .context("failed to parse MCP config from JSON")?;

        Ok(result)
    }

    pub fn to_json(&self) -> Result<String> {
        let result =
            serde_json::to_string_pretty(self).context("failed to serialize MCP config to JSON")?;

        Ok(result)
    }

    pub fn to_json_value(&self) -> Result<Value> {
        let result = to_value(&self).context("failed to convert to json value")?;
        Ok(result)
    }

    pub fn from_application() -> Result<Self> {
        let dir = get_application_dir()?;

        let config_path = dir.join(MCP_FILE);
        let config = fs::read_to_string(config_path).context("failed to read MCP config file")?;

        Self::from_json(&config)
    }
}
