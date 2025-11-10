use anyhow::{Context, Result, anyhow};
use std::env;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use crate::constants::dir::ROOT_DIR;

fn get_dir_or_die(path: PathBuf) -> Result<PathBuf> {
    if path.is_dir() {
        Ok(path)
    } else {
        Err(anyhow!(format!(
            "{} is not a directory or needs permission",
            path.to_str().unwrap()
        )))
    }
}

pub fn get_workspace_dir() -> Result<PathBuf> {
    let mut current = env::current_dir().context("failed to get current directory")?;

    loop {
        let marker = current.join(ROOT_DIR);

        if marker.is_dir() {
            return Ok(current);
        }

        if !current.pop() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("No `{}` directory found in any parent directory", ROOT_DIR),
            )
            .into());
        }
    }
}

pub fn get_home_dir() -> Result<PathBuf> {
    home::home_dir().ok_or_else(|| anyhow!("failed to get user home directory"))
}

// TODO: Valid only for Unix as of Now. Make Win Compatible
pub fn get_config_dir() -> Result<PathBuf> {
    let home_dir = get_home_dir()?;
    let config_dir = home_dir.join(".config");
    get_dir_or_die(config_dir)
}

pub fn get_application_dir() -> Result<PathBuf> {
    let app_dir = get_workspace_dir()?.join(ROOT_DIR);
    get_dir_or_die(app_dir)
}
