use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    constants::file::MCP_FILE, core::features::traits::FeatureTrait,
    utils::path::get_application_dir,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_timeout_sec: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_timeout_sec: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token_env_var: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_vars: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_connect: Option<bool>,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },

    #[serde(rename = "sse")]
    Sse {
        #[serde(flatten)]
        common: Option<CommonConfig>,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },

    #[serde(rename = "stdio")]
    Stdio {
        #[serde(flatten)]
        common: Option<CommonConfig>,
        command: String,
        args: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        env: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
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

    fn resolve_source_path(_name: Option<&str>) -> Result<PathBuf> {
        Ok(get_application_dir()?.join(MCP_FILE))
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
    fn test_from_json_sse() {
        let json = r#"{
            "$schema": "https://schema.example.com",
            "servers": {
                "sse-server": {
                    "type": "sse",
                    "url": "https://example.com/sse",
                    "headers": {
                        "Authorization": "Bearer token"
                    }
                }
            }
        }"#;

        let mcp = McpFeature::from_json(json).unwrap();
        let server = mcp.servers.get("sse-server").unwrap();
        match server {
            ServerConfig::Sse { url, headers, .. } => {
                assert_eq!(url, "https://example.com/sse");
                assert_eq!(
                    headers.as_ref().unwrap().get("Authorization"),
                    Some(&"Bearer token".to_string())
                );
            }
            _ => panic!("Expected Sse server config"),
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
    fn test_expanded_common_fields_roundtrip_as_camel_case() {
        let json = r#"{
            "$schema": "https://schema.example.com",
            "servers": {
                "expanded": {
                    "type": "http",
                    "url": "https://example.com/mcp",
                    "disabled": false,
                    "disabledTools": ["slow-tool"],
                    "enabledTools": ["fast-tool"],
                    "required": true,
                    "startupTimeoutSec": 10,
                    "toolTimeoutSec": 20,
                    "bearerTokenEnvVar": "MCP_TOKEN",
                    "envVars": ["MCP_TOKEN"],
                    "alwaysAllow": ["fast-tool"],
                    "autoConnect": true
                }
            }
        }"#;

        let mcp = McpFeature::from_json(json).unwrap();
        let server = mcp.servers.get("expanded").unwrap();
        match server {
            ServerConfig::Http {
                common: Some(common),
                ..
            } => {
                assert_eq!(common.disabled, Some(false));
                assert_eq!(common.disabled_tools, Some(vec!["slow-tool".to_string()]));
                assert_eq!(common.enabled_tools, Some(vec!["fast-tool".to_string()]));
                assert_eq!(common.required, Some(true));
                assert_eq!(common.startup_timeout_sec, Some(10));
                assert_eq!(common.tool_timeout_sec, Some(20));
                assert_eq!(common.bearer_token_env_var, Some("MCP_TOKEN".to_string()));
                assert_eq!(common.env_vars, Some(vec!["MCP_TOKEN".to_string()]));
                assert_eq!(common.always_allow, Some(vec!["fast-tool".to_string()]));
                assert_eq!(common.auto_connect, Some(true));
            }
            _ => panic!("Expected Http server config with common fields"),
        }

        let serialized = mcp.to_json().unwrap();
        assert!(serialized.contains("\"enabledTools\""));
        assert!(serialized.contains("\"startupTimeoutSec\""));
        assert!(serialized.contains("\"toolTimeoutSec\""));
        assert!(serialized.contains("\"bearerTokenEnvVar\""));
        assert!(serialized.contains("\"envVars\""));
        assert!(!serialized.contains("startup_timeout_sec"));
        assert!(McpFeature::from_json(&serialized).is_ok());
    }

    #[test]
    fn test_unset_optional_fields_are_omitted() {
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

        let serialized = mcp.to_json().unwrap();
        assert!(!serialized.contains("\"cwd\": null"));
        assert!(!serialized.contains("\"env\": null"));
        assert!(!serialized.contains("\"envFile\": null"));
    }

    #[test]
    fn test_rejects_invalid_transport_specific_configs() {
        let missing_command = r#"{
            "$schema": "https://schema.example.com",
            "servers": {
                "bad-stdio": {
                    "type": "stdio",
                    "args": []
                }
            }
        }"#;
        let missing_url = r#"{
            "$schema": "https://schema.example.com",
            "servers": {
                "bad-http": {
                    "type": "http"
                }
            }
        }"#;

        assert!(McpFeature::from_json(missing_command).is_err());
        assert!(McpFeature::from_json(missing_url).is_err());
    }

    #[test]
    fn test_public_schema_includes_expanded_mcp_fields() {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../public/v1/schemas/mcp.schema.json"))
                .unwrap();
        let server_schema =
            &schema["properties"]["servers"]["patternProperties"]["^[a-zA-Z0-9._-]+$"];
        let transport_enum = server_schema["properties"]["type"]["enum"]
            .as_array()
            .unwrap();
        assert!(transport_enum.contains(&serde_json::json!("stdio")));
        assert!(transport_enum.contains(&serde_json::json!("http")));
        assert!(transport_enum.contains(&serde_json::json!("sse")));
        assert!(server_schema["properties"].get("enabledTools").is_some());
        assert!(server_schema["properties"].get("disabledTools").is_some());
        assert!(
            server_schema["properties"]
                .get("startupTimeoutSec")
                .is_some()
        );
        assert!(server_schema["properties"].get("toolTimeoutSec").is_some());
        assert!(
            server_schema["properties"]
                .get("bearerTokenEnvVar")
                .is_some()
        );
        assert!(server_schema["properties"].get("envVars").is_some());
        assert!(server_schema["properties"].get("alwaysAllow").is_some());
        assert!(server_schema["properties"].get("autoConnect").is_some());
        assert!(server_schema["allOf"].is_array());
    }
}
