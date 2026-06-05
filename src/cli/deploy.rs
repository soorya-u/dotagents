use crate::prelude::*;
use rayon::prelude::*;
use serde_json::{Value, to_value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

use crate::cli::options::DeployOptions;
use crate::cli::ui::deploy::{deploy_outro, print_deploy_summary, prompt_gitignore_update};
use crate::cli::ui::dry_run::{
    DeployDryRunStatus, DryRunDeployEntry, print_dry_run_deploy_summary,
};
use crate::core::config::{
    AppConfig, CACHE_SINGLETON_KEY, CacheConfig, CacheEntry, CacheUpdate, FeatureSettings,
};
use crate::core::features::{
    Feature, command::CommandFeature, ignore::IgnoreFeature, instruction::InstructionFeature,
    mcp::McpFeature, skill::SkillFeature, traits::FeatureTrait,
};
use crate::schema::registry::Registry;
use crate::templates::variables::set_env_paths;
use crate::templates::{
    TemplateCache, Templater, get_templater, registry_url, render_feature_with_settings,
    resolve_provider_defaults, resolve_target_path,
};
use crate::utils::gitignore::{collapse_paths, rebuild_fence_from_cache};
use crate::utils::hash::{hash_content, hash_file};
use crate::utils::json::merge_json;
use crate::utils::path::{get_workspace_dir, make_workspace_relative, override_workspace_dir};
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
/// Returns `(target_path, winner, losers)` for each unique path.
pub(crate) fn dedup_by_path(
    providers: &[(String, PathBuf)],
) -> Vec<(PathBuf, String, Vec<String>)> {
    let mut path_groups: HashMap<PathBuf, Vec<String>> = HashMap::new();
    for (provider_name, target_path) in providers {
        path_groups
            .entry(target_path.clone())
            .or_default()
            .push(provider_name.clone());
    }

    let mut result: Vec<(PathBuf, String, Vec<String>)> = path_groups
        .into_iter()
        .map(|(path, mut group)| {
            group.sort();
            let winner = group.remove(0);
            (path, winner, group)
        })
        .collect();
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

    let provider_pairs: Vec<(String, PathBuf)> = path_groups
        .into_iter()
        .flat_map(|(path, group)| {
            group
                .into_iter()
                .map(|(name, _)| (name.clone(), path.clone()))
                .collect::<Vec<_>>()
        })
        .collect();

    let dedup_results = dedup_by_path(&provider_pairs);

    let mut work_list = Vec::new();
    for (_path, winner, losers) in dedup_results {
        let settings = providers.get(&winner).unwrap();
        work_list.push(DeployWorkItem {
            provider_name: winner.clone(),
            settings,
            item,
            dedup: None,
        });
        for loser in losers {
            let settings = providers.get(&loser).unwrap();
            work_list.push(DeployWorkItem {
                provider_name: loser,
                settings,
                item,
                dedup: Some(DedupInfo {
                    winner: winner.clone(),
                }),
            });
        }
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
        CacheUpdate::Skipped => {
            stats.skipped += 1;
        }
        CacheUpdate::UserEditedSkipped { path } => {
            debug!("user-edited skip: path={}", path.display());
            stats.skipped += 1;
        }
        CacheUpdate::MergeSkipped { path, reason } => {
            debug!("merge skipped: path={}, reason={}", path.display(), reason);
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

    let feature_name = feature.as_ref();
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

    let relative_paths: Vec<String> = cache_targets
        .iter()
        .filter_map(|p| make_workspace_relative(p, &workspace_root))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let pattern_count = collapse_paths(&relative_paths, &workspace_root).len();

    let should_update = if opts.gitignore {
        true
    } else {
        stats.written > 0 && prompt_gitignore_update(pattern_count)
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

    stats = stats.merge(deploy_feature::<IgnoreFeature>(
        ctx,
        &Feature::AgentIgnore,
        || {
            let ignore = IgnoreFeature::from_application()?;
            if ignore.to_string()?.trim().is_empty() {
                return Ok(vec![]);
            }
            Ok(vec![ignore])
        },
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

    let has_any_provider =
        Feature::iter().any(|f| !app_config.get_provider_feature_settings(&f).is_empty());
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
    use std::path::PathBuf;

    // alphabetical winner selected when 3 providers target same path
    #[test]
    fn dedup_alphabetical_winner_three_providers() {
        let providers = vec![
            ("zebra".to_string(), PathBuf::from("AGENTS.md")),
            ("alpha".to_string(), PathBuf::from("AGENTS.md")),
            ("middle".to_string(), PathBuf::from("AGENTS.md")),
        ];
        let result = dedup_by_path(&providers);
        assert_eq!(result.len(), 1);
        let (path, winner, losers) = &result[0];
        assert_eq!(path, &PathBuf::from("AGENTS.md"));
        assert_eq!(winner, "alpha");
        assert_eq!(losers, &vec!["middle".to_string(), "zebra".to_string()]);
    }

    // no dedup when providers target different paths
    #[test]
    fn dedup_no_collision_different_paths() {
        let providers = vec![
            ("claude".to_string(), PathBuf::from(".claude/AGENTS.md")),
            ("codex".to_string(), PathBuf::from(".openai/AGENTS.md")),
        ];
        let result = dedup_by_path(&providers);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, _, losers)| losers.is_empty()));
    }

    // mixed: some providers share path, others don't
    #[test]
    fn dedup_mixed_some_collide() {
        let providers = vec![
            ("a".to_string(), PathBuf::from("shared.md")),
            ("b".to_string(), PathBuf::from("shared.md")),
            ("c".to_string(), PathBuf::from("unique.md")),
        ];
        let result = dedup_by_path(&providers);
        assert_eq!(result.len(), 2);
        let shared = result
            .iter()
            .find(|(p, _, _)| p == &PathBuf::from("shared.md"))
            .unwrap();
        let unique = result
            .iter()
            .find(|(p, _, _)| p == &PathBuf::from("unique.md"))
            .unwrap();
        assert_eq!(shared.1, "a");
        assert_eq!(shared.2, vec!["b"]);
        assert_eq!(unique.1, "c");
        assert_eq!(unique.2, Vec::<String>::new());
    }
}
