use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    constants::file::AGENTIGNORE_FILE, core::features::traits::FeatureTrait,
    utils::path::get_application_dir,
};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct IgnoreFeature {
    patterns: Vec<String>,
}

impl IgnoreFeature {
    pub(crate) fn mock() -> &'static str {
        crate::constants::mocks::AGENTIGNORE_MOCK
    }

    pub fn from_application() -> Result<Self> {
        let dir = get_application_dir()?;
        let path = dir.join(AGENTIGNORE_FILE);
        let content = if path.exists() {
            fs::read_to_string(&path)?
        } else {
            String::new()
        };
        Self::from_string(&content)
    }
}

impl FeatureTrait for IgnoreFeature {
    fn from_string(value: &str) -> Result<Self> {
        let patterns: Vec<String> = value
            .lines()
            .map(|line| line.to_string())
            .filter(|line| !line.is_empty())
            .collect();
        Ok(Self { patterns })
    }

    fn to_string(&self) -> Result<String> {
        let mut output = String::new();
        for pattern in &self.patterns {
            output.push_str(pattern);
            output.push('\n');
        }
        Ok(output)
    }

    fn to_value(&self) -> Value {
        json!({
            "ignore": {
                "patterns": self.patterns
            }
        })
    }

    fn is_symlinkable(&self) -> bool {
        true
    }

    fn is_provider_agnostic() -> bool {
        true
    }

    fn resolve_source_path(_name: Option<&str>) -> Result<PathBuf> {
        Ok(get_application_dir()?.join(AGENTIGNORE_FILE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_parses_newline_separated_patterns() {
        let input = "node_modules/\n*.log\n.env\n";
        let ignore = IgnoreFeature::from_string(input).unwrap();
        assert_eq!(ignore.patterns, vec!["node_modules/", "*.log", ".env"]);
    }

    #[test]
    fn from_string_skips_empty_lines() {
        let input = "node_modules/\n\n*.log\n";
        let ignore = IgnoreFeature::from_string(input).unwrap();
        assert_eq!(ignore.patterns, vec!["node_modules/", "*.log"]);
    }

    #[test]
    fn from_string_handles_empty_input() {
        let ignore = IgnoreFeature::from_string("").unwrap();
        assert!(ignore.patterns.is_empty());
    }

    #[test]
    fn to_string_produces_newline_separated_patterns() {
        let ignore = IgnoreFeature {
            patterns: vec!["node_modules/".to_string(), "*.log".to_string()],
        };
        let result = ignore.to_string().unwrap();
        assert_eq!(result, "node_modules/\n*.log\n");
    }

    #[test]
    fn to_string_empty_patterns_produces_empty_string() {
        let ignore = IgnoreFeature { patterns: vec![] };
        let result = ignore.to_string().unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn to_value_returns_correct_json_structure() {
        let ignore = IgnoreFeature {
            patterns: vec!["node_modules/".to_string(), "*.log".to_string()],
        };
        let value = ignore.to_value();
        assert_eq!(
            value,
            json!({
                "ignore": {
                    "patterns": ["node_modules/", "*.log"]
                }
            })
        );
    }

    #[test]
    fn roundtrip_preserves_patterns() {
        let original = "node_modules/\n*.log\n.env\n";
        let ignore = IgnoreFeature::from_string(original).unwrap();
        let result = ignore.to_string().unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn get_file_name_returns_none() {
        let ignore = IgnoreFeature {
            patterns: vec!["node_modules/".to_string()],
        };
        assert!(ignore.get_file_name().is_none());
    }
}
