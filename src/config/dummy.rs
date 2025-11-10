use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fmt::format;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::InitOptions;
use crate::constants::dir::CACHE_DIR;
use crate::constants::{
    dir::{COMMANDS_DIR, ROOT_DIR},
    file::{GLOBAL_CONFIG_FILE, INSTRUCTIONS_FILE, LOCAL_CONFIG_FILE, MCP_FILE},
};
use crate::schema::builder::{
    command::CommandBuilder, config::ApplicationConfigBuilder, mcp::McpConfigBuilder,
};
use crate::schema::{
    common::Target,
    config::{ConfigAgentAbilitySettings, ConfigAgentSettings, TomlConfig},
};

fn get_root_relative_path<P: AsRef<Path>>(relative_path: P) -> PathBuf {
    let main_dir = Path::new(ROOT_DIR);
    main_dir.join(relative_path)
}

fn set_dummy_data(filename: &str, content: &str, dir_name: Option<&str>) -> Result<()> {
    let path = if dir_name.is_some() {
        let absolute_dir = get_root_relative_path(dir_name.unwrap());
        fs::create_dir_all(&absolute_dir).context("unable to create commands directory")?;
        absolute_dir.join(filename)
    } else {
        get_root_relative_path(filename)
    };

    fs::write(path, content).context(format!("unable to write dummy data into {}", content))?;

    Ok(())
}

pub(crate) fn set_dummy_command() -> Result<()> {
    let commands = CommandBuilder::new("hello", "A Hello Command to greet the User.")
        .add_content(
            r#"# Hello Command

Greet the User with his name if present, else greet user as stranger.

Context: $USER_INPUT"#,
        )
        .build();

    let content = commands.to_markdown()?;

    set_dummy_data("dummy.md", &content, Some(COMMANDS_DIR))?;

    Ok(())
}

pub(crate) fn set_dummy_instructions() -> Result<()> {
    let content = "# Instructions for {{ agent_name }}\n\nThis is a custom instructions for {{ agent_name }} for a given repository.\n";

    set_dummy_data(INSTRUCTIONS_FILE, content, None)?;

    Ok(())
}

pub(crate) fn set_dummy_mcp() -> Result<()> {
    let config = McpConfigBuilder::new()
        .add_http_server(
            "server-mcp",
            "http://localhost:9000",
            Some(HashMap::from([(
                "Authorization".into(),
                "Bearer ${API_KEY}".into(),
            )])),
            None,
        )
        .add_stdio_server(
            "server-stdio",
            "python",
            vec![],
            Some("{{ workspace_folder }}"),
            None,
        )
        .build();

    let content = config.to_json()?;

    set_dummy_data(MCP_FILE, &content, None)?;

    Ok(())
}

pub(crate) fn set_dummy_config(opts: InitOptions) -> Result<()> {
    let config_builder = ApplicationConfigBuilder::new()
        .add_features(!opts.no_command, !opts.no_instruction, !opts.no_mcp)
        .add_targets(
            vec!["gemini".to_string()],
            vec!["vscode".to_string(), "windsurf".to_string()],
            vec![],
        );

    let global_config = config_builder.clone().build();

    let local_config = config_builder
        .add_target(Target::Custom, vec!["opencode".into()])
        .add_provider(
            Target::Custom,
            "opencode",
            ConfigAgentAbilitySettings {
                mcp: Some(ConfigAgentSettings {
                    template: Some("templates/opencode".into()),
                    target: Some("{{ workspace_dir }}/.opencode/mcp.json".into()),
                    ..Default::default()
                }),
                instructions: Some(ConfigAgentSettings {
                    template: Some("templates/INSTRUCTION.md".into()),
                    target: Some("{{ workspace_dir }}/.opencode/instructions.md".into()),
                    ..Default::default()
                }),
                commands: Some(ConfigAgentSettings {
                    template: Some("templates/commands-template".into()),
                    target: Some("{{ workspace_dir }}/.opencode/commands".into()),
                    ..Default::default()
                }),
            },
        )
        .build_local();

    let local_content = local_config.to_toml()?;
    let global_content = global_config.to_toml()?;

    set_dummy_data(GLOBAL_CONFIG_FILE, &global_content, None)?;
    set_dummy_data(LOCAL_CONFIG_FILE, &local_content, None)?;

    Ok(())
}

pub(crate) fn set_gitignore() -> Result<()> {
    let content = format!("{}/\n{}", CACHE_DIR, LOCAL_CONFIG_FILE);

    set_dummy_data(".gitignore", &content, None)?;

    Ok(())
}
