use anyhow::Error;
use log;
use std::fmt::Write;

pub(crate) fn display_error(error: Error) {
    let mut chain = error.chain();
    let mut error_message = format!("Failed to {}\nCaused by:\n", chain.next().unwrap());

    for e in chain {
        writeln!(error_message, "    {e}").unwrap();
    }

    error_message.pop();

    log::error!("{}", error_message);
}
