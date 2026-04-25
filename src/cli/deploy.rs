use std::path::PathBuf;

use anyhow::{Context, Result};
use log::warn;
use rayon::prelude::*;
use serde_json::{Value, to_value};

use crate::cli::options::DeployOptions;
use crate::constants::features::{
    COMMANDS_FEATURE, INSTRUCTION_FEATURE, MCP_FEATURE, SKILLS_FEATURE,
};
use crate::schema::config::AppConfig;
use crate::schema::features::{
    command::CommandFeature, instruction::InstructionFeature, mcp::McpFeature, skill::SkillFeature,
    traits::FeatureTrait,
};
use crate::templates::{Templater, get_templater, render_feature_with_settings};
use crate::utils::gitignore::{
    parse_fenced_section, prompt_gitignore_update, read_gitignore, write_gitignore,
};
use crate::utils::path::{get_workspace_dir, make_workspace_relative};

/// Deploy a single feature across all configured providers, returning written paths.
fn deploy_feature<T>(
    app_config: &AppConfig,
    templater: &Templater,
    variables: Option<&Value>,
    feature_name: &str,
    loader: impl FnOnce() -> Result<Vec<T>>,
) -> Result<Vec<PathBuf>>
where
    T: FeatureTrait + Sync,
{
    if !app_config.has_feature(feature_name) {
        return Ok(Vec::new());
    }

    let features = loader().context(format!("Failed to load {} feature", feature_name))?;
    let providers = app_config.get_provider_feature_settings(feature_name);

    let paths: Vec<PathBuf> = providers
        .par_iter()
        .try_fold(
            Vec::new,
            |mut acc, (provider_name, settings)| -> Result<Vec<PathBuf>> {
                for feature in features.iter() {
                    let path = render_feature_with_settings(
                        provider_name,
                        feature,
                        settings,
                        templater,
                        variables,
                    )?;
                    acc.push(path);
                }
                Ok(acc)
            },
        )
        .try_reduce(Vec::new, |mut a, b| {
            a.extend(b);
            Ok(a)
        })?;

    Ok(paths)
}

pub(super) fn deploy(opts: DeployOptions) -> Result<()> {
    let templater = get_templater();
    let app_config =
        AppConfig::from_application(templater).context("Failed to load application config")?;
    let variables =
        Some(to_value(app_config.variables.clone()).context("Failed to extract variables")?);

    let mut all_paths = Vec::new();

    all_paths.extend(deploy_feature::<CommandFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        COMMANDS_FEATURE,
        CommandFeature::from_application,
    )?);

    all_paths.extend(deploy_feature::<SkillFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        SKILLS_FEATURE,
        SkillFeature::from_application,
    )?);

    all_paths.extend(deploy_feature::<McpFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        MCP_FEATURE,
        || McpFeature::from_application().map(|mcp| vec![mcp]),
    )?);

    all_paths.extend(deploy_feature::<InstructionFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        INSTRUCTION_FEATURE,
        || InstructionFeature::from_application().map(|inst| vec![inst]),
    )?);

    // Skip gitignore update when no files were written or the user opted out.
    if all_paths.is_empty() || opts.no_gitignore {
        return Ok(());
    }

    let workspace_root = get_workspace_dir().context("Failed to get workspace directory")?;

    let should_update = if opts.gitignore {
        true
    } else {
        // Compute how many paths would be added to decide whether to prompt.
        let gitignore_path = workspace_root.join(".gitignore");
        let current_content = read_gitignore(&gitignore_path)?;
        let existing = parse_fenced_section(&current_content);
        let new_count = all_paths
            .iter()
            .filter_map(|p| make_workspace_relative(p, &workspace_root))
            .filter(|s| !existing.contains(s.as_str()))
            .count();

        if new_count == 0 {
            return Ok(());
        }

        prompt_gitignore_update(new_count)
    };

    if should_update && let Err(e) = write_gitignore(&workspace_root, &all_paths) {
        warn!("Failed to update .gitignore: {}", e);
    }

    Ok(())
}
