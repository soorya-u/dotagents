mod cli;
mod constants;
mod core;
mod prelude;
mod schema;
mod templates;
pub(crate) mod utils;

use std::io::Write;

fn main() {
    let opts = cli::get_options();
    let ci_from_env = std::env::var("DOTAGENTS_CI")
        .ok()
        .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes"))
        .unwrap_or(false);
    utils::tui::set_ci_mode(opts.ci || ci_from_env);
    utils::set_log_config(opts.quiet, opts.verbosity);

    match cli::run(opts) {
        Ok(success) if success => std::process::exit(0),
        Ok(_) => std::process::exit(1),
        Err(e) => {
            utils::display_error(e);
            if utils::logs::log_config().is_some_and(|c| c.is_tty) {
                cliclack::outro_cancel("Fatal error").ok();
                let _ = std::io::stdout().flush();
            }
            std::process::exit(1);
        }
    }
}
