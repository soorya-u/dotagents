//! Integration tests for the render pipeline.

use super::{TestWorkspace, init_with_mycode_provider};
use std::fs;

/// Adds template mode for instruction and command features to the local config.
fn enable_template_mode_for_type2(ws: &TestWorkspace) {
    let config_path = ws.active_root_dir().join("local.config.toml");
    let mut config = fs::read_to_string(&config_path).unwrap();
    config.push_str("\n[feature-maps.instruction]\nmode = \"template\"\n");
    config.push_str("[feature-maps.command]\nmode = \"template\"\n");
    fs::write(&config_path, config).unwrap();
}

// ─────────────────────────────────────────────────────────────────────────────
// Variable interpolation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn provider_variable_interpolated_in_instructions_output() {
    // The mycode provider sets agent_name = "Mycode"; INSTRUCTIONS.md uses {{ var.agent_name }}.
    // Deployed output should contain the resolved value.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);
    enable_template_mode_for_type2(&ws);

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
    enable_template_mode_for_type2(&ws);

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
    enable_template_mode_for_type2(&ws);

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
    // Skill output is provider-agnostic — rendered directly with frontmatter from to_markdown().
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

// ─────────────────────────────────────────────────────────────────────────────
// HookFeature integration (embedded merge + standalone write + TOML)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn hook_embedded_merge_gemini() {
    // Pre-create .gemini/settings.json with model; hooks merge into it.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);
    let d = ws.root_dir_name();
    ws.write_file(
        format!("{d}/hooks.jsonc"),
        r#"{
          "$schema": "https://dotagents.soorya-u.dev/v1/schemas/hooks.schema.json",
          "hooks": [{"event":"PreToolUse","type":"command","command":"./x.sh","timeout":5000,"matcher":"Bash"}]
        }"#,
    );
    let config_path = ws.active_root_dir().join("local.config.toml");
    let mut cfg = fs::read_to_string(&config_path).unwrap();
    // Enable hook feature + target gemini (init_with_mycode_provider clears targets to [] for offline mycode)
    // Use canonical feature name "hook".
    let root = env!("CARGO_MANIFEST_DIR");
    cfg = cfg
        .replace(r#"targets = []"#, r#"targets = ["gemini"]"#)
        .replace("features = [", "features = [\"hook\", ");
    cfg.push_str(&format!(
        r#"

[providers.gemini.hooks]
template = "{root}/public/v1/templates/gemini/hooks.hbs"
target = "{{{{ dir.workspace }}}}/.gemini/settings.json"
"#
    ));
    fs::write(&config_path, cfg).unwrap();
    ws.write_file(".gemini/settings.json", r#"{"model":"gemini-2.5"}"#);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let out = ws.read_file(".gemini/settings.json");
    let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        parsed["model"], "gemini-2.5",
        "model preserved; full content:\n{out}"
    );
    if !out.contains("PreToolUse") {
        eprintln!("DEBUG gemini hooks output:\n{out}");
        panic!("expected PreToolUse in gemini hooks merge output");
    }
    assert!(out.contains("./x.sh"));
}

#[test]
fn hook_standalone_write_cursor() {
    // Cursor hooks deploy to standalone .cursor/hooks.json (flattened).
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);
    let d = ws.root_dir_name();
    ws.write_file(
        format!("{d}/hooks.jsonc"),
        r#"{"hooks":[{"event":"Stop","type":"command","command":"echo hi","timeout":1000}]}"#,
    );
    let config_path = ws.active_root_dir().join("local.config.toml");
    let mut cfg = fs::read_to_string(&config_path).unwrap();
    let root = env!("CARGO_MANIFEST_DIR");
    cfg = cfg
        .replace(r#"targets = []"#, r#"targets = ["cursor"]"#)
        .replace("features = [", "features = [\"hook\", ");
    cfg.push_str(&format!(
        r#"

[providers.cursor.hooks]
template = "{root}/public/v1/templates/cursor/hooks.hbs"
target = "{{{{ dir.workspace }}}}/.cursor/hooks.json"
"#
    ));
    fs::write(&config_path, cfg).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let out = ws.read_file(".cursor/hooks.json");
    if !out.contains("stop") {
        eprintln!("DEBUG cursor hooks output:\n{out}");
        panic!("expected lowercase-first 'stop' in cursor hooks output");
    }
    assert!(out.contains("version"));
    assert!(out.contains("echo hi"));
}

#[test]
fn hook_toml_output_kimi() {
    // Kimi emits [[hooks]] TOML table.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);
    let d = ws.root_dir_name();
    ws.write_file(
        format!("{d}/hooks.jsonc"),
        r#"{"hooks":[{"event":"PreToolUse","type":"command","command":"guard","matcher":"Bash","timeout":2000}]}"#,
    );
    let config_path = ws.active_root_dir().join("local.config.toml");
    let mut cfg = fs::read_to_string(&config_path).unwrap();
    let root = env!("CARGO_MANIFEST_DIR");
    cfg = cfg
        .replace(r#"targets = []"#, r#"targets = ["kimi"]"#)
        .replace("features = [", "features = [\"hook\", ");
    cfg.push_str(&format!(
        r#"

[providers.kimi.hooks]
template = "{root}/public/v1/templates/kimi/hooks.hbs"
target = "{{{{ dir.workspace }}}}/.kimi-code/config.toml"
"#
    ));
    fs::write(&config_path, cfg).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let out = ws.read_file(".kimi-code/config.toml");
    assert!(out.contains("[[hooks]]"));
    assert!(out.contains("event = \"PreToolUse\""));
    assert!(out.contains("command = \"guard\""));
}
