use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants::resources;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    schema: String,
    features: Vec<String>,
    targets: Targets,
    providers: Option<Provider>,
}

pub enum Target {
    IDE,
    CLI,
    Custom,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Targets {
    pub ide: Vec<String>,
    pub cli: Vec<String>,
    pub custom: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Provider {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ide: Option<HashMap<String, ProviderSettings>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<HashMap<String, ProviderSettings>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, ProviderSettings>>,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            ide: Some(HashMap::new()),
            cli: Some(HashMap::new()),
            custom: Some(HashMap::new()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ProviderSettings {
    pub mcp: ConfigAgentSettings,
    pub instructions: ConfigAgentSettings,
    pub commands: ConfigAgentSettings,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub(crate) struct ConfigAgentSettings {
    pub template: Option<String>,
    pub target: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub include: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct ConfigBuilder {
    pub schema: String,
    pub features: Vec<String>,
    pub targets: Option<Targets>,
    pub providers: Option<Provider>,
}

impl Config {
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self).context("Failed to serialize config to TOML")
    }

    pub fn from_toml(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to deserialize config from TOML")
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            schema: resources::CONFIG_SCHEMA.into(),
            features: vec![],
            targets: None,
            providers: None,
        }
    }

    pub fn add_features(mut self, is_commands: bool, is_instructions: bool, is_mcp: bool) -> Self {
        if is_commands {
            self.features.push(resources::COMMANDS_FEATURE.into());
        }
        if is_instructions {
            self.features.push(resources::INSTRUCTION_FEATURE.into());
        }
        if is_mcp {
            self.features.push(resources::MCP_FEATURE.into());
        }
        self
    }

    pub fn add_targets(mut self, ide: Vec<String>, cli: Vec<String>, custom: Vec<String>) -> Self {
        self.targets = Some(Targets { ide, cli, custom });
        self
    }

    pub fn add_target(mut self, target_name: Target, targets: Vec<String>) -> Self {
        match target_name {
            Target::CLI => self.targets.as_mut().unwrap().cli.extend(targets),
            Target::IDE => self.targets.as_mut().unwrap().ide.extend(targets),
            Target::Custom => self.targets.as_mut().unwrap().custom.extend(targets),
        };
        self
    }

    pub fn add_provider(
        mut self,
        target_name: Target,
        provider_name: &str,
        providers: ProviderSettings,
    ) -> Self {
        let provider = self.providers.get_or_insert_with(Provider::default);
        
        match target_name {
            Target::CLI => {
                if let Some(ref mut cli) = provider.cli {
                    cli.insert(provider_name.into(), providers);
                }
            }
            Target::IDE => {
                if let Some(ref mut ide) = provider.ide {
                    ide.insert(provider_name.into(), providers);
                }
            }
            Target::Custom => {
                if let Some(ref mut custom) = provider.custom {
                    custom.insert(provider_name.into(), providers);
                }
            }
        };
        self
    }

    pub fn build(self) -> Config {
        Config {
            schema: self.schema,
            features: self.features,
            targets: self.targets.unwrap_or_default(),
            providers: self.providers,
        }
    }
}
