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

    // Phase 2: populate feature with variables (skip var injection in link mode)
    let populated_feature: Option<T> = if mode == FeatureMode::Template {
        Some(
            feature
                .populate_with_values(templater, Some(&user_vars))
                .context("unable to render feature variables")?,
        )
    } else {
        None
    };

    let content_to_render = populated_feature
        .as_ref()
        .map(|f| f.to_value())
        .unwrap_or_else(|| feature.to_value());

    // Phase 3: template rendering (skip for provider-agnostic features; no .hbs template)
    let feature_as_variables = content_to_render;
    let provider_agnostic = T::is_provider_agnostic();

    let content = if provider_agnostic {
        // Type 1: no .hbs template; content is source (vars injected in template mode, raw in link mode)
        if let Some(ref populated) = populated_feature {
            populated.to_string()?
        } else {
            feature.to_string()?
        }
    } else {
        // Type 2: .hbs template required for both link and template modes
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
                debug!(
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
    use crate::core::features::ignore::IgnoreFeature;
    use crate::core::features::instruction::InstructionFeature;
    use crate::core::features::skill::SkillFeature;
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

    // link_feature_with_settings creates a symlink at target pointing to source
    #[test]
    fn link_feature_creates_symlink_at_target() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let source_path = tmp.path().join("source.txt");
        fs::write(&source_path, "hello world").unwrap();

        let feature = InstructionFeature::from_string("plain content").unwrap();
        let settings = FeatureSettings {
            target: Some(tmp.path().join("linked.md").to_string_lossy().to_string()),
            ..Default::default()
        };

        let result = link_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            &templater,
            None,
            false,
            &source_path,
        );

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let CacheUpdate::Linked { target } = result.unwrap() else {
            panic!("expected CacheUpdate::Linked");
        };
        assert_eq!(target, tmp.path().join("linked.md"));
        assert!(target.is_symlink(), "target should be a symlink");
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "hello world",
            "symlink should resolve to source content"
        );
    }

    // link_feature_with_settings creates parent directories for the target
    #[test]
    fn link_feature_creates_parent_directories() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let source_path = tmp.path().join("source.txt");
        fs::write(&source_path, "content").unwrap();

        let feature = InstructionFeature::from_string("plain content").unwrap();
        let target = tmp.path().join("nested").join("dirs").join("linked.md");
        let settings = FeatureSettings {
            target: Some(target.to_string_lossy().to_string()),
            ..Default::default()
        };

        let result = link_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            &templater,
            None,
            false,
            &source_path,
        );
        assert!(result.is_ok());
        assert!(target.is_symlink(), "target should be a symlink");
    }

    // link_feature_with_settings overwrites an existing target file as a symlink
    #[test]
    fn link_feature_overwrites_existing_target() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let source_path = tmp.path().join("source.txt");
        fs::write(&source_path, "new content").unwrap();

        let target = tmp.path().join("linked.md");
        fs::write(&target, "old regular content").unwrap();
        assert!(!target.is_symlink(), "precondition: target is regular file");

        let feature = InstructionFeature::from_string("plain content").unwrap();
        let settings = FeatureSettings {
            target: Some(target.to_string_lossy().to_string()),
            ..Default::default()
        };

        let result = link_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            &templater,
            None,
            false,
            &source_path,
        );
        assert!(result.is_ok());
        assert!(target.is_symlink(), "target should now be a symlink");
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "new content",
            "symlink should resolve to new source content"
        );
    }

    // link_feature_with_settings in dry-run mode returns Linked without creating the symlink
    #[test]
    fn link_feature_dry_run_does_not_create_symlink() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let source_path = tmp.path().join("source.txt");
        fs::write(&source_path, "hello world").unwrap();

        let target = tmp.path().join("linked.md");
        let feature = InstructionFeature::from_string("plain content").unwrap();
        let settings = FeatureSettings {
            target: Some(target.to_string_lossy().to_string()),
            ..Default::default()
        };

        let result = link_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            &templater,
            None,
            true,
            &source_path,
        );
        assert!(result.is_ok());
        let CacheUpdate::Linked { target: ret_target } = result.unwrap() else {
            panic!("expected CacheUpdate::Linked");
        };
        assert_eq!(ret_target, target);
        assert!(!target.exists(), "dry-run should not create the symlink");
    }

    // link_feature_with_settings errors when target config is missing
    #[test]
    fn link_feature_errors_when_target_missing() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let source_path = tmp.path().join("source.txt");
        fs::write(&source_path, "content").unwrap();

        let feature = InstructionFeature::from_string("plain content").unwrap();
        let settings = FeatureSettings {
            target: None,
            ..Default::default()
        };

        let result = link_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            &templater,
            None,
            false,
            &source_path,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Target config not found"),
            "expected 'Target config not found' in error, got: {err}"
        );
    }

    // link_feature_with_settings errors when target template cannot be rendered
    #[test]
    fn link_feature_errors_on_broken_target_template() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let source_path = tmp.path().join("source.txt");
        fs::write(&source_path, "content").unwrap();

        let feature = InstructionFeature::from_string("plain content").unwrap();
        let settings = FeatureSettings {
            target: Some("{{invalid".to_string()),
            ..Default::default()
        };

        let result = link_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            &templater,
            None,
            false,
            &source_path,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("unable to render target path"),
            "expected 'unable to render target path' in error, got: {err}"
        );
    }

    // link_feature_with_settings renders skill name variable into the target path
    #[test]
    fn link_feature_renders_skill_name_into_target() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let skills_dir = tmp.path().join(ROOT_DIR).join("skills").join("my-skill");
        fs::create_dir_all(&skills_dir).unwrap();
        let source_path = skills_dir.join("SKILL.md");
        fs::write(&source_path, "skill body").unwrap();

        let skill =
            SkillFeature::from_string("---\nname: my-skill\ndescription: test\n---\n\nskill body")
                .unwrap();
        let target = tmp
            .path()
            .join(".claude")
            .join("skills")
            .join("{{ skill.name }}")
            .join("SKILL.md");
        let settings = FeatureSettings {
            target: Some(target.to_string_lossy().to_string()),
            ..Default::default()
        };

        let result = link_feature_with_settings(
            "claude",
            &skill,
            &settings,
            &templater,
            None,
            false,
            &source_path,
        );
        assert!(result.is_ok());
        let CacheUpdate::Linked { target } = result.unwrap() else {
            panic!("expected CacheUpdate::Linked");
        };
        let expected = tmp
            .path()
            .join(".claude")
            .join("skills")
            .join("my-skill")
            .join("SKILL.md");
        assert_eq!(target, expected);
        assert!(target.is_symlink());
    }

    // render_feature Type 2 + template mode injects vars into content and renders template
    #[test]
    fn render_feature_type2_template_mode_injects_vars_and_renders_template() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let template_path = tmp.path().join("tmpl.hbs");
        fs::write(&template_path, "{{ instruction.content }}").unwrap();

        let feature = InstructionFeature::from_string("Hello {{ var.name }}").unwrap();
        let target = tmp.path().join("out.md");
        let settings = FeatureSettings {
            template: Some(template_path.to_string_lossy().to_string()),
            target: Some(target.to_string_lossy().to_string()),
            ..Default::default()
        };
        let vars = serde_json::json!({ "name": "world" });

        let result = render_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            FeatureMode::Template,
            &templater,
            Some(&vars),
            None,
            true,
            false,
        );
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        assert!(target.exists(), "target file should be written");
        let content = fs::read_to_string(&target).unwrap();
        assert_eq!(
            content, "Hello world",
            "template mode should inject vars into content before rendering"
        );
    }

    // render_feature Type 2 + link mode renders template but skips var injection into content
    #[test]
    fn render_feature_type2_link_mode_skips_var_injection() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let template_path = tmp.path().join("tmpl.hbs");
        fs::write(&template_path, "{{ instruction.content }}").unwrap();

        let feature = InstructionFeature::from_string("Hello {{ var.name }}").unwrap();
        let target = tmp.path().join("out.md");
        let settings = FeatureSettings {
            template: Some(template_path.to_string_lossy().to_string()),
            target: Some(target.to_string_lossy().to_string()),
            ..Default::default()
        };
        let vars = serde_json::json!({ "name": "world" });

        let result = render_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            FeatureMode::Link,
            &templater,
            Some(&vars),
            None,
            true,
            false,
        );
        assert!(result.is_ok());
        let content = fs::read_to_string(&target).unwrap();
        assert_eq!(
            content, "Hello {{ var.name }}",
            "link mode should NOT inject vars into feature content"
        );
    }

    // render_feature Type 1 + template mode writes source with vars injected, no .hbs template
    #[test]
    fn render_feature_type1_template_mode_injects_vars_without_template() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let feature = IgnoreFeature::from_string("node_modules/\n{{ var.pattern }}\n").unwrap();
        let target = tmp.path().join(".agentignore");
        let settings = FeatureSettings {
            template: None,
            target: Some(target.to_string_lossy().to_string()),
            ..Default::default()
        };
        let vars = serde_json::json!({ "pattern": "*.log" });

        let result = render_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            FeatureMode::Template,
            &templater,
            Some(&vars),
            None,
            true,
            false,
        );
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        assert!(target.exists(), "target file should be written");
        let content = fs::read_to_string(&target).unwrap();
        assert_eq!(
            content, "node_modules/\n*.log\n",
            "Type 1 template mode should inject vars into content without a .hbs template"
        );
    }

    // render_feature Type 1 + link mode writes raw source without var injection or template
    #[test]
    fn render_feature_type1_link_mode_writes_raw_source() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let feature = IgnoreFeature::from_string("node_modules/\n{{ var.pattern }}\n").unwrap();
        let target = tmp.path().join(".agentignore");
        let settings = FeatureSettings {
            template: None,
            target: Some(target.to_string_lossy().to_string()),
            ..Default::default()
        };
        let vars = serde_json::json!({ "pattern": "*.log" });

        let result = render_feature_with_settings(
            "test-provider",
            &feature,
            &settings,
            FeatureMode::Link,
            &templater,
            Some(&vars),
            None,
            true,
            false,
        );
        assert!(result.is_ok());
        let content = fs::read_to_string(&target).unwrap();
        assert_eq!(
            content, "node_modules/\n{{ var.pattern }}\n",
            "Type 1 link mode should write raw source without var injection"
        );
    }

    // render_feature Type 2 without template errors
    #[test]
    fn render_feature_type2_without_template_errors() {
        let Ok(tmp) = setup_test_workspace() else {
            return;
        };
        let templater = Templater::new().unwrap();

        let feature = InstructionFeature::from_string("content").unwrap();
        let target = tmp.path().join("out.md");
        let settings = FeatureSettings {
            template: None,
            target: Some(target.to_string_lossy().to_string()),
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
            false,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Template config not found"),
            "expected 'Template config not found' in error, got: {err}"
        );
    }
}
