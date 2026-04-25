use anyhow::Result;
use serde_json::Value;

use crate::templates::{RenderType, Templater, variables::get_command_name_variable};

pub trait FeatureTrait: Sized {
    fn from_string(value: &str) -> Result<Self>;
    fn to_string(&self) -> Result<String>;
    fn to_value(&self) -> Value;

    // Can't mutate it as we have to populate the same struct with different values for different providers
    fn populate_with_values(&self, templater: &Templater, values: Option<&Value>) -> Result<Self> {
        let content = self.to_string()?;
        let rendered_content = templater.render_template(RenderType::Content(content), values)?;
        Self::from_string(&rendered_content)
    }

    fn get_file_name(&self) -> Option<String> {
        None
    }

    fn get_name_variable(&self, filename: &str) -> Result<Value> {
        get_command_name_variable(filename)
    }
}
