use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};

use super::cache::CacheConfig;
use super::common::{Providers, Targets};
use super::global::GlobalConfig;
use super::local::LocalConfig;
use crate::constants::features::{COMMANDS_FEATURE, INSTRUCTION_FEATURE, MCP_FEATURE};
use crate::constants::file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE};
use crate::constants::schema::CONFIG_SCHEMA;
use crate::schema::config::{FeatureSettings, TomlConfig};
use crate::templates::{RenderType, Templater, get_templater};
use crate::utils::merge::{merge_optional, merge_optional_or_default};
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
            schema: CONFIG_SCHEMA.into(),
            features: HashSet::new(),
            targets: Targets::new(),
            providers: None,
            variables: None,
        }
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.contains(feature)
    }

    pub fn get_provider_feature_settings(&self, feature: &str) -> HashMap<String, FeatureSettings> {
        let Some(providers) = &self.providers else {
            return HashMap::new();
        };

        let has_feature = self.has_feature(feature);

        let ide_iter = providers.ide.iter().flat_map(|m| m.iter());
        let cli_iter = providers.cli.iter().flat_map(|m| m.iter());
        let custom_iter = providers.custom.iter().flat_map(|m| m.iter());

        ide_iter
            .chain(cli_iter)
            .chain(custom_iter)
            .filter_map(|(name, settings)| {
                let config = settings.get_config(feature)?;
                let is_disabled = config.disabled.unwrap_or(false);

                if has_feature && !is_disabled {
                    Some((name.clone(), config.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn to_cache(&self) -> CacheConfig {
        CacheConfig {
            schema: self.schema.clone(),
            providers: self.providers.clone(),
        }
    }

    pub fn from_application(templater: &Templater) -> Result<Self> {
        let global_config_content =
            templater.render_template(RenderType::Name(GLOBAL_CONFIG_FILE.into()), None)?;
        let local_config_content =
            templater.render_template(RenderType::Name(LOCAL_CONFIG_FILE.into()), None)?;

        let local_config = LocalConfig::from_toml(&local_config_content)?;
        local_config.validate().context("invalid local config")?;
        let global_config = GlobalConfig::from_toml(&global_config_content)?;
        global_config.validate().context("invalid local config")?;

        let app_config = AppConfig::from((&global_config, &local_config));

        Ok(app_config)
    }
}

impl From<(&GlobalConfig, &LocalConfig)> for AppConfig {
    fn from((global, local): (&GlobalConfig, &LocalConfig)) -> Self {
        let schema = local
            .schema
            .clone()
            .or_else(|| global.schema.clone())
            .unwrap_or_else(|| CONFIG_SCHEMA.into());

        let features = local
            .features
            .clone()
            .unwrap_or_else(|| global.features.clone());

        let targets =
            merge_optional_or_default(global.targets.as_ref(), local.targets.as_ref(), |g, l| {
                g.merge(l)
            });

        let providers = merge_optional(
            global.providers.as_ref(),
            local.providers.as_ref(),
            |g, l| g.merge(l),
        );

        let variables = merge_optional(
            global.variables.as_ref(),
            local.variables.as_ref(),
            |g, l| {
                let mut merged = g.clone();
                merged.extend(l.clone());
                merged
            },
        );

        Self {
            schema,
            features,
            targets,
            providers,
            variables,
        }
    }
}

impl From<&CacheConfig> for AppConfig {
    fn from(cache: &CacheConfig) -> Self {
        Self {
            schema: cache.schema.clone(),
            features: HashSet::new(),
            targets: Targets::new(),
            providers: cache.providers.clone(),
            variables: None,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(debug_assertions)]
impl TomlConfig for AppConfig {}
