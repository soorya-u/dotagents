use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CacheConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ide: Option<HashMap<String, ProviderSettings>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<HashMap<String, ProviderSettings>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, ProviderSettings>>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ide: Some(HashMap::new()),
            cli: Some(HashMap::new()),
            custom: Some(HashMap::new()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ProviderSettings {
    pub mcp: AgentAbilitySettings,
    pub instructions: AgentAbilitySettings,
    pub commands: AgentAbilitySettings,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub(crate) struct AgentAbilitySettings {
    pub template: String,
    pub target: String,
    pub variables: HashMap<String, String>,
    pub hash: String,
}
