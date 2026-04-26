use std::collections::{HashMap, HashSet};

use super::common::{PackageRunner, Providers, Targets};
use super::traits::TomlConfig;
use crate::constants::schema::CONFIG_SCHEMA;
use crate::schema::features::Feature;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default)]
    pub features: HashSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Targets>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_runner: Option<PackageRunner>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: HashSet::new(),
            targets: None,
            providers: None,
            variables: None,
            package_runner: None,
        }
    }

    pub fn with_features(features: HashSet<String>, targets: Targets) -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features,
            targets: Some(targets),
            providers: None,
            variables: None,
            package_runner: None,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for feature in &self.features {
            if Feature::from_str(feature).is_none() {
                anyhow::bail!(
                    "Invalid feature: {}. Valid features are: {}",
                    feature,
                    Feature::all_names().join(", ")
                );
            }
        }

        Ok(())
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TomlConfig for GlobalConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_runner_field_deserialises_in_global_config() {
        // all four runner values parse correctly from TOML
        for (toml_val, expected) in [
            ("npm", PackageRunner::Npm),
            ("pnpm", PackageRunner::Pnpm),
            ("yarn", PackageRunner::Yarn),
            ("bun", PackageRunner::Bun),
        ] {
            let toml = format!("package-runner = \"{toml_val}\"\n");
            let config: GlobalConfig = toml::from_str(&toml).unwrap();
            assert_eq!(config.package_runner, Some(expected));
        }
    }

    #[test]
    fn package_runner_absent_yields_none_in_global_config() {
        // omitting the field deserialises to None
        let config: GlobalConfig = toml::from_str("features = []\n").unwrap();
        assert_eq!(config.package_runner, None);
    }

    #[test]
    fn invalid_package_runner_value_fails_deserialisation() {
        // an unrecognised runner value should fail with a serde error
        let result: Result<GlobalConfig, _> = toml::from_str("package-runner = \"cargo\"\n");
        assert!(result.is_err());
    }
}
