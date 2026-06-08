use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::templates::{RenderType, Templater};

pub trait FeatureTrait: Sized {
    fn from_string(value: &str) -> Result<Self>;
    fn to_string(&self) -> Result<String>;
    fn to_value(&self) -> Value;
    /// Derives the filesystem path to the source file from an optional item name, for symlink creation.
    fn resolve_source_path(name: Option<&str>) -> Result<PathBuf>;

    // Can't mutate it as we have to populate the same struct with different values for different providers
    fn populate_with_values(&self, templater: &Templater, values: Option<&Value>) -> Result<Self> {
        let content = self.to_string()?;
        let rendered_content = templater
            .render_template(RenderType::Content(content), values)
            .context("unable to render feature content")?;
        Self::from_string(&rendered_content)
    }

    fn get_file_name(&self) -> Option<String> {
        None
    }

    /// Must be overridden by any feature that returns `Some` from `get_file_name`.
    fn get_name_variable(&self, filename: &str) -> Result<Option<Value>> {
        let value = self.to_value();
        if let Value::Object(ref map) = value
            && let Some((namespace, _)) = map.iter().next()
        {
            return Ok(Some(
                serde_json::json!({ namespace.as_str(): { "name": filename } }),
            ));
        }
        Ok(None)
    }

    /// Returns true if this feature's content format is identical across providers, making it eligible for symlink deployment.
    fn is_symlinkable(&self) -> bool {
        false
    }

    /// Returns true if this feature type's content is the same across all providers.
    fn is_provider_agnostic() -> bool {
        false
    }

    /// Returns the source directory for this feature item, if the feature
    /// stores its files in a multi-file directory (e.g. skills). The deployer
    /// uses this to symlink extra files alongside the main source file.
    fn source_dir(_name: Option<&str>) -> Option<PathBuf> {
        None
    }
}
