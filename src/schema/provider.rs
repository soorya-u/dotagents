use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub(crate) struct Provider {
    pub schema: String,
    pub providers: ProviderOption,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ProviderOption {
    pub ide: Option<HashMap<String, ProviderSettings>>,
    pub cli: Option<HashMap<String, ProviderSettings>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ProviderSettings {
    pub mcp: ProviderAgentSettings,
    pub instructions: ProviderAgentSettings,
    pub commands: ProviderAgentSettings,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ProviderAgentSettings {
    pub template: String,
    pub target: String,
}

impl Provider {
    fn from_str(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to deserialize provider from TOML")
    }
}
