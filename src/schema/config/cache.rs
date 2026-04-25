use super::common::Providers;
use super::traits::TomlConfig;
use crate::constants::schema::CONFIG_SCHEMA;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct CacheConfig {
    pub schema: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
}

impl CacheConfig {
    pub fn new() -> Self {
        Self {
            schema: CONFIG_SCHEMA.into(),
            providers: None,
        }
    }

    pub fn with_providers(providers: Providers) -> Self {
        Self {
            schema: CONFIG_SCHEMA.into(),
            providers: Some(providers),
        }
    }

    pub fn has_valid_hash(&self, target_name: &str, feature: &str) -> bool {
        if let Some(providers) = &self.providers
            && let Some(map) = &providers.0
            && let Some(settings) = map.get(target_name)
        {
            let feature_settings = match feature {
                "mcp" => settings.mcp.as_ref(),
                "instructions" => settings.instructions.as_ref(),
                "commands" => settings.commands.as_ref(),
                _ => return false,
            };

            return feature_settings
                .and_then(|s| s.hash.as_ref())
                .map(|h| !h.is_empty())
                .unwrap_or(false);
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        self.providers.is_none()
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TomlConfig for CacheConfig {}
