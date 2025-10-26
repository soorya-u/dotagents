use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{constants::resources, templates::helpers::get_templater};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub schema: String,
    pub features: Vec<String>,
    pub targets: Targets,
    pub providers: Option<Provider>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Targets {
    pub ide: Option<Vec<String>>,
    pub cli: Option<Vec<String>>,
    pub custom: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Provider {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ide: Option<HashMap<String, ConfigAgentAbilitySettings>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<HashMap<String, ConfigAgentAbilitySettings>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, ConfigAgentAbilitySettings>>,
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
pub(crate) struct ConfigAgentAbilitySettings {
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

impl ConfigAgentSettings {
    pub fn merge(&mut self, other: &ConfigAgentSettings) -> Self {
        Self {
            template: self.template.clone().or_else(|| other.template.clone()),
            target: self.target.clone().or_else(|| other.target.clone()),
            disabled: self.disabled.or(other.disabled),
            include: match (&self.include, &other.include) {
                (Some(self_vec), Some(other_vec)) => {
                    let mut merged = self_vec.clone();
                    merged.extend(other_vec.clone());
                    Some(merged)
                }
                (Some(local_vec), None) => Some(local_vec.clone()),
                (None, Some(global_vec)) => Some(global_vec.clone()),
                (None, None) => None,
            },
        }
    }
}

impl ConfigAgentAbilitySettings {
    fn merge(&mut self, other: &ConfigAgentAbilitySettings) {
        self.mcp = self.mcp.merge(&other.mcp);
        self.instructions = self.instructions.merge(&other.instructions);
        self.commands = self.commands.merge(&other.commands);
    }
}

fn merge_provider_map(
    global_map: &mut Option<HashMap<String, ConfigAgentAbilitySettings>>,
    local_map: Option<HashMap<String, ConfigAgentAbilitySettings>>,
) {
    if let Some(local) = local_map {
        let map = global_map.get_or_insert_with(HashMap::new);
        for (key, local_settings) in local {
            match map.get_mut(&key) {
                Some(global_settings) => {
                    global_settings.merge(&local_settings);
                }
                None => {
                    map.insert(key, local_settings);
                }
            }
        }
    }
}

impl ApplicationConfig {
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self).context("Failed to serialize config to TOML")
    }

    fn from_toml(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to deserialize config from TOML")
    }

    fn load_global_config() -> Result<Self> {
        let templater = get_templater();
        let global_config = templater.render_template(resources::GLOBAL_CONFIG_FILE, None)?;

        Self::from_toml(&global_config)
    }

    fn load_local_config() -> Result<Self> {
        let templater = get_templater();
        let local_config = templater.render_template(resources::LOCAL_CONFIG_FILE, None)?;

        Self::from_toml(&local_config)
    }

    fn merge_config(global: Self, local: Self) -> Self {
        Self {
            schema: global.schema,
            features: if !local.features.is_empty() {
                local.features
            } else {
                global.features
            },
            targets: Targets {
                ide: if local.targets.ide.is_some() {
                    local.targets.ide
                } else {
                    global.targets.ide
                },
                cli: if local.targets.cli.is_some() {
                    local.targets.cli
                } else {
                    global.targets.cli
                },
                custom: if local.targets.custom.is_some() {
                    local.targets.custom
                } else {
                    global.targets.custom
                },
            },
            providers: match (global.providers, local.providers) {
                (Some(mut g), Some(l)) => {
                    // Merge each category (ide, cli, custom)
                    merge_provider_map(&mut g.ide, l.ide);
                    merge_provider_map(&mut g.cli, l.cli);
                    merge_provider_map(&mut g.custom, l.custom);
                    Some(g)
                }
                (None, Some(l)) => Some(l),
                (Some(g), None) => Some(g),
                (None, None) => None,
            },
        }
    }

    pub fn new() -> Result<Self> {
        let global_config = Self::load_global_config()?;
        let local_config = Self::load_local_config()?;
        Ok(Self::merge_config(global_config, local_config))
    }
}
