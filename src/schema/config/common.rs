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
            variables: merge_optional(self.variables.as_ref(), other.variables.as_ref(), |b, o| {
                b.clone().into_iter().chain(o.clone()).collect()
            }),
            hash: other.hash.clone().or_else(|| self.hash.clone()),
        }
    }

    pub fn render_template<T: FeatureTrait>(
        &self,
        templater: &Templater,
        name: &str,
        variables: Option<&Value>,
        feature: &T,
    ) -> Result<()> {
        let template_str = self
            .template
            .as_deref()
            .ok_or_else(|| anyhow!("Template config not found for provider {}", name))?;

        let target_str = self
            .target
            .as_deref()
            .ok_or_else(|| anyhow!("Target config not found for provider {}", name))?;

        // Only create PathBuf when needed for filesystem operations
        let template_path = PathBuf::from(template_str);
        let mut target_path = if let Some(filename) = feature.get_file_name() {
            let command_var = get_command_name_variable(&filename)?;
            PathBuf::from(templater.render_template(
                RenderType::Content(target_str.to_string()),
                Some(&command_var),
            )?)
        } else {
            PathBuf::from(target_str)
        };

        if !template_path.exists() {
            return Err(anyhow!(
                "Template file not found for {} provider at {}",
                name,
                template_path.display()
            ));
        }

        if target_path.exists() {
            warn!("Replacing existing file at {}", target_path.display());
        }

        let local_vars = self.variables.as_ref().map(to_value).transpose()?;

        let user_vars =
            get_user_defined_variables(Some(merge_json(variables, local_vars.as_ref())))?;

        let populate_config = feature.populate_with_values(templater, Some(&user_vars))?;

        let feature_as_variables = populate_config.to_value();

        let template_file_content = read_file(&template_path).context(format!(
            "failed to read file in {}",
            template_path.display()
        ))?;

        let vars = merge_json(Some(&user_vars), Some(&feature_as_variables));
        let content =
            templater.render_template(RenderType::Content(template_file_content), Some(&vars))?;

        write_file(&target_path, &content)
            .context(format!("failed to write file in {}", target_path.display()))?;

        Ok(())
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
        let base = ConfigAgentSettings {
            template: Some("base.tmpl".to_string()),
            target: Some("base.target".to_string()),
            disabled: Some(false),
            variables: Some(HashMap::from([("key1".to_string(), "value1".to_string())])),
            hash: None,
        };

        let override_settings = ConfigAgentSettings {
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
        let settings = ConfigAgentAbilitySettings {
            mcp: Some(ConfigAgentSettings {
                template: Some("mcp.tmpl".to_string()),
                ..Default::default()
            }),
            instructions: Some(ConfigAgentSettings {
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
