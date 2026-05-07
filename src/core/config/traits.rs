use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub trait TomlConfig: Serialize + for<'de> Deserialize<'de> {
    #[allow(dead_code)]
    fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize configuration to TOML")
    }

    fn from_toml(toml_str: &str) -> Result<Self> {
        toml::from_str(toml_str).context("Failed to deserialize configuration from TOML")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    impl TomlConfig for TestConfig {}

    #[test]
    fn test_to_toml() {
        let config = TestConfig {
            name: "test".to_string(),
            value: 42,
        };
        let toml_str = config.to_toml().unwrap();
        assert!(toml_str.contains("name = \"test\""));
        assert!(toml_str.contains("value = 42"));
    }

    #[test]
    fn test_from_toml() {
        let toml_str = r#"
            name = "test"
            value = 42
        "#;
        let config = TestConfig::from_toml(toml_str).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
    }

    #[test]
    fn test_roundtrip() {
        let original = TestConfig {
            name: "roundtrip".to_string(),
            value: 123,
        };
        let toml_str = original.to_toml().unwrap();
        let decoded = TestConfig::from_toml(&toml_str).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_from_toml_invalid() {
        let invalid_toml = "not valid toml {]";
        let result = TestConfig::from_toml(invalid_toml);
        assert!(result.is_err());
    }
}
