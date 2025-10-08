use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants::resources;

#[derive(Serialize, Deserialize)]
pub(crate) struct McpConfig {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub servers: HashMap<String, ServerConfig>,
}

pub(crate) struct McpConfigBuilder {
    schema: String,
    servers: HashMap<String, ServerConfig>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerConfig {
    #[serde(rename = "http")]
    Http {
        url: String,
        headers: Option<HashMap<String, String>>,
    },

    #[serde(rename = "stdio")]
    Stdio {
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

impl McpConfigBuilder {
    pub fn new() -> Self {
        Self {
            schema: resources::MCP_SCHEMA.into(),
            servers: HashMap::new(),
        }
    }

    pub fn add_http_server(
        mut self,
        name: &str,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Self {
        self.servers.insert(
            name.to_string(),
            ServerConfig::Http {
                url: url.into(),
                headers,
            },
        );
        self
    }

    pub fn add_stdio_server(
        mut self,
        name: &str,
        command: &str,
        args: Vec<String>,
        cwd: Option<&str>,
    ) -> Self {
        self.servers.insert(
            name.to_string(),
            ServerConfig::Stdio {
                command: command.into(),
                args,
                cwd: cwd.map(|s| s.into()),
                env: Some(HashMap::new()),
                env_file: None,
            },
        );
        self
    }

    pub fn build(self) -> McpConfig {
        McpConfig {
            schema: self.schema,
            servers: self.servers,
        }
    }
}
