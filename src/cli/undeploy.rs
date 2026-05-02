use std::path::PathBuf;

use crate::prelude::*;

use crate::cli::options::UndeployOptions;
use crate::cli::ui::undeploy::{
    print_undeploy_summary, prompt_confirm_undeploy, prompt_delete_edited,
};
use crate::schema::config::CacheConfig;
use crate::utils::fs::{delete_file, hash_file, prune_empty_dir};
use crate::utils::gitignore::clear_gitignore_fence;
use crate::utils::path::get_workspace_dir;
use crate::utils::tty::is_tty;

pub(super) fn undeploy(opts: UndeployOptions) -> Result<()> {
    let workspace_root = get_workspace_dir().context("Failed to get workspace directory")?;

    let mut cache = CacheConfig::load().context("Failed to load cache")?;

    // Collect all target paths and their expected hashes from the cache.
    let entries: Vec<(PathBuf, String)> = cache
        .iter_entries()
        .map(|(_, _, _, entry)| (PathBuf::from(&entry.target), entry.hash.clone()))
        .collect();

    if entries.is_empty() {
        if is_tty() {
            println!("Nothing to undeploy.");
        }
        return Ok(());
    }

    // Interactive confirmation (skipped when --force or non-TTY).
    if !opts.force && !prompt_confirm_undeploy(entries.len()) {
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
        .context("Failed to save cache after undeploy")?;

    // Remove the dotagents-managed fence from .gitignore.
    if !opts.no_gitignore
        && let Err(e) = clear_gitignore_fence(&workspace_root)
    {
        warn!("Failed to update .gitignore: {}", e);
    }

    print_undeploy_summary(removed, skipped);

    Ok(())
}
