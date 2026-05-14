use std::path::Path;

use cliclack::select;

use crate::utils::tui::is_tui_enabled;

/// Prompts the user to confirm deletion of `count` deployed files.
/// Returns false immediately in non-TTY environments.
pub(crate) fn prompt_confirm_undeploy(count: usize) -> bool {
    if !is_tui_enabled() {
        return true; // non-TTY: proceed without prompting
    }
    let msg = format!("Remove {} deployed file(s)?", count);
    let mut sel = select(msg)
        .item(false, "No", "")
        .item(true, "Yes", "")
        .initial_value(false);
    sel.interact().unwrap_or(false)
}

/// Prompts the user to confirm deletion of a single user-edited file.
/// Returns false immediately in non-TTY environments.
pub(crate) fn prompt_delete_edited(path: &Path) -> bool {
    if !is_tui_enabled() {
        return false; // non-TTY: skip without prompting
    }
    let msg = format!("{} was manually edited. Delete it anyway?", path.display());
    let mut sel = select(msg)
        .item(false, "No, keep it", "")
        .item(true, "Yes, delete it", "");
    sel.interact().unwrap_or(false)
}

/// Prints undeploy completion summary to stdout; no-op in non-TTY environments.
pub(crate) fn print_undeploy_summary(removed: usize, skipped: usize) {
    if !is_tui_enabled() {
        return;
    }
    if skipped > 0 {
        println!("✓ {} file(s) removed ({} skipped)", removed, skipped);
    } else {
        println!("✓ {} file(s) removed", removed);
    }
}
