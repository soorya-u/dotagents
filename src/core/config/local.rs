use std::collections::{HashMap, HashSet};

use super::common::{PackageRunner, Providers, Targets};
use super::traits::TomlConfig;
use crate::constants::schema::CONFIG_SCHEMA;
use crate::core::features::Feature;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct LocalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<HashSet<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Targets>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_runner: Option<PackageRunner>,
}

impl LocalConfig {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: None,
            targets: None,
            providers: None,
            variables: None,
            package_runner: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_features(features: HashSet<String>) -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: Some(features),
            targets: None,
            providers: None,
            variables: None,
            package_runner: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_providers(providers: Providers) -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: None,
            targets: None,
            providers: Some(providers),
            variables: None,
            package_runner: None,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        // Check that features are valid if present
        if let Some(features) = &self.features {
            for feature in features {
                if Feature::from_str(feature).is_none() {
                    anyhow::bail!(
                        "Invalid feature: {}. Valid features are: {}",
                        feature,
                        Feature::all_names().join(", ")
                    );
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.schema.is_none()
            && self.features.is_none()
            && self.targets.is_none()
            && self.providers.is_none()
            && self.package_runner.is_none()
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TomlConfig for LocalConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_runner_field_deserialises_in_local_config() {
        // all four runner values parse correctly from TOML
        for (toml_val, expected) in [
            ("npm", PackageRunner::Npm),
            ("pnpm", PackageRunner::Pnpm),
            ("yarn", PackageRunner::Yarn),
            ("bun", PackageRunner::Bun),
        ] {
            let toml = format!("package-runner = \"{toml_val}\"\n");
            let config: LocalConfig = toml::from_str(&toml).unwrap();
            assert_eq!(config.package_runner, Some(expected));
        }
    }

    #[test]
    fn package_runner_absent_yields_none_in_local_config() {
        // omitting the field deserialises to None
        let config: LocalConfig = toml::from_str("").unwrap();
        assert_eq!(config.package_runner, None);
    }
}
