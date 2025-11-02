use super::cache::CacheConfig;
use super::common::{Providers, Targets};
use super::global::GlobalConfig;
use super::local::LocalConfig;
use crate::constants::resources::CONFIG_SCHEMA;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub schema: String,

    pub features: Vec<String>,

    pub targets: Targets,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            schema: CONFIG_SCHEMA.to_string(),
            features: Vec::new(),
            targets: Targets::new(),
            providers: None,
        }
    }

    pub fn from_configs(global: &GlobalConfig, local: &LocalConfig) -> Self {
        let schema = local
            .schema
            .clone()
            .or_else(|| global.schema.clone())
            .unwrap_or_else(|| CONFIG_SCHEMA.to_string());

        let features = local
            .features
            .clone()
            .unwrap_or_else(|| global.features.clone());

        let targets = match (&global.targets, &local.targets) {
            (None, None) => Targets::new(),
            (Some(g), None) => g.clone(),
            (None, Some(l)) => l.clone(),
            (Some(g), Some(l)) => g.merge(l),
        };

        let providers = match (&global.providers, &local.providers) {
            (None, None) => None,
            (Some(g), None) => Some(g.clone()),
            (None, Some(l)) => Some(l.clone()),
            (Some(g), Some(l)) => Some(g.merge(l)),
        };

        Self {
            schema,
            features,
            targets,
            providers,
        }
    }

    pub fn from_cache(cache: &CacheConfig) -> Self {
        Self {
            schema: cache.schema.clone(),
            features: Vec::new(),
            targets: Targets::new(),
            providers: cache.providers.clone(),
        }
    }

    pub fn to_cache(&self) -> CacheConfig {
        CacheConfig {
            schema: self.schema.clone(),
            providers: self.providers.clone(),
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

        if let Some(custom_targets) = &self.targets.custom {
            if let Some(providers) = &self.providers {
                if let Some(custom_providers) = &providers.custom {
                    for target in custom_targets {
                        if !custom_providers.contains_key(target) {
                            anyhow::bail!(
                                "Custom target '{}' is defined in targets but has no provider configuration",
                                target
                            );
                        }
                    }
                } else if !custom_targets.is_empty() {
                    anyhow::bail!(
                        "Custom targets are defined but no custom providers are configured"
                    );
                }
            }
        }

        Ok(())
    }

    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    pub fn get_provider(
        &self,
        target_type: &str,
        target_name: &str,
    ) -> Option<&super::common::ConfigAgentAbilitySettings> {
        let providers = self.providers.as_ref()?;

        let provider_map = match target_type {
            "ide" => providers.ide.as_ref(),
            "cli" => providers.cli.as_ref(),
            "custom" => providers.custom.as_ref(),
            _ => return None,
        };

        provider_map.and_then(|map| map.get(target_name))
    }

    pub fn is_target_enabled(&self, target_type: &str, target_name: &str) -> bool {
        let targets = match target_type {
            "ide" => self.targets.ide.as_ref(),
            "cli" => self.targets.cli.as_ref(),
            "custom" => self.targets.custom.as_ref(),
            _ => return false,
        };

        targets
            .map(|t| t.iter().any(|name| name == target_name))
            .unwrap_or(false)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}
