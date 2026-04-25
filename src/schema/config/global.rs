use std::collections::{HashMap, HashSet};

use super::common::{Providers, Targets};
use super::traits::TomlConfig;
use crate::constants::schema::CONFIG_SCHEMA;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default)]
    pub features: HashSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Targets>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: HashSet::new(),
            targets: None,
            providers: None,
            variables: None,
        }
    }

    pub fn with_features(features: HashSet<String>, targets: Targets) -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features,
            targets: Some(targets),
            providers: None,
            variables: None,
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

        Ok(())
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TomlConfig for GlobalConfig {}
