use std::collections::{HashMap, HashSet};

#[cfg(feature = "skills-add")]
use super::common::PackageRunner;
use super::common::Providers;
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
    pub targets: Option<HashSet<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
    #[cfg(feature = "skills-add")]
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
            #[cfg(feature = "skills-add")]
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
            #[cfg(feature = "skills-add")]
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
            #[cfg(feature = "skills-add")]
            package_runner: None,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
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
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TomlConfig for LocalConfig {}

#[cfg(feature = "skills-add")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_runner_field_deserialises_in_local_config() {
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
        let config: LocalConfig = toml::from_str("").unwrap();
        assert_eq!(config.package_runner, None);
    }
}
