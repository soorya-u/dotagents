use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};

use super::cache::CacheConfig;
use super::common::{Providers, Targets};
use super::global::GlobalConfig;
use super::local::LocalConfig;
use crate::constants::features::{COMMANDS_FEATURE, INSTRUCTION_FEATURE, MCP_FEATURE};
use crate::constants::file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE};
use crate::constants::schema::CONFIG_SCHEMA;
use crate::schema::config::{ConfigAgentSettings, TomlConfig};
use crate::templates::helpers::{RenderType, Templater, get_templater};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub schema: String,
    pub features: HashSet<String>,
    pub targets: Targets,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
    pub variables: Option<HashMap<String, String>>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            schema: CONFIG_SCHEMA.to_string(),
            features: HashSet::new(),
            targets: Targets::new(),
            providers: None,
            variables: None,
        }
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.contains(feature)
    }

    pub fn get_feature_providers(&self, feature: &str) -> HashMap<String, ConfigAgentSettings> {
        let Some(providers) = &self.providers else {
            return HashMap::new();
        };

        let has_feature = self.has_feature(feature);

        [
            providers.cli.clone(),
            providers.ide.clone(),
            providers.custom.clone(),
        ]
        .into_iter()
        .flatten()
        .flat_map(|map| map.into_iter())
        .filter_map(|(name, settings)| {
            let config = settings.get_config(feature)?;
            let is_enabled = config.disabled.unwrap_or(false);

            if has_feature || is_enabled {
                Some((name.clone(), config.clone()))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>()
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

        let variables = match (&global.variables, &local.variables) {
            (None, None) => None,
            (Some(g), None) => Some(g.clone()),
            (None, Some(l)) => Some(l.clone()),
            (Some(g), Some(l)) => {
                let mut var = g.clone();
                var.extend(l.clone());
                Some(var)
            }
        };

        Self {
            schema,
            features,
            targets,
            providers,
            variables,
        }
    }

    pub fn from_cache(cache: &CacheConfig) -> Self {
        Self {
            schema: cache.schema.clone(),
            features: HashSet::new(),
            targets: Targets::new(),
            providers: cache.providers.clone(),
            variables: None,
        }
    }

    pub fn to_cache(&self) -> CacheConfig {
        CacheConfig {
            schema: self.schema.clone(),
            providers: self.providers.clone(),
        }
    }

    pub fn from_application(templater: &Templater) -> Result<Self> {
        let global_config_content =
            templater.render_template(RenderType::Name(GLOBAL_CONFIG_FILE.to_string()), None)?;
        let local_config_content =
            templater.render_template(RenderType::Name(LOCAL_CONFIG_FILE.to_string()), None)?;

        let local_config = LocalConfig::from_toml(&local_config_content)?;
        local_config.validate().context("invalid local config")?;
        let global_config = GlobalConfig::from_toml(&global_config_content)?;
        global_config.validate().context("invalid local config")?;

        let app_config = AppConfig::from_configs(&global_config, &local_config);

        Ok(app_config)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(debug_assertions)]
impl TomlConfig for AppConfig {}
