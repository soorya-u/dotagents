use anyhow::Error;
use std::fmt::Write;

use crate::prelude::*;

/// Formats an anyhow error chain into a display string.
fn format_error_chain(error: &Error) -> String {
    let mut chain = error.chain();
    let mut error_message = format!("Failed to {}\nCaused by:\n", chain.next().unwrap());

    for e in chain {
        writeln!(error_message, "    {e}").unwrap();
    }

    error_message.pop();
    error_message
}

pub(crate) fn display_error(error: Error) {
    error!("{}", format_error_chain(&error));
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_display_error_single_error() {
        let error = anyhow!("test error");
        // This test primarily checks that display_error doesn't panic
        display_error(error);
    }

    #[test]
    fn test_display_error_with_context() {
        let error = anyhow!("root cause")
            .context("intermediate error")
            .context("top level error");
        // This test primarily checks that display_error doesn't panic with chained errors
        display_error(error);
    }

    #[test]
    fn test_display_error_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = anyhow::Error::from(io_error).context("failed to read file");
        display_error(error);
    }

    // single-message error chain produces "Failed to <msg>" with no double prefix
    #[test]
    fn format_error_chain_single_message_no_double_prefix() {
        let error = anyhow!("complete 'skills add' command");
        let output = format_error_chain(&error);
        assert!(output.starts_with("Failed to complete 'skills add' command"));
        assert!(!output.contains("Failed to Failed to"));
    }

    // two-level error chain produces "Failed to <outer>" and "Caused by:\n    <inner>"
    #[test]
    fn format_error_chain_two_level_shows_outer_and_inner() {
        let error = anyhow!("No .dotagents directory found")
            .context("unable to resolve workspace directory")
            .context("complete 'skills add' command");
        let output = format_error_chain(&error);
        assert!(output.starts_with("Failed to complete 'skills add' command"));
        assert!(output.contains("Caused by:"));
        assert!(output.contains("unable to resolve workspace directory"));
        assert!(output.contains("No .dotagents directory found"));
        assert!(!output.contains("Failed to Failed to"));
    }
}
