use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};

use super::common::Providers;
use super::global::GlobalConfig;
use super::local::LocalConfig;
use crate::constants::file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE};
use crate::constants::schema::CONFIG_SCHEMA;
use crate::schema::config::{FeatureSettings, TomlConfig};
use crate::schema::features::Feature;
use crate::templates::{RenderType, Templater, get_templater};
use crate::utils::merge::merge_optional;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub schema: String,
    pub features: HashSet<String>,
    pub targets: HashSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
    pub variables: Option<HashMap<String, String>>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            schema: CONFIG_SCHEMA.into(),
            features: HashSet::new(),
            targets: HashSet::new(),
            providers: None,
            variables: None,
        }
    }

    pub fn has_feature(&self, feature: &Feature) -> bool {
        self.features.contains(feature.as_str())
    }

    pub fn get_provider_feature_settings(
        &self,
        feature: &Feature,
    ) -> HashMap<String, FeatureSettings> {
        let Some(providers) = &self.providers else {
            return HashMap::new();
        };

        let Some(provider_map) = &providers.0 else {
            return HashMap::new();
        };

        let has_feature = self.has_feature(feature);

        provider_map
            .iter()
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

        let targets = local
            .targets
            .as_ref()
            .and_then(|t| t.providers.clone())
            .or_else(|| global.targets.as_ref().and_then(|t| t.providers.clone()))
            .unwrap_or_default();

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

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(debug_assertions)]
impl TomlConfig for AppConfig {}
