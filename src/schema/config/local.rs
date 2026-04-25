use std::collections::{HashMap, HashSet};

use super::common::{Providers, Targets};
use super::traits::TomlConfig;
use crate::constants::schema::CONFIG_SCHEMA;
use crate::schema::features::Feature;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct LocalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<HashSet<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Targets>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}

impl LocalConfig {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: None,
            targets: None,
            providers: None,
            variables: None,
        }
    }

    pub fn with_features(features: HashSet<String>) -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: Some(features),
            targets: None,
            providers: None,
            variables: None,
        }
    }

    pub fn with_providers(providers: Providers) -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: None,
            targets: None,
            providers: Some(providers),
            variables: None,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        // Check that features are valid if present
        if let Some(features) = &self.features {
            for feature in features {
                if Feature::from_str(feature).is_none() {
                    anyhow::bail!(
                        "Invalid feature: {}. Valid features are: {}",
                        feature,
                        Feature::all_names().join(", ")
                    );
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
