use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

static CI_MODE: AtomicBool = AtomicBool::new(false);

/// Sets CI mode for the process; call once at startup before any TUI check.
pub(crate) fn set_ci_mode(enabled: bool) {
    CI_MODE.store(enabled, Ordering::Relaxed);
}

/// Returns true when both stdin and stdout are interactive terminals.
pub(crate) fn is_tty() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Returns true when TUI should be shown: terminal is interactive and CI mode is not active.
pub(crate) fn is_tui_enabled() -> bool {
    if CI_MODE.load(Ordering::Relaxed) {
        return false;
    }
    is_tty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_tty_returns_bool_without_panicking() {
        // Value depends on environment; just ensure no panic.
        let _ = is_tty();
    }

    #[test]
    fn is_tui_enabled_returns_false_when_ci_mode_set() {
        // CI mode overrides terminal detection.
        set_ci_mode(true);
        assert!(!is_tui_enabled());
        set_ci_mode(false);
    }
}
