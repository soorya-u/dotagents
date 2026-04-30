use std::collections::HashMap;
use std::fs;

use crate::prelude::*;
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    constants::file::SKILL_FILE,
    schema::features::traits::FeatureTrait,
    templates::variables::get_skill_name_variable,
    utils::{gitignore::GitignoreScope, path::get_skills_dir},
};

#[derive(Serialize, Deserialize)]
pub(crate) struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(rename = "allowed-tools", skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SkillFeature {
    pub metadata: SkillMetadata,
    pub content: String,
}

impl SkillFeature {
    pub fn to_markdown(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self.metadata)
            .context("failed to serialize skill metadata to YAML")?;

        Ok(format!("---\n{}---\n\n{}", yaml, self.content))
    }

    pub fn from_markdown(md: &str) -> Result<Self> {
        let matter = Matter::<YAML>::new();
        let parsed = matter.parse(md).context("failed to parse skill markdown")?;

        let metadata: SkillMetadata = parsed
            .data
            .context("failed to parse skill markdown metadata")?;

        Ok(SkillFeature {
            metadata,
            content: parsed.content,
        })
    }

    pub fn from_application() -> Result<Vec<Self>> {
        let skills_dir = get_skills_dir()?;
        let mut skills = Vec::<Self>::new();

        for entry in fs::read_dir(&skills_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let skill_md = path.join(SKILL_FILE);

            if !skill_md.is_file() {
                warn!(
                    "Skill directory '{}' has no {}, skipping",
                    path.display(),
                    SKILL_FILE
                );
                continue;
            }

            let content = fs::read_to_string(&skill_md).context("failed to read skill SKILL.md")?;
            let skill = Self::from_markdown(&content)?;

            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
                && dir_name != skill.metadata.name
            {
                warn!(
                    "Skill directory name '{}' does not match name '{}' in SKILL.md (path: {})",
                    dir_name,
                    skill.metadata.name,
                    skill_md.display()
                );
            }

            skills.push(skill);
        }

        Ok(skills)
    }
}

impl FeatureTrait for SkillFeature {
    fn to_string(&self) -> Result<String> {
        self.to_markdown()
    }

    fn from_string(value: &str) -> Result<Self> {
        Self::from_markdown(value)
    }

    fn to_value(&self) -> Value {
        let mut skill = serde_json::to_value(&self.metadata).unwrap_or_default();
        if let Value::Object(ref mut map) = skill {
            map.insert("content".to_string(), json!(self.content));
        }
        json!({ "skill": skill })
    }

    fn get_file_name(&self) -> Option<String> {
        Some(self.metadata.name.clone())
    }

    fn get_name_variable(&self, filename: &str) -> Result<Option<Value>> {
        Ok(Some(get_skill_name_variable(filename)?))
    }

    fn gitignore_scope(&self) -> GitignoreScope {
        GitignoreScope::Directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_all_fields() {
        let md = r#"---
name: full-skill
description: A skill with all fields
license: MIT
compatibility: Any agent
metadata:
  author: tester
  version: "1.0"
allowed-tools: Read Write Edit
---

Body content here"#;

        let skill = SkillFeature::from_markdown(md).unwrap();
        assert_eq!(skill.metadata.name, "full-skill");
        assert_eq!(skill.metadata.description, "A skill with all fields");
        assert_eq!(skill.metadata.license, Some("MIT".to_string()));
        assert_eq!(skill.metadata.compatibility, Some("Any agent".to_string()));
        assert_eq!(
            skill.metadata.allowed_tools,
            Some("Read Write Edit".to_string())
        );
        let metadata = skill.metadata.metadata.unwrap();
        assert_eq!(metadata.get("author"), Some(&"tester".to_string()));
        assert_eq!(metadata.get("version"), Some(&"1.0".to_string()));
        assert_eq!(skill.content, "Body content here");
    }

    #[test]
    fn test_parse_required_only() {
        let md = r#"---
name: minimal-skill
description: Only required fields
---

Minimal body"#;

        let skill = SkillFeature::from_markdown(md).unwrap();
        assert_eq!(skill.metadata.name, "minimal-skill");
        assert_eq!(skill.metadata.description, "Only required fields");
        assert!(skill.metadata.license.is_none());
        assert!(skill.metadata.compatibility.is_none());
        assert!(skill.metadata.metadata.is_none());
        assert!(skill.metadata.allowed_tools.is_none());
        assert_eq!(skill.content, "Minimal body");
    }

    #[test]
    fn test_serialize_omits_absent_optionals() {
        let skill = SkillFeature {
            metadata: SkillMetadata {
                name: "no-optionals".to_string(),
                description: "No optional fields set".to_string(),
                license: None,
                compatibility: None,
                metadata: None,
                allowed_tools: None,
            },
            content: "Body".to_string(),
        };

        let md = skill.to_markdown().unwrap();
        assert!(md.contains("name: no-optionals"));
        assert!(md.contains("description: No optional fields set"));
        assert!(!md.contains("license"));
        assert!(!md.contains("compatibility"));
        assert!(!md.contains("metadata"));
        assert!(!md.contains("allowed-tools"));
        assert!(!md.contains("null"));
    }

    #[test]
    fn test_roundtrip() {
        let mut metadata_map = HashMap::new();
        metadata_map.insert("key".to_string(), "value".to_string());

        let original = SkillFeature {
            metadata: SkillMetadata {
                name: "roundtrip".to_string(),
                description: "Roundtrip skill".to_string(),
                license: Some("Apache-2.0".to_string()),
                compatibility: Some("Claude, Codex".to_string()),
                metadata: Some(metadata_map),
                allowed_tools: Some("Read Write".to_string()),
            },
            content: "Multi\nline\ncontent".to_string(),
        };

        let md = original.to_markdown().unwrap();
        let parsed = SkillFeature::from_markdown(&md).unwrap();

        assert_eq!(parsed.metadata.name, original.metadata.name);
        assert_eq!(parsed.metadata.description, original.metadata.description);
        assert_eq!(parsed.metadata.license, original.metadata.license);
        assert_eq!(
            parsed.metadata.compatibility,
            original.metadata.compatibility
        );
        assert_eq!(
            parsed.metadata.allowed_tools,
            original.metadata.allowed_tools
        );
        assert_eq!(parsed.metadata.metadata, original.metadata.metadata);
        assert_eq!(parsed.content, original.content);
    }

    #[test]
    fn test_to_value_includes_all_metadata_fields() {
        let mut meta_map = HashMap::new();
        meta_map.insert("author".to_string(), "tester".to_string());

        let skill = SkillFeature {
            metadata: SkillMetadata {
                name: "value-skill".to_string(),
                description: "Value test".to_string(),
                license: Some("MIT".to_string()),
                compatibility: Some("Claude, Codex".to_string()),
                metadata: Some(meta_map),
                allowed_tools: Some("Read Write".to_string()),
            },
            content: "Skill body".to_string(),
        };

        let value = skill.to_value();
        assert_eq!(
            value,
            json!({
                "skill": {
                    "name": "value-skill",
                    "description": "Value test",
                    "license": "MIT",
                    "compatibility": "Claude, Codex",
                    "metadata": { "author": "tester" },
                    "allowed-tools": "Read Write",
                    "content": "Skill body"
                }
            })
        );
    }

    #[test]
    fn test_to_value_omits_absent_optional_fields() {
        let skill = SkillFeature {
            metadata: SkillMetadata {
                name: "minimal-skill".to_string(),
                description: "Minimal".to_string(),
                license: None,
                compatibility: None,
                metadata: None,
                allowed_tools: None,
            },
            content: "Body".to_string(),
        };

        let value = skill.to_value();
        let skill_obj = value.get("skill").unwrap();
        assert_eq!(skill_obj.get("name").unwrap(), "minimal-skill");
        assert_eq!(skill_obj.get("content").unwrap(), "Body");
        assert!(skill_obj.get("license").is_none());
        assert!(skill_obj.get("compatibility").is_none());
        assert!(skill_obj.get("metadata").is_none());
        assert!(skill_obj.get("allowed-tools").is_none());
    }

    #[test]
    fn test_get_file_name() {
        let skill = SkillFeature {
            metadata: SkillMetadata {
                name: "file-name".to_string(),
                description: "Test".to_string(),
                license: None,
                compatibility: None,
                metadata: None,
                allowed_tools: None,
            },
            content: "Body".to_string(),
        };

        assert_eq!(skill.get_file_name(), Some("file-name".to_string()));
    }

    #[test]
    fn test_gitignore_scope_is_directory() {
        // skills deploy many files into an owned directory → Directory scope
        let skill = SkillFeature {
            metadata: SkillMetadata {
                name: "scope-test".to_string(),
                description: "Test".to_string(),
                license: None,
                compatibility: None,
                metadata: None,
                allowed_tools: None,
            },
            content: "Body".to_string(),
        };
        assert!(matches!(skill.gitignore_scope(), GitignoreScope::Directory));
    }

    #[test]
    fn test_from_string() {
        let md = r#"---
name: from-string
description: From string test
---

Body"#;

        let skill = SkillFeature::from_string(md).unwrap();
        assert_eq!(skill.metadata.name, "from-string");
        assert_eq!(skill.metadata.description, "From string test");
        assert_eq!(skill.content, "Body");
    }

    #[test]
    fn test_to_string() {
        let skill = SkillFeature {
            metadata: SkillMetadata {
                name: "to-string".to_string(),
                description: "To string test".to_string(),
                license: None,
                compatibility: None,
                metadata: None,
                allowed_tools: None,
            },
            content: "Body".to_string(),
        };

        let result = skill.to_string().unwrap();
        assert!(result.contains("name: to-string"));
        assert!(result.contains("description: To string test"));
        assert!(result.contains("Body"));
    }

    #[test]
    fn test_allowed_tools_rename() {
        let md = r#"---
name: tools-skill
description: Allowed tools test
allowed-tools: Read Grep
---

Body"#;

        let skill = SkillFeature::from_markdown(md).unwrap();
        assert_eq!(skill.metadata.allowed_tools, Some("Read Grep".to_string()));

        let serialized = skill.to_markdown().unwrap();
        assert!(serialized.contains("allowed-tools: Read Grep"));
        assert!(!serialized.contains("allowed_tools"));
    }

    #[test]
    fn test_get_name_variable_uses_skill_namespace() {
        let skill = SkillFeature {
            metadata: SkillMetadata {
                name: "ignored".to_string(),
                description: "Test".to_string(),
                license: None,
                compatibility: None,
                metadata: None,
                allowed_tools: None,
            },
            content: "Body".to_string(),
        };

        let value = skill.get_name_variable("my-skill").unwrap();
        assert_eq!(value, Some(json!({"skill": {"name": "my-skill"}})));
    }
}
