use super::commands;
use super::completions::generate_cli_completions;
use super::deploy::deploy;
use super::init::initialize_agents_dir;
use super::options::{Action, Options};
use super::skills;
use anyhow::Result;
use clap::CommandFactory;

pub(crate) fn run(opts: Options) -> Result<bool> {
    let default_action = || {
        Options::command().print_help().unwrap();
        std::process::exit(0);
    };

    let success = match opts.action.unwrap_or_else(default_action) {
        Action::Init(opts) => {
            initialize_agents_dir(opts)?;
            true
        }
        Action::GenCompletions { shell, to } => {
            generate_cli_completions(shell, to)?;
            true
        }
        Action::Deploy(opts) => {
            deploy(opts)?;
            true
        }
        Action::Skills { action } => skills::run_skills(action)?,
        Action::Commands { action } => commands::run_commands(action)?,
    };

    Ok(success)
}
