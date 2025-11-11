use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{constants::file::INSTRUCTIONS_FILE, utils::path::get_application_dir};

#[derive(Serialize, Deserialize)]
pub(crate) struct Instruction {
    content: String,
}

impl Instruction {
    pub fn from_application() -> Result<Self> {
        let dir = get_application_dir()?;
        let path = dir.join(INSTRUCTIONS_FILE);
        let content = fs::read_to_string(path)?;
        Ok(Self { content })
    }
}
