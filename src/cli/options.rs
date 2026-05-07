use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

use crate::core::config::common::PackageRunner;

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
    /// Initialize .dotagents directory with a single package containing all the files in the current
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

    /// Manage skills
    Skills {
        #[clap(subcommand)]
        action: SkillsAction,
    },

    /// Manage commands
    Commands {
        #[clap(subcommand)]
        action: CommandsAction,
    },

    /// Remove all files deployed by the last `dotagents deploy` run
    Undeploy(UndeployOptions),
}

/// Features that can be scaffolded by `dotagents init`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum Feature {
    /// Enable command templating.
    Commands,
    /// Enable instruction templating.
    Instructions,
    /// Enable MCP templating.
    Mcp,
    /// Enable skill templating.
    Skills,
    /// Disable all features (exclusive — cannot be combined with other values).
    None,
}

/// Subcommands for `dotagents skills`.
#[derive(Subcommand)]
pub(crate) enum SkillsAction {
    /// Install a skill from skills.sh or a GitHub owner/repo into .dotagents/skills/
    Add(SkillsAddOptions),
    /// Create a new local skill scaffold in .dotagents/skills/
    New(AddSkillOptions),
    /// Remove a local skill from .dotagents/skills/
    Rm(RmSkillOptions),
    /// List local skills in .dotagents/skills/
    Ls(SubLsOptions),
}

/// Subcommands for `dotagents commands`.
#[derive(Subcommand)]
pub(crate) enum CommandsAction {
    /// Create a new command in .dotagents/commands/
    New(AddCommandOptions),
    /// Remove a command from .dotagents/commands/
    Rm(RmCommandOptions),
    /// List commands in .dotagents/commands/
    Ls(SubLsOptions),
}

/// Options shared by `commands ls` and `skills ls`.
#[derive(Args, Default)]
pub(crate) struct SubLsOptions {
    /// Show full descriptions (word-wrapped) instead of truncating.
    #[clap(long = "full")]
    pub full: bool,
}

#[derive(Args)]
pub(crate) struct SkillsAddOptions {
    /// Skill name or owner/repo to install (e.g. vercel-labs/agent-skills)
    pub name: String,

    /// Package runner to use for this invocation [npm, pnpm, yarn, bun]
    #[clap(long, short)]
    pub runner: Option<PackageRunner>,
}

#[derive(Args, Default)]
pub(crate) struct DeployOptions {
    /// Workspace root directory containing `.dotagents/` (default: inferred from current directory).
    #[clap(value_name = "PATH")]
    pub dir: Option<PathBuf>,

    /// Force overwrite all target files regardless of cache state.
    #[clap(long, short)]
    pub force: bool,

    /// Skip hash comparison; always re-render and write all target files.
    /// cache.toml is still written at the end of the run.
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

    /// Preview what would be deployed without writing any files, saving cache, or updating .gitignore.
    #[clap(long)]
    pub dry_run: bool,

    /// Custom .env file(s) to load instead of .dotagents/.env. Repeatable; later files override earlier ones on duplicate keys.
    #[clap(long)]
    pub env: Vec<std::path::PathBuf>,
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
    /// Workspace root directory to initialise (default: current directory). Created if it does not exist.
    #[clap(value_name = "PATH")]
    pub dir: Option<PathBuf>,

    /// Features to scaffold. Accepts comma-separated values and/or repeated flags.
    /// Valid values: commands, instructions, mcp, skills, none.
    /// When omitted, features are chosen interactively when possible; otherwise all features are enabled.
    /// Use `none` to disable all features.
    #[clap(long, value_delimiter = ',', num_args = 1..)]
    pub features: Option<Vec<Feature>>,

    /// Force overwriting existing configuration.
    #[clap(long, short, default_value_t = cfg!(debug_assertions))]
    pub force: bool,

    /// Scaffolding template: 'starter' (core files only) or 'with-custom-provider' (adds mycode example).
    /// When omitted in an interactive terminal, the wizard will prompt for a choice.
    #[clap(long, value_enum)]
    pub template: Option<InitTemplate>,

    /// Provider targets selected interactively by the wizard; not a CLI flag.
    #[clap(skip)]
    pub targets: Vec<String>,
}

impl InitOptions {
    /// Returns true if the given feature is enabled based on the `--features` flag.
    pub(crate) fn has_feature(&self, feature: Feature) -> bool {
        match &self.features {
            None => true, // all features enabled when flag is absent
            Some(list) => {
                if list.iter().any(|f| matches!(f, Feature::None)) {
                    false // none sentinel: all features disabled
                } else {
                    list.contains(&feature)
                }
            }
        }
    }
}

#[derive(Args)]
pub(crate) struct AddCommandOptions {
    /// Name of the command (used as the filename, e.g. "hello" → hello.md).
    pub name: String,

    /// Short description of the command.
    #[clap(long, short = 'd')]
    pub description: Option<String>,

    /// Category for the command.
    #[clap(long, short = 'c')]
    pub category: Option<String>,

    /// Comma-separated tags for the command.
    #[clap(long, short = 't')]
    pub tags: Option<String>,

    /// Overwrite if the file already exists.
    #[clap(long, short = 'f')]
    pub force: bool,

    /// Run deploy after creating the command.
    #[clap(long)]
    pub deploy: bool,
}

#[derive(Args)]
pub(crate) struct AddSkillOptions {
    /// Name of the skill (used as the directory name).
    pub name: String,

    /// Short description of the skill.
    #[clap(long, short = 'd')]
    pub description: Option<String>,

    /// License for the skill (e.g. MIT).
    #[clap(long, short = 'l')]
    pub license: Option<String>,

    /// Compatibility notes for the skill.
    #[clap(long)]
    pub compatibility: Option<String>,

    /// Overwrite if the skill already exists.
    #[clap(long, short = 'f')]
    pub force: bool,

    /// Run deploy after creating the skill.
    #[clap(long)]
    pub deploy: bool,
}

#[derive(Args)]
pub(crate) struct RmCommandOptions {
    /// Name of the command to remove.
    pub name: String,

    /// Skip the confirmation prompt.
    #[clap(long, short = 'f')]
    pub force: bool,

    /// Run deploy after removing the command.
    #[clap(long)]
    pub deploy: bool,
}

#[derive(Args)]
pub(crate) struct RmSkillOptions {
    /// Name of the skill to remove.
    pub name: String,

    /// Skip the confirmation prompt.
    #[clap(long, short = 'f')]
    pub force: bool,

    /// Run deploy after removing the skill.
    #[clap(long)]
    pub deploy: bool,
}

/// Options for `dotagents undeploy`.
#[derive(Args, Default)]
pub(crate) struct UndeployOptions {
    /// Workspace root directory containing `.dotagents/` (default: inferred from current directory).
    #[clap(value_name = "PATH")]
    pub dir: Option<PathBuf>,

    /// Skip confirmation prompt and delete user-edited files without asking.
    #[clap(long, short)]
    pub force: bool,

    /// Do not remove entries from .gitignore.
    #[clap(long)]
    pub no_gitignore: bool,

    /// Preview what would be undeployed without deleting any files, clearing cache, or touching .gitignore.
    #[clap(long)]
    pub dry_run: bool,
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
        // features defaults to None (all features enabled, TUI mode possible)
        let init_options = InitOptions {
            dir: None,
            features: None,
            force: false,
            template: None,
            targets: vec![],
        };

        assert!(init_options.features.is_none());
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

    #[test]
    fn has_feature_returns_true_when_features_absent() {
        // None (flag absent) enables all features
        let opts = InitOptions {
            dir: None,
            features: None,
            force: false,
            template: None,
            targets: vec![],
        };
        assert!(opts.has_feature(Feature::Commands));
        assert!(opts.has_feature(Feature::Instructions));
        assert!(opts.has_feature(Feature::Mcp));
        assert!(opts.has_feature(Feature::Skills));
    }

    #[test]
    fn has_feature_returns_false_for_unlisted_feature() {
        // Only Commands is listed → Mcp is disabled
        let opts = InitOptions {
            dir: None,
            features: Some(vec![Feature::Commands]),
            force: false,
            template: None,
            targets: vec![],
        };
        assert!(opts.has_feature(Feature::Commands));
        assert!(!opts.has_feature(Feature::Mcp));
        assert!(!opts.has_feature(Feature::Instructions));
        assert!(!opts.has_feature(Feature::Skills));
    }

    #[test]
    fn has_feature_returns_false_for_all_when_none_sentinel() {
        // Feature::None sentinel disables everything
        let opts = InitOptions {
            dir: None,
            features: Some(vec![Feature::None]),
            force: false,
            template: None,
            targets: vec![],
        };
        assert!(!opts.has_feature(Feature::Commands));
        assert!(!opts.has_feature(Feature::Instructions));
        assert!(!opts.has_feature(Feature::Mcp));
        assert!(!opts.has_feature(Feature::Skills));
    }
}
