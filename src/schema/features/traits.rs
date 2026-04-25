use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::templates::{RenderType, Templater};

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

    /// Returns the name variable for target path interpolation.
    /// Must be overridden by any feature that returns `Some` from `get_file_name`.
    fn get_name_variable(&self, _filename: &str) -> Result<Option<Value>> {
        Ok(None)
    }
}
