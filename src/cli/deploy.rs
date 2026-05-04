use std::collections::HashSet;
use std::path::PathBuf;

use crate::prelude::*;
use rayon::prelude::*;
use serde_json::{Value, to_value};
use std::sync::{Arc, Mutex};

use crate::cli::options::DeployOptions;
use crate::cli::ui::deploy::{print_deploy_summary, prompt_gitignore_update, prompt_offline};
use crate::schema::config::{AppConfig, CACHE_SINGLETON_KEY, CacheConfig, CacheEntry, CacheUpdate};
use crate::schema::features::{
    Feature, command::CommandFeature, instruction::InstructionFeature, mcp::McpFeature,
    skill::SkillFeature, traits::FeatureTrait,
};
use crate::schema::registry::Registry;
use crate::templates::variables::set_env_paths;
use crate::templates::{
    TemplateCache, Templater, get_templater, registry_url, render_feature_with_settings,
    resolve_provider_defaults,
};
use crate::utils::gitignore::{
    GitignorePath, gitignore_path_to_pattern, parse_fenced_section, read_gitignore, write_gitignore,
};
use crate::utils::path::get_workspace_dir;
use crate::utils::tty::is_tty;

/// Aggregated result of deploying one feature across all providers.
#[derive(Debug, Default)]
pub(crate) struct DeployStats {
    pub written: usize,
    pub skipped: usize,
    pub paths: Vec<GitignorePath>,
}

impl DeployStats {
    /// Merge another `DeployStats` into this one, consuming both.
    fn merge(mut self, other: Self) -> Self {
        self.written += other.written;
        self.skipped += other.skipped;
        self.paths.extend(other.paths);
        self
    }
}

/// Deploys one feature across all enabled providers, collecting gitignore entries and updating cache.
#[allow(clippy::too_many_arguments)]
fn deploy_feature<T>(
    app_config: &AppConfig,
    templater: &Templater,
    variables: Option<&Value>,
    feature: &Feature,
    cache: &Arc<Mutex<CacheConfig>>,
    force: bool,
    no_cache: bool,
    loader: impl FnOnce() -> Result<Vec<T>>,
) -> Result<DeployStats>
where
    T: FeatureTrait + Sync,
{
    if !app_config.has_feature(feature) {
        return Ok(DeployStats::default());
    }

    let feature_name = feature.as_str();
    let items = loader().context(format!("Failed to load {} feature", feature))?;
    let providers = app_config.get_provider_feature_settings(feature);

    let stats: DeployStats = providers
        .par_iter()
        .try_fold(
            DeployStats::default,
            |mut acc, (provider_name, settings)| -> Result<DeployStats> {
                for item in items.iter() {
                    let file_name = item.get_file_name();
                    let item_key = file_name.as_deref().unwrap_or(CACHE_SINGLETON_KEY);

                    // Skip cache lookup when --no-cache: treat as a miss so comparison is bypassed.
                    let cached_entry: Option<CacheEntry> = if no_cache {
                        None
                    } else {
                        cache
                            .lock()
                            .unwrap()
                            .get(provider_name, feature_name, item_key)
                            .cloned()
                    };

                    let update = render_feature_with_settings(
                        provider_name,
                        item,
                        settings,
                        templater,
                        variables,
                        cached_entry.as_ref(),
                        force,
                    )?;

                    match update {
                        CacheUpdate::Written { hash, target } => {
                            acc.paths.push(GitignorePath::File(PathBuf::from(&target)));
                            acc.written += 1;
                            cache.lock().unwrap().set(
                                provider_name,
                                feature_name,
                                item_key,
                                CacheEntry { hash, target },
                            );
                        }
                        CacheUpdate::Skipped | CacheUpdate::UserEditedSkipped { .. } => {
                            acc.skipped += 1;
                        }
                    }
                }

                Ok(acc)
            },
        )
        .try_reduce(DeployStats::default, |a, b| Ok(a.merge(b)))?;

    Ok(stats)
}

pub(super) fn deploy(mut opts: DeployOptions) -> Result<()> {
    // Validate env paths before the LazyLock fires so missing files surface as clean errors.
    for path in &opts.env {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "load env file '{}': file not found",
                path.display()
            ));
        }
    }
    // Must be called before get_templater() so ENV_PATHS is set before the LazyLock fires.
    set_env_paths(std::mem::take(&mut opts.env));
    let templater = get_templater();
    let mut app_config =
        AppConfig::from_application(templater).context("Failed to load application config")?;

    // In interactive sessions, ask whether to run offline before the registry fetch.
    if !opts.offline && is_tty() {
        opts.offline = prompt_offline();
    }

    // Resolve missing template/target fields from the official provider registry.
    // registry.json is fetched at most once here; the result is shared across all providers.
    let template_cache = TemplateCache::new().context("Failed to initialise template cache")?;
    let registry: Option<Registry> = if opts.offline {
        None // --offline: skip fetch entirely, resolve from cache only
    } else {
        match Registry::fetch(registry_url()) {
            Ok(r) => Some(r),
            Err(e) => {
                warn!(
                    "Failed to fetch provider registry: {} — falling back to local cache",
                    e
                );
                None
            }
        }
    };
    resolve_provider_defaults(
        &mut app_config,
        registry.as_ref(),
        &template_cache,
        opts.offline,
        opts.no_cache,
    )
    .context("Failed to resolve provider template defaults")?;

    // Serialize user-defined variables only when present; None stays None so the
    // renderer doesn't receive Value::Null, which would wipe the Templater globals.
    let variables: Option<Value> = app_config
        .variables
        .as_ref()
        .map(|v| to_value(v).context("Failed to extract variables"))
        .transpose()?;

    // Always initialise cache; --no-cache only suppresses the hash-comparison read.
    let cache = Arc::new(Mutex::new(
        CacheConfig::load().context("Failed to load cache")?,
    ));

    let mut stats = DeployStats::default();

    stats = stats.merge(deploy_feature::<CommandFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        &Feature::Command,
        &cache,
        opts.force,
        opts.no_cache,
        CommandFeature::from_application,
    )?);

    stats = stats.merge(deploy_feature::<SkillFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        &Feature::Skill,
        &cache,
        opts.force,
        opts.no_cache,
        SkillFeature::from_application,
    )?);

    stats = stats.merge(deploy_feature::<McpFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        &Feature::Mcp,
        &cache,
        opts.force,
        opts.no_cache,
        || McpFeature::from_application().map(|mcp| vec![mcp]),
    )?);

    stats = stats.merge(deploy_feature::<InstructionFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        &Feature::Instruction,
        &cache,
        opts.force,
        opts.no_cache,
        || InstructionFeature::from_application().map(|inst| vec![inst]),
    )?);

    // Always persist cache to disk (--no-cache only skips hash comparison, not persistence).
    cache
        .lock()
        .unwrap()
        .save()
        .context("Failed to save cache")?;

    // Print deploy summary before the gitignore step.
    print_deploy_summary(&stats);

    // Skip gitignore update when no files were written or the user opted out.
    if stats.paths.is_empty() || opts.no_gitignore {
        return Ok(());
    }

    let workspace_root = get_workspace_dir().context("Failed to get workspace directory")?;

    let should_update = if opts.gitignore {
        true
    } else {
        // Compute how many patterns would be added to decide whether to prompt.
        let gitignore_path = workspace_root.join(".gitignore");
        let current_content = read_gitignore(&gitignore_path)?;
        let existing = parse_fenced_section(&current_content);
        let new_count = stats
            .paths
            .iter()
            .filter_map(|entry| gitignore_path_to_pattern(entry, &workspace_root))
            .filter(|s| !existing.contains(s.as_str()))
            .collect::<HashSet<_>>()
            .len();

        if new_count == 0 {
            return Ok(());
        }

        prompt_gitignore_update(new_count)
    };

    if should_update && let Err(e) = write_gitignore(&workspace_root, &stats.paths) {
        warn!("Failed to update .gitignore: {}", e);
    }

    Ok(())
}
