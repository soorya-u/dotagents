use std::collections::HashSet;
use std::io::Write;
use std::sync::{Arc, Mutex};

use cliclack::{outro, select, spinner};

use crate::cli::deploy::DeployStats;
use crate::cli::options::DeployOptions;
use crate::core::config::CacheConfig;
use crate::prelude::*;
use crate::schema::registry::Registry;
use crate::templates::registry_url;
use crate::utils::gitignore::{collapse_paths, rebuild_fence_from_cache};
use crate::utils::path::{get_workspace_dir, make_workspace_relative};
use crate::utils::tui::is_tui_enabled;

/// Prints deploy completion summary to stdout; in TTY uses `"✓ "` prefix, in non-TTY uses plain text.
pub(crate) fn print_deploy_summary(stats: &DeployStats) {
    let stdout = std::io::stdout();
    write_summary(stats, &mut stdout.lock(), is_tui_enabled());
}

fn write_summary<W: std::io::Write>(stats: &DeployStats, writer: &mut W, tty: bool) {
    let _ = if stats.written == 0 && stats.skipped == 0 {
        if tty {
            writeln!(writer, "✓ Nothing deployed")
        } else {
            writeln!(writer, "Nothing deployed")
        }
    } else if tty {
        if stats.user_edited > 0 {
            writeln!(
                writer,
                "✓ {} written, {} skipped, {} edited",
                stats.written, stats.skipped, stats.user_edited
            )
        } else {
            writeln!(
                writer,
                "✓ {} written, {} skipped",
                stats.written, stats.skipped
            )
        }
    } else if stats.user_edited > 0 {
        writeln!(
            writer,
            "{} written, {} skipped, {} edited",
            stats.written, stats.skipped, stats.user_edited
        )
    } else {
        writeln!(
            writer,
            "{} written, {} skipped",
            stats.written, stats.skipped
        )
    };
}

/// Prompts whether to add deployed paths to .gitignore using a cliclack select.
/// Returns false (skip) immediately in non-TTY environments or on error.
pub(crate) fn prompt_gitignore_update(new_path_count: usize) -> bool {
    if !is_tui_enabled() {
        return false;
    }
    let msg = format!("Add {} deployed path(s) to .gitignore?", new_path_count);
    let mut sel = select(msg)
        .item(false, "No", "")
        .item(true, "Yes", "")
        .initial_value(false);
    sel.interact().unwrap_or(false)
}

/// Formats deploy summary as a string for use with `cliclack::outro`.
pub(crate) fn deploy_outro(stats: &DeployStats) -> String {
    if stats.written == 0 && stats.skipped == 0 {
        "Nothing deployed".to_string()
    } else if stats.user_edited > 0 {
        format!(
            "{} written, {} skipped, {} edited",
            stats.written, stats.skipped, stats.user_edited
        )
    } else {
        format!("{} written, {} skipped", stats.written, stats.skipped)
    }
}

/// Prints the deploy summary using TUI or plain output depending on context.
pub(crate) fn print_summary(stats: &DeployStats) {
    if is_tui_enabled() {
        outro(deploy_outro(stats)).ok();
        let _ = std::io::stdout().flush();
    } else {
        print_deploy_summary(stats);
    }
}

/// Fetches the provider registry, showing a spinner in TUI mode. Returns None on failure or when offline.
pub(crate) fn fetch_registry(offline: bool) -> Option<Registry> {
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
pub(crate) fn finalize_deploy(
    opts: &DeployOptions,
    stats: &DeployStats,
    cache: &Arc<Mutex<CacheConfig>>,
) -> Result<()> {
    if opts.no_gitignore {
        print_summary(stats);
        return check_user_edited(stats, opts.force);
    }

    let workspace_root = get_workspace_dir().context("unable to get workspace directory")?;
    let mut cache_targets = cache.lock().unwrap().all_targets();
    cache_targets.extend(stats.linked_targets.clone());

    if cache_targets.is_empty() {
        print_summary(stats);
        return check_user_edited(stats, opts.force);
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
    check_user_edited(stats, opts.force)
}

fn check_user_edited(stats: &DeployStats, force: bool) -> Result<()> {
    if stats.user_edited > 0 && !is_tui_enabled() && !force {
        return Err(anyhow::anyhow!(
            "{} file(s) were manually edited. Use --force to override.",
            stats.user_edited
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // non-TTY summary with written and skipped outputs counts as plain text
    #[test]
    fn non_tty_summary_with_written_and_skipped() {
        let stats = DeployStats {
            written: 2,
            skipped: 1,
            user_edited: 0,
            dry_run_entries: vec![],
            linked_targets: vec![],
            hooks_written: 0,
        };
        let mut buf = Vec::new();
        write_summary(&stats, &mut buf, false);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("2"));
        assert!(output.contains("1"));
        assert!(!output.contains("✓"));
    }

    // non-TTY summary with zero written and skipped prints "Nothing deployed"
    #[test]
    fn non_tty_summary_with_nothing_deployed() {
        let stats = DeployStats {
            written: 0,
            skipped: 0,
            user_edited: 0,
            dry_run_entries: vec![],
            linked_targets: vec![],
            hooks_written: 0,
        };
        let mut buf = Vec::new();
        write_summary(&stats, &mut buf, false);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Nothing deployed"));
        assert!(!output.contains("✓"));
    }

    // non-TTY summary includes edited count when user_edited > 0
    #[test]
    fn non_tty_summary_with_user_edited() {
        let stats = DeployStats {
            written: 3,
            skipped: 2,
            user_edited: 1,
            dry_run_entries: vec![],
            linked_targets: vec![],
            hooks_written: 0,
        };
        let mut buf = Vec::new();
        write_summary(&stats, &mut buf, false);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("3 written"));
        assert!(output.contains("2 skipped"));
        assert!(output.contains("1 edited"));
        assert!(!output.contains("✓"));
    }

    // TTY summary includes edited count when user_edited > 0
    #[test]
    fn tty_summary_with_user_edited() {
        let stats = DeployStats {
            written: 2,
            skipped: 1,
            user_edited: 3,
            dry_run_entries: vec![],
            linked_targets: vec![],
            hooks_written: 0,
        };
        let mut buf = Vec::new();
        write_summary(&stats, &mut buf, true);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("✓"));
        assert!(output.contains("2 written"));
        assert!(output.contains("1 skipped"));
        assert!(output.contains("3 edited"));
    }
}
