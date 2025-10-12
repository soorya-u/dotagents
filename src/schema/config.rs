use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub schema: String,
    pub features: Vec<String>,
    pub targets: Targets,
    pub providers: Option<Provider>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Targets {
    pub ide: Vec<String>,
    pub cli: Vec<String>,
    pub custom: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Provider {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ide: Option<HashMap<String, ConfigAgentAbilitySettings>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<HashMap<String, ConfigAgentAbilitySettings>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, ConfigAgentAbilitySettings>>,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            ide: Some(HashMap::new()),
            cli: Some(HashMap::new()),
            custom: Some(HashMap::new()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ConfigAgentAbilitySettings {
    pub mcp: ConfigAgentSettings,
    pub instructions: ConfigAgentSettings,
    pub commands: ConfigAgentSettings,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub(crate) struct ConfigAgentSettings {
    pub template: Option<String>,
    pub target: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub include: Option<Vec<String>>,
}

impl ApplicationConfig {
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self).context("Failed to serialize config to TOML")
    }

    fn from_toml(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to deserialize config from TOML")
    }

    // fn load_global_config() -> Result<Self> {}
}
