use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;
use strum_macros::{AsRefStr, EnumString};

#[cfg(feature = "skills-add")]
use crate::core::config::common::PackageRunner;

#[derive(Parser, Default)]
#[clap(author, version, about, long_about=None)]
pub(crate) struct Options {
    /// Verbosity level — specify up to 2 times (-v for debug, -vv for trace).
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    pub verbosity: u8,

    /// Quiet - only print errors
    #[clap(short, long, value_parser, global = true)]
    pub quiet: bool,

    /// Force non-interactive CI mode; suppresses all prompts (equivalent to DOTAGENTS_CI=true)
    #[clap(long, global = true)]
    pub ci: bool,

    #[clap(subcommand)]
    pub action: Option<Action>,
}

#[derive(Subcommand)]
pub(crate) enum Action {
    /// Initialize .dotagents directory with the given template.
    Init(InitOptions),

    /// Generate completions for the given shell
    GenCompletions {
        /// Set the shell for generating completions
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

    /// List providers from the registry
    Providers(ProvidersLsOptions),

    /// Remove all files deployed by the last `dotagents deploy` run
    Undeploy(UndeployOptions),

    /// Inspect or edit the workspace configuration
    Config(ConfigOptions),
}

/// Features that can be scaffolded by `dotagents init`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, EnumString, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Feature {
    /// Enable custom slash commands.
    Command,
    /// Enable global instruction.
    Instruction,
    /// Enable MCP configurations.
    Mcp,
    /// Enable skills.
    Skill,
    /// Enable Agent Ignore Files.
    AgentIgnore,
}

/// Subcommands for `dotagents skills`.
#[derive(Subcommand)]
pub(crate) enum SkillsAction {
    /// Install a skill from skills.sh or a GitHub owner/repo into .dotagents/skills/
    #[cfg(feature = "skills-add")]
    Add(SkillsAddOptions),
    /// Create a new skill scaffold
    New(AddSkillOptions),
    /// Remove a skill
    Rm(RmSkillOptions),
    /// List skills
    Ls(SubLsOptions),
}

/// Subcommands for `dotagents commands`.
#[derive(Subcommand)]
pub(crate) enum CommandsAction {
    /// Create a new command
    New(AddCommandOptions),
    /// Remove a command
    Rm(RmCommandOptions),
    /// List commands
    Ls(SubLsOptions),
}

/// Options for `dotagents providers`.
#[derive(Args, Default)]
pub(crate) struct ProvidersLsOptions {
    /// Output as JSON array instead of text.
    #[clap(long)]
    pub json: bool,
}

/// Shared workspace path argument for `--cwd`.
#[derive(Args, Default)]
pub(crate) struct WorkspaceDirArgs {
    /// Workspace root directory containing `.dotagents/`.
    #[clap(long = "cwd", value_name = "PATH")]
    pub cwd: Option<PathBuf>,
}

/// Options shared by `commands ls` and `skills ls`.
#[derive(Args, Default)]
pub(crate) struct SubLsOptions {
    #[clap(flatten)]
    pub workspace: WorkspaceDirArgs,
    /// Show descriptions and content of each item.
    #[clap(long = "content")]
    pub content: bool,

    /// Output as JSON array.
    #[clap(long = "json")]
    pub json: bool,

    /// Filter by command name (for `commands ls`).
    #[clap(long = "command")]
    pub command: Option<String>,

    /// Filter by skill name (for `skills ls`).
    #[clap(long = "skill")]
    pub skill: Option<String>,
}

#[cfg(feature = "skills-add")]
#[derive(Args)]
pub(crate) struct SkillsAddOptions {
    #[clap(flatten)]
    pub workspace: WorkspaceDirArgs,

    /// Skill name or owner/repo to install (e.g. vercel-labs/agent-skills)
    pub name: String,

    /// Package runner to use for this invocation [npm, pnpm, yarn, bun]
    #[clap(long, short)]
    pub runner: Option<PackageRunner>,
}

#[derive(Args, Default)]
pub(crate) struct DeployOptions {
    /// Workspace root directory containing `.dotagents/`.
    #[clap(value_name = "PATH")]
    pub dir: Option<PathBuf>,

    /// Force overwrite all target files regardless of cache state.
    #[clap(long, short)]
    pub force: bool,

    /// re-render without hash lookup and write all target files.
    #[clap(long)]
    pub no_cache: bool,

    /// Always update .gitignore without prompting.
    #[clap(long, conflicts_with = "no_gitignore")]
    pub gitignore: bool,

    /// Never update .gitignore.
    #[clap(long)]
    pub no_gitignore: bool,

    /// resolve templates only from the local cache.
    #[clap(long)]
    pub offline: bool,

    /// Preview what would be deployed without writing any files.
    #[clap(long)]
    pub dry_run: bool,

    /// Custom `.env` file(s) to load; later files override earlier ones when duplicate keys exist.
    #[clap(long)]
    pub env: Vec<std::path::PathBuf>,
}

/// Scaffolding template to use when running `dotagents init`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum InitTemplate {
    /// Minimal scaffolding.
    Blank,
    /// Variables, env & rendering.
    Starter,
    /// Custom provider & overrides.
    Advanced,
}

#[derive(Args)]
pub(crate) struct InitOptions {
    /// Workspace root directory to initialise (default: current directory).
    #[clap(value_name = "PATH")]
    pub dir: Option<PathBuf>,

    /// Features to scaffold.
    #[clap(long, value_delimiter = ',')]
    pub features: Option<Vec<Feature>>,

    /// Force overwriting existing configuration.
    #[clap(long, short, default_value_t = cfg!(debug_assertions))]
    pub force: bool,

    /// Template to scaffold from.
    #[clap(long, value_enum)]
    pub template: Option<InitTemplate>,

    /// Provider targets to deploy to.
    #[clap(long, value_delimiter = ',')]
    pub targets: Option<Vec<String>>,
}

impl InitOptions {
    /// Returns true if the given feature is enabled based on the `--features` flag.
    pub(crate) fn has_feature(&self, feature: Feature) -> bool {
        match &self.features {
            None => false,
            Some(list) => list.contains(&feature),
        }
    }
}

#[derive(Args)]
pub(crate) struct AddCommandOptions {
    #[clap(flatten)]
    pub workspace: WorkspaceDirArgs,

    /// Name of the command.
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

    /// Deploy automatically after creating the command.
    #[clap(long, conflicts_with = "no_deploy")]
    pub deploy: bool,

    /// Skip automatic deploy after creating the command.
    #[clap(long, default_value_t = false)]
    pub no_deploy: bool,
}

#[derive(Args)]
pub(crate) struct AddSkillOptions {
    #[clap(flatten)]
    pub workspace: WorkspaceDirArgs,

    /// Name of the skill.
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

    /// Deploy automatically after creating the skill.
    #[clap(long, conflicts_with = "no_deploy")]
    pub deploy: bool,

    /// Skip automatic deploy after creating the skill.
    #[clap(long, default_value_t = false)]
    pub no_deploy: bool,
}

#[derive(Args)]
pub(crate) struct RmCommandOptions {
    #[clap(flatten)]
    pub workspace: WorkspaceDirArgs,

    /// Name of the command to remove.
    pub name: String,

    /// Skip the confirmation prompt.
    #[clap(long, short = 'f')]
    pub force: bool,

    /// Deploy automatically after removing the command.
    #[clap(long, conflicts_with = "no_deploy")]
    pub deploy: bool,

    /// Skip automatic deploy after removing the command.
    #[clap(long, default_value_t = false)]
    pub no_deploy: bool,
}

#[derive(Args)]
pub(crate) struct RmSkillOptions {
    #[clap(flatten)]
    pub workspace: WorkspaceDirArgs,

    /// Name of the skill to remove.
    pub name: String,

    /// Skip the confirmation prompt.
    #[clap(long, short = 'f')]
    pub force: bool,

    /// Deploy automatically after removing the skill.
    #[clap(long, conflicts_with = "no_deploy")]
    pub deploy: bool,

    /// Skip automatic deploy after removing the skill.
    #[clap(long, default_value_t = false)]
    pub no_deploy: bool,
}

/// Options for `dotagents undeploy`.
#[derive(Args, Default)]
pub(crate) struct UndeployOptions {
    /// Workspace root directory containing `.dotagents/`.
    #[clap(value_name = "PATH")]
    pub dir: Option<PathBuf>,

    /// Skip confirmation prompt and delete user-edited files without asking.
    #[clap(long, short)]
    pub force: bool,

    /// Preview what would be undeployed without deleting any files, clearing cache, or touching .gitignore.
    #[clap(long)]
    pub dry_run: bool,
}

/// Target config layer to inspect/edit.
#[derive(Clone, Debug, Default, PartialEq, Eq, ValueEnum)]
pub(crate) enum ConfigTarget {
    /// runtime config (default).
    #[default]
    App,
    /// config.toml.
    Global,
    /// local.config.toml.
    Local,
}

/// Options for `dotagents config`.
#[derive(Args, Default)]
pub(crate) struct ConfigOptions {
    #[clap(flatten)]
    pub workspace: WorkspaceDirArgs,

    /// Config target
    #[clap(default_value = "app", value_enum)]
    pub target: ConfigTarget,

    /// Output as JSON
    #[clap(long, conflicts_with = "edit")]
    pub json: bool,

    /// Edit config interactively (only for global/local targets)
    #[clap(long)]
    pub edit: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_clamping() {
        // Test that verbosity is clamped to max 2
        let verbosity_values = [0, 1, 2, 3, 4, 5, 10];
        for v in verbosity_values {
            let clamped = std::cmp::min(2, v);
            if v <= 2 {
                assert_eq!(clamped, v);
            } else {
                assert_eq!(clamped, 2);
            }
        }
    }

    #[test]
    fn test_init_options_defaults() {
        let init_options = InitOptions {
            dir: None,
            features: None,
            force: false,
            template: None,
            targets: None,
        };

        assert!(init_options.features.is_none());
        assert!(init_options.template.is_none());
        assert!(init_options.targets.is_none());
    }

    #[test]
    fn test_init_template_variants() {
        // All three template variants are distinct and correct
        assert_eq!(InitTemplate::Blank, InitTemplate::Blank);
        assert_eq!(InitTemplate::Starter, InitTemplate::Starter);
        assert_eq!(InitTemplate::Advanced, InitTemplate::Advanced);
        assert_ne!(InitTemplate::Blank, InitTemplate::Starter);
        assert_ne!(InitTemplate::Starter, InitTemplate::Advanced);
        assert_ne!(InitTemplate::Blank, InitTemplate::Advanced);
    }

    #[test]
    fn test_options_default() {
        let options = Options::default();
        assert_eq!(options.verbosity, 0);
        assert!(!options.quiet);
        assert!(options.action.is_none());
    }

    #[test]
    fn has_feature_returns_false_when_features_absent() {
        let opts = InitOptions {
            dir: None,
            features: None,
            force: false,
            template: None,
            targets: None,
        };
        assert!(!opts.has_feature(Feature::Command));
        assert!(!opts.has_feature(Feature::Instruction));
        assert!(!opts.has_feature(Feature::Mcp));
        assert!(!opts.has_feature(Feature::Skill));
        assert!(!opts.has_feature(Feature::AgentIgnore));
    }

    #[test]
    fn has_feature_returns_false_for_unlisted_feature() {
        let opts = InitOptions {
            dir: None,
            features: Some(vec![Feature::Command]),
            force: false,
            template: None,
            targets: None,
        };
        assert!(opts.has_feature(Feature::Command));
        assert!(!opts.has_feature(Feature::Mcp));
        assert!(!opts.has_feature(Feature::Instruction));
        assert!(!opts.has_feature(Feature::Skill));
        assert!(!opts.has_feature(Feature::AgentIgnore));
    }

    #[test]
    fn has_feature_returns_true_for_all_listed() {
        let opts = InitOptions {
            dir: None,
            features: Some(vec![
                Feature::Command,
                Feature::Instruction,
                Feature::Mcp,
                Feature::Skill,
                Feature::AgentIgnore,
            ]),
            force: false,
            template: None,
            targets: None,
        };
        assert!(opts.has_feature(Feature::Command));
        assert!(opts.has_feature(Feature::Instruction));
        assert!(opts.has_feature(Feature::Mcp));
        assert!(opts.has_feature(Feature::Skill));
        assert!(opts.has_feature(Feature::AgentIgnore));
    }

    #[test]
    fn config_target_default_is_app() {
        // ConfigTarget::default() is App
        assert_eq!(ConfigTarget::default(), ConfigTarget::App);
    }

    #[test]
    fn config_target_value_enum_variants() {
        // ConfigTarget parses from string variants
        assert_eq!(
            ConfigTarget::from_str("app", true).unwrap(),
            ConfigTarget::App
        );
        assert_eq!(
            ConfigTarget::from_str("global", true).unwrap(),
            ConfigTarget::Global
        );
        assert_eq!(
            ConfigTarget::from_str("local", true).unwrap(),
            ConfigTarget::Local
        );
    }

    #[test]
    fn config_options_default() {
        // ConfigOptions::default() has app target, json=false, edit=false
        let opts = ConfigOptions::default();
        assert_eq!(opts.target, ConfigTarget::App);
        assert!(!opts.json);
        assert!(!opts.edit);
    }
}
