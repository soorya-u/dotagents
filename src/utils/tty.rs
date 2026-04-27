use std::io::IsTerminal;

/// Returns true when both stdin and stdout are interactive terminals.
pub(crate) fn is_tty() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_tty_returns_bool_without_panicking() {
        // Value depends on environment; just ensure no panic.
        let _ = is_tty();
    }
}
