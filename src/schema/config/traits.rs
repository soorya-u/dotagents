use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub trait TomlConfig: Serialize + for<'de> Deserialize<'de> {
    fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize configuration to TOML")
    }

    fn from_toml(toml_str: &str) -> Result<Self> {
        toml::from_str(toml_str).context("Failed to deserialize configuration from TOML")
    }
}
