use crate::prelude::*;
use rayon::prelude::*;
use serde_json::{Value, to_value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::cli::options::DeployOptions;
use crate::cli::ui::deploy::{
    deploy_outro, print_deploy_summary, prompt_gitignore_update, prompt_offline,
};
use crate::cli::ui::dry_run::{
    DeployDryRunStatus, DryRunDeployEntry, print_dry_run_deploy_summary,
};
use crate::core::config::{
    AppConfig, CACHE_SINGLETON_KEY, CacheConfig, CacheEntry, CacheUpdate, FeatureSettings,
};
use crate::core::features::{
    Feature, command::CommandFeature, instruction::InstructionFeature, mcp::McpFeature,
    skill::SkillFeature, traits::FeatureTrait,
};
use crate::schema::registry::Registry;
use crate::templates::variables::set_env_paths;
use crate::templates::{
    TemplateCache, Templater, get_templater, registry_url, render_feature_with_settings,
    resolve_provider_defaults, resolve_target_path,
};
use crate::utils::gitignore::rebuild_fence_from_cache;
use crate::utils::hash::{hash_content, hash_file};
use crate::utils::json::merge_json;
use crate::utils::path::{get_workspace_dir, override_workspace_dir};
use crate::utils::tui::is_tui_enabled;
use cliclack::{outro, spinner};
use std::io::Write;

/// Tracks deduplication info for a skipped provider.
#[derive(Debug, Clone)]
struct DedupInfo {
    winner: String,
}

/// A single unit of work for deploy_feature: one (provider, item) pair.
struct DeployWorkItem<'a, T: FeatureTrait> {
    provider_name: String,
    settings: &'a FeatureSettings,
    item: &'a T,
    dedup: Option<DedupInfo>,
}

/// Aggregated result of deploying one feature across all providers.
#[derive(Debug, Default)]
pub(crate) struct DeployStats {
    pub written: usize,
    pub skipped: usize,
    /// Populated only during `--dry-run`; empty in normal deploys.
    pub dry_run_entries: Vec<DryRunDeployEntry>,
}

impl DeployStats {
    /// Merge another `DeployStats` into this one, consuming both.
    fn merge(mut self, other: Self) -> Self {
        self.written += other.written;
        self.skipped += other.skipped;
        self.dry_run_entries.extend(other.dry_run_entries);
        self
    }
}

/// Shared context passed to `deploy_feature` to reduce argument count.
struct DeployContext<'a> {
    app_config: &'a AppConfig,
    templater: &'a Templater,
    variables: Option<&'a Value>,
    cache: &'a Arc<Mutex<CacheConfig>>,
    force: bool,
    no_cache: bool,
    dry_run: bool,
}

/// Resolves target paths for all providers for a single item, grouping by path.
fn resolve_provider_paths<'a, T: FeatureTrait>(
    item: &'a T,
    providers: &'a HashMap<String, FeatureSettings>,
    templater: &Templater,
    variables: Option<&Value>,
) -> Result<HashMap<PathBuf, Vec<(&'a String, &'a FeatureSettings)>>> {
    let name_var: Option<Value> = item
        .get_file_name()
        .map(|filename| item.get_name_variable(&filename))
        .transpose()?
        .flatten();
    let item_base_vars = merge_json(variables, name_var.as_ref());

    let mut path_groups: HashMap<PathBuf, Vec<(&String, &FeatureSettings)>> = HashMap::new();
    for (provider_name, settings) in providers {
        let target_str = settings
            .target
            .as_deref()
            .ok_or_else(|| anyhow!("Target config not found for provider {}", provider_name))?;
        let target_path = resolve_target_path(templater, target_str, Some(&item_base_vars))?;
        path_groups
            .entry(target_path)
            .or_default()
            .push((provider_name, settings));
    }
    Ok(path_groups)
}

/// Groups providers by resolved target path and marks dedup winners/losers.
/// Returns `(provider_name, is_winner, dedup_winner_name)`.
pub(crate) fn dedup_by_path(providers: &[(String, String)]) -> Vec<(String, bool, Option<String>)> {
    let mut path_groups: HashMap<String, Vec<String>> = HashMap::new();
    for (provider_name, target_path) in providers {
        path_groups
            .entry(target_path.clone())
            .or_default()
            .push(provider_name.clone());
    }

    let mut result = Vec::new();
    for (_path, mut group) in path_groups {
        group.sort();
        let winner = group[0].clone();
        result.push((winner.clone(), true, None));
        for loser in &group[1..] {
            result.push((loser.clone(), false, Some(winner.clone())));
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Builds the dedup-aware work list for a single item across all providers.
fn build_item_work_items<'a, T: FeatureTrait>(
    item: &'a T,
    providers: &'a HashMap<String, FeatureSettings>,
    templater: &Templater,
    variables: Option<&Value>,
) -> Result<Vec<DeployWorkItem<'a, T>>> {
    let path_groups = resolve_provider_paths(item, providers, templater, variables)?;

    let provider_pairs: Vec<(String, String)> = path_groups
        .into_iter()
        .flat_map(|(path, group)| {
            group
                .into_iter()
                .map(|(name, _)| (name.clone(), path.to_string_lossy().to_string()))
                .collect::<Vec<_>>()
        })
        .collect();

    let dedup_results = dedup_by_path(&provider_pairs);

    let mut work_list = Vec::new();
    for (provider_name, is_winner, winner_name) in dedup_results {
        let settings = providers.get(&provider_name).unwrap();
        let dedup = if is_winner {
            None
        } else {
            Some(DedupInfo {
                winner: winner_name.unwrap(),
            })
        };
        work_list.push(DeployWorkItem {
            provider_name,
            settings,
            item,
            dedup,
        });
    }
    Ok(work_list)
}

/// Builds the full dedup-aware work list for all items.
fn build_work_list<'a, T: FeatureTrait>(
    items: &'a [T],
    providers: &'a HashMap<String, FeatureSettings>,
    templater: &Templater,
    variables: Option<&Value>,
) -> Result<Vec<DeployWorkItem<'a, T>>> {
    let mut all_work = Vec::new();
    for item in items {
        all_work.extend(build_item_work_items(
            item, providers, templater, variables,
        )?);
    }
    Ok(all_work)
}

/// Handles a dedup-skipped work item: logs, increments skip count, and records dry-run entry.
fn handle_dedup_skip<T: FeatureTrait>(
    work: &DeployWorkItem<'_, T>,
    templater: &Templater,
    variables: Option<&Value>,
    dry_run: bool,
    stats: &mut DeployStats,
) -> Result<()> {
    debug!(
        "provider {} targets same file as {} — deduplicating",
        work.provider_name,
        work.dedup.as_ref().unwrap().winner
    );
    stats.skipped += 1;

    if dry_run {
        let target_str = work.settings.target.as_deref().unwrap_or("");
        let name_var: Option<Value> = work
            .item
            .get_file_name()
            .map(|filename| work.item.get_name_variable(&filename))
            .transpose()?
            .flatten();
        let target_vars = merge_json(variables, name_var.as_ref());
        if let Ok(target_path) = resolve_target_path(templater, target_str, Some(&target_vars)) {
            stats.dry_run_entries.push(DryRunDeployEntry {
                path: target_path,
                status: DeployDryRunStatus::DedupSkipped {
                    winner: work.dedup.as_ref().unwrap().winner.clone(),
                },
            });
        }
    }
    Ok(())
}

/// Processes a single cache update result, updating stats and cache accordingly.
fn process_cache_update<T: FeatureTrait>(
    work: &DeployWorkItem<'_, T>,
    feature_name: &str,
    item_key: &str,
    update: CacheUpdate,
    cache: &Arc<Mutex<CacheConfig>>,
    stats: &mut DeployStats,
) -> Result<()> {
    match update {
        CacheUpdate::Written { hash, target } => {
            stats.written += 1;
            cache.lock().unwrap().set(
                &work.provider_name,
                feature_name,
                item_key,
                CacheEntry { hash, target },
            );
        }
        CacheUpdate::DryRun { target, content } => {
            let rendered_hash = hash_content(&content);
            if !target.exists() {
                stats.dry_run_entries.push(DryRunDeployEntry {
                    path: target,
                    status: DeployDryRunStatus::New,
                });
            } else {
                match hash_file(&target)? {
                    Some(disk_hash) if disk_hash == rendered_hash => {}
                    _ => {
                        stats.dry_run_entries.push(DryRunDeployEntry {
                            path: target,
                            status: DeployDryRunStatus::Modified,
                        });
                    }
                }
            }
        }
        CacheUpdate::Skipped | CacheUpdate::UserEditedSkipped { .. } => {
            stats.skipped += 1;
        }
    }
    Ok(())
}

/// Deploys one feature across all enabled providers, collecting gitignore entries and updating cache.
fn deploy_feature<T>(
    ctx: &DeployContext<'_>,
    feature: &Feature,
    loader: impl FnOnce() -> Result<Vec<T>>,
) -> Result<DeployStats>
where
    T: FeatureTrait + Sync,
{
    if !ctx.app_config.has_feature(feature) {
        return Ok(DeployStats::default());
    }

    let items = loader().context(format!("unable to load {} feature", feature))?;
    let providers = ctx.app_config.get_provider_feature_settings(feature);

    if providers.is_empty() {
        return Ok(DeployStats::default());
    }

    let work_list = build_work_list(&items, &providers, ctx.templater, ctx.variables)?;

    let feature_name = feature.as_str();
    let cache_ref = ctx.cache.clone();

    let stats: DeployStats = work_list
        .par_iter()
        .try_fold(DeployStats::default, |acc, work| -> Result<DeployStats> {
            let mut stats = acc;

            if let Some(ref _dedup) = work.dedup {
                handle_dedup_skip(work, ctx.templater, ctx.variables, ctx.dry_run, &mut stats)?;
                return Ok(stats);
            }

            let file_name = work.item.get_file_name();
            let item_key = file_name.as_deref().unwrap_or(CACHE_SINGLETON_KEY);

            let cached_entry: Option<CacheEntry> = if ctx.no_cache {
                None
            } else {
                cache_ref
                    .lock()
                    .unwrap()
                    .get(&work.provider_name, feature_name, item_key)
                    .cloned()
            };

            let update = render_feature_with_settings(
                &work.provider_name,
                work.item,
                work.settings,
                ctx.templater,
                ctx.variables,
                cached_entry.as_ref(),
                ctx.force,
                ctx.dry_run,
            )?;

            process_cache_update(work, feature_name, item_key, update, &cache_ref, &mut stats)?;

            Ok(stats)
        })
        .try_reduce(DeployStats::default, |a, b| Ok(a.merge(b)))?;

    Ok(stats)
}

/// Fetches the provider registry, showing a spinner in TUI mode. Returns None on failure or when offline.
fn fetch_registry(offline: bool) -> Option<Registry> {
    if offline {
        return None;
    }

    let sp = if is_tui_enabled() {
        let s = spinner();
        s.start("Fetching provider registry…");
        Some(s)
    } else {
        None
    };

    match Registry::fetch(registry_url()) {
        Ok(r) => {
            if let Some(s) = sp {
                s.clear();
            }
            Some(r)
        }
        Err(e) => {
            if let Some(s) = sp {
                s.error(format!("Could not reach registry: {}", e));
            }
            warn!(
                "Failed to fetch provider registry: {} — falling back to local cache",
                e
            );
            None
        }
    }
}

/// Prints the deploy summary and optionally updates .gitignore from cache targets.
fn finalize_deploy(
    opts: &DeployOptions,
    stats: &DeployStats,
    cache: &Arc<Mutex<CacheConfig>>,
) -> Result<()> {
    if opts.no_gitignore {
        print_summary(stats);
        return Ok(());
    }

    let workspace_root = get_workspace_dir().context("unable to get workspace directory")?;
    let cache_targets = cache.lock().unwrap().all_targets();

    if cache_targets.is_empty() {
        print_summary(stats);
        return Ok(());
    }

    let should_update = if opts.gitignore {
        true
    } else {
        stats.written > 0 && prompt_gitignore_update(cache_targets.len())
    };

    if should_update && let Err(e) = rebuild_fence_from_cache(&cache_targets, &workspace_root) {
        warn!("Failed to update .gitignore: {}", e);
    }

    print_summary(stats);
    Ok(())
}

/// Prints the deploy summary using TUI or plain output depending on context.
fn print_summary(stats: &DeployStats) {
    if is_tui_enabled() {
        outro(deploy_outro(stats)).ok();
        let _ = std::io::stdout().flush();
    } else {
        print_deploy_summary(stats);
    }
}

/// Deploys all enabled features and returns aggregated stats.
fn deploy_all_features(ctx: &DeployContext<'_>) -> Result<DeployStats> {
    let mut stats = DeployStats::default();

    stats = stats.merge(deploy_feature::<CommandFeature>(
        ctx,
        &Feature::Command,
        CommandFeature::from_application,
    )?);

    stats = stats.merge(deploy_feature::<SkillFeature>(
        ctx,
        &Feature::Skill,
        SkillFeature::from_application,
    )?);

    stats = stats.merge(deploy_feature::<McpFeature>(ctx, &Feature::Mcp, || {
        McpFeature::from_application().map(|mcp| vec![mcp])
    })?);

    stats = stats.merge(deploy_feature::<InstructionFeature>(
        ctx,
        &Feature::Instruction,
        || InstructionFeature::from_application().map(|inst| vec![inst]),
    )?);

    Ok(stats)
}

pub(super) fn deploy(mut opts: DeployOptions) -> Result<()> {
    for path in &opts.env {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "load env file '{}': file not found",
                path.display()
            ));
        }
    }
    set_env_paths(std::mem::take(&mut opts.env));

    if let Some(dir) = opts.dir.take() {
        let workspace = std::env::current_dir()
            .context("failed to get current directory")?
            .join(dir);
        override_workspace_dir(workspace).context("unable to set workspace directory")?;
    }

    let templater = get_templater().context("unable to initialise templater")?;
    let mut app_config =
        AppConfig::from_application(templater).context("unable to load application config")?;

    if !opts.offline && is_tui_enabled() {
        opts.offline = prompt_offline();
    }

    let template_cache = TemplateCache::new().context("unable to initialise template cache")?;
    let registry = fetch_registry(opts.offline);

    resolve_provider_defaults(
        &mut app_config,
        registry.as_ref(),
        &template_cache,
        opts.offline,
        opts.no_cache,
    )
    .context("unable to resolve provider template defaults")?;

    let variables: Option<Value> = app_config
        .variables
        .as_ref()
        .map(|v| to_value(v).context("unable to extract variables"))
        .transpose()?;

    let has_any_provider = Feature::all()
        .iter()
        .any(|f| !app_config.get_provider_feature_settings(f).is_empty());
    if !has_any_provider {
        warn!("No providers configured — nothing to deploy. Add providers to config.toml.");
    }

    let cache = Arc::new(Mutex::new(
        CacheConfig::load().context("unable to load cache")?,
    ));

    let ctx = DeployContext {
        app_config: &app_config,
        templater,
        variables: variables.as_ref(),
        cache: &cache,
        force: opts.force,
        no_cache: opts.no_cache,
        dry_run: opts.dry_run,
    };

    let stats = deploy_all_features(&ctx)?;

    if opts.dry_run {
        print_dry_run_deploy_summary(&stats.dry_run_entries);
        return Ok(());
    }

    cache
        .lock()
        .unwrap()
        .save()
        .context("unable to save cache")?;

    finalize_deploy(&opts, &stats, &cache)
}

#[cfg(test)]
mod tests {
    use super::dedup_by_path;

    // alphabetical winner selected when 3 providers target same path
    #[test]
    fn dedup_alphabetical_winner_three_providers() {
        let providers = vec![
            ("zebra".to_string(), "AGENTS.md".to_string()),
            ("alpha".to_string(), "AGENTS.md".to_string()),
            ("middle".to_string(), "AGENTS.md".to_string()),
        ];
        let result = dedup_by_path(&providers);
        let winner = result.iter().find(|(_, is_winner, _)| *is_winner).unwrap();
        assert_eq!(winner.0, "alpha");
        let losers: Vec<_> = result
            .iter()
            .filter(|(_, is_winner, _)| !is_winner)
            .collect();
        assert_eq!(losers.len(), 2);
        assert!(
            losers
                .iter()
                .all(|(_, _, w)| w.as_ref().unwrap() == "alpha")
        );
    }

    // no dedup when providers target different paths
    #[test]
    fn dedup_no_collision_different_paths() {
        let providers = vec![
            ("claude".to_string(), ".claude/AGENTS.md".to_string()),
            ("codex".to_string(), ".openai/AGENTS.md".to_string()),
        ];
        let result = dedup_by_path(&providers);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, is_winner, _)| *is_winner));
        assert!(result.iter().all(|(_, _, w)| w.is_none()));
    }

    // mixed: some providers share path, others don't
    #[test]
    fn dedup_mixed_some_collide() {
        let providers = vec![
            ("a".to_string(), "shared.md".to_string()),
            ("b".to_string(), "shared.md".to_string()),
            ("c".to_string(), "unique.md".to_string()),
        ];
        let result = dedup_by_path(&providers);
        let winners: Vec<_> = result
            .iter()
            .filter(|(_, is_winner, _)| *is_winner)
            .collect();
        let losers: Vec<_> = result
            .iter()
            .filter(|(_, is_winner, _)| !is_winner)
            .collect();
        assert_eq!(winners.len(), 2);
        assert_eq!(losers.len(), 1);
        assert_eq!(losers[0].2.as_ref().unwrap(), "a");
    }
}
