use anyhow::{Context, Result};
use rayon::prelude::*;
use serde_json::{Value, to_value};

use crate::constants::features::{COMMANDS_FEATURE, INSTRUCTION_FEATURE, MCP_FEATURE};
use crate::schema::config::AppConfig;
use crate::schema::features::{
    command::CommandFeature, instruction::InstructionFeature, mcp::McpFeature, traits::FeatureTrait,
};
use crate::templates::{Templater, get_templater};

fn deploy_feature<T>(
    app_config: &AppConfig,
    templater: &Templater,
    variables: Option<&Value>,
    feature_name: &str,
    loader: impl FnOnce() -> Result<Vec<T>>,
) -> Result<()>
where
    T: FeatureTrait + Sync,
{
    if !app_config.has_feature(feature_name) {
        return Ok(());
    }

    let features = loader().context(format!("Failed to load {} feature", feature_name))?;
    let providers = app_config.get_feature_providers(feature_name);

    providers
        .par_iter()
        .map(|(provider_name, config)| {
            features.iter().try_for_each(|feature| {
                config.render_template(templater, provider_name, variables, feature)
            })
        })
        .collect::<Result<()>>()?;

    Ok(())
}

pub(super) fn deploy() -> Result<()> {
    let templater = get_templater();
    let app_config =
        AppConfig::from_application(templater).context("Failed to load application config")?;
    let variables =
        Some(to_value(app_config.variables.clone()).context("Failed to extract variables")?);

    deploy_feature::<CommandFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        COMMANDS_FEATURE,
        CommandFeature::from_application,
    )?;

    deploy_feature::<McpFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        MCP_FEATURE,
        || McpFeature::from_application().map(|mcp| vec![mcp]),
    )?;

    deploy_feature::<InstructionFeature>(
        &app_config,
        templater,
        variables.as_ref(),
        INSTRUCTION_FEATURE,
        || InstructionFeature::from_application().map(|inst| vec![inst]),
    )?;

    Ok(())
}
