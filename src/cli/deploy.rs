use anyhow::{Context, Result};
use serde_yaml::to_value;

use crate::constants::features::{COMMANDS_FEATURE, INSTRUCTION_FEATURE, MCP_FEATURE};
use crate::schema::command::Command;
use crate::schema::config::{AppConfig, TomlConfig};
use crate::schema::instruction::Instruction;
use crate::schema::mcp::McpConfig;
use crate::templates::helpers::get_templater;

pub(super) fn deploy() -> Result<()> {
    let templater = get_templater();
    let app_config =
        AppConfig::from_application(templater).context("Failed to load application config")?;
    let variables =
        to_value(app_config.variables.clone()).context("Failed to extract variables")?;

    if app_config.has_feature(COMMANDS_FEATURE) {
        let commands = Command::from_application().context("Failed to load commands")?;
        let providers_with_config = app_config.get_feature_providers(COMMANDS_FEATURE);
    }

    if app_config.has_feature(MCP_FEATURE) {
        let mcp = McpConfig::from_application().context("Failed to load mcp config")?;
        let providers_with_config = app_config.get_feature_providers(MCP_FEATURE);
    }

    if app_config.has_feature(INSTRUCTION_FEATURE) {
        let instruction = Instruction::from_application().context("Failed to load instruction")?;
        let providers_with_config = app_config.get_feature_providers(INSTRUCTION_FEATURE);
    }

    println!("# Application Config: \n\n{}\n\n", app_config.to_toml()?);
    println!(
        "# Cache Config: \n\n{}\n\n",
        app_config.to_cache().to_toml()?
    );

    todo!()
}
