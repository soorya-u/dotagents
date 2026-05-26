use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{core::features::Feature, utils::merge::merge_optional};

/// Package runner used to invoke the `skills` CLI.
#[cfg(feature = "skills-add")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum PackageRunner {
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

#[cfg(feature = "skills-add")]
impl PackageRunner {
    /// Returns the executable name to check on PATH and use as the first argv element.
    pub(crate) fn binary(&self) -> &str {
        match self {
            PackageRunner::Npm => "npx",
            PackageRunner::Pnpm => "pnpm",
            PackageRunner::Yarn => "yarn",
            PackageRunner::Bun => "bunx",
        }
    }

    /// Returns the full argument list for `skills add <skill_name>`.
    /// When `ci` is true, appends `--yes` to skip interactive confirmation prompts.
    pub(crate) fn args(&self, skill_name: &str, ci: bool) -> Vec<String> {
        let mut v = match self {
            PackageRunner::Npm => vec![
                "npx".into(),
                "skills".into(),
                "add".into(),
                skill_name.into(),
                "--agent".into(),
                "claude-code".into(),
            ],
            PackageRunner::Pnpm => vec![
                "pnpm".into(),
                "dlx".into(),
                "skills".into(),
                "add".into(),
                skill_name.into(),
                "--agent".into(),
                "claude-code".into(),
            ],
            PackageRunner::Yarn => vec![
                "yarn".into(),
                "dlx".into(),
                "skills".into(),
                "add".into(),
                skill_name.into(),
                "--agent".into(),
                "claude-code".into(),
            ],
            PackageRunner::Bun => vec![
                "bunx".into(),
                "skills".into(),
                "add".into(),
                skill_name.into(),
                "--agent".into(),
                "claude-code".into(),
            ],
        };
        if ci {
            v.push("--yes".into());
        }
        v
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
        }
    }

    pub fn get_config(&self, feature: &Feature) -> Option<FeatureSettings> {
        match feature {
            Feature::Mcp => self.mcp.clone(),
            Feature::Instruction => self.instructions.clone(),
            Feature::Command => self.commands.clone(),
            Feature::Skill => self.skills.clone(),
            Feature::AgentIgnore => self.ignore.clone(),
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
        };

        let mcp_config = settings.get_config(&Feature::Mcp);
        assert!(mcp_config.is_some());
        assert_eq!(mcp_config.unwrap().template, Some("mcp.tmpl".to_string()));

        let cmd_config = settings.get_config(&Feature::Command);
        assert!(cmd_config.is_none());

        let skill_config = settings.get_config(&Feature::Skill);
        assert!(skill_config.is_none());
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_serialises_to_lowercase() {
        #[derive(Serialize, Deserialize)]
        struct W {
            r: PackageRunner,
        }
        for (variant, expected) in [
            (PackageRunner::Npm, "npm"),
            (PackageRunner::Pnpm, "pnpm"),
            (PackageRunner::Yarn, "yarn"),
            (PackageRunner::Bun, "bun"),
        ] {
            let s = toml::to_string(&W { r: variant }).unwrap();
            assert!(
                s.contains(&format!("\"{expected}\"")),
                "expected \"{expected}\" in: {s}"
            );
        }
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_deserialises_from_lowercase() {
        #[derive(Serialize, Deserialize)]
        struct W {
            r: PackageRunner,
        }
        for (toml_val, expected) in [
            ("npm", PackageRunner::Npm),
            ("pnpm", PackageRunner::Pnpm),
            ("yarn", PackageRunner::Yarn),
            ("bun", PackageRunner::Bun),
        ] {
            let w: W = toml::from_str(&format!("r = \"{toml_val}\"\n")).unwrap();
            assert_eq!(w.r, expected);
        }
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_args_npm() {
        let args = PackageRunner::Npm.args("vercel-labs/agent-skills", false);
        assert_eq!(
            args,
            vec![
                "npx",
                "skills",
                "add",
                "vercel-labs/agent-skills",
                "--agent",
                "claude-code"
            ]
        );
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_args_npm_ci() {
        let args = PackageRunner::Npm.args("vercel-labs/agent-skills", true);
        assert_eq!(
            args,
            vec![
                "npx",
                "skills",
                "add",
                "vercel-labs/agent-skills",
                "--agent",
                "claude-code",
                "--yes"
            ]
        );
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_args_pnpm() {
        let args = PackageRunner::Pnpm.args("my-skill", false);
        assert_eq!(
            args,
            vec![
                "pnpm",
                "dlx",
                "skills",
                "add",
                "my-skill",
                "--agent",
                "claude-code"
            ]
        );
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_args_pnpm_ci() {
        let args = PackageRunner::Pnpm.args("my-skill", true);
        assert_eq!(
            args,
            vec![
                "pnpm",
                "dlx",
                "skills",
                "add",
                "my-skill",
                "--agent",
                "claude-code",
                "--yes"
            ]
        );
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_args_yarn() {
        let args = PackageRunner::Yarn.args("my-skill", false);
        assert_eq!(
            args,
            vec![
                "yarn",
                "dlx",
                "skills",
                "add",
                "my-skill",
                "--agent",
                "claude-code"
            ]
        );
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_args_yarn_ci() {
        let args = PackageRunner::Yarn.args("my-skill", true);
        assert_eq!(
            args,
            vec![
                "yarn",
                "dlx",
                "skills",
                "add",
                "my-skill",
                "--agent",
                "claude-code",
                "--yes"
            ]
        );
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_args_bun() {
        let args = PackageRunner::Bun.args("my-skill", false);
        assert_eq!(
            args,
            vec![
                "bunx",
                "skills",
                "add",
                "my-skill",
                "--agent",
                "claude-code"
            ]
        );
    }

    #[cfg(feature = "skills-add")]
    #[test]
    fn package_runner_args_bun_ci() {
        let args = PackageRunner::Bun.args("my-skill", true);
        assert_eq!(
            args,
            vec![
                "bunx",
                "skills",
                "add",
                "my-skill",
                "--agent",
                "claude-code",
                "--yes"
            ]
        );
    }
}
