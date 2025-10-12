use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub(crate) struct McpConfig {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub servers: HashMap<String, ServerConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct CommonConfig {
    pub disabled: Option<bool>,
    #[serde(rename = "disabledTools")]
    pub disabled_tools: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
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
}
