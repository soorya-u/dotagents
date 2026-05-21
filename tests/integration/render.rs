//! Integration tests for the render pipeline.

use super::{TestWorkspace, init_with_mycode_provider};

// ─────────────────────────────────────────────────────────────────────────────
// Variable interpolation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn provider_variable_interpolated_in_instructions_output() {
    // The mycode provider sets agent_name = "Mycode"; INSTRUCTIONS.md uses {{ var.agent_name }}.
    // Deployed output should contain the resolved value.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let content = ws.read_file(".mycode/instructions.md");
    assert!(
        content.contains("Mycode"),
        "provider variable agent_name should be interpolated; got:\n{content}"
    );
}

#[test]
fn env_variable_interpolated_in_instructions_output() {
    // INSTRUCTIONS.md uses {{ env.app_name }}; .env contains APP_NAME=dotagents.
    // Deployed output should contain the resolved env value.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let content = ws.read_file(".mycode/instructions.md");
    assert!(
        content.contains("dotagents"),
        "env variable APP_NAME should be interpolated as 'dotagents'; got:\n{content}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Frontmatter stripping
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deployed_command_has_no_yaml_frontmatter_delimiter() {
    // The command template (command.hbs) renders only {{command.content}}, not frontmatter.
    // Deployed command file should not start with or contain `---`.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let content = ws.read_file(".mycode/commands/hello.md");
    assert!(
        !content.starts_with("---"),
        "deployed command should not start with YAML frontmatter delimiter; got:\n{content}"
    );
    // Content should not contain the raw YAML block.
    assert!(
        !content.contains("name: hello"),
        "YAML frontmatter key 'name' should be absent from deployed output; got:\n{content}"
    );
}

#[test]
fn command_content_body_preserved_in_deployed_output() {
    // The hello command body references {{ var.agent_name }}; after rendering it should
    // contain "Mycode" and the static prose from the source file.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let content = ws.read_file(".mycode/commands/hello.md");
    assert!(
        content.contains("Mycode"),
        "command content should have agent_name variable resolved; got:\n{content}"
    );
    assert!(
        content.contains("Greet the User"),
        "static prose from source should appear in deployed output; got:\n{content}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP output format
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mcp_output_is_valid_json_with_servers_key() {
    // mcp.hbs renders a JSON object with an "mcpServers" top-level key.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let content = ws.read_file(".mycode/mcp.json");
    assert!(
        content.contains("mcpServers"),
        "deployed mcp.json should contain the mcpServers key; got:\n{content}"
    );
    // Validate that the output is parseable JSON.
    serde_json::from_str::<serde_json::Value>(&content)
        .expect("deployed mcp.json should be valid JSON");
}

#[test]
fn mcp_expanded_fields_render_to_toml_and_json_providers() {
    // Expanded MCP fields should map to representative TOML and JSON provider outputs.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);
    let d = ws.root_dir_name();
    let root = env!("CARGO_MANIFEST_DIR");
    let config_path = ws.active_root_dir().join("local.config.toml");
    let mut config =
        std::fs::read_to_string(&config_path).expect("failed to read local.config.toml");
    config.push_str(&format!(
        r#"

[providers.codex.mcp]
template = "{root}/public/v1/templates/codex/mcp.hbs"
target = "{{{{ dir.workspace }}}}/.codex/config.toml"

[providers.gemini.mcp]
template = "{root}/public/v1/templates/gemini/mcp.hbs"
target = "{{{{ dir.workspace }}}}/.gemini/settings.json"
"#
    ));
    std::fs::write(&config_path, config).expect("failed to write local.config.toml");
    ws.write_file(
        format!("{d}/mcp.jsonc"),
        r#"{
          "$schema": "https://dotagents.soorya-u.dev/v1/schemas/mcp.schema.json",
          "servers": {
            "stdio-server": {
              "type": "stdio",
              "command": "node",
              "args": ["server.js"],
              "enabledTools": ["read"],
              "disabledTools": ["delete"],
              "required": true,
              "startupTimeoutSec": 11,
              "toolTimeoutSec": 22,
              "bearerTokenEnvVar": "TOKEN",
              "envVars": ["TOKEN"]
            },
            "http-server": {
              "type": "http",
              "url": "https://example.com/mcp",
              "headers": {"Authorization": "Bearer token"}
            },
            "sse-server": {
              "type": "sse",
              "url": "https://example.com/sse",
              "headers": {"X-Test": "1"}
            }
          }
        }"#,
    );

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let codex = ws.read_file(".codex/config.toml");
    assert!(codex.contains("enabled_tools = [\"read\"]"));
    assert!(codex.contains("disabled_tools = [\"delete\"]"));
    assert!(codex.contains("startup_timeout_sec = 11"));
    assert!(codex.contains("tool_timeout_sec = 22"));
    assert!(codex.contains("bearer_token_env_var = \"TOKEN\""));
    assert!(codex.contains("env_vars = [\"TOKEN\"]"));
    toml::from_str::<toml::Value>(&codex).expect("codex output should be valid TOML");

    let gemini = ws.read_file(".gemini/settings.json");
    let parsed: serde_json::Value =
        serde_json::from_str(&gemini).expect("gemini output should be valid JSON");
    assert_eq!(
        parsed["mcpServers"]["http-server"]["httpUrl"],
        serde_json::json!("https://example.com/mcp")
    );
    assert_eq!(
        parsed["mcpServers"]["sse-server"]["url"],
        serde_json::json!("https://example.com/sse")
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Skill output
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deployed_skill_output_retains_frontmatter_from_template() {
    // The skill.hbs template re-emits frontmatter with name/description fields.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let content = ws.read_file(".mycode/skills/hello-skill/SKILL.md");
    assert!(
        content.contains("name: hello-skill"),
        "deployed skill output should contain the skill name in frontmatter; got:\n{content}"
    );
}
