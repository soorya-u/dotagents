use std::{
    fs,
    path::{Path, PathBuf},
};

use super::options::InitOptions;
use crate::constants::{
    dir::{COMMANDS_DIR, MOCK_CUSTOM_AGENT_DIR, ROOT_DIR, SKILLS_DIR, TEMPLATE_DIR},
    file::{
        ENV_EXAMPLE_FILE, ENV_FILE, GITIGNORE_FILE, GLOBAL_CONFIG_FILE, INSTRUCTIONS_FILE,
        LOCAL_CONFIG_FILE, MCP_FILE, MOCK_COMMAND_FILE, MOCK_COMMAND_TEMPLATE_FILE,
        MOCK_INSTRUCTION_TEMPLATE_FILE, MOCK_MCP_TEMPLATE_FILE, MOCK_SKILL_FILE,
        MOCK_SKILL_TEMPLATE_FILE,
    },
    mocks,
};
use crate::utils::fs::write_file;
use anyhow::{Context, Result};

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

pub(super) fn initialize_agents_dir(opts: InitOptions) -> Result<()> {
    let main_dir = Path::new(ROOT_DIR);

    if main_dir
        .try_exists()
        .context("failed to check if .dotagents directory exists")?
    {
        if !opts.force {
            anyhow::bail!(format!(
                "Configuration already exists: {}",
                main_dir.display()
            ));
        } else {
            log::warn!("Overwriting existing configuration");
            fs::remove_dir_all(main_dir).context("failed to remove .dotagents directory")?;
        }
    }

    fs::create_dir(main_dir).context("failed to create .dotagents directory")?;

    let init_files = vec![
        InitFile::new(ENV_EXAMPLE_FILE, mocks::ENV_EXAMPLE),
        InitFile::new(ENV_FILE, mocks::ENV_EXAMPLE),
        InitFile::new(GITIGNORE_FILE, mocks::GITIGNORE),
        InitFile::new(GLOBAL_CONFIG_FILE, mocks::CONFIG),
        InitFile::new(LOCAL_CONFIG_FILE, mocks::LOCAL_CONFIG),
        InitFile::new(INSTRUCTIONS_FILE, mocks::INSTRUCTIONS)
            .with_skip_if(|opts| opts.no_instruction),
        InitFile::new(MCP_FILE, mocks::MCP).with_skip_if(|opts| opts.no_mcp),
        InitFile::new(
            Path::new(COMMANDS_DIR).join(MOCK_COMMAND_FILE),
            mocks::COMMAND_HELLO,
        )
        .with_skip_if(|opts| opts.no_command),
        InitFile::new(
            Path::new(SKILLS_DIR).join(MOCK_SKILL_FILE),
            mocks::SKILL_HELLO,
        )
        .with_skip_if(|opts| opts.no_skill),
        InitFile::new(
            Path::new(TEMPLATE_DIR)
                .join(MOCK_CUSTOM_AGENT_DIR)
                .join(MOCK_COMMAND_TEMPLATE_FILE),
            mocks::TEMPLATE_MYCODE_COMMAND,
        ),
        InitFile::new(
            Path::new(TEMPLATE_DIR)
                .join(MOCK_CUSTOM_AGENT_DIR)
                .join(MOCK_SKILL_TEMPLATE_FILE),
            mocks::TEMPLATE_MYCODE_SKILL,
        ),
        InitFile::new(
            Path::new(TEMPLATE_DIR)
                .join(MOCK_CUSTOM_AGENT_DIR)
                .join(MOCK_INSTRUCTION_TEMPLATE_FILE),
            mocks::TEMPLATE_MYCODE_INSTRUCTIONS,
        ),
        InitFile::new(
            Path::new(TEMPLATE_DIR)
                .join(MOCK_CUSTOM_AGENT_DIR)
                .join(MOCK_MCP_TEMPLATE_FILE),
            mocks::TEMPLATE_MYCODE_MCP,
        ),
    ];

    for file in init_files {
        if file.should_skip(&opts) {
            log::info!("Skipping {}", file.path.display());
            continue;
        }
        write_file(&main_dir.join(&file.path), file.content)?;
    }

    Ok(())
}
