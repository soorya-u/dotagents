use anyhow::{Context, Result};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct CommandMetadata {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Command {
    pub metadata: CommandMetadata,
    pub content: String,
}

pub(crate) struct CommandBuilder {
    metadata: CommandMetadata,
    content: Option<String>,
}

impl Command {
    pub fn to_markdown(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self.metadata)
            .context("failed to serialize metadata to YAML")?;

        Ok(format!("---\n{}---\n\n{}", yaml, self.content))
    }

    pub fn from_markdown(md: &str) -> Result<Self> {
        let matter = Matter::<YAML>::new();
        let parsed = matter.parse(md).context("failed to parse markdown")?;

        let metadata: CommandMetadata = parsed.data.context("failed to parse markdown metadata")?;

        Ok(Command {
            metadata,
            content: parsed.content,
        })
    }
}

impl CommandBuilder {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            metadata: CommandMetadata {
                name: name.into(),
                description: description.into(),
            },
            content: None,
        }
    }

    pub fn add_content(mut self, content: &str) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn build(self) -> Command {
        Command {
            metadata: self.metadata,
            content: self.content.unwrap_or_default(),
        }
    }
}
