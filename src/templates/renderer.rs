use std::path::{Path, PathBuf};

use crate::prelude::*;
use serde_json::{Value, to_value};

use crate::{
    core::{
        config::{CacheEntry, CacheUpdate, FeatureMode, FeatureSettings},
        features::traits::FeatureTrait,
    },
    templates::{RenderType, Templater, variables::get_user_defined_variables},
    utils::format::MergeFormat,
    utils::http::fetch_template,
    utils::merge::merge_into_existing,
    utils::{
        fs::{read_file, write_file, write_symlink},
        hash::{hash_content, hash_file},
        json::merge_json,
    },
};

/// Resolves a Handlebars target path template to a concrete `PathBuf`.
pub(crate) fn resolve_target_path(
    templater: &Templater,
    target_str: &str,
    variables: Option<&Value>,
) -> Result<PathBuf> {
    Ok(PathBuf::from(
        templater
            .render_template(RenderType::Content(target_str.to_string()), variables)
            .context("unable to render target path")?,
    ))
}

/// Symlinks a feature's source file to the resolved target path.
#[allow(clippy::too_many_arguments)]
pub fn link_feature_with_settings<T: FeatureTrait>(
    provider_name: &str,
    feature: &T,
    feature_settings: &FeatureSettings,
    templater: &Templater,
    variables: Option<&Value>,
    dry_run: bool,
    source_path: &Path,
) -> Result<CacheUpdate> {
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
    let target_path = resolve_target_path(templater, target_str, Some(&target_vars))
        .context("unable to render target path")?;

    if dry_run {
        return Ok(CacheUpdate::Linked {
            target: target_path,
        });
    }

    write_symlink(source_path, &target_path).context(format!(
        "unable to symlink {} -> {}",
        source_path.display(),
        target_path.display()
    ))?;

    Ok(CacheUpdate::Linked {
        target: target_path,
    })
}

/// Renders a feature for a provider, applying cache skip/detect logic.
#[allow(clippy::too_many_arguments)]
pub fn render_feature_with_settings<T: FeatureTrait>(
    provider_name: &str,
    feature: &T,
    feature_settings: &FeatureSettings,
    mode: FeatureMode,
    templater: &Templater,
    variables: Option<&Value>,
    cache: Option<&CacheEntry>,
    force: bool,
    dry_run: bool,
) -> Result<CacheUpdate> {
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
    let target_path = resolve_target_path(templater, target_str, Some(&target_vars))
        .context("unable to render target path")?;

    let local_vars = feature_settings
        .variables
        .as_ref()
        .map(to_value)
        .transpose()?;

    let user_vars = get_user_defined_variables(Some(merge_json(variables, local_vars.as_ref())))?;

    // Phase 2: populate feature with variables (skip in link mode)
    let content_to_render = if mode == FeatureMode::Link {
        feature.to_value()
    } else {
        let populate_config = feature
            .populate_with_values(templater, Some(&user_vars))
            .context("unable to render feature variables")?;
        populate_config.to_value()
    };

    // Phase 3: template rendering (skip for provider-agnostic features in link mode)
    let feature_as_variables = content_to_render;
    let provider_agnostic = T::is_provider_agnostic();

    let content = if provider_agnostic && mode == FeatureMode::Link {
        // Provider-agnostic features have no .hbs template; content is the source rendered with vars
        feature.to_string()?
    } else {
        let template_str = feature_settings.template.as_deref().ok_or_else(|| {
            anyhow!(
                "Template config not found for provider {} (template mode requires a template)",
                provider_name
            )
        })?;

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
        templater
            .render_template(RenderType::Content(template_file_content), Some(&vars))
            .context(format!(
                "unable to render template content for provider '{}'",
                provider_name
            ))?
    };

    let is_mergeable = target_path.exists() && MergeFormat::from_extension(&target_path).is_some();

    let final_content = if is_mergeable {
        let existing = read_file(&target_path).context(format!(
            "failed to read existing file at {}",
            target_path.display()
        ))?;
        match merge_into_existing(&target_path, &existing, &content) {
            Ok(merged) => merged,
            Err(e) => {
                let reason = e.to_string();
                warn!("Skipping merge for {}: {}", target_path.display(), reason);
                return Ok(CacheUpdate::MergeSkipped {
                    path: target_path,
                    reason,
                });
            }
        }
    } else {
        content
    };

    let final_hash = hash_content(&final_content);

    // Cache-aware skip / user-edit detection (bypassed when --force)
    if !force
        && let Some(entry) = cache
        && final_hash == entry.hash
    {
        match hash_file(&target_path)? {
            None => {}
            Some(disk_hash) if disk_hash == entry.hash => {
                return Ok(CacheUpdate::Skipped);
            }
            Some(_) if !is_mergeable => {
                warn!(
                    "Target file {} was manually edited; skipping",
                    target_path.display()
                );
                return Ok(CacheUpdate::UserEditedSkipped { path: target_path });
            }
            Some(_) => {}
        }
    }

    if dry_run {
        return Ok(CacheUpdate::DryRun {
            target: target_path,
            content: final_content,
        });
    }

    write_file(&target_path, &final_content)
        .context(format!("failed to write file in {}", target_path.display()))?;

    Ok(CacheUpdate::Written {
        hash: final_hash,
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
            FeatureMode::Template,
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
            FeatureMode::Template,
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

    // resolve_target_path renders a plain path without variables
    #[test]
    fn resolve_target_path_plain_string() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();
        let path = resolve_target_path(&templater, ".claude/AGENTS.md", None).unwrap();
        assert_eq!(path, PathBuf::from(".claude/AGENTS.md"));
        let _ = tmp;
    }

    // resolve_target_path renders variables in the target template
    #[test]
    fn resolve_target_path_with_variables() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();
        let vars = serde_json::json!({ "dir": { "workspace": tmp.path().to_string_lossy() } });
        let path =
            resolve_target_path(&templater, "{{ dir.workspace }}/AGENTS.md", Some(&vars)).unwrap();
        assert!(path.to_string_lossy().ends_with("AGENTS.md"));
    }
}
