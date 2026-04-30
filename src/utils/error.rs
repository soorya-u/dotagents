use anyhow::Error;
use std::fmt::Write;

use crate::prelude::*;

pub(crate) fn display_error(error: Error) {
    let mut chain = error.chain();
    let mut error_message = format!("Failed to {}\nCaused by:\n", chain.next().unwrap());

    for e in chain {
        writeln!(error_message, "    {e}").unwrap();
    }

    error_message.pop();

    error!("{}", error_message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, anyhow};

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
}
