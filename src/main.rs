mod cli;
mod constants;
mod core;
mod prelude;
mod schema;
mod templates;
pub(crate) mod utils;

use clap::Parser;
use std::io::Write;
use utils::{error, logs, tui};

fn main() {
    let opts = cli::Options::parse();
    tui::set_ci_config(opts.ci);
    logs::set_log_config(opts.quiet, opts.verbosity);

    match cli::run(opts) {
        Ok(success) if success => std::process::exit(0),
        Ok(_) => std::process::exit(1),
        Err(e) => {
            error::display_error(e);
            if utils::logs::log_config().is_some_and(|c| c.is_tty) {
                cliclack::outro_cancel("Fatal error").ok();
                let _ = std::io::stdout().flush();
            }
            std::process::exit(1);
        }
    }
}
