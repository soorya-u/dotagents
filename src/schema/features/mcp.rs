use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use serde_json5::from_str;
use std::{collections::HashMap, fs};

use crate::{
    constants::file::MCP_FILE,
    schema::features::traits::FeatureTrait,
    utils::{gitignore::GitignoreScope, path::get_application_dir},
};

#[derive(Serialize, Deserialize)]
pub(crate) struct McpFeature {
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

impl McpFeature {
    /// Returns the starter mcp.jsonc mock content used during `init`.
    pub(crate) fn mock() -> &'static str {
        crate::constants::mocks::MCP_MOCK
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let result = serde_json5::from_str::<McpFeature>(json)
            .context("failed to parse MCP config from JSON")?;

        Ok(result)
    }

    pub fn to_json(&self) -> Result<String> {
        let result =
            serde_json::to_string_pretty(self).context("failed to serialize MCP config to JSON")?;

        Ok(result)
    }

    pub fn from_application() -> Result<Self> {
        let dir = get_application_dir()?;

        let config_path = dir.join(MCP_FILE);
        let config = fs::read_to_string(config_path).context("failed to read MCP config file")?;

        Self::from_json(&config)
    }
}

impl FeatureTrait for McpFeature {
    fn from_string(value: &str) -> Result<Self> {
        Self::from_json(value)
    }

    fn to_string(&self) -> Result<String> {
        self.to_json()
    }

    fn to_value(&self) -> Value {
        json!({
            "mcp": self,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json_stdio() {
        let json = r#"{
            "$schema": "https://schema.example.com",
            "servers": {
                "test-server": {
                    "type": "stdio",
                    "command": "node",
                    "args": ["server.js"]
                }
            }
        }"#;

        let mcp = McpFeature::from_json(json).unwrap();
        assert_eq!(mcp.schema, "https://schema.example.com");
        assert_eq!(mcp.servers.len(), 1);

        let server = mcp.servers.get("test-server").unwrap();
        match server {
            ServerConfig::Stdio { command, args, .. } => {
                assert_eq!(command, "node");
                assert_eq!(args, &vec!["server.js"]);
            }
            _ => panic!("Expected Stdio server config"),
        }
    }

    #[test]
    fn test_from_json_http() {
        let json = r#"{
            "$schema": "https://schema.example.com",
            "servers": {
                "http-server": {
                    "type": "http",
                    "url": "https://example.com/mcp"
                }
            }
        }"#;

        let mcp = McpFeature::from_json(json).unwrap();
        let server = mcp.servers.get("http-server").unwrap();
        match server {
            ServerConfig::Http { url, .. } => {
                assert_eq!(url, "https://example.com/mcp");
            }
            _ => panic!("Expected Http server config"),
        }
    }

    #[test]
    fn test_from_json_with_env() {
        let json = r#"{
            "$schema": "https://schema.example.com",
            "servers": {
                "env-server": {
                    "type": "stdio",
                    "command": "python",
                    "args": ["app.py"],
                    "cwd": "/path/to/dir",
                    "env": {
                        "API_KEY": "secret"
                    }
                }
            }
        }"#;

        let mcp = McpFeature::from_json(json).unwrap();
        let server = mcp.servers.get("env-server").unwrap();
        match server {
            ServerConfig::Stdio {
                command, env, cwd, ..
            } => {
                assert_eq!(command, "python");
                assert_eq!(cwd, &Some("/path/to/dir".to_string()));
                assert!(env.is_some());
                assert_eq!(
                    env.as_ref().unwrap().get("API_KEY"),
                    Some(&"secret".to_string())
                );
            }
            _ => panic!("Expected Stdio server config"),
        }
    }

    #[test]
    fn test_to_json() {
        let mut servers = HashMap::new();
        servers.insert(
            "test".to_string(),
            ServerConfig::Stdio {
                common: None,
                command: "node".to_string(),
                args: vec!["index.js".to_string()],
                cwd: None,
                env: None,
                env_file: None,
            },
        );

        let mcp = McpFeature {
            schema: "https://test.com".to_string(),
            servers,
        };

        let json = mcp.to_json().unwrap();
        assert!(json.contains("https://test.com"));
        assert!(json.contains("node"));
        assert!(json.contains("index.js"));
    }

    #[test]
    fn test_roundtrip() {
        let json = r#"{
            "$schema": "https://schema.example.com",
            "servers": {
                "roundtrip": {
                    "type": "stdio",
                    "command": "bash",
                    "args": ["script.sh"]
                }
            }
        }"#;

        let mcp = McpFeature::from_json(json).unwrap();
        let serialized = mcp.to_json().unwrap();
        let deserialized = McpFeature::from_json(&serialized).unwrap();

        assert_eq!(mcp.schema, deserialized.schema);
        assert_eq!(mcp.servers.len(), deserialized.servers.len());
    }

    #[test]
    fn test_from_string() {
        let json = r#"{
            "$schema": "https://schema.example.com",
            "servers": {}
        }"#;

        let mcp = McpFeature::from_string(json).unwrap();
        assert_eq!(mcp.schema, "https://schema.example.com");
        assert_eq!(mcp.servers.len(), 0);
    }

    #[test]
    fn test_to_string() {
        let mcp = McpFeature {
            schema: "https://test.com".to_string(),
            servers: HashMap::new(),
        };

        let result = mcp.to_string().unwrap();
        assert!(result.contains("https://test.com"));
    }

    #[test]
    fn test_disabled_tools() {
        let json = r#"{
            "$schema": "https://schema.example.com",
            "servers": {
                "disabled-server": {
                    "type": "stdio",
                    "command": "node",
                    "args": ["server.js"],
                    "disabled": true,
                    "disabledTools": ["tool1", "tool2"]
                }
            }
        }"#;

        let mcp = McpFeature::from_json(json).unwrap();
        let server = mcp.servers.get("disabled-server").unwrap();
        match server {
            ServerConfig::Stdio { common, .. } => {
                assert!(common.is_some());
                let common = common.as_ref().unwrap();
                assert_eq!(common.disabled, Some(true));
                assert_eq!(
                    common.disabled_tools,
                    Some(vec!["tool1".to_string(), "tool2".to_string()])
                );
            }
            _ => panic!("Expected Stdio server config"),
        }
    }

    #[test]
    fn test_gitignore_scope_is_file() {
        // mcp config is a single file per provider → File scope (default)
        let json = r#"{"$schema":"","servers":{"s":{"type":"stdio","command":"x","args":[]}}}"#;
        let mcp = McpFeature::from_json(json).unwrap();
        assert!(matches!(mcp.gitignore_scope(), GitignoreScope::File));
    }
}
