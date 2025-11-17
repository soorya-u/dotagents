use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    constants::file::INSTRUCTIONS_FILE, schema::features::traits::FeatureTrait,
    utils::path::get_application_dir,
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
