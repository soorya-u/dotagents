use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::integrations::skills_sh::PackageRunner;
use crate::{core::features::Feature, utils::merge::merge_optional};

/// Configuration for external integrations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct IntegrationsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills_sh: Option<SkillsShConfig>,
}

/// Configuration for the skills.sh integration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SkillsShConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_runner: Option<PackageRunner>,
}

impl IntegrationsConfig {
    pub fn merge(&self, other: &IntegrationsConfig) -> IntegrationsConfig {
        IntegrationsConfig {
            skills_sh: merge_optional(self.skills_sh.as_ref(), other.skills_sh.as_ref(), |g, l| {
                SkillsShConfig {
                    package_runner: l
                        .package_runner
                        .clone()
                        .or_else(|| g.package_runner.clone()),
                }
            }),
        }
    }
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

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore: Option<FeatureSettings>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<FeatureSettings>,
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
    /// Returns true when at least one feature override is configured.
    pub fn has_configured_overrides(&self) -> bool {
        self.mcp.is_some()
            || self.instructions.is_some()
            || self.commands.is_some()
            || self.skills.is_some()
            || self.ignore.is_some()
            || self.hooks.is_some()
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
            ignore: Self::merge_settings(self.ignore.as_ref(), other.ignore.as_ref()),
            hooks: Self::merge_settings(self.hooks.as_ref(), other.hooks.as_ref()),
        }
    }

    pub fn get_config(&self, feature: &Feature) -> Option<FeatureSettings> {
        match feature {
            Feature::Mcp => self.mcp.clone(),
            Feature::Instruction => self.instructions.clone(),
            Feature::Command => self.commands.clone(),
            Feature::Skill => self.skills.clone(),
            Feature::AgentIgnore => self.ignore.clone(),
            Feature::Hook => self.hooks.clone(),
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
        use crate::core::features::Feature;

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
            ignore: None,
            hooks: None,
        };

        let mcp_config = settings.get_config(&Feature::Mcp);
        assert!(mcp_config.is_some());
        assert_eq!(mcp_config.unwrap().template, Some("mcp.tmpl".to_string()));

        let cmd_config = settings.get_config(&Feature::Command);
        assert!(cmd_config.is_none());

        let skill_config = settings.get_config(&Feature::Skill);
        assert!(skill_config.is_none());
    }

    #[test]
    fn test_features_has_configured_overrides_false_when_empty() {
        // empty feature overrides are reported as absent
        assert!(!Features::default().has_configured_overrides());
    }

    #[test]
    fn test_features_has_configured_overrides_true_when_any_override_present() {
        // any configured feature override is reported as present
        let features = Features {
            commands: Some(FeatureSettings::default()),
            ..Default::default()
        };
        assert!(features.has_configured_overrides());
    }
}
