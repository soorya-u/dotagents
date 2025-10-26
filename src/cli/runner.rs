use super::completions::generate_cli_completions;
use super::deploy::deploy;
use super::init::initialize_agents_dir;
use super::options::{Action, Options};
use anyhow::Result;
use clap::CommandFactory;

pub(crate) fn run(opts: Options) -> Result<bool> {
    let default_action = || {
        Options::command().print_help().unwrap();
        std::process::exit(0);
    };

    match opts.action.unwrap_or_else(default_action) {
        Action::Init(opts) => initialize_agents_dir(opts),
        Action::GenCompletions { shell, to } => generate_cli_completions(shell, to),
        Action::Deploy => deploy(),
    }?;

    Ok(true)
}
