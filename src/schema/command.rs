use std::fs;

use anyhow::{Context, Result};
use gray_matter::Matter;
use gray_matter::engine::YAML;
use serde::{Deserialize, Serialize};

use crate::utils::path::get_commands_dir;

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
