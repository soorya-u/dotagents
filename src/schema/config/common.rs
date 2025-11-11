use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::constants::features::{COMMANDS_FEATURE, INSTRUCTION_FEATURE, MCP_FEATURE};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Targets {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ide: Option<HashSet<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<HashSet<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashSet<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Providers {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ide: Option<HashMap<String, ConfigAgentAbilitySettings>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<HashMap<String, ConfigAgentAbilitySettings>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, ConfigAgentAbilitySettings>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigAgentAbilitySettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<ConfigAgentSettings>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<ConfigAgentSettings>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<ConfigAgentSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigAgentSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl Targets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&self, other: &Targets) -> Targets {
        Targets {
            ide: other.ide.clone().or_else(|| self.ide.clone()),
            cli: other.cli.clone().or_else(|| self.cli.clone()),
            custom: other.custom.clone().or_else(|| self.custom.clone()),
        }
    }
}

impl Providers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&self, other: &Providers) -> Providers {
        Providers {
            ide: Self::merge_provider_maps(self.ide.as_ref(), other.ide.as_ref()),
            cli: Self::merge_provider_maps(self.cli.as_ref(), other.cli.as_ref()),
            custom: Self::merge_provider_maps(self.custom.as_ref(), other.custom.as_ref()),
        }
    }

    fn merge_provider_maps(
        base: Option<&HashMap<String, ConfigAgentAbilitySettings>>,
        override_map: Option<&HashMap<String, ConfigAgentAbilitySettings>>,
    ) -> Option<HashMap<String, ConfigAgentAbilitySettings>> {
        match (base, override_map) {
            (None, None) => None,
            (Some(b), None) => Some(b.clone()),
            (None, Some(o)) => Some(o.clone()),
            (Some(b), Some(o)) => {
                let mut merged = b.clone();
                for (key, value) in o {
                    merged
                        .entry(key.clone())
                        .and_modify(|existing| *existing = existing.merge(value))
                        .or_insert_with(|| value.clone());
                }
                Some(merged)
            }
        }
    }
}

impl ConfigAgentAbilitySettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&self, other: &ConfigAgentAbilitySettings) -> ConfigAgentAbilitySettings {
        ConfigAgentAbilitySettings {
            mcp: Self::merge_settings(self.mcp.as_ref(), other.mcp.as_ref()),
            instructions: Self::merge_settings(
                self.instructions.as_ref(),
                other.instructions.as_ref(),
            ),
            commands: Self::merge_settings(self.commands.as_ref(), other.commands.as_ref()),
        }
    }

    pub fn get_config(&self, feature: &str) -> Option<ConfigAgentSettings> {
        match feature {
            MCP_FEATURE => self.mcp.clone(),
            INSTRUCTION_FEATURE => self.instructions.clone(),
            COMMANDS_FEATURE => self.commands.clone(),
            _ => None,
        }
    }

    fn merge_settings(
        base: Option<&ConfigAgentSettings>,
        override_settings: Option<&ConfigAgentSettings>,
    ) -> Option<ConfigAgentSettings> {
        match (base, override_settings) {
            (None, None) => None,
            (Some(b), None) => Some(b.clone()),
            (None, Some(o)) => Some(o.clone()),
            (Some(b), Some(o)) => Some(b.merge(o)),
        }
    }
}

impl ConfigAgentSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&self, other: &ConfigAgentSettings) -> ConfigAgentSettings {
        ConfigAgentSettings {
            template: other.template.clone().or_else(|| self.template.clone()),
            target: other.target.clone().or_else(|| self.target.clone()),
            disabled: other.disabled.or(self.disabled),
            variables: Self::merge_variables(self.variables.as_ref(), other.variables.as_ref()),
            hash: other.hash.clone().or_else(|| self.hash.clone()),
        }
    }

    fn merge_variables(
        base: Option<&HashMap<String, String>>,
        override_vars: Option<&HashMap<String, String>>,
    ) -> Option<HashMap<String, String>> {
        match (base, override_vars) {
            (None, None) => None,
            (Some(b), None) => Some(b.clone()),
            (None, Some(o)) => Some(o.clone()),
            (Some(b), Some(o)) => {
                let mut merged = b.clone();
                merged.extend(o.clone());
                Some(merged)
            }
        }
    }
}
