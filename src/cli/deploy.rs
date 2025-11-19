use anyhow::{Context, Result, anyhow};
use serde_json::to_value;

use crate::constants::features::{COMMANDS_FEATURE, INSTRUCTION_FEATURE, MCP_FEATURE};
use crate::schema::config::{AppConfig, TomlConfig};
use crate::schema::features::{
    command::CommandFeature, instruction::InstructionFeature, mcp::McpFeature,
};
use crate::templates::{RenderType, get_templater};
use crate::utils::fs::read_file;

pub(super) fn deploy() -> Result<()> {
    let templater = get_templater();
    let app_config =
        AppConfig::from_application(templater).context("Failed to load application config")?;
    let variables =
        to_value(app_config.variables.clone()).context("Failed to extract variables")?;

    // TODO: Command has much complicated process since commands is iterative
    if app_config.has_feature(COMMANDS_FEATURE) {
        let commands = CommandFeature::from_application().context("Failed to load commands")?;
        let providers_with_config = app_config.get_feature_providers(COMMANDS_FEATURE);

        providers_with_config
            .into_iter()
            .try_for_each::<_, Result<()>>(|(provider_name, config)| {
                commands.iter().try_for_each(|command| {
                    config.render_template(templater, &provider_name, command)
                })
            })
            .context("failed to deploy commands feature")?;
    };

    if app_config.has_feature(MCP_FEATURE) {
        let mcp = McpFeature::from_application().context("load mcp config")?;
        let providers_with_config = app_config.get_feature_providers(MCP_FEATURE);

        providers_with_config
            .into_iter()
            .try_for_each::<_, Result<()>>(|(provider_name, config)| {
                config.render_template(templater, &provider_name, &mcp)
            })
            .context("failed to deploy mcp feature")?;
    };

    if app_config.has_feature(INSTRUCTION_FEATURE) {
        let instruction =
            InstructionFeature::from_application().context("Failed to load instruction")?;
        let providers_with_config = app_config.get_feature_providers(INSTRUCTION_FEATURE);

        providers_with_config
            .into_iter()
            .try_for_each::<_, Result<()>>(|(provider_name, config)| {
                config.render_template(templater, &provider_name, &instruction)
            })
            .context("failed to deploy instruction feature")?;
    };

    Ok(())
}
