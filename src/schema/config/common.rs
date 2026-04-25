use anyhow::{Context, Result, anyhow};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    schema::features::{Feature, traits::FeatureTrait},
    templates::{RenderType, Templater, variables::get_user_defined_variables},
    utils::{
        fs::{read_file, write_file},
        merge::merge_optional,
        merge_json,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(transparent)]
pub struct Targets {
    pub providers: Option<HashSet<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(transparent)]
pub struct Providers(pub Option<HashMap<String, Features>>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Features {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<FeatureSettings>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<FeatureSettings>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<FeatureSettings>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<FeatureSettings>,
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
            providers: other.providers.clone().or_else(|| self.providers.clone()),
        }
    }
}

impl Providers {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn merge(&self, other: &Providers) -> Providers {
        Providers(Self::merge_provider_maps(self.0.as_ref(), other.0.as_ref()))
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
            skills: Self::merge_settings(self.skills.as_ref(), other.skills.as_ref()),
        }
    }

    pub fn get_config(&self, feature: &Feature) -> Option<FeatureSettings> {
        match feature {
            Feature::Mcp => self.mcp.clone(),
            Feature::Instruction => self.instructions.clone(),
            Feature::Command => self.commands.clone(),
            Feature::Skill => self.skills.clone(),
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
            providers: Some(HashSet::from([
                "vscode".to_string(),
                "windsurf".to_string(),
            ])),
        };

        let override_targets = Targets {
            providers: Some(HashSet::from(["cursor".to_string(), "claude".to_string()])),
        };

        let merged = base.merge(&override_targets);
        assert_eq!(
            merged.providers,
            Some(HashSet::from(["cursor".to_string(), "claude".to_string()]))
        );
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
        use crate::schema::features::Feature;

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
            skills: None,
        };

        let mcp_config = settings.get_config(&Feature::Mcp);
        assert!(mcp_config.is_some());
        assert_eq!(mcp_config.unwrap().template, Some("mcp.tmpl".to_string()));

        let cmd_config = settings.get_config(&Feature::Command);
        assert!(cmd_config.is_none());

        let skill_config = settings.get_config(&Feature::Skill);
        assert!(skill_config.is_none());
    }
}
