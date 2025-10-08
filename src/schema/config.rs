use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants::resources;

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub features: Features,
    pub targets: Option<Targets>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Features {
    pub command: bool,
    pub instructions: bool,
    pub mcp: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Targets {
    pub ide: Option<HashMap<String, Option<TargetOption>>>,
    pub cli: Option<HashMap<String, Option<TargetOption>>>,
    pub custom: Option<HashMap<String, Option<TargetOption>>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct TargetOption {
    pub overrides: Option<Overrides>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Overrides {
    pub command: Option<bool>,
    pub instructions: Option<bool>,
    pub mcp: Option<bool>,
}

#[derive(Clone)]
pub(crate) struct ConfigBuilder {
    pub schema: String,
    features: Features,
    targets: Option<Targets>,
}

impl Config {
    pub fn to_yaml(&self) -> Result<String> {
        let result = serde_yaml::to_string(self).context("failed to serialize config to YAML")?;

        Ok(result)
    }
}

impl ConfigBuilder {
    pub fn new(mcp: bool, command: bool, instructions: bool) -> Self {
        Self {
            schema: resources::CONFIG_SCHEMA.into(),
            features: Features {
                command,
                instructions,
                mcp,
            },
            targets: None,
        }
    }

    pub fn add_target(mut self, target: Targets) -> Self {
        self.targets = Some(target);
        self
    }

    pub fn build(self) -> Config {
        Config {
            schema: self.schema,
            features: self.features,
            targets: self.targets,
        }
    }
}
