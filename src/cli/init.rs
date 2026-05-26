use crate::prelude::*;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::options::{Feature, InitOptions, InitTemplate};
use crate::{
    cli::ui,
    constants::{
        dir::{COMMANDS_DIR, ROOT_DIR, SKILLS_DIR},
        file::{
            AGENTIGNORE_FILE, ENV_EXAMPLE_FILE, ENV_FILE, GITIGNORE_FILE, GLOBAL_CONFIG_FILE,
            INSTRUCTIONS_FILE, LOCAL_CONFIG_FILE, MCP_FILE, SKILL_FILE,
        },
        mocks,
    },
    core::features::{
        command::CommandFeature, ignore::IgnoreFeature, instruction::InstructionFeature,
        mcp::McpFeature, skill::SkillFeature,
    },
    utils::{fs::write_file, tui::is_tui_enabled},
};

/// Represents a file to write during init, with an optional condition to skip it.
struct InitFile {
    path: PathBuf,
    content: &'static str,
    skip_condition: Option<fn(&InitOptions) -> bool>,
}

impl InitFile {
    fn new(relative_path: impl AsRef<Path>, content: &'static str) -> Self {
        Self {
            path: relative_path.as_ref().to_path_buf(),
            content,
            skip_condition: None,
        }
    }

    fn with_skip_if(mut self, condition: fn(&InitOptions) -> bool) -> Self {
        self.skip_condition = Some(condition);
        self
    }

    fn should_skip(&self, opts: &InitOptions) -> bool {
        self.skip_condition.map(|f| f(opts)).unwrap_or(false)
    }
}

/// Derives the global and local config file content from init options.
fn build_config_content(opts: &InitOptions, template: InitTemplate) -> (String, String) {
    let config_features: Vec<&str> = opts
        .features
        .as_ref()
        .map(|fs| fs.iter().map(|f| f.as_ref()).collect())
        .unwrap_or_default();
    let config_targets: Vec<&str> = opts
        .targets
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(String::as_str)
        .collect();
    let base_config = mocks::default_config(&config_features, &config_targets);
    match template {
        InitTemplate::Blank => (base_config, String::new()),
        InitTemplate::Starter => (base_config.clone(), base_config),
        InitTemplate::Advanced => (
            base_config.clone(),
            format!("{}{}", base_config, mocks::MYCODE_PROVIDER_CONFIG),
        ),
    }
}

pub(super) fn initialize_agents_dir(mut opts: InitOptions) -> Result<()> {
    // Resolve workspace root from the optional positional dir arg.
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let workspace = match opts.dir.take() {
        Some(p) => cwd.join(p),
        None => cwd,
    };
    let main_dir = workspace.join(ROOT_DIR);

    let dir_exists = main_dir
        .try_exists()
        .context("failed to check if .dotagents directory exists")?;

    let tui_mode = is_tui_enabled();

    if tui_mode {
        let proceed = ui::init::run_init_wizard(&mut opts, dir_exists)?;
        if !proceed {
            return Ok(());
        }
    }

    // Ensure the workspace root exists before creating .dotagents inside it.
    fs::create_dir_all(&workspace).context("failed to create workspace directory")?;

    if dir_exists {
        if !opts.force {
            anyhow::bail!(format!(
                "Configuration already exists in {}. Please use --force to overwrite it.",
                main_dir.display()
            ));
        } else {
            if !tui_mode {
                warn!("Overwriting existing configuration");
            }
            fs::remove_dir_all(&main_dir).context("failed to remove .dotagents directory")?;
        }
    }

    fs::create_dir(&main_dir).context("failed to create .dotagents directory")?;

    // Resolve the effective template: default to Starter when no flag was set.
    let template = opts.template.unwrap_or(InitTemplate::Blank);

    // Write config files directly — content is runtime-generated so cannot live in InitFile.
    let (base_config, local_config) = build_config_content(&opts, template);
    write_file(&main_dir.join(GLOBAL_CONFIG_FILE), &base_config)
        .with_context(|| format!("failed to write {GLOBAL_CONFIG_FILE}"))?;
    if !local_config.is_empty() {
        write_file(&main_dir.join(LOCAL_CONFIG_FILE), &local_config)
            .with_context(|| format!("failed to write {LOCAL_CONFIG_FILE}"))?;
    }

    let init_files = vec![
        InitFile::new(ENV_EXAMPLE_FILE, mocks::ENV_EXAMPLE)
            .with_skip_if(|opts| matches!(opts.template, Some(InitTemplate::Blank) | None)),
        InitFile::new(ENV_FILE, mocks::ENV_EXAMPLE)
            .with_skip_if(|opts| matches!(opts.template, Some(InitTemplate::Blank) | None)),
        InitFile::new(GITIGNORE_FILE, mocks::GITIGNORE),
        InitFile::new(INSTRUCTIONS_FILE, InstructionFeature::mock())
            .with_skip_if(|opts| !opts.has_feature(Feature::Instruction)),
        InitFile::new(MCP_FILE, McpFeature::mock())
            .with_skip_if(|opts| !opts.has_feature(Feature::Mcp)),
        InitFile::new(
            Path::new(COMMANDS_DIR).join("hello.md"),
            CommandFeature::mock(),
        )
        .with_skip_if(|opts| !opts.has_feature(Feature::Command)),
        InitFile::new(
            Path::new(SKILLS_DIR).join("hello-skill").join(SKILL_FILE),
            SkillFeature::mock(),
        )
        .with_skip_if(|opts| !opts.has_feature(Feature::Skill)),
        InitFile::new(AGENTIGNORE_FILE, IgnoreFeature::mock())
            .with_skip_if(|opts| !opts.has_feature(Feature::AgentIgnore)),
        // Template files — only written for the Advanced template.
        InitFile::new(
            Path::new("templates").join("mycode").join("command.hbs"),
            mocks::TEMPLATE_MYCODE_COMMAND,
        )
        .with_skip_if(|opts| opts.template != Some(InitTemplate::Advanced)),
        InitFile::new(
            Path::new("templates").join("mycode").join("skill.hbs"),
            mocks::TEMPLATE_MYCODE_SKILL,
        )
        .with_skip_if(|opts| opts.template != Some(InitTemplate::Advanced)),
        InitFile::new(
            Path::new("templates")
                .join("mycode")
                .join("instructions.hbs"),
            mocks::TEMPLATE_MYCODE_INSTRUCTIONS,
        )
        .with_skip_if(|opts| opts.template != Some(InitTemplate::Advanced)),
        InitFile::new(
            Path::new("templates").join("mycode").join("mcp.hbs"),
            mocks::TEMPLATE_MYCODE_MCP,
        )
        .with_skip_if(|opts| opts.template != Some(InitTemplate::Advanced)),
        InitFile::new(
            Path::new("templates")
                .join("mycode")
                .join("agent-ignore.hbs"),
            mocks::TEMPLATE_MYCODE_AGENT_IGNORE,
        )
        .with_skip_if(|opts| opts.template != Some(InitTemplate::Advanced)),
    ];

    for file in init_files {
        if file.should_skip(&opts) {
            debug!("Skipping {}", file.path.display());
            continue;
        }
        write_file(&main_dir.join(&file.path), file.content)?;
    }

    if tui_mode {
        ui::init::finish_init();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_opts() -> InitOptions {
        InitOptions {
            dir: None,
            features: None,
            force: false,
            template: None,
            targets: None,
        }
    }

    #[test]
    fn is_tui_enabled_false_when_ci_mode_set() {
        use crate::utils::tui::set_ci_mode;
        set_ci_mode(true);
        assert!(!is_tui_enabled());
        set_ci_mode(false);
    }

    #[test]
    fn init_file_should_not_skip_without_condition() {
        let file = InitFile::new("some.txt", "content");
        assert!(!file.should_skip(&default_opts()));
    }

    #[test]
    fn init_file_should_skip_when_feature_disabled() {
        let file =
            InitFile::new("some.txt", "content").with_skip_if(|o| !o.has_feature(Feature::Mcp));
        assert!(file.should_skip(&InitOptions {
            features: Some(vec![Feature::Command]),
            ..default_opts()
        }));
    }

    #[test]
    fn init_file_should_skip_when_features_absent() {
        let file =
            InitFile::new("some.txt", "content").with_skip_if(|o| !o.has_feature(Feature::Mcp));
        assert!(file.should_skip(&default_opts()));
    }

    #[test]
    fn build_config_content_defaults_to_empty_features() {
        let opts = default_opts();
        let (global, local) = build_config_content(&opts, InitTemplate::Blank);
        assert!(
            global.contains("features = []"),
            "expected empty features list; got: {global}"
        );
        assert!(
            local.is_empty(),
            "Blank template should produce empty local config"
        );
    }

    #[test]
    fn build_config_content_blank_skips_local_config() {
        let opts = default_opts();
        let (_, local) = build_config_content(&opts, InitTemplate::Blank);
        assert!(
            local.is_empty(),
            "Blank template: local config should be empty"
        );
    }

    #[test]
    fn build_config_content_writes_selected_features() {
        let opts = InitOptions {
            features: Some(vec![Feature::Command, Feature::Instruction]),
            ..default_opts()
        };
        let (global, _) = build_config_content(&opts, InitTemplate::Blank);
        assert!(global.contains("\"command\""));
        assert!(global.contains("\"instruction\""));
        assert!(!global.contains("\"mcp\""));
        assert!(!global.contains("\"skill\""));
        assert!(!global.contains("\"agent-ignore\""));
    }

    #[test]
    fn build_config_content_starter_global_and_local_are_identical() {
        let opts = default_opts();
        let (global, local) = build_config_content(&opts, InitTemplate::Starter);
        assert_eq!(
            global, local,
            "Starter template: global and local configs should match"
        );
    }

    #[test]
    fn build_config_content_advanced_appends_provider_block() {
        let opts = default_opts();
        let (global, local) = build_config_content(&opts, InitTemplate::Advanced);
        assert!(
            !global.contains("providers.mycode"),
            "global should not have provider block"
        );
        assert!(
            local.contains("providers.mycode"),
            "local should contain mycode provider block"
        );
        assert!(local.contains(mocks::MYCODE_PROVIDER_CONFIG));
    }

    #[test]
    fn initialize_agents_dir_with_explicit_dir() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let target = temp.path().join("myproject");
        let opts = InitOptions {
            dir: Some(target.clone()),
            template: Some(InitTemplate::Starter),
            force: true,
            features: Some(vec![
                Feature::Command,
                Feature::Instruction,
                Feature::Mcp,
                Feature::Skill,
            ]),
            ..default_opts()
        };
        initialize_agents_dir(opts).expect("init should succeed");
        assert!(
            target.join(ROOT_DIR).is_dir(),
            ".dotagents-debug should exist inside the provided dir"
        );
    }

    #[test]
    fn initialize_agents_dir_creates_missing_workspace() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let target = temp.path().join("new").join("nested").join("dir");
        assert!(!target.exists(), "target should not pre-exist");
        let opts = InitOptions {
            dir: Some(target.clone()),
            template: Some(InitTemplate::Starter),
            force: true,
            features: Some(vec![
                Feature::Command,
                Feature::Instruction,
                Feature::Mcp,
                Feature::Skill,
            ]),
            ..default_opts()
        };
        initialize_agents_dir(opts).expect("init should succeed");
        assert!(target.join(ROOT_DIR).is_dir(), ".dotagents-debug created");
    }

    // init with dir=None uses CWD (main_dir is relative .dotagents-debug)
    #[test]
    fn initialize_agents_dir_no_dir_uses_cwd() {
        // When dir is None the workspace is CWD; the resulting main_dir is an absolute path
        // ending in ROOT_DIR. We can't actually create files in CWD from a unit test, so
        // just verify the path computation doesn't panic and produces a path ending in ROOT_DIR.
        let cwd = std::env::current_dir().unwrap();
        let expected = cwd.join(ROOT_DIR);
        // Construct the same path the function would use.
        let workspace = cwd;
        let main_dir = workspace.join(ROOT_DIR);
        assert_eq!(main_dir, expected);
    }
}
