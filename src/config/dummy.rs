use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::InitOptions;
use crate::constants::resources;
use crate::schema::config::{ConfigAgentSettings, ConfigBuilder, ProviderSettings, Target};
use crate::schema::{command::CommandBuilder, mcp::McpConfigBuilder};

fn get_root_relative_path<P: AsRef<Path>>(relative_path: P) -> PathBuf {
    let main_dir = Path::new(resources::ROOT_DIR);
    main_dir.join(relative_path)
}

fn set_dummy_data(filename: &str, content: &str, dir_name: Option<&str>) -> Result<()> {
    if dir_name.is_some() && Path::new(dir_name.unwrap()).exists() {}

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

    set_dummy_data("dummy.md", &content, Some(resources::COMMANDS_DIR))?;

    Ok(())
}

pub(crate) fn set_dummy_instructions() -> Result<()> {
    let content = "# Instructions for {{ agent_name }}\n\nThis is a custom instructions for {{ agent_name }} for a given repository.\n";

    set_dummy_data(resources::INSTRUCTIONS_FILE, content, None)?;

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

    set_dummy_data(resources::MCP_FILE, &content, None)?;

    Ok(())
}

pub(crate) fn set_dummy_config(opts: InitOptions) -> Result<()> {
    let config_builder = ConfigBuilder::new()
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
            ProviderSettings {
                mcp: ConfigAgentSettings {
                    template: Some("templates/opencode".into()),
                    target: Some("{{ workspace_dir }}/.opencode/mcp.json".into()),
                    ..Default::default()
                },
                instructions: ConfigAgentSettings {
                    template: Some("templates/INSTRUCTION.md".into()),
                    target: Some("{{ workspace_dir }}/.opencode/instructions.md".into()),
                    ..Default::default()
                },
                commands: ConfigAgentSettings {
                    template: Some("templates/commands-template".into()),
                    target: Some("{{ workspace_dir }}/.opencode/commands".into()),
                    ..Default::default()
                },
            },
        )
        .build();

    let local_content = local_config.to_toml()?;
    let global_content = global_config.to_toml()?;

    set_dummy_data(resources::GLOBAL_CONFIG_FILE, &global_content, None)?;

    set_dummy_data(resources::LOCAL_CONFIG_FILE, &local_content, None)?;

    Ok(())
}

pub(crate) fn set_gitignore() -> Result<()> {
    let content = "cache/\nlocal.config.yml";

    set_dummy_data(".gitignore", content, None)?;

    Ok(())
}
