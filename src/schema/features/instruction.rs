use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    constants::file::INSTRUCTIONS_FILE,
    schema::features::traits::FeatureTrait,
    utils::{gitignore::GitignoreScope, path::get_application_dir},
};

#[derive(Serialize, Deserialize)]
pub(crate) struct InstructionFeature {
    content: String,
}

impl InstructionFeature {
    pub fn from_application() -> Result<Self> {
        let dir = get_application_dir()?;
        let path = dir.join(INSTRUCTIONS_FILE);
        let content = fs::read_to_string(path)?;
        Ok(Self { content })
    }
}

impl FeatureTrait for InstructionFeature {
    fn from_string(value: &str) -> Result<Self> {
        Ok(Self {
            content: value.into(),
        })
    }

    fn to_string(&self) -> Result<String> {
        Ok(self.content.clone())
    }

    fn to_value(&self) -> Value {
        json!({
            "instruction": {
                "content": self.content
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        let content = "This is an instruction";
        let instruction = InstructionFeature::from_string(content).unwrap();
        assert_eq!(instruction.content, content);
    }

    #[test]
    fn test_to_string() {
        let instruction = InstructionFeature {
            content: "Test instruction".to_string(),
        };
        let result = instruction.to_string().unwrap();
        assert_eq!(result, "Test instruction");
    }

    #[test]
    fn test_to_value() {
        let instruction = InstructionFeature {
            content: "Sample instruction".to_string(),
        };
        let value = instruction.to_value();
        assert_eq!(
            value,
            json!({
                "instruction": {
                    "content": "Sample instruction"
                }
            })
        );
    }

    #[test]
    fn test_roundtrip() {
        let original = "Complex instruction\nwith multiple lines\nand content";
        let instruction = InstructionFeature::from_string(original).unwrap();
        let result = instruction.to_string().unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_empty_content() {
        let instruction = InstructionFeature::from_string("").unwrap();
        assert_eq!(instruction.content, "");
        assert_eq!(instruction.to_string().unwrap(), "");
    }

    #[test]
    fn test_gitignore_scope_is_file() {
        // instructions are a single file per provider → File scope (default)
        let instruction = InstructionFeature::from_string("content").unwrap();
        assert!(matches!(
            instruction.gitignore_scope(),
            GitignoreScope::File
        ));
    }
}
