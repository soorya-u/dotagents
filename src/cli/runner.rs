use super::add::run_add;
use super::completions::generate_cli_completions;
use super::deploy::deploy;
use super::init::initialize_agents_dir;
use super::ls::run_ls;
use super::options::{Action, Options, SkillsAction};
use super::rm::run_rm;
use super::skills;
use super::undeploy::undeploy;
use anyhow::Result;
use clap::CommandFactory;

pub(crate) fn run(opts: Options) -> Result<bool> {
    let verbosity = opts.verbosity;

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
        Action::Skills { action } => match action {
            SkillsAction::Add(opts) => skills::add(opts)?,
        },
        Action::Ls(mut opts) => {
            // -v / --verbose global flag also enables full descriptions.
            if verbosity > 0 {
                opts.verbose = true;
            }
            run_ls(opts)?
        }
        Action::Add { action } => run_add(action)?,
        Action::Rm { action } => run_rm(action)?,
        Action::Undeploy(opts) => {
            undeploy(opts)?;
            true
        }
    };

    Ok(success)
}
