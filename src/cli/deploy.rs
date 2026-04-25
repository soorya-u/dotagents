use std::path::PathBuf;

use anyhow::{Context, Result};
use log::warn;
use rayon::prelude::*;
use serde_json::{Value, to_value};
use std::sync::{Arc, Mutex};

use crate::cli::options::DeployOptions;
use crate::constants::features::{
    COMMANDS_FEATURE, INSTRUCTION_FEATURE, MCP_FEATURE, SKILLS_FEATURE,
};
use crate::schema::config::{AppConfig, CACHE_SINGLETON_KEY, CacheConfig, CacheEntry, CacheUpdate};
use crate::schema::features::{
    command::CommandFeature, instruction::InstructionFeature, mcp::McpFeature, skill::SkillFeature,
    traits::FeatureTrait,
};
use crate::templates::{Templater, get_templater, render_feature_with_settings};
use crate::utils::gitignore::{
    parse_fenced_section, prompt_gitignore_update, read_gitignore, write_gitignore,
};
use crate::utils::path::{get_workspace_dir, make_workspace_relative};

/// Deploys one feature across all enabled providers, collecting written paths and updating cache.
fn deploy_feature<T>(
    app_config: &AppConfig,
    templater: &Templater,
    variables: Option<&Value>,
    feature_name: &str,
    cache: &Option<Arc<Mutex<CacheConfig>>>,
    force: bool,
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
                    let file_name = feature.get_file_name();
                    let item_key = file_name.as_deref().unwrap_or(CACHE_SINGLETON_KEY);

                    // Read-only cache lookup: acquire lock, clone entry, drop lock.
                    let cached_entry: Option<CacheEntry> = cache.as_ref().and_then(|c| {
                        c.lock()
                            .unwrap()
                            .get(provider_name, feature_name, item_key)
                            .cloned()
                    });

                    let update = render_feature_with_settings(
                        provider_name,
                        feature,
                        settings,
                        templater,
                        variables,
                        cached_entry.as_ref(),
                        force,
                    )?;

                    // If a file was written, collect the path and update cache.
                    if let CacheUpdate::Written { hash, target } = update {
                        acc.push(PathBuf::from(&target));
                        if let Some(c) = cache {
                            c.lock().unwrap().set(
                                provider_name,
                                feature_name,
                                item_key,
                                CacheEntry { hash, target },
                            );
                        }
                    }
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

    // Load cache unless --no-cache is specified.
    let cache: Option<Arc<Mutex<CacheConfig>>> = if opts.no_cache {
        None
    } else {
        Some(Arc::new(Mutex::new(
            CacheConfig::load().context("Failed to load cache")?,
        )))
    };

    let mut all_paths = Vec::new();

    all_paths.extend(deploy_feature::<CommandFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        COMMANDS_FEATURE,
        &cache,
        opts.force,
        CommandFeature::from_application,
    )?);

    all_paths.extend(deploy_feature::<SkillFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        SKILLS_FEATURE,
        &cache,
        opts.force,
        SkillFeature::from_application,
    )?);

    all_paths.extend(deploy_feature::<McpFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        MCP_FEATURE,
        &cache,
        opts.force,
        || McpFeature::from_application().map(|mcp| vec![mcp]),
    )?);

    all_paths.extend(deploy_feature::<InstructionFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        INSTRUCTION_FEATURE,
        &cache,
        opts.force,
        || InstructionFeature::from_application().map(|inst| vec![inst]),
    )?);

    // Persist cache to disk (skipped when --no-cache).
    if let Some(c) = &cache {
        c.lock().unwrap().save().context("Failed to save cache")?;
    }

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
