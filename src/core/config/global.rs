use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use strum::VariantNames;

use super::common::IntegrationsConfig;
use super::common::Providers;
use super::mode::FeatureModeConfig;
use super::traits::TomlConfig;
use crate::constants::schema::CONFIG_SCHEMA;
use crate::core::features::Feature;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default)]
    pub features: HashSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<HashSet<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Providers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrations: Option<IntegrationsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_maps: Option<HashMap<String, FeatureModeConfig>>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: HashSet::new(),
            targets: None,
            providers: None,
            variables: None,
            integrations: None,
            feature_maps: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_features(features: HashSet<String>, targets: HashSet<String>) -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features,
            targets: Some(targets),
            providers: None,
            variables: None,
            integrations: None,
            feature_maps: None,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for feature in &self.features {
            if Feature::from_str(feature.as_str()).is_err() {
                anyhow::bail!(
                    "Invalid feature: {}. Valid features are: {}",
                    feature,
                    Feature::VARIANTS.join(", ")
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
    use crate::integrations::skills_sh::PackageRunner;

    #[test]
    fn integrations_skills_sh_package_runner_deserialises_in_global_config() {
        // [integrations.skills-sh] package-runner deserialises correctly
        for (toml_val, expected) in [
            ("npm", PackageRunner::Npm),
            ("pnpm", PackageRunner::Pnpm),
            ("yarn", PackageRunner::Yarn),
            ("bun", PackageRunner::Bun),
        ] {
            let toml = format!("[integrations.skills-sh]\npackage-runner = \"{toml_val}\"\n");
            let config: GlobalConfig = toml::from_str(&toml).unwrap();
            assert_eq!(
                config
                    .integrations
                    .and_then(|i| i.skills_sh)
                    .and_then(|s| s.package_runner),
                Some(expected)
            );
        }
    }

    #[test]
    fn integrations_absent_yields_none_in_global_config() {
        // no [integrations] table yields None
        let config: GlobalConfig = toml::from_str("features = []\n").unwrap();
        assert_eq!(config.integrations, None);
    }

    #[test]
    fn invalid_package_runner_value_fails_deserialisation() {
        // invalid package-runner value under [integrations.skills-sh] fails
        let result: Result<GlobalConfig, _> =
            toml::from_str("[integrations.skills-sh]\npackage-runner = \"cargo\"\n");
        assert!(result.is_err());
    }

    #[test]
    fn top_level_package_runner_is_ignored() {
        // top-level package-runner is no longer read — serde ignores unknown fields
        let config: GlobalConfig = toml::from_str("package-runner = \"bun\"\n").unwrap();
        assert_eq!(config.integrations, None);
    }
}
