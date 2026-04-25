use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use log::warn;
use serde_json::{Value, to_value};

use crate::{
    schema::{config::FeatureSettings, features::traits::FeatureTrait},
    templates::{RenderType, Templater, variables::get_user_defined_variables},
    utils::{
        fs::{read_file, write_file},
        merge_json,
    },
};

pub fn render_feature_with_settings<T: FeatureTrait>(
    provider_name: &str,
    feature: &T,
    feature_settings: &FeatureSettings,
    templater: &Templater,
    variables: Option<&Value>,
) -> Result<PathBuf> {
    let template_str = feature_settings
        .template
        .as_deref()
        .ok_or_else(|| anyhow!("Template config not found for provider {}", provider_name))?;

    let target_str = feature_settings
        .target
        .as_deref()
        .ok_or_else(|| anyhow!("Target config not found for provider {}", provider_name))?;

    let template_path = PathBuf::from(template_str);
    let target_path = if let Some(filename) = feature.get_file_name() {
        let name_var = feature.get_name_variable(&filename)?;
        PathBuf::from(
            templater
                .render_template(RenderType::Content(target_str.to_string()), Some(&name_var))?,
        )
    } else {
        PathBuf::from(target_str)
    };

    if !template_path.exists() {
        return Err(anyhow!(
            "Template file not found for {} provider at {}",
            provider_name,
            template_path.display()
        ));
    }

    if target_path.exists() {
        warn!("Replacing existing file at {}", target_path.display());
    }

    let local_vars = feature_settings
        .variables
        .as_ref()
        .map(to_value)
        .transpose()?;

    let user_vars = get_user_defined_variables(Some(merge_json(variables, local_vars.as_ref())))?;

    let populate_config = feature.populate_with_values(templater, Some(&user_vars))?;

    let feature_as_variables = populate_config.to_value();

    let template_file_content = read_file(&template_path).context(format!(
        "failed to read file in {}",
        template_path.display()
    ))?;

    let vars = merge_json(Some(&user_vars), Some(&feature_as_variables));
    let content =
        templater.render_template(RenderType::Content(template_file_content), Some(&vars))?;

    write_file(&target_path, &content)
        .context(format!("failed to write file in {}", target_path.display()))?;

    Ok(target_path)
}
