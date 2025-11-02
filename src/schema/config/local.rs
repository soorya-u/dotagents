use super::common::{Providers, Targets};
use super::traits::TomlConfig;
use crate::constants::resources::CONFIG_SCHEMA;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct LocalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Targets>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
}

impl LocalConfig {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.to_string()),
            features: None,
            targets: None,
            providers: None,
        }
    }

    pub fn with_features(features: Vec<String>) -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.to_string()),
            features: Some(features),
            targets: None,
            providers: None,
        }
    }

    pub fn with_providers(providers: Providers) -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.to_string()),
            features: None,
            targets: None,
            providers: Some(providers),
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        // Check that features are valid if present
        if let Some(features) = &self.features {
            for feature in features {
                if !["commands", "instructions", "mcp"].contains(&feature.as_str()) {
                    anyhow::bail!(
                        "Invalid feature: {}. Valid features are: commands, instructions, mcp",
                        feature
                    );
                }
            }
        }

        if let (Some(targets), Some(providers)) = (&self.targets, &self.providers) {
            if let Some(custom_targets) = &targets.custom {
                if let Some(custom_providers) = &providers.custom {
                    for target in custom_targets {
                        if !custom_providers.contains_key(target) {
                            anyhow::bail!(
                                "Custom target '{}' is defined in targets but has no provider configuration",
                                target
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.schema.is_none()
            && self.features.is_none()
            && self.targets.is_none()
            && self.providers.is_none()
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TomlConfig for LocalConfig {}
