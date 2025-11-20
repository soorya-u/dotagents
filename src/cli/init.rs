use std::{
    fs,
    path::{Path, PathBuf},
};

use super::options::InitOptions;
use crate::constants::{
    dir::{COMMANDS_DIR, ROOT_DIR, TEMPLATE_DIR},
    file::{GLOBAL_CONFIG_FILE, INSTRUCTIONS_FILE, LOCAL_CONFIG_FILE, MCP_FILE},
    mocks,
};
use crate::utils::fs::write_file;
use anyhow::{Context, Result};
use log;

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

    // Write .env.example
    write_file(&main_dir.join(".env.example"), mocks::ENV_EXAMPLE)?;
    write_file(&main_dir.join(".env"), mocks::ENV_EXAMPLE)?;

    // Write .gitignore
    write_file(&main_dir.join(".gitignore"), mocks::GITIGNORE)?;

    // Write config files
    write_file(&main_dir.join(GLOBAL_CONFIG_FILE), mocks::CONFIG)?;
    write_file(&main_dir.join(LOCAL_CONFIG_FILE), mocks::LOCAL_CONFIG)?;

    // Write INSTRUCTIONS.md if not skipped
    if !opts.no_instruction {
        write_file(&main_dir.join(INSTRUCTIONS_FILE), mocks::INSTRUCTIONS)?;
    } else {
        log::info!("Skipping {}", INSTRUCTIONS_FILE);
    }

    // Write mcp.jsonc if not skipped
    if !opts.no_mcp {
        write_file(&main_dir.join(MCP_FILE), mocks::MCP)?;
    } else {
        log::info!("Skipping {}", MCP_FILE);
    }

    // Write commands/hello.md if not skipped
    if !opts.no_command {
        write_file(
            &main_dir.join(COMMANDS_DIR).join("hello.md"),
            mocks::COMMAND_HELLO,
        )?;
    } else {
        log::info!("Skipping {}", COMMANDS_DIR);
    }

    // Write templates
    let templates_base = main_dir.join(TEMPLATE_DIR).join("mycode");
    write_file(
        &templates_base.join("command.hbs"),
        mocks::TEMPLATE_MYCODE_COMMAND,
    )?;
    write_file(
        &templates_base.join("instructions.hbs"),
        mocks::TEMPLATE_MYCODE_INSTRUCTIONS,
    )?;
    write_file(&templates_base.join("mcp.hbs"), mocks::TEMPLATE_MYCODE_MCP)?;

    Ok(())
}
