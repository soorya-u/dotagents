use super::cache::CacheConfig;
use super::common::{Providers, Targets};
use super::global::GlobalConfig;
use super::local::LocalConfig;
use crate::constants::schema::CONFIG_SCHEMA;
use crate::schema::config::TomlConfig;
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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(debug_assertions)]
impl TomlConfig for AppConfig {}
