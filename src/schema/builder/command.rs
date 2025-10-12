use crate::schema::command::{Command, CommandMetadata};

pub(crate) struct CommandBuilder {
    metadata: CommandMetadata,
    content: Option<String>,
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
