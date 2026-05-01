use std::io::IsTerminal;

use crate::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

use super::options::{Feature, InitOptions, InitTemplate};
use crate::cli::ui;
use crate::constants::{
    dir::{COMMANDS_DIR, MOCK_CUSTOM_AGENT_DIR, ROOT_DIR, SKILLS_DIR, TEMPLATE_DIR},
    file::{
        ENV_EXAMPLE_FILE, ENV_FILE, GITIGNORE_FILE, GLOBAL_CONFIG_FILE, INSTRUCTIONS_FILE,
        LOCAL_CONFIG_FILE, MCP_FILE, MOCK_COMMAND_FILE, MOCK_COMMAND_TEMPLATE_FILE,
        MOCK_INSTRUCTION_TEMPLATE_FILE, MOCK_MCP_TEMPLATE_FILE, MOCK_SKILL_DIR,
        MOCK_SKILL_TEMPLATE_FILE, SKILL_FILE,
    },
    mocks,
};
use crate::utils::fs::write_file;
use anyhow::{Context, Result};

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

/// Returns true when init should run in interactive TUI mode.
fn is_tui_mode(opts: &InitOptions) -> bool {
    opts.features.is_none() && opts.template.is_none() && std::io::stdin().is_terminal()
}

/// Validates the `--features` flag: errors on empty value or `none` combined with others.
fn validate_features(opts: &InitOptions) -> Result<()> {
    let Some(features) = &opts.features else {
        return Ok(());
    };
    if features.is_empty() {
        anyhow::bail!(
            "--features requires at least one value. Use '--features none' to disable all features."
        );
    }
    let has_none = features.iter().any(|f| matches!(f, Feature::None));
    if has_none && features.len() > 1 {
        anyhow::bail!("'none' cannot be combined with other feature names in --features.");
    }
    Ok(())
}

/// Updates the `targets` array in the given TOML config file.
fn update_config_targets(config_path: &Path, targets: &[String]) -> Result<()> {
    let content =
        fs::read_to_string(config_path).context("Failed to read config.toml for target update")?;
    let mut value: toml::Value = toml::from_str(&content).context("Failed to parse config.toml")?;
    let toml::Value::Table(ref mut table) = value else {
        anyhow::bail!("config.toml root is not a TOML table");
    };
    table.insert(
        "targets".to_owned(),
        toml::Value::Array(
            targets
                .iter()
                .map(|s| toml::Value::String(s.clone()))
                .collect(),
        ),
    );
    let new_content =
        toml::to_string_pretty(&value).context("Failed to serialise updated config.toml")?;
    fs::write(config_path, new_content).context("Failed to write updated config.toml")?;
    Ok(())
}

pub(super) fn initialize_agents_dir(mut opts: InitOptions) -> Result<()> {
    validate_features(&opts)?;

    let main_dir = Path::new(ROOT_DIR);

    let dir_exists = main_dir
        .try_exists()
        .context("failed to check if .dotagents directory exists")?;

    let tui_mode = is_tui_mode(&opts);

    if tui_mode {
        let proceed = ui::init::run_init_wizard(&mut opts, dir_exists)?;
        if !proceed {
            return Ok(());
        }
    }

    if dir_exists {
        if !opts.force {
            anyhow::bail!(format!(
                "Configuration already exists: {}",
                main_dir.display()
            ));
        } else {
            if !tui_mode {
                warn!("Overwriting existing configuration");
            }
            fs::remove_dir_all(main_dir).context("failed to remove .dotagents directory")?;
        }
    }

    fs::create_dir(main_dir).context("failed to create .dotagents directory")?;

    // Resolve the effective template: default to Starter when no flag was set.
    let template = opts.template.unwrap_or(InitTemplate::Starter);

    let init_files = vec![
        InitFile::new(ENV_EXAMPLE_FILE, mocks::ENV_EXAMPLE),
        InitFile::new(ENV_FILE, mocks::ENV_EXAMPLE),
        InitFile::new(GITIGNORE_FILE, mocks::GITIGNORE),
        InitFile::new(GLOBAL_CONFIG_FILE, mocks::CONFIG),
        InitFile::new(
            LOCAL_CONFIG_FILE,
            match template {
                InitTemplate::WithCustomProvider => mocks::LOCAL_CONFIG_WITH_PROVIDER,
                InitTemplate::Starter => mocks::CONFIG,
            },
        ),
        InitFile::new(INSTRUCTIONS_FILE, mocks::INSTRUCTIONS)
            .with_skip_if(|opts| !opts.has_feature(Feature::Instructions)),
        InitFile::new(MCP_FILE, mocks::MCP).with_skip_if(|opts| !opts.has_feature(Feature::Mcp)),
        InitFile::new(
            Path::new(COMMANDS_DIR).join(MOCK_COMMAND_FILE),
            mocks::COMMAND_HELLO,
        )
        .with_skip_if(|opts| !opts.has_feature(Feature::Commands)),
        InitFile::new(
            Path::new(SKILLS_DIR).join(MOCK_SKILL_DIR).join(SKILL_FILE),
            mocks::SKILL_HELLO,
        )
        .with_skip_if(|opts| !opts.has_feature(Feature::Skills)),
        // Template files — only written for the WithCustomProvider template.
        InitFile::new(
            Path::new(TEMPLATE_DIR)
                .join(MOCK_CUSTOM_AGENT_DIR)
                .join(MOCK_COMMAND_TEMPLATE_FILE),
            mocks::TEMPLATE_MYCODE_COMMAND,
        )
        .with_skip_if(|opts| opts.template != Some(InitTemplate::WithCustomProvider)),
        InitFile::new(
            Path::new(TEMPLATE_DIR)
                .join(MOCK_CUSTOM_AGENT_DIR)
                .join(MOCK_SKILL_TEMPLATE_FILE),
            mocks::TEMPLATE_MYCODE_SKILL,
        )
        .with_skip_if(|opts| opts.template != Some(InitTemplate::WithCustomProvider)),
        InitFile::new(
            Path::new(TEMPLATE_DIR)
                .join(MOCK_CUSTOM_AGENT_DIR)
                .join(MOCK_INSTRUCTION_TEMPLATE_FILE),
            mocks::TEMPLATE_MYCODE_INSTRUCTIONS,
        )
        .with_skip_if(|opts| opts.template != Some(InitTemplate::WithCustomProvider)),
        InitFile::new(
            Path::new(TEMPLATE_DIR)
                .join(MOCK_CUSTOM_AGENT_DIR)
                .join(MOCK_MCP_TEMPLATE_FILE),
            mocks::TEMPLATE_MYCODE_MCP,
        )
        .with_skip_if(|opts| opts.template != Some(InitTemplate::WithCustomProvider)),
    ];

    for file in init_files {
        if file.should_skip(&opts) {
            info!("Skipping {}", file.path.display());
            continue;
        }
        write_file(&main_dir.join(&file.path), file.content)?;
    }

    if tui_mode {
        if !opts.targets.is_empty() {
            update_config_targets(&main_dir.join(GLOBAL_CONFIG_FILE), &opts.targets)?;
            update_config_targets(&main_dir.join(LOCAL_CONFIG_FILE), &opts.targets)?;
        }
        ui::init::finish_init();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn default_opts() -> InitOptions {
        InitOptions {
            features: None,
            force: false,
            template: None,
            targets: vec![],
        }
    }

    // is_tui_mode returns false when --features is set
    #[test]
    fn is_tui_mode_false_when_features_set() {
        assert!(!is_tui_mode(&InitOptions {
            features: Some(vec![Feature::Commands]),
            ..default_opts()
        }));
    }

    // is_tui_mode returns false when --features none is set
    #[test]
    fn is_tui_mode_false_when_features_none_set() {
        assert!(!is_tui_mode(&InitOptions {
            features: Some(vec![Feature::None]),
            ..default_opts()
        }));
    }

    // is_tui_mode returns false when --template is set
    #[test]
    fn is_tui_mode_false_when_template_set() {
        assert!(!is_tui_mode(&InitOptions {
            template: Some(InitTemplate::Starter),
            ..default_opts()
        }));
        assert!(!is_tui_mode(&InitOptions {
            template: Some(InitTemplate::WithCustomProvider),
            ..default_opts()
        }));
    }

    // is_tui_mode returns false whenever any headless flag is set
    #[test]
    fn is_tui_mode_false_when_any_headless_flag_set() {
        let cases = [
            InitOptions {
                features: Some(vec![Feature::Commands]),
                ..default_opts()
            },
            InitOptions {
                features: Some(vec![Feature::Commands, Feature::Mcp]),
                ..default_opts()
            },
            InitOptions {
                features: Some(vec![Feature::None]),
                ..default_opts()
            },
            InitOptions {
                template: Some(InitTemplate::Starter),
                ..default_opts()
            },
            InitOptions {
                template: Some(InitTemplate::WithCustomProvider),
                ..default_opts()
            },
        ];
        for opts in &cases {
            assert!(
                !is_tui_mode(opts),
                "expected TUI mode disabled when a headless flag is set"
            );
        }
    }

    // validate_features errors on none combined with other values
    #[test]
    fn validate_features_errors_on_none_combined() {
        let opts = InitOptions {
            features: Some(vec![Feature::None, Feature::Commands]),
            ..default_opts()
        };
        assert!(validate_features(&opts).is_err());
    }

    // validate_features succeeds for a valid explicit list
    #[test]
    fn validate_features_ok_for_explicit_list() {
        let opts = InitOptions {
            features: Some(vec![Feature::Commands, Feature::Mcp]),
            ..default_opts()
        };
        assert!(validate_features(&opts).is_ok());
    }

    // validate_features succeeds for none sentinel alone
    #[test]
    fn validate_features_ok_for_none_alone() {
        let opts = InitOptions {
            features: Some(vec![Feature::None]),
            ..default_opts()
        };
        assert!(validate_features(&opts).is_ok());
    }

    // validate_features succeeds when flag is absent
    #[test]
    fn validate_features_ok_when_absent() {
        assert!(validate_features(&default_opts()).is_ok());
    }

    // update_config_targets writes the targets array into the TOML file
    #[test]
    fn update_config_targets_sets_targets_array() {
        let f = NamedTempFile::new().expect("temp file");
        fs::write(f.path(), mocks::CONFIG).expect("write config");
        let targets = vec!["claude".to_string(), "codex".to_string()];
        update_config_targets(f.path(), &targets).expect("update should succeed");
        let result = fs::read_to_string(f.path()).unwrap();
        assert!(result.contains("claude"), "targets should contain 'claude'");
        assert!(result.contains("codex"), "targets should contain 'codex'");
    }

    // update_config_targets replaces a previously set targets array
    #[test]
    fn update_config_targets_replaces_existing_targets() {
        let f = NamedTempFile::new().expect("temp file");
        fs::write(f.path(), mocks::CONFIG).expect("write config");
        update_config_targets(f.path(), &["new-provider".to_string()])
            .expect("update should succeed");
        let result = fs::read_to_string(f.path()).unwrap();
        assert!(
            result.contains("new-provider"),
            "new target should be present"
        );
        assert!(
            !result.contains("windsurf"),
            "old targets should be replaced"
        );
    }

    // update_config_targets errors on a missing file
    #[test]
    fn update_config_targets_errors_on_missing_file() {
        let result = update_config_targets(Path::new("/nonexistent/config.toml"), &[]);
        assert!(result.is_err(), "should error when file does not exist");
    }

    // InitFile::should_skip returns false when no condition is set
    #[test]
    fn init_file_should_not_skip_without_condition() {
        let file = InitFile::new("some.txt", "content");
        assert!(!file.should_skip(&default_opts()));
    }

    // InitFile::should_skip returns true when the feature is not in the list
    #[test]
    fn init_file_should_skip_when_feature_disabled() {
        let file =
            InitFile::new("some.txt", "content").with_skip_if(|o| !o.has_feature(Feature::Mcp));
        assert!(file.should_skip(&InitOptions {
            features: Some(vec![Feature::Commands]),
            ..default_opts()
        }));
    }

    // InitFile::should_skip returns false when the feature is enabled
    #[test]
    fn init_file_should_not_skip_when_feature_enabled() {
        let file =
            InitFile::new("some.txt", "content").with_skip_if(|o| !o.has_feature(Feature::Mcp));
        assert!(!file.should_skip(&default_opts()));
    }
}
