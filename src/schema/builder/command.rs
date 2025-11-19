use crate::schema::features::command::{CommandFeature, CommandMetadata};

pub(crate) struct CommandFeatureBuilder {
    metadata: CommandMetadata,
    content: Option<String>,
}

impl CommandFeatureBuilder {
    pub fn new(name: &str, description: &str, extension: &str) -> Self {
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

    pub fn build(self) -> CommandFeature {
        CommandFeature {
            metadata: self.metadata,
            content: self.content.unwrap_or_default(),
        }
    }
}
