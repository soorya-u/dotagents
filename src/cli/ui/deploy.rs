use cliclack::select;

use crate::utils::tty::is_tty;

/// Prompts whether to run deploy in offline mode using a cliclack select.
/// Returns false (online) immediately in non-TTY environments or on error.
pub(crate) fn prompt_offline() -> bool {
    if !is_tty() {
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

/// Prompts whether to add deployed paths to .gitignore using a cliclack select.
/// Returns false (skip) immediately in non-TTY environments or on error.
pub(crate) fn prompt_gitignore_update(new_path_count: usize) -> bool {
    if !is_tty() {
        return false;
    }
    let msg = format!("Add {} deployed path(s) to .gitignore?", new_path_count);
    let mut sel = select(msg)
        .item(false, "No", "")
        .item(true, "Yes", "")
        .initial_value(false);
    sel.interact().unwrap_or(false)
}
