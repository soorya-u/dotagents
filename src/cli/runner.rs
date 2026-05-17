use super::commands;
use super::completions::generate_cli_completions;
use super::config;
use super::deploy::deploy;
use super::init::initialize_agents_dir;
use super::options::{Action, CommandsAction, Options, SkillsAction};
use super::providers;
use super::skills;
use super::undeploy::undeploy;
use anyhow::{Context, Result};
use clap::CommandFactory;

pub(crate) fn run(opts: Options) -> Result<bool> {
    let global_quiet = opts.quiet;
    let default_action = || {
        Options::command().print_help().unwrap();
        std::process::exit(0);
    };

    let success = match opts.action.unwrap_or_else(default_action) {
        Action::Init(opts) => {
            initialize_agents_dir(opts).context("complete 'init' command")?;
            true
        }
        Action::GenCompletions { shell, to } => {
            generate_cli_completions(shell, to)?;
            true
        }
        Action::Deploy(opts) => {
            deploy(opts).context("complete 'deploy' command")?;
            true
        }
        Action::Skills { action } => match action {
            #[cfg(feature = "skills-add")]
            SkillsAction::Add(opts) => {
                skills::add(opts).context("complete 'skills add' command")?
            }
            SkillsAction::New(opts) => {
                skills::new_skill(opts).context("complete 'skills new' command")?
            }
            SkillsAction::Rm(opts) => {
                skills::rm_skill(opts).context("complete 'skills rm' command")?
            }
            SkillsAction::Ls(opts) => {
                skills::ls_skills(opts).context("complete 'skills ls' command")?
            }
        },
        Action::Commands { action } => match action {
            CommandsAction::New(opts) => {
                commands::new_command(opts).context("complete 'commands new' command")?
            }
            CommandsAction::Rm(opts) => {
                commands::rm_command(opts).context("complete 'commands rm' command")?
            }
            CommandsAction::Ls(opts) => {
                commands::ls_commands(opts).context("complete 'commands ls' command")?
            }
        },
        Action::Providers(opts) => {
            providers::run_providers(opts, global_quiet).context("complete 'providers' command")?
        }
        Action::Undeploy(opts) => {
            undeploy(opts).context("complete 'undeploy' command")?;
            true
        }
        Action::Config(opts) => config::handle(opts).context("complete 'config' command")?,
    };

    Ok(success)
}
