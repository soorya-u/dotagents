use anyhow::{Context, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants::resources;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    schema: String,
    features: Vec<String>,
    targets: Targets,
    ide: Option<HashMap<String, IdeOverride>>,
    cli: Option<HashMap<String, CliOverride>>,
    custom: Option<HashMap<String, CustomOverride>>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Targets {
    #[serde(default)]
    cli: Vec<String>,
    #[serde(default)]
    ide: Vec<String>,
    #[serde(default)]
    custom: Vec<String>,
}

// Shared subset of fields
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommonOverride {
    #[serde(rename = "commands-dir")]
    pub commands_dir: Option<Vec<String>>,
    #[serde(rename = "instruction-file")]
    pub instruction_file: Option<String>,
    #[serde(rename = "mcp-file")]
    pub mcp_file: Option<String>,
    pub commands: Option<bool>,
    pub instruction: Option<bool>,
    pub mcp: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdeOverride {
    #[serde(flatten)]
    pub common: CommonOverride,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliOverride {
    #[serde(flatten)]
    pub common: CommonOverride,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomOverride {
    #[serde(flatten)]
    pub common: CommonOverride,
    pub path: Option<String>,
}

#[derive(Clone)]
pub struct ConfigBuilder {
    schema: String,
    features: Option<Vec<String>>,
    targets: Option<Targets>,
    ide: Option<HashMap<String, IdeOverride>>,
    cli: Option<HashMap<String, CliOverride>>,
    custom: Option<HashMap<String, CustomOverride>>,
}

impl Config {
    pub fn to_toml(&self) -> Result<String> {
        let result = toml::to_string_pretty(self).context("failed to serialize config to TOML")?;

        Ok(result)
    }

    pub fn from_toml(toml: &str) -> Result<Self> {
        let result = toml::from_str(toml).context("failed to parse config from TOML")?;

        Ok(result)
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            schema: resources::CONFIG_SCHEMA.into(),
            features: None,
            targets: None,
            ide: None,
            cli: None,
            custom: None,
        }
    }

    pub fn add_features(mut self, is_commands: bool, is_instruction: bool, is_mcp: bool) -> Self {
        let mut vec: Vec<String> = vec![];
        if is_commands {
            vec.push(resources::COMMANDS_FEATURE.into());
        }
        if is_instruction {
            vec.push(resources::INSTRUCTION_FEATURE.into());
        }
        if is_mcp {
            vec.push(resources::MCP_FEATURE.into());
        }

        self.features = Some(vec);

        self
    }

    pub fn add_targets(mut self, cli: Vec<String>, ide: Vec<String>, custom: Vec<String>) -> Self {
        self.targets = Some(Targets { cli, ide, custom });

        self
    }

    pub fn add_cli_target(mut self, value: Vec<String>) -> Self {
        if self.targets.is_none() {
            debug!("targets is not initialized");
            self
        } else {
            self.targets.as_mut().unwrap().cli.extend(value);
            self
        }
    }

    pub fn add_ide_target(mut self, value: Vec<String>) -> Self {
        if self.targets.is_none() {
            debug!("targets is not initialized");
            self
        } else {
            self.targets.as_mut().unwrap().ide.extend(value);
            self
        }
    }

    pub fn add_custom_target(mut self, value: Vec<String>) -> Self {
        if self.targets.is_none() {
            debug!("targets is not initialized");
            self
        } else {
            self.targets.as_mut().unwrap().custom.extend(value);
            self
        }
    }

    pub fn add_cli_override(mut self, name: &str, overrides: CliOverride) -> Self {
        self.cli
            .get_or_insert_with(|| HashMap::new())
            .insert(name.into(), overrides);

        self
    }

    pub fn add_ide_override(mut self, name: &str, overrides: IdeOverride) -> Self {
        self.ide
            .get_or_insert_with(|| HashMap::new())
            .insert(name.into(), overrides);

        self
    }

    pub fn add_custom_override(mut self, name: &str, overrides: CustomOverride) -> Self {
        self.custom
            .get_or_insert_with(|| HashMap::new())
            .insert(name.into(), overrides);

        self
    }

    pub fn build(self) -> Config {
        Config {
            schema: self.schema,
            features: self.features.unwrap_or_default(),
            targets: self.targets.unwrap_or_default(),
            ide: self.ide,
            cli: self.cli,
            custom: self.custom,
        }
    }
}
