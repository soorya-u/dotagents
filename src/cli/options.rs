use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser, Default)]
#[clap(author, version, about, long_about=None)]
pub(crate) struct Options {
    /// Verbosity level - specify up to 3 times to get more detailed output.
    /// Specifying at least once prints the differences between what was before and after Dotter's run
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    pub verbosity: u8,

    /// Quiet - only print errors
    #[clap(short, long, value_parser, global = true)]
    pub quiet: bool,

    #[clap(subcommand)]
    pub action: Option<Action>,
}

#[derive(Subcommand)]
pub(crate) enum Action {
    /// Initialize .agents directory with a single package containing all the files in the current
    /// directory creating a mock templates for commands, instructions and mcp configuration.
    Init(InitOptions),

    /// Generate completions for the given shell
    GenCompletions {
        /// Set the shell for generating completions [values: bash, elvish, fish, powerShell, zsh]
        #[clap(long, short)]
        shell: Shell,

        /// Set the out directory for writing completions file
        #[clap(long)]
        to: Option<PathBuf>,
    },

    /// Deploy templates
    Deploy(DeployOptions),
}

#[derive(Args, Default)]
pub(crate) struct DeployOptions {
    /// Force overwrite all target files regardless of cache state.
    #[clap(long, short)]
    pub force: bool,

    /// Bypass cache entirely; do not read or update cache.toml.
    #[clap(long)]
    pub no_cache: bool,

    /// Always update .gitignore without prompting.
    #[clap(long)]
    pub gitignore: bool,

    /// Never update .gitignore.
    #[clap(long)]
    pub no_gitignore: bool,

    /// Skip the remote registry fetch; resolve missing templates from the local cache only.
    /// Errors if a required template has not been cached by a previous online deploy.
    #[clap(long)]
    pub offline: bool,
}

/// Scaffolding template to use when running `dotagents init`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum InitTemplate {
    /// Core files only — no example custom-provider templates.
    Starter,
    /// Core files plus a mycode example provider (templates/ dir + local.config.toml provider block).
    WithCustomProvider,
}

#[derive(Args)]
pub(crate) struct InitOptions {
    /// Disables the MCP Templating for all the Targets.
    /// You can override this later in config.toml file.
    #[clap(long)]
    pub no_mcp: bool,

    /// Disables the Command Templating for all the Targets.
    /// You can override this later in config.toml file.
    #[clap(long)]
    pub no_command: bool,

    /// Disables the Instruction Templating for all the Targets.
    /// You can override this later in config.toml file.
    #[clap(long)]
    pub no_instruction: bool,

    /// Disables the Skill Templating for all the Targets.
    /// You can override this later in config.toml file.
    #[clap(long)]
    pub no_skill: bool,

    /// Force overwriting existing configuration.
    #[clap(long, short, default_value_t = cfg!(debug_assertions))]
    pub force: bool,

    /// Scaffolding template: 'starter' (core files only) or 'with-custom-provider' (adds mycode example).
    /// When omitted in an interactive terminal, the wizard will prompt for a choice.
    #[clap(long, value_enum)]
    pub template: Option<InitTemplate>,
}

pub fn get_options() -> Options {
    let mut opt = Options::parse();

    opt.verbosity = std::cmp::min(3, opt.verbosity);

    opt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_clamping() {
        // Test that verbosity is clamped to max 3
        let verbosity_values = [0, 1, 2, 3, 4, 5, 10];
        for v in verbosity_values {
            let clamped = std::cmp::min(3, v);
            if v <= 3 {
                assert_eq!(clamped, v);
            } else {
                assert_eq!(clamped, 3);
            }
        }
    }

    #[test]
    fn test_init_options_defaults() {
        // Test default values for InitOptions — template defaults to None (wizard mode)
        let init_options = InitOptions {
            no_mcp: false,
            no_command: false,
            no_instruction: false,
            no_skill: false,
            force: false,
            template: None,
        };

        assert!(!init_options.no_mcp);
        assert!(!init_options.no_command);
        assert!(!init_options.no_instruction);
        assert!(!init_options.no_skill);
        assert!(init_options.template.is_none());
    }

    #[test]
    fn test_init_template_variants() {
        // Both template variants are distinct and correct
        assert_eq!(InitTemplate::Starter, InitTemplate::Starter);
        assert_eq!(
            InitTemplate::WithCustomProvider,
            InitTemplate::WithCustomProvider
        );
        assert_ne!(InitTemplate::Starter, InitTemplate::WithCustomProvider);
    }

    #[test]
    fn test_options_default() {
        let options = Options::default();
        assert_eq!(options.verbosity, 0);
        assert!(!options.quiet);
        assert!(options.action.is_none());
    }
}
