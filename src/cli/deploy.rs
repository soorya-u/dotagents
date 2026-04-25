use anyhow::{Context, Result};
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

/// Deploys one feature across all enabled providers, updating the in-memory cache.
fn deploy_feature<T>(
    app_config: &AppConfig,
    templater: &Templater,
    variables: Option<&Value>,
    feature_name: &str,
    cache: &Option<Arc<Mutex<CacheConfig>>>,
    force: bool,
    loader: impl FnOnce() -> Result<Vec<T>>,
) -> Result<()>
where
    T: FeatureTrait + Sync,
{
    if !app_config.has_feature(feature_name) {
        return Ok(());
    }

    let features = loader().context(format!("Failed to load {} feature", feature_name))?;
    let providers = app_config.get_provider_feature_settings(feature_name);

    providers
        .par_iter()
        .map(|(provider_name, settings)| {
            features.iter().try_for_each(|feature| {
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

                // If a file was written, store the new cache entry.
                if let CacheUpdate::Written { hash, target } = update
                    && let Some(c) = cache
                {
                    c.lock().unwrap().set(
                        provider_name,
                        feature_name,
                        item_key,
                        CacheEntry { hash, target },
                    );
                }

                Ok(())
            })
        })
        .collect::<Result<()>>()?;

    Ok(())
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

    deploy_feature::<CommandFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        COMMANDS_FEATURE,
        &cache,
        opts.force,
        CommandFeature::from_application,
    )?;

    deploy_feature::<SkillFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        SKILLS_FEATURE,
        &cache,
        opts.force,
        SkillFeature::from_application,
    )?;

    deploy_feature::<McpFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        MCP_FEATURE,
        &cache,
        opts.force,
        || McpFeature::from_application().map(|mcp| vec![mcp]),
    )?;

    deploy_feature::<InstructionFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        INSTRUCTION_FEATURE,
        &cache,
        opts.force,
        || InstructionFeature::from_application().map(|inst| vec![inst]),
    )?;

    // Persist cache to disk (skipped when --no-cache).
    if let Some(c) = cache {
        c.lock().unwrap().save().context("Failed to save cache")?;
    }

    Ok(())
}
