use crate::prelude::*;
use rayon::prelude::*;
use serde_json::{Value, to_value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

use crate::cli::options::DeployOptions;
use crate::cli::ui::deploy::{fetch_registry, finalize_deploy};
use crate::cli::ui::dry_run::{
    DeployDryRunStatus, DryRunDeployEntry, print_dry_run_deploy_summary,
};
use crate::core::config::{
    AppConfig, CACHE_SINGLETON_KEY, CacheConfig, CacheEntry, CacheUpdate, FeatureMode,
};
use crate::core::features::{
    Feature, command::CommandFeature, hook::HookFeature, ignore::IgnoreFeature,
    instruction::InstructionFeature, mcp::McpFeature, skill::SkillFeature, traits::FeatureTrait,
};
use crate::templates::variables::set_env_paths;
use crate::templates::{
    TemplateCache, Templater, get_templater, link_feature_with_settings,
    render_feature_with_settings, resolve_provider_defaults, resolve_target_path,
};
use crate::utils::dedup::{DeployWorkItem, build_work_list};
use crate::utils::fs::write_symlink;
use crate::utils::hash::{hash_content, hash_file};
use crate::utils::json::merge_json;
use crate::utils::path::override_workspace_dir;

/// Aggregated result of deploying one feature across all providers.
#[derive(Debug, Default)]
pub(crate) struct DeployStats {
    pub written: usize,
    pub skipped: usize,
    pub user_edited: usize,
    /// Populated only during `--dry-run`; empty in normal deploys.
    pub dry_run_entries: Vec<DryRunDeployEntry>,
    /// Target paths of symlinked items (for .gitignore fence).
    pub linked_targets: Vec<PathBuf>,
}

impl DeployStats {
    /// Merge another `DeployStats` into this one, consuming both.
    fn merge(mut self, other: Self) -> Self {
        self.written += other.written;
        self.skipped += other.skipped;
        self.user_edited += other.user_edited;
        self.dry_run_entries.extend(other.dry_run_entries);
        self.linked_targets.extend(other.linked_targets);
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
    dry_run: bool,
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
        CacheUpdate::Linked { target } => {
            stats.written += 1;
            stats.linked_targets.push(target.clone());
            if dry_run {
                stats.dry_run_entries.push(DryRunDeployEntry {
                    path: target,
                    status: DeployDryRunStatus::Linked,
                });
            }
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
            stats.user_edited += 1;
        }
        CacheUpdate::MergeSkipped { path, reason } => {
            debug!("merge skipped: path={}, reason={}", path.display(), reason);
            stats.skipped += 1;
        }
    }
    Ok(())
}

/// Walks `source_dir` recursively and symlinks every file except the `skip_path`
/// into the corresponding location under `target_dir`.
fn deploy_extra_files(
    source_dir: &Path,
    target_dir: &Path,
    skip_path: &Path,
    dry_run: bool,
) -> Result<Vec<PathBuf>> {
    let canonical = source_dir
        .canonicalize()
        .with_context(|| format!("unable to canonicalize source dir {}", source_dir.display()))?;
    let canonical_skip = skip_path
        .canonicalize()
        .with_context(|| format!("unable to canonicalize skip path {}", skip_path.display()))?;
    let mut linked = Vec::new();
    deploy_extra_files_recursive(
        &canonical,
        &canonical,
        target_dir,
        &canonical_skip,
        dry_run,
        &mut linked,
    )?;
    Ok(linked)
}

fn deploy_extra_files_recursive(
    base_source: &Path,
    current_source: &Path,
    target_base: &Path,
    skip_path: &Path,
    dry_run: bool,
    linked: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(current_source)? {
        let entry = entry?;
        let src_path = entry.path();
        if src_path == skip_path {
            continue;
        }
        let relative = src_path
            .strip_prefix(base_source)
            .context("unable to compute relative path for extra file")?;
        let target_path = target_base.join(relative);

        if src_path.is_dir() {
            if !dry_run {
                fs::create_dir_all(&target_path).with_context(|| {
                    format!("unable to create target dir {}", target_path.display())
                })?;
            }
            deploy_extra_files_recursive(
                base_source,
                &src_path,
                target_base,
                skip_path,
                dry_run,
                linked,
            )?;
        } else if src_path.is_file() {
            if !dry_run {
                write_symlink(&src_path, &target_path).with_context(|| {
                    format!(
                        "unable to symlink extra file {} -> {}",
                        src_path.display(),
                        target_path.display()
                    )
                })?;
            }
            linked.push(target_path);
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
    let provider_agnostic = T::is_provider_agnostic();
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

            let mode = ctx
                .app_config
                .resolve_mode(feature_name, file_name.as_deref());

            let update =
                if provider_agnostic && work.item.is_symlinkable() && mode == FeatureMode::Link {
                    let source_path = T::resolve_source_path(file_name.as_deref())
                        .context("unable to resolve source path for symlink")?;

                    link_feature_with_settings(
                        &work.provider_name,
                        work.item,
                        work.settings,
                        ctx.templater,
                        ctx.variables,
                        ctx.dry_run,
                        &source_path,
                    )?
                } else {
                    let cached_entry: Option<CacheEntry> = if ctx.no_cache {
                        None
                    } else {
                        cache_ref
                            .lock()
                            .unwrap()
                            .get(&work.provider_name, feature_name, item_key)
                            .cloned()
                    };

                    render_feature_with_settings(
                        &work.provider_name,
                        work.item,
                        work.settings,
                        mode,
                        ctx.templater,
                        ctx.variables,
                        cached_entry.as_ref(),
                        ctx.force,
                        ctx.dry_run,
                    )?
                };

            // Extract target path for extra file resolution (Type 1 features)
            let main_target: Option<PathBuf> = match &update {
                CacheUpdate::Linked { target } => Some(target.clone()),
                CacheUpdate::Written { target, .. } => Some(PathBuf::from(target)),
                CacheUpdate::DryRun { target, .. } => Some(target.clone()),
                _ => None,
            };

            process_cache_update(
                work,
                feature_name,
                item_key,
                update,
                &cache_ref,
                ctx.dry_run,
                &mut stats,
            )?;

            // Deploy extra files for Type 1 features regardless of mode (spec: "regardless of the mode setting")
            if provider_agnostic
                && let Some(main_target) = main_target
                && let Some(source_dir) = T::source_dir(file_name.as_deref())
            {
                let source_path = T::resolve_source_path(file_name.as_deref())
                    .context("unable to resolve source path for extra files")?;
                let target_dir = main_target
                    .parent()
                    .with_context(|| "target path has no parent directory")?;
                let extra_targets =
                    deploy_extra_files(&source_dir, target_dir, &source_path, ctx.dry_run)?;
                if !ctx.dry_run || !extra_targets.is_empty() {
                    stats.written += extra_targets.len();
                    stats.linked_targets.extend(extra_targets.clone());
                    if ctx.dry_run {
                        for target in extra_targets {
                            stats.dry_run_entries.push(DryRunDeployEntry {
                                path: target,
                                status: DeployDryRunStatus::Linked,
                            });
                        }
                    }
                }
            }

            Ok(stats)
        })
        .try_reduce(DeployStats::default, |a, b| Ok(a.merge(b)))?;

    Ok(stats)
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

    stats = stats.merge(deploy_feature::<HookFeature>(ctx, &Feature::Hook, || {
        HookFeature::from_application().map(|h| vec![h])
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

    finalize_deploy(&opts, &stats, &cache)?;

    // Trust-hash warning for hooks (codex, claude, trae) — one line per provider if any hooks written.
    const TRUST_HASH_PROVIDERS: &[&str] = &["codex", "claude", "trae"];
    if stats.written > 0 {
        for p in TRUST_HASH_PROVIDERS {
            if app_config
                .get_provider_feature_settings(&Feature::Hook)
                .contains_key(*p)
            {
                warn!(
                    "{}: re-trust required — run /hooks in {} to review changed hooks",
                    p, p
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ui::dry_run::DeployDryRunStatus;
    use crate::core::config::FeatureSettings;
    use crate::core::features::instruction::InstructionFeature;
    use crate::utils::dedup::DeployWorkItem;

    // process_cache_update increments user_edited on UserEditedSkipped
    #[test]
    fn process_cache_update_user_edited_skipped() {
        let feature = InstructionFeature::from_string("test").unwrap();
        let settings = FeatureSettings::default();
        let work = DeployWorkItem {
            provider_name: "test".to_string(),
            settings: &settings,
            item: &feature,
            dedup: None,
        };
        let cache = Arc::new(Mutex::new(CacheConfig::default()));
        let mut stats = DeployStats::default();

        process_cache_update(
            &work,
            "instruction",
            "key",
            CacheUpdate::UserEditedSkipped {
                path: PathBuf::from("/tmp/test"),
            },
            &cache,
            false,
            &mut stats,
        )
        .unwrap();

        assert_eq!(stats.user_edited, 1);
        assert_eq!(stats.skipped, 1);
    }

    // process_cache_update on Linked increments written, tracks target, and writes no cache entry
    #[test]
    fn process_cache_update_linked_skips_cache() {
        let feature = InstructionFeature::from_string("test").unwrap();
        let settings = FeatureSettings::default();
        let work = DeployWorkItem {
            provider_name: "test".to_string(),
            settings: &settings,
            item: &feature,
            dedup: None,
        };
        let cache = Arc::new(Mutex::new(CacheConfig::default()));
        let mut stats = DeployStats::default();

        let target = PathBuf::from("/tmp/linked.md");
        process_cache_update(
            &work,
            "skill",
            "my-skill",
            CacheUpdate::Linked {
                target: target.clone(),
            },
            &cache,
            false,
            &mut stats,
        )
        .unwrap();

        assert_eq!(stats.written, 1, "Linked should increment written");
        assert_eq!(
            stats.linked_targets,
            vec![target.clone()],
            "Linked should track target in linked_targets"
        );
        assert!(
            cache
                .lock()
                .unwrap()
                .get("test", "skill", "my-skill")
                .is_none(),
            "Linked variant must NOT write a cache entry"
        );
        assert_eq!(stats.skipped, 0, "Linked should not increment skipped");
        assert!(
            stats.dry_run_entries.is_empty(),
            "non-dry-run should not add entries"
        );
    }

    // process_cache_update on Linked in dry-run mode adds a Linked dry-run entry
    #[test]
    fn process_cache_update_linked_dry_run_adds_entry() {
        let feature = InstructionFeature::from_string("test").unwrap();
        let settings = FeatureSettings::default();
        let work = DeployWorkItem {
            provider_name: "test".to_string(),
            settings: &settings,
            item: &feature,
            dedup: None,
        };
        let cache = Arc::new(Mutex::new(CacheConfig::default()));
        let mut stats = DeployStats::default();

        let target = PathBuf::from("/tmp/linked.md");
        process_cache_update(
            &work,
            "skill",
            "my-skill",
            CacheUpdate::Linked {
                target: target.clone(),
            },
            &cache,
            true,
            &mut stats,
        )
        .unwrap();

        assert_eq!(stats.written, 1);
        assert_eq!(stats.linked_targets, vec![target.clone()]);
        assert_eq!(stats.dry_run_entries.len(), 1);
        assert_eq!(stats.dry_run_entries[0].path, target);
        assert_eq!(stats.dry_run_entries[0].status, DeployDryRunStatus::Linked);
        assert!(
            cache
                .lock()
                .unwrap()
                .get("test", "skill", "my-skill")
                .is_none(),
            "dry-run Linked must NOT write a cache entry"
        );
    }

    // process_cache_update on Written writes a cache entry (contrast with Linked)
    #[test]
    fn process_cache_update_written_writes_cache_entry() {
        let feature = InstructionFeature::from_string("test").unwrap();
        let settings = FeatureSettings::default();
        let work = DeployWorkItem {
            provider_name: "test".to_string(),
            settings: &settings,
            item: &feature,
            dedup: None,
        };
        let cache = Arc::new(Mutex::new(CacheConfig::default()));
        let mut stats = DeployStats::default();

        process_cache_update(
            &work,
            "command",
            "hello",
            CacheUpdate::Written {
                hash: "abc123".to_string(),
                target: "/tmp/out.md".to_string(),
            },
            &cache,
            false,
            &mut stats,
        )
        .unwrap();

        assert_eq!(stats.written, 1);
        assert!(
            cache
                .lock()
                .unwrap()
                .get("test", "command", "hello")
                .is_some(),
            "Written variant should write a cache entry"
        );
        assert!(
            stats.linked_targets.is_empty(),
            "Written should not add to linked_targets"
        );
    }

    // deploy_extra_files with only SKILL.md returns empty list and creates no symlinks
    #[test]
    fn deploy_extra_files_empty_dir_returns_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source_dir = tmp.path().join("my-skill");
        fs::create_dir_all(&source_dir).unwrap();
        let skill_md = source_dir.join("SKILL.md");
        fs::write(&skill_md, "skill body").unwrap();

        let target_dir = tmp.path().join("target").join("my-skill");
        let linked = deploy_extra_files(&source_dir, &target_dir, &skill_md, false).unwrap();

        assert!(linked.is_empty(), "no extra files should be linked");
        assert!(
            !target_dir.exists(),
            "target dir should not be created when there are no extra files"
        );
    }

    // deploy_extra_files mirrors nested directories and symlinks deep files
    #[test]
    fn deploy_extra_files_deep_nesting_mirrors_structure() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source_dir = tmp.path().join("my-skill");
        let deep_dir = source_dir.join("data").join("sub");
        fs::create_dir_all(&deep_dir).unwrap();
        let skill_md = source_dir.join("SKILL.md");
        fs::write(&skill_md, "skill body").unwrap();
        fs::write(source_dir.join("script.py"), "print('hi')").unwrap();
        fs::write(deep_dir.join("config.json"), "{}").unwrap();
        fs::write(deep_dir.join("nested.md"), "deep").unwrap();

        let target_dir = tmp.path().join("target").join("my-skill");
        let linked = deploy_extra_files(&source_dir, &target_dir, &skill_md, false).unwrap();

        assert_eq!(
            linked.len(),
            3,
            "should link script.py, config.json, nested.md"
        );
        let script_target = target_dir.join("script.py");
        let config_target = target_dir.join("data").join("sub").join("config.json");
        let nested_target = target_dir.join("data").join("sub").join("nested.md");
        assert!(script_target.is_symlink(), "script.py should be symlinked");
        assert!(
            config_target.is_symlink(),
            "deeply nested config.json should be symlinked"
        );
        assert!(
            nested_target.is_symlink(),
            "deeply nested nested.md should be symlinked"
        );
        assert_eq!(
            fs::read_to_string(&config_target).unwrap(),
            "{}",
            "deeply nested symlink should resolve to source content"
        );
    }

    // deploy_extra_files overwrites existing regular files at target with symlinks
    #[test]
    fn deploy_extra_files_overwrites_existing_target() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source_dir = tmp.path().join("my-skill");
        fs::create_dir_all(&source_dir).unwrap();
        let skill_md = source_dir.join("SKILL.md");
        fs::write(&skill_md, "skill body").unwrap();
        fs::write(source_dir.join("helper.py"), "new content").unwrap();

        let target_dir = tmp.path().join("target").join("my-skill");
        fs::create_dir_all(&target_dir).unwrap();
        let existing_target = target_dir.join("helper.py");
        fs::write(&existing_target, "old regular content").unwrap();
        assert!(
            !existing_target.is_symlink(),
            "precondition: target is regular file"
        );

        let linked = deploy_extra_files(&source_dir, &target_dir, &skill_md, false).unwrap();

        assert_eq!(linked.len(), 1);
        assert!(
            existing_target.is_symlink(),
            "existing regular file should be overwritten with symlink"
        );
        assert_eq!(
            fs::read_to_string(&existing_target).unwrap(),
            "new content",
            "overwritten symlink should resolve to new source content"
        );
    }

    // deploy_extra_files in dry-run mode tracks paths without creating symlinks
    #[test]
    fn deploy_extra_files_dry_run_tracks_without_creating() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source_dir = tmp.path().join("my-skill");
        fs::create_dir_all(&source_dir).unwrap();
        let skill_md = source_dir.join("SKILL.md");
        fs::write(&skill_md, "skill body").unwrap();
        fs::write(source_dir.join("extra.txt"), "extra").unwrap();

        let target_dir = tmp.path().join("target").join("my-skill");
        let linked = deploy_extra_files(&source_dir, &target_dir, &skill_md, true).unwrap();

        assert_eq!(linked.len(), 1, "dry-run should track the extra file path");
        assert_eq!(linked[0], target_dir.join("extra.txt"));
        assert!(
            !target_dir.exists(),
            "dry-run should not create target directory or symlinks"
        );
    }

    // deploy_extra_files skips the SKILL.md path and links only extras
    #[test]
    fn deploy_extra_files_skips_skill_md() {
        let tmp = tempfile::TempDir::new().unwrap();
        let source_dir = tmp.path().join("my-skill");
        fs::create_dir_all(&source_dir).unwrap();
        let skill_md = source_dir.join("SKILL.md");
        fs::write(&skill_md, "skill body").unwrap();
        fs::write(source_dir.join("extra.txt"), "extra").unwrap();

        let target_dir = tmp.path().join("target").join("my-skill");
        let linked = deploy_extra_files(&source_dir, &target_dir, &skill_md, false).unwrap();

        assert_eq!(linked.len(), 1, "only extra.txt should be linked");
        assert!(
            !target_dir.join("SKILL.md").exists(),
            "SKILL.md should be skipped"
        );
        assert!(
            target_dir.join("extra.txt").is_symlink(),
            "extra.txt should be symlinked"
        );
    }
}
