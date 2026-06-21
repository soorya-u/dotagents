use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use super::common::IntegrationsConfig;
use super::common::Providers;
use super::mode::FeatureModeConfig;
use super::traits::TomlConfig;
use crate::constants::schema::CONFIG_SCHEMA;
use crate::core::features::Feature;
use serde::{Deserialize, Serialize};
use strum::VariantNames;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrations: Option<IntegrationsConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_maps: Option<HashMap<String, FeatureModeConfig>>,
}

impl LocalConfig {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: None,
            targets: None,
            providers: None,
            variables: None,
            integrations: None,
            feature_maps: None,
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
            integrations: None,
            feature_maps: None,
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
            integrations: None,
            feature_maps: None,
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if let Some(features) = &self.features {
            for feature in features {
                if Feature::from_str(feature.as_str()).is_err() {
                    anyhow::bail!(
                        "Invalid feature: {}. Valid features are: {}",
                        feature,
                        Feature::VARIANTS.join(", ")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::skills_sh::PackageRunner;

    #[test]
    fn integrations_skills_sh_package_runner_deserialises_in_local_config() {
        // [integrations.skills-sh] package-runner deserialises in local config
        for (toml_val, expected) in [
            ("npm", PackageRunner::Npm),
            ("pnpm", PackageRunner::Pnpm),
            ("yarn", PackageRunner::Yarn),
            ("bun", PackageRunner::Bun),
        ] {
            let toml = format!("[integrations.skills-sh]\npackage-runner = \"{toml_val}\"\n");
            let config: LocalConfig = toml::from_str(&toml).unwrap();
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
    fn integrations_absent_yields_none_in_local_config() {
        // no [integrations] table yields None
        let config: LocalConfig = toml::from_str("").unwrap();
        assert_eq!(config.integrations, None);
    }
}
