use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::utils::merge::merge_optional;

/// Deploy mode for a feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FeatureMode {
    Link,
    Template,
}

impl fmt::Display for FeatureMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureMode::Link => write!(f, "link"),
            FeatureMode::Template => write!(f, "template"),
        }
    }
}

/// Per-feature mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct FeatureModeConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<FeatureMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode_override: Option<HashMap<String, FeatureMode>>,
}

impl FeatureModeConfig {
    /// Merges local overrides into the global base, preferring non-None local fields.
    pub fn merge(&self, other: &FeatureModeConfig) -> FeatureModeConfig {
        FeatureModeConfig {
            mode: other.mode.or(self.mode),
            mode_override: merge_optional(
                self.mode_override.as_ref(),
                other.mode_override.as_ref(),
                |b, o| {
                    let mut merged = b.clone();
                    merged.extend(o.clone());
                    merged
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_mode_display_link() {
        assert_eq!(FeatureMode::Link.to_string(), "link");
    }

    #[test]
    fn feature_mode_display_template() {
        assert_eq!(FeatureMode::Template.to_string(), "template");
    }

    // serializes "link" to lowercase
    #[test]
    fn feature_mode_serialize_link() {
        let json = serde_json::to_string(&FeatureMode::Link).unwrap();
        assert_eq!(json, "\"link\"");
    }

    // serializes "template" to lowercase
    #[test]
    fn feature_mode_serialize_template() {
        let json = serde_json::to_string(&FeatureMode::Template).unwrap();
        assert_eq!(json, "\"template\"");
    }

    // deserializes "link" to Link variant
    #[test]
    fn feature_mode_deserialize_link() {
        let mode: FeatureMode = serde_json::from_str("\"link\"").unwrap();
        assert_eq!(mode, FeatureMode::Link);
    }

    // deserializes "template" to Template variant
    #[test]
    fn feature_mode_deserialize_template() {
        let mode: FeatureMode = serde_json::from_str("\"template\"").unwrap();
        assert_eq!(mode, FeatureMode::Template);
    }

    // rejects invalid mode value
    #[test]
    fn feature_mode_deserialize_invalid() {
        let result: Result<FeatureMode, _> = serde_json::from_str("\"invalid\"");
        assert!(result.is_err());
    }

    // mode_override merges local over global
    #[test]
    fn feature_mode_config_merge_local_wins() {
        let global = FeatureModeConfig {
            mode: Some(FeatureMode::Link),
            mode_override: Some(HashMap::from([("cmd1".to_string(), FeatureMode::Template)])),
        };
        let local = FeatureModeConfig {
            mode: Some(FeatureMode::Template),
            mode_override: Some(HashMap::from([("cmd2".to_string(), FeatureMode::Link)])),
        };
        let merged = global.merge(&local);
        assert_eq!(merged.mode, Some(FeatureMode::Template));
        let overrides = merged.mode_override.unwrap();
        assert_eq!(overrides.get("cmd1"), Some(&FeatureMode::Template));
        assert_eq!(overrides.get("cmd2"), Some(&FeatureMode::Link));
    }

    // merge local None preserves global values
    #[test]
    fn feature_mode_config_merge_local_absent_preserves_global() {
        let global = FeatureModeConfig {
            mode: Some(FeatureMode::Template),
            mode_override: Some(HashMap::from([("a".to_string(), FeatureMode::Link)])),
        };
        let local = FeatureModeConfig::default();
        let merged = global.merge(&local);
        assert_eq!(merged.mode, Some(FeatureMode::Template));
        assert_eq!(
            merged.mode_override.unwrap().get("a"),
            Some(&FeatureMode::Link)
        );
    }

    // TOML deserialization of kebab-case FeatureModeConfig
    #[test]
    fn feature_mode_config_toml_deserialize() {
        let toml = r#"
mode = "link"
[mode-override]
hello = "template"
"#;
        let config: FeatureModeConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.mode, Some(FeatureMode::Link));
        let overrides = config.mode_override.unwrap();
        assert_eq!(overrides.get("hello"), Some(&FeatureMode::Template));
    }

    // TOML rejects invalid mode value
    #[test]
    fn feature_mode_config_toml_rejects_invalid_mode() {
        let toml = "mode = \"invalid\"\n";
        let result: Result<FeatureModeConfig, _> = toml::from_str(toml);
        assert!(result.is_err());
    }

    // TOML rejects invalid mode_override value
    #[test]
    fn feature_mode_config_toml_rejects_invalid_override() {
        let toml = "[mode-override]\nhello = \"invalid\"\n";
        let result: Result<FeatureModeConfig, _> = toml::from_str(toml);
        assert!(result.is_err());
    }
}
