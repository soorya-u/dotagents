use super::options::InitOptions;
use crate::config::dummy;
use crate::constants::resources::{COMMANDS_DIR, INSTRUCTIONS_FILE, MCP_FILE, ROOT_DIR};
use anyhow::{Context, Result};
use log;
use std::{fs, path::Path};

fn seed_dummy<F>(skip: bool, name: &str, path: &str, f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if !skip {
        f()?;
    } else {
        log::info!("Skipping {} {}", path, name);
    }

    Ok(())
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
    

    seed_dummy(
        opts.no_command,
        "directory",
        COMMANDS_DIR,
        dummy::set_dummy_command,
    )?;

    seed_dummy(
        opts.no_instruction,
        "file",
        INSTRUCTIONS_FILE,
        dummy::set_dummy_instructions,
    )?;

    seed_dummy(opts.no_mcp, "file", MCP_FILE, dummy::set_dummy_mcp)?;

    let set_dummy_config = || dummy::set_dummy_config(opts);

    seed_dummy(false, "", "", set_dummy_config)?;

    seed_dummy(false, "", "", dummy::set_gitignore)?;

    Ok(())
}

