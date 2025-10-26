use anyhow::{Context, Result};

use crate::schema::config::ApplicationConfig;

pub(super) fn deploy() -> Result<()> {
    let app_config = ApplicationConfig::new()?;
    let content = app_config.to_toml()?;

    println!("Application config: \n\n{}", content);
    Ok(())
}
