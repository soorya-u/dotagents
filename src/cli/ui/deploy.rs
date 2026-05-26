use cliclack::select;

use crate::cli::deploy::DeployStats;
use crate::utils::tui::is_tui_enabled;

/// Prompts whether to run deploy in offline mode using a cliclack select.
pub(crate) fn prompt_offline() -> bool {
    if !is_tui_enabled() {
        return false;
    }
    let mut sel = select("Run in offline mode?")
        .item(false, "No, fetch latest templates", "")
        .item(
            true,
            "Yes, use cached templates only",
            "skips registry fetch",
        );
    sel.interact().unwrap_or(false)
}

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
        writeln!(
            writer,
            "✓ {} written, {} skipped",
            stats.written, stats.skipped
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
    } else {
        format!("{} written, {} skipped", stats.written, stats.skipped)
    }
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
            dry_run_entries: vec![],
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
            dry_run_entries: vec![],
        };
        let mut buf = Vec::new();
        write_summary(&stats, &mut buf, false);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Nothing deployed"));
        assert!(!output.contains("✓"));
    }
}
