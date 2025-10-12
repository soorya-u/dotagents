use anyhow::{Context, Result, anyhow};
use std::env;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn get_workspace_dir() -> Result<PathBuf> {
    let mut current = env::current_dir().context("failed to get current directory")?;

    loop {
        let marker = current.join(".dotagents");

        if marker.is_dir() {
            return Ok(current);
        }

        if !current.pop() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "No `.dotfiles` directory found in any parent directory",
            )
            .into());
        }
    }
}

// TODO: Valid only for Unix as of Now. Make Win
pub fn get_config_dir() -> Result<PathBuf> {
    let home_dir = get_home_dir()?;
    let config_dir = home_dir.join(".config");
    match config_dir.try_exists() {
        Ok(true) => Ok(config_dir),
        Ok(false) => Err(anyhow!("config file does not exist")),
        Err(e) => Err(anyhow!(e)),
    }
}

pub fn get_home_dir() -> Result<PathBuf> {
    home::home_dir().ok_or_else(|| anyhow!("failed to get user home directory"))
}
