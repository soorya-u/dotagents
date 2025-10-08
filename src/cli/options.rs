use clap::{Args, Parser, Subcommand};
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
pub(super) enum Action {
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
}

#[derive(Args)]
pub(crate) struct InitOptions {
    #[clap(long)]
    pub no_mcp: bool,

    #[clap(long)]
    pub no_command: bool,

    #[clap(long)]
    pub no_instruction: bool,
}

pub fn get_options() -> Options {
    let mut opt = Options::parse();

    opt.verbosity = std::cmp::min(3, opt.verbosity);

    opt
}
