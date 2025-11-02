use super::common::Providers;
use super::traits::TomlConfig;
use crate::constants::resources::CONFIG_SCHEMA;
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
            schema: CONFIG_SCHEMA.to_string(),
            providers: None,
        }
    }

    pub fn with_providers(providers: Providers) -> Self {
        Self {
            schema: CONFIG_SCHEMA.to_string(),
            providers: Some(providers),
        }
    }

    pub fn has_valid_hash(&self, target_type: &str, target_name: &str, feature: &str) -> bool {
        if let Some(providers) = &self.providers {
            let provider_map = match target_type {
                "ide" => providers.ide.as_ref(),
                "cli" => providers.cli.as_ref(),
                "custom" => providers.custom.as_ref(),
                _ => return false,
            };

            if let Some(map) = provider_map {
                if let Some(settings) = map.get(target_name) {
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
            }
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
