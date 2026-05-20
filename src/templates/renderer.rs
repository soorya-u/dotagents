use std::path::PathBuf;

use crate::prelude::*;
use serde_json::{Value, to_value};

use crate::{
    core::{
        config::{CacheEntry, CacheUpdate, FeatureSettings},
        features::traits::FeatureTrait,
    },
    templates::{RenderType, Templater, variables::get_user_defined_variables},
    utils::http::fetch_template,
    utils::{
        fs::{read_file, write_file},
        hash::{hash_content, hash_file},
        json::merge_json,
    },
};

/// Renders a feature for a provider, applying cache skip/detect logic.
#[allow(clippy::too_many_arguments)]
pub fn render_feature_with_settings<T: FeatureTrait>(
    provider_name: &str,
    feature: &T,
    feature_settings: &FeatureSettings,
    templater: &Templater,
    variables: Option<&Value>,
    cache: Option<&CacheEntry>,
    force: bool,
    dry_run: bool,
) -> Result<CacheUpdate> {
    let template_str = feature_settings
        .template
        .as_deref()
        .ok_or_else(|| anyhow!("Template config not found for provider {}", provider_name))?;

    let target_str = feature_settings
        .target
        .as_deref()
        .ok_or_else(|| anyhow!("Target config not found for provider {}", provider_name))?;

    let name_var: Option<Value> = feature
        .get_file_name()
        .map(|filename| feature.get_name_variable(&filename))
        .transpose()?
        .flatten();
    let target_vars = merge_json(variables, name_var.as_ref());
    let target_path = PathBuf::from(
        templater
            .render_template(
                RenderType::Content(target_str.to_string()),
                Some(&target_vars),
            )
            .context("unable to render target path")?,
    );

    let local_vars = feature_settings
        .variables
        .as_ref()
        .map(to_value)
        .transpose()?;

    let user_vars = get_user_defined_variables(Some(merge_json(variables, local_vars.as_ref())))?;

    let populate_config = feature
        .populate_with_values(templater, Some(&user_vars))
        .context("unable to render feature variables")?;

    let feature_as_variables = populate_config.to_value();

    let template_file_content =
        if template_str.starts_with("https://") || template_str.starts_with("http://") {
            fetch_template(template_str)?
        } else {
            let template_path = PathBuf::from(template_str);
            if !template_path.exists() {
                return Err(anyhow!(
                    "Template file not found for {} provider at {}",
                    provider_name,
                    template_path.display()
                ));
            }
            read_file(&template_path).context(format!(
                "failed to read file in {}",
                template_path.display()
            ))?
        };

    let vars = merge_json(Some(&user_vars), Some(&feature_as_variables));
    let content = templater
        .render_template(RenderType::Content(template_file_content), Some(&vars))
        .context(format!(
            "unable to render template content for provider '{}'",
            provider_name
        ))?;

    let rendered_hash = hash_content(&content);

    // Cache-aware skip / user-edit detection (bypassed when --force)
    if !force
        && let Some(entry) = cache
        && rendered_hash == entry.hash
    {
        // Rendered content is identical to what was last written.
        // Check the on-disk file to detect user edits.
        match hash_file(&target_path)? {
            None => {
                // File is missing despite a valid cache entry → re-write it.
            }
            Some(disk_hash) if disk_hash == entry.hash => {
                // On-disk file still matches the cache → nothing to do.
                return Ok(CacheUpdate::Skipped);
            }
            Some(_) => {
                // On-disk file differs from cache → user manually edited it.
                warn!(
                    "Target file {} was manually edited; skipping",
                    target_path.display()
                );
                return Ok(CacheUpdate::UserEditedSkipped { path: target_path });
            }
        }
        // rendered_hash == entry.hash but file was missing → fall through to write.
    }
    // No cache entry, force=true, or inputs changed → fall through to write.

    // In dry-run mode skip the actual write; return content for caller classification.
    if dry_run {
        return Ok(CacheUpdate::DryRun {
            target: target_path,
            content,
        });
    }

    write_file(&target_path, &content)
        .context(format!("failed to write file in {}", target_path.display()))?;

    Ok(CacheUpdate::Written {
        hash: rendered_hash,
        target: target_path.to_string_lossy().into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    use crate::constants::dir::ROOT_DIR;
    use crate::constants::file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE};
    use crate::constants::mocks::default_config;
    use crate::core::features::instruction::InstructionFeature;
    use crate::utils::path::override_workspace_dir;

    fn setup_test_workspace() -> anyhow::Result<TempDir> {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join(ROOT_DIR);
        fs::create_dir_all(&root)?;
        fs::write(
            root.join(GLOBAL_CONFIG_FILE),
            default_config(&["commands", "instructions", "mcp", "skills"], &["claude"]),
        )?;
        fs::write(root.join(LOCAL_CONFIG_FILE), "")?;
        override_workspace_dir(tmp.path().to_path_buf())?;
        Ok(tmp)
    }

    // broken target path expression produces "unable to render target path" in error chain
    #[test]
    fn render_feature_broken_target_path_emits_phase_context() {
        let Ok(tmp) = setup_test_workspace() else {
            return; // WORKSPACE_DIR OnceLock already set by a prior test; skip
        };
        let templater = Templater::new().unwrap();

        let valid_template_path = tmp.path().join("valid.hbs");
        fs::write(&valid_template_path, "{{ instruction.content }}").unwrap();

        let feature = InstructionFeature::from_string("plain content").unwrap();
        let settings = FeatureSettings {
            template: Some(valid_template_path.to_string_lossy().to_string()),
            target: Some("{{invalid".to_string()),
            ..Default::default()
        };

        let result = render_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            &templater,
            None,
            None,
            true,
            true,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("unable to render target path"),
            "expected 'unable to render target path' in error, got: {err}"
        );
    }

    // broken template content produces "unable to render template content for provider" in error chain
    #[test]
    fn render_feature_broken_template_content_emits_phase_context() {
        let Ok(tmp) = setup_test_workspace() else {
            return; // WORKSPACE_DIR OnceLock already set by a prior test; skip
        };
        let templater = Templater::new().unwrap();

        let broken_template_path = tmp.path().join("broken.hbs");
        fs::write(&broken_template_path, "{{#if}}").unwrap();

        let feature = InstructionFeature::from_string("plain content").unwrap();
        let settings = FeatureSettings {
            template: Some(broken_template_path.to_string_lossy().to_string()),
            target: Some("out.md".to_string()),
            ..Default::default()
        };

        let result = render_feature_with_settings(
            "my-provider",
            &feature,
            &settings,
            &templater,
            None,
            None,
            true,
            true,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("unable to render template content for provider 'my-provider'"),
            "expected 'unable to render template content for provider' in error, got: {err}"
        );
    }
}
