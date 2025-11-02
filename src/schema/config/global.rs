use super::common::{Providers, Targets};
use super::traits::TomlConfig;
use crate::constants::resources::CONFIG_SCHEMA;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(default)]
    pub features: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Targets>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.to_string()),
            features: Vec::new(),
            targets: Some(Targets::new()),
            providers: None,
        }
    }

    pub fn with_features(features: Vec<String>, targets: Targets) -> Self {
        Self {
            schema: Some("https://dotagents.soorya-u.dev/schemas/config.schema.json".to_string()),
            features,
            targets: Some(targets),
            providers: None,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for feature in &self.features {
            if !["commands", "instructions", "mcp"].contains(&feature.as_str()) {
                anyhow::bail!(
                    "Invalid feature: {}. Valid features are: commands, instructions, mcp",
                    feature
                );
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
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TomlConfig for GlobalConfig {}
