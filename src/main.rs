mod cli;
mod config;
mod constants;
mod core;
mod schema;
mod templates;
mod utils;

fn main() {
    let opts = cli::get_options();
    utils::set_log_config(opts.quiet, opts.verbosity);

    match cli::run(opts) {
        Ok(success) if success => std::process::exit(0),
        Ok(_) => std::process::exit(1),
        Err(e) => {
            utils::display_error(e);
            std::process::exit(1);
        }
    }
}
