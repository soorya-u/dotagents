mod cli;
mod constants;
mod core;
mod prelude;
mod schema;
mod templates;
pub(crate) mod utils;

fn main() {
    let opts = cli::get_options();
    utils::set_log_config(opts.quiet, opts.verbosity);

    match cli::run(opts) {
        Ok(success) if success => std::process::exit(0),
        Ok(_) => std::process::exit(1),
        Err(e) => {
            utils::display_error(e);
            if utils::logs::log_config().is_some_and(|c| c.is_tty) {
                cliclack::outro_cancel("Fatal error").ok();
            }
            std::process::exit(1);
        }
    }
}
