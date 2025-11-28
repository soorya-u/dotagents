use anyhow::{Context, Result, anyhow};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    constants::features::{COMMANDS_FEATURE, INSTRUCTION_FEATURE, MCP_FEATURE},
    schema::features::traits::FeatureTrait,
    templates::{
        RenderType, Templater,
        variables::{get_command_name_variable, get_user_defined_variables},
    },
    utils::{
        fs::{read_file, write_file},
        merge::merge_optional,
        merge_json,
    },
};

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
    pub ide: Option<HashMap<String, Features>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<HashMap<String, Features>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, Features>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Features {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<FeatureSettings>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<FeatureSettings>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<FeatureSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct FeatureSettings {
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
        base: Option<&HashMap<String, Features>>,
        override_map: Option<&HashMap<String, Features>>,
    ) -> Option<HashMap<String, Features>> {
        match (base, override_map) {
            (None, None) => None,
            (Some(b), None) => Some(b.clone()),
            (None, Some(o)) => Some(o.clone()),
            (Some(b), Some(o)) => {
                let mut merged = HashMap::with_capacity(b.len() + o.len());
                merged.extend(b.iter().map(|(k, v)| (k.clone(), v.clone())));

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

impl Features {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&self, other: &Features) -> Features {
        Features {
            mcp: Self::merge_settings(self.mcp.as_ref(), other.mcp.as_ref()),
            instructions: Self::merge_settings(
                self.instructions.as_ref(),
                other.instructions.as_ref(),
            ),
            commands: Self::merge_settings(self.commands.as_ref(), other.commands.as_ref()),
        }
    }

    pub fn get_config(&self, feature: &str) -> Option<FeatureSettings> {
        match feature {
            MCP_FEATURE => self.mcp.clone(),
            INSTRUCTION_FEATURE => self.instructions.clone(),
            COMMANDS_FEATURE => self.commands.clone(),
            _ => None,
        }
    }

    fn merge_settings(
        base: Option<&FeatureSettings>,
        override_settings: Option<&FeatureSettings>,
    ) -> Option<FeatureSettings> {
        match (base, override_settings) {
            (None, None) => None,
            (Some(b), None) => Some(b.clone()),
            (None, Some(o)) => Some(o.clone()),
            (Some(b), Some(o)) => Some(b.merge(o)),
        }
    }
}

impl FeatureSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&self, other: &FeatureSettings) -> FeatureSettings {
        FeatureSettings {
            template: other.template.clone().or_else(|| self.template.clone()),
            target: other.target.clone().or_else(|| self.target.clone()),
            disabled: other.disabled.or(self.disabled),
            variables: merge_optional(self.variables.as_ref(), other.variables.as_ref(), |b, o| {
                b.clone().into_iter().chain(o.clone()).collect()
            }),
            hash: other.hash.clone().or_else(|| self.hash.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_targets_merge() {
        let base = Targets {
            ide: Some(HashSet::from(["vscode".to_string()])),
            cli: None,
            custom: None,
        };

        let override_targets = Targets {
            ide: Some(HashSet::from(["cursor".to_string()])),
            cli: Some(HashSet::from(["anthropic".to_string()])),
            custom: None,
        };

        let merged = base.merge(&override_targets);
        assert_eq!(merged.ide, Some(HashSet::from(["cursor".to_string()])));
        assert_eq!(merged.cli, Some(HashSet::from(["anthropic".to_string()])));
    }

    #[test]
    fn test_config_agent_settings_merge() {
        let base = FeatureSettings {
            template: Some("base.tmpl".to_string()),
            target: Some("base.target".to_string()),
            disabled: Some(false),
            variables: Some(HashMap::from([("key1".to_string(), "value1".to_string())])),
            hash: None,
        };

        let override_settings = FeatureSettings {
            template: None,
            target: Some("override.target".to_string()),
            disabled: Some(true),
            variables: Some(HashMap::from([("key2".to_string(), "value2".to_string())])),
            hash: Some("hash123".to_string()),
        };

        let merged = base.merge(&override_settings);
        assert_eq!(merged.template, Some("base.tmpl".to_string()));
        assert_eq!(merged.target, Some("override.target".to_string()));
        assert_eq!(merged.disabled, Some(true));
        assert_eq!(merged.hash, Some("hash123".to_string()));

        let vars = merged.variables.unwrap();
        assert_eq!(vars.get("key1"), Some(&"value1".to_string()));
        assert_eq!(vars.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_config_agent_ability_settings_get_config() {
        let settings = Features {
            mcp: Some(FeatureSettings {
                template: Some("mcp.tmpl".to_string()),
                ..Default::default()
            }),
            instructions: Some(FeatureSettings {
                template: Some("inst.tmpl".to_string()),
                ..Default::default()
            }),
            commands: None,
        };

        let mcp_config = settings.get_config("mcp");
        assert!(mcp_config.is_some());
        assert_eq!(mcp_config.unwrap().template, Some("mcp.tmpl".to_string()));

        let cmd_config = settings.get_config("commands");
        assert!(cmd_config.is_none());

        let unknown = settings.get_config("unknown");
        assert!(unknown.is_none());
    }
}
