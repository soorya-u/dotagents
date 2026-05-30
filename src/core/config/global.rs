use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use strum::VariantNames;

#[cfg(feature = "skills-add")]
use super::common::PackageRunner;
use super::common::Providers;
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
    #[cfg(feature = "skills-add")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_runner: Option<PackageRunner>,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            schema: Some(CONFIG_SCHEMA.into()),
            features: HashSet::new(),
            targets: None,
            providers: None,
            variables: None,
            #[cfg(feature = "skills-add")]
            package_runner: None,
            extra: HashMap::new(),
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
            #[cfg(feature = "skills-add")]
            package_runner: None,
            extra: HashMap::new(),
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

#[cfg(feature = "skills-add")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_runner_field_deserialises_in_global_config() {
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
        let config: GlobalConfig = toml::from_str("features = []\n").unwrap();
        assert_eq!(config.package_runner, None);
    }

    #[test]
    fn invalid_package_runner_value_fails_deserialisation() {
        let result: Result<GlobalConfig, _> = toml::from_str("package-runner = \"cargo\"\n");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;

    // unknown string key survives parse/serialize round-trip
    #[test]
    fn unknown_string_key_roundtrip() {
        let toml_str = "features = []\nmy-custom-key = \"hello\"\n";
        let config: GlobalConfig = TomlConfig::from_toml(toml_str).unwrap();
        assert_eq!(
            config.extra.get("my-custom-key"),
            Some(&toml::Value::String("hello".into()))
        );
        let serialized = config.to_toml().unwrap();
        assert!(serialized.contains("my-custom-key"));
        assert!(serialized.contains("hello"));
    }

    // unknown table survives parse/serialize round-trip
    #[test]
    fn unknown_table_roundtrip() {
        let toml_str = "features = []\n\n[metadata]\nauthor = \"alice\"\n";
        let config: GlobalConfig = TomlConfig::from_toml(toml_str).unwrap();
        assert!(config.extra.contains_key("metadata"));
        let serialized = config.to_toml().unwrap();
        assert!(serialized.contains("metadata"));
        assert!(serialized.contains("alice"));
    }

    // unknown array survives parse/serialize round-trip
    #[test]
    fn unknown_array_roundtrip() {
        let toml_str = "features = []\ntags = [\"rust\", \"cli\"]\n";
        let config: GlobalConfig = TomlConfig::from_toml(toml_str).unwrap();
        assert!(config.extra.contains_key("tags"));
        let serialized = config.to_toml().unwrap();
        assert!(serialized.contains("tags"));
        assert!(serialized.contains("rust"));
    }
}
