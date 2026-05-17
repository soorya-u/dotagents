use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::prelude::*;

use crate::cli::options::UndeployOptions;
use crate::cli::ui::dry_run::{
    DryRunUndeployEntry, UndeployDryRunStatus, print_dry_run_undeploy_summary,
};
use crate::cli::ui::undeploy::{
    print_undeploy_summary, prompt_confirm_undeploy, prompt_delete_edited,
};
use crate::core::config::CacheConfig;
use crate::utils::fs::{delete_file, hash_file, prune_empty_dir};
use crate::utils::gitignore::{clear_gitignore_fence, rebuild_fence_from_cache};
use crate::utils::path::{get_workspace_dir, override_workspace_dir};
use crate::utils::tui::is_tui_enabled;

pub(super) fn undeploy(mut opts: UndeployOptions) -> Result<()> {
    // Override workspace root before any path resolution is triggered.
    if let Some(dir) = opts.dir.take() {
        let workspace = std::env::current_dir()
            .context("failed to get current directory")?
            .join(dir);
        override_workspace_dir(workspace).context("unable to set workspace directory")?;
    }

    let workspace_root = get_workspace_dir().context("unable to get workspace directory")?;

    let mut cache = CacheConfig::load().context("unable to load cache")?;

    // Collect all target paths and their expected hashes from the cache.
    let entries: Vec<(PathBuf, String)> = cache
        .iter_entries()
        .map(|(_, _, _, entry)| (PathBuf::from(&entry.target), entry.hash.clone()))
        .collect();

    if entries.is_empty() {
        if !opts.no_gitignore
            && let Err(e) = clear_gitignore_fence(&workspace_root)
        {
            warn!("Failed to update .gitignore: {}", e);
        }
        if opts.dry_run {
            print_dry_run_undeploy_summary(&[]);
        } else if is_tui_enabled() {
            println!("Nothing to undeploy.");
        }
        return Ok(());
    }

    // Dry-run: classify each entry and print summary without touching anything.
    if opts.dry_run {
        let mut dry_run_entries: Vec<DryRunUndeployEntry> = Vec::new();

        for (path, expected_hash) in &entries {
            if !path.exists() {
                warn!("already removed: {}", path.display());
                continue;
            }

            let disk_hash = hash_file(path)
                .with_context(|| format!("failed to hash {}", path.display()))?
                .unwrap_or_default();

            let status = if disk_hash != *expected_hash {
                UndeployDryRunStatus::Edited
            } else {
                UndeployDryRunStatus::WouldDelete
            };

            dry_run_entries.push(DryRunUndeployEntry {
                path: path.clone(),
                status,
            });
        }

        print_dry_run_undeploy_summary(&dry_run_entries);
        return Ok(());
    }

    // Interactive confirmation (skipped when --force or non-TTY).
    if !opts.force && !prompt_confirm_undeploy(entries.len()) {
        if is_tui_enabled() {
            cliclack::outro_cancel("Undeploy cancelled.").ok();
        }
        return Ok(());
    }

    let mut removed = 0usize;
    let mut skipped = 0usize;

    for (path, expected_hash) in &entries {
        if !path.exists() {
            warn!("already removed: {}", path.display());
            skipped += 1;
            continue;
        }

        // Detect user edits by comparing on-disk hash to cached hash.
        let disk_hash = hash_file(path)
            .with_context(|| format!("failed to hash {}", path.display()))?
            .unwrap_or_default();
        let was_edited = disk_hash != *expected_hash;

        if was_edited && !opts.force {
            if prompt_delete_edited(path) {
                // User confirmed deletion of edited file in TTY.
            } else {
                warn!("skipping user-edited file: {}", path.display());
                skipped += 1;
                continue;
            }
        }

        delete_file(path).with_context(|| format!("failed to delete {}", path.display()))?;
        prune_empty_dir(path)
            .with_context(|| format!("failed to prune directory for {}", path.display()))?;
        removed += 1;
    }

    // Clear and persist cache.
    cache.clear();
    cache
        .save()
        .context("unable to save cache after undeploy")?;

    // Remove the dotagents-managed fence from .gitignore.
    if !opts.no_gitignore
        && let Err(e) = clear_gitignore_fence(&workspace_root)
    {
        warn!("Failed to update .gitignore: {}", e);
    }

    print_undeploy_summary(removed, skipped);

    Ok(())
}

/// Removes deployed files, cache entries, and .gitignore fence paths for one removed item across all providers.
pub(crate) fn undeploy_item(
    feature: &str,
    item_key: &str,
    cache: &mut CacheConfig,
    workspace_dir: &Path,
) -> Result<()> {
    let entries: Vec<(String, String)> = cache
        .iter_entries()
        .filter(|(_, f, i, _)| *f == feature && *i == item_key)
        .map(|(p, _, _, e)| (p.to_string(), e.target.clone()))
        .collect();

    if entries.is_empty() {
        warn!(
            "No deployed files found for '{}' — was it ever deployed?",
            item_key
        );
        return Ok(());
    }

    for (provider, target_path) in &entries {
        match std::fs::remove_file(target_path) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => {
                warn!("Failed to delete deployed file {}: {}", target_path, e);
            }
        }
        cache.remove(provider, feature, item_key);
    }

    let remaining_targets = cache.all_targets();
    if remaining_targets.is_empty() {
        if let Err(e) = clear_gitignore_fence(workspace_dir) {
            warn!("Failed to update .gitignore: {}", e);
        }
    } else if let Err(e) = rebuild_fence_from_cache(&remaining_targets, workspace_dir) {
        warn!("Failed to update .gitignore: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // empty cache → warn and return Ok without touching files
    #[test]
    fn undeploy_item_warns_when_no_cache_entries() {
        let mut cache = CacheConfig::default();
        let tmp = tempfile::TempDir::new().unwrap();
        let result = undeploy_item("skills", "my-skill", &mut cache, tmp.path());
        assert!(result.is_ok());
    }

    // file exists and cache entry exists → file deleted, cache entry removed
    #[test]
    fn undeploy_item_deletes_deployed_file_and_clears_cache() {
        use crate::core::config::CacheEntry;
        let mut cache = CacheConfig::default();
        let tmp = tempfile::TempDir::new().unwrap();
        let file = tmp.path().join("deployed.md");
        std::fs::write(&file, "content").unwrap();
        let file_str = file.to_str().unwrap().to_string();
        cache.set(
            "mycode",
            "commands",
            "hello",
            CacheEntry {
                hash: "abc".to_string(),
                target: file_str,
            },
        );
        undeploy_item("commands", "hello", &mut cache, tmp.path()).unwrap();
        assert!(!file.exists(), "deployed file should be deleted");
        assert!(cache.get("mycode", "commands", "hello").is_none());
    }

    // file is missing but cache entry exists → Ok, cache entry removed, no panic
    #[test]
    fn undeploy_item_continues_when_file_already_deleted() {
        use crate::core::config::CacheEntry;
        let mut cache = CacheConfig::default();
        let tmp = tempfile::TempDir::new().unwrap();
        let file_str = tmp
            .path()
            .join("nonexistent.md")
            .to_str()
            .unwrap()
            .to_string();
        cache.set(
            "mycode",
            "commands",
            "hello",
            CacheEntry {
                hash: "abc".to_string(),
                target: file_str,
            },
        );
        let result = undeploy_item("commands", "hello", &mut cache, tmp.path());
        assert!(result.is_ok(), "should not fail when file is missing");
        assert!(cache.get("mycode", "commands", "hello").is_none());
    }
}
