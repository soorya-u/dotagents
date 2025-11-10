use anyhow::{Context, Result};

use crate::constants::file::{GLOBAL_CONFIG_FILE, LOCAL_CONFIG_FILE};
use crate::schema::config::{AppConfig, GlobalConfig, LocalConfig, TomlConfig};
use crate::templates::helpers::get_templater;

pub(super) fn deploy() -> Result<()> {
    let templater = get_templater();

    let global_config_content = templater.render_template(GLOBAL_CONFIG_FILE, None)?;
    let local_config_content = templater.render_template(LOCAL_CONFIG_FILE, None)?;

    let local_config = LocalConfig::from_toml(&local_config_content)?;
    local_config.validate().context("invalid local config")?;
    let global_config = GlobalConfig::from_toml(&global_config_content)?;
    global_config.validate().context("invalid local config")?;

    let app_config = AppConfig::from_configs(&global_config, &local_config);

    println!("# Application Config: \n\n{}\n\n", app_config.to_toml()?);
    println!(
        "# Cache Config: \n\n{}\n\n",
        app_config.to_cache().to_toml()?
    );

    todo!()
}
