use std::fs;

use anyhow::{Context, Result};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    constants::templates::{COMMAND_STARTER, render_starter},
    core::features::traits::FeatureTrait,
    templates::variables::get_command_name_variable,
    utils::path::get_commands_dir,
};

#[derive(Serialize, Deserialize)]
pub(crate) struct CommandMetadata {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CommandFeature {
    pub metadata: CommandMetadata,
    pub content: String,
}

impl CommandFeature {
    /// Returns the starter hello.md mock content used during `init`.
    pub(crate) fn mock() -> &'static str {
        crate::constants::mocks::COMMAND_MOCK
    }

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

    /// Build the file content for a new command from the given fields.
    pub fn scaffold(
        name: &str,
        description: &str,
        category: &str,
        tags: &[String],
    ) -> Result<String> {
        let feature = CommandFeature {
            metadata: CommandMetadata {
                name: name.to_string(),
                description: description.to_string(),
                category: if category.is_empty() {
                    None
                } else {
                    Some(category.to_string())
                },
                tags: if tags.is_empty() {
                    None
                } else {
                    Some(tags.to_vec())
                },
            },
            content: render_starter(COMMAND_STARTER, name),
        };
        feature.to_markdown()
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
        let mut meta = serde_json::to_value(&self.metadata).unwrap_or_default();
        if let Value::Object(ref mut map) = meta {
            map.insert("content".to_string(), json!(self.content));
        }
        json!({ "command": meta })
    }

    fn get_file_name(&self) -> Option<String> {
        Some(self.metadata.name.clone())
    }

    fn get_name_variable(&self, filename: &str) -> Result<Option<Value>> {
        Ok(Some(get_command_name_variable(filename)?))
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
                category: None,
                tags: None,
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
                category: None,
                tags: None,
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
                category: None,
                tags: None,
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
    fn test_to_value_with_category_and_tags() {
        // to_value includes category and tags when set
        let command = CommandFeature {
            metadata: CommandMetadata {
                name: "cmd".to_string(),
                description: "desc".to_string(),
                category: Some("Workflow".to_string()),
                tags: Some(vec!["a".to_string(), "b".to_string()]),
            },
            content: "body".to_string(),
        };

        let value = command.to_value();
        assert_eq!(value["command"]["category"], "Workflow");
        assert_eq!(value["command"]["tags"][0], "a");
    }

    #[test]
    fn test_get_file_name() {
        let command = CommandFeature {
            metadata: CommandMetadata {
                name: "file-name-test".to_string(),
                description: "Test".to_string(),
                category: None,
                tags: None,
            },
            content: "Content".to_string(),
        };

        assert_eq!(command.get_file_name(), Some("file-name-test".to_string()));
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
                category: None,
                tags: None,
            },
            content: "Content".to_string(),
        };

        let result = command.to_string().unwrap();
        assert!(result.contains("name: to-string-test"));
        assert!(result.contains("description: To string test"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_scaffold_returns_markdown_with_body() {
        // scaffold produces valid markdown with name, description, and starter body
        let content =
            CommandFeature::scaffold("my-cmd", "Does things", "Workflow", &["tag1".to_string()])
                .unwrap();
        assert!(content.starts_with("---"));
        assert!(content.contains("name: my-cmd"));
        assert!(content.contains("description: Does things"));
        assert!(content.contains("category: Workflow"));
        assert!(content.contains("tag1"));
    }

    #[test]
    fn test_scaffold_empty_optional_fields_omitted() {
        // scaffold omits category and tags when empty
        let content = CommandFeature::scaffold("cmd", "desc", "", &[]).unwrap();
        assert!(!content.contains("category"));
        assert!(!content.contains("tags"));
    }
}
