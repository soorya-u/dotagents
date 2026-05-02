use std::fs;

use anyhow::{Context, Result};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    schema::features::traits::FeatureTrait,
    templates::variables::get_command_name_variable,
    utils::{gitignore::GitignoreScope, path::get_commands_dir},
};

#[derive(Serialize, Deserialize)]
pub(crate) struct CommandMetadata {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CommandFeature {
    pub metadata: CommandMetadata,
    pub content: String,
}

impl CommandFeature {
    pub fn to_markdown(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self.metadata)
            .context("failed to serialize metadata to YAML")?;

        Ok(format!("---\n{}---\n\n{}", yaml, self.content))
    }

    pub fn from_markdown(md: &str) -> Result<Self> {
        let matter = Matter::<YAML>::new();
        let parsed = matter.parse(md).context("failed to parse markdown")?;

        let metadata: CommandMetadata = parsed.data.context("failed to parse markdown metadata")?;

        Ok(CommandFeature {
            metadata,
            content: parsed.content,
        })
    }

    pub fn from_application() -> Result<Vec<Self>> {
        let dir = get_commands_dir()?;
        let mut commands = Vec::<Self>::new();

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let content = fs::read_to_string(&path).context("failed to read file")?;
                let command = Self::from_markdown(&content)?;
                commands.push(command);
            }
        }

        Ok(commands)
    }
}

impl FeatureTrait for CommandFeature {
    fn to_string(&self) -> Result<String> {
        self.to_markdown()
    }

    fn from_string(value: &str) -> Result<Self> {
        Self::from_markdown(value)
    }

    fn to_value(&self) -> Value {
        json!({
            "command": {
                "name": self.metadata.name,
                "description": self.metadata.description,
                "content": self.content
            }
        })
    }

    fn get_file_name(&self) -> Option<String> {
        Some(self.metadata.name.clone())
    }

    fn get_name_variable(&self, filename: &str) -> Result<Option<Value>> {
        Ok(Some(get_command_name_variable(filename)?))
    }

    fn gitignore_scope(&self) -> GitignoreScope {
        GitignoreScope::File
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_markdown() {
        let command = CommandFeature {
            metadata: CommandMetadata {
                name: "test-command".to_string(),
                description: "A test command".to_string(),
            },
            content: "Command content here".to_string(),
        };

        let md = command.to_markdown().unwrap();
        assert!(md.starts_with("---"));
        assert!(md.contains("name: test-command"));
        assert!(md.contains("description: A test command"));
        assert!(md.contains("Command content here"));
    }

    #[test]
    fn test_from_markdown() {
        let md = r#"---
name: test-command
description: A test command
---

Command content here"#;

        let command = CommandFeature::from_markdown(md).unwrap();
        assert_eq!(command.metadata.name, "test-command");
        assert_eq!(command.metadata.description, "A test command");
        assert_eq!(command.content, "Command content here");
    }

    #[test]
    fn test_roundtrip() {
        let original = CommandFeature {
            metadata: CommandMetadata {
                name: "roundtrip-test".to_string(),
                description: "Testing roundtrip conversion".to_string(),
            },
            content: "Content with\nmultiple lines".to_string(),
        };

        let md = original.to_markdown().unwrap();
        let parsed = CommandFeature::from_markdown(&md).unwrap();

        assert_eq!(parsed.metadata.name, original.metadata.name);
        assert_eq!(parsed.metadata.description, original.metadata.description);
        assert_eq!(parsed.content, original.content);
    }

    #[test]
    fn test_to_value() {
        let command = CommandFeature {
            metadata: CommandMetadata {
                name: "value-test".to_string(),
                description: "Testing value conversion".to_string(),
            },
            content: "Value content".to_string(),
        };

        let value = command.to_value();
        assert_eq!(
            value,
            json!({
                "command": {
                    "name": "value-test",
                    "description": "Testing value conversion",
                    "content": "Value content"
                }
            })
        );
    }

    #[test]
    fn test_get_file_name() {
        let command = CommandFeature {
            metadata: CommandMetadata {
                name: "file-name-test".to_string(),
                description: "Test".to_string(),
            },
            content: "Content".to_string(),
        };

        assert_eq!(command.get_file_name(), Some("file-name-test".to_string()));
    }

    #[test]
    fn test_gitignore_scope_is_file() {
        // commands use File scope — individual file paths in .gitignore
        let command = CommandFeature {
            metadata: CommandMetadata {
                name: "scope-test".to_string(),
                description: "Test".to_string(),
            },
            content: "Content".to_string(),
        };
        assert!(matches!(command.gitignore_scope(), GitignoreScope::File));
    }

    #[test]
    fn test_from_string() {
        let md = r#"---
name: string-test
description: From string test
---

String content"#;

        let command = CommandFeature::from_string(md).unwrap();
        assert_eq!(command.metadata.name, "string-test");
        assert_eq!(command.metadata.description, "From string test");
        assert_eq!(command.content, "String content");
    }

    #[test]
    fn test_to_string() {
        let command = CommandFeature {
            metadata: CommandMetadata {
                name: "to-string-test".to_string(),
                description: "To string test".to_string(),
            },
            content: "Content".to_string(),
        };

        let result = command.to_string().unwrap();
        assert!(result.contains("name: to-string-test"));
        assert!(result.contains("description: To string test"));
        assert!(result.contains("Content"));
    }
}
