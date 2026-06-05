//! Integration tests for merge-aware deploy.
//!
//! These tests verify that deploy correctly merges rendered content into existing
//! config files (JSON, JSONC, TOML) while preserving user-managed keys and comments.

use super::{TestWorkspace, init_with_mycode_provider};
use std::fs;

// ─────────────────────────────────────────────────────────────────────────────
// JSON merge
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deploy_mcp_to_existing_json_preserves_non_mcp_keys() {
    // Scenario: User has a settings.json with model config and other keys.
    // Deploy should merge MCP servers into the file while preserving existing keys.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Modify the mycode provider to output MCP to settings.json instead of mcp.json
    let config_path = ws.active_root_dir().join("local.config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let modified = config.replace(
        "target = \"{{ dir.workspace }}/.mycode/mcp.json\"",
        "target = \"{{ dir.workspace }}/settings.json\"",
    );
    fs::write(&config_path, &modified).expect("failed to write modified config");

    // Pre-populate settings.json with existing content
    let existing_settings = r#"{
  "model": "gpt-4",
  "temperature": 0.7,
  "customKey": "user-value"
}"#;
    fs::write(ws.root().join("settings.json"), existing_settings).unwrap();

    // Deploy
    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Verify the merged content
    let merged = ws.read_file("settings.json");
    assert!(
        merged.contains("\"model\": \"gpt-4\""),
        "existing model key should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("\"temperature\": 0.7"),
        "existing temperature key should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("\"customKey\": \"user-value\""),
        "existing customKey should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("\"mcpServers\""),
        "mcpServers should be added; got:\n{merged}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// JSONC merge
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deploy_mcp_to_existing_jsonc_preserves_comments() {
    // Scenario: User has a config.jsonc with comments explaining config.
    // Deploy should merge MCP servers while preserving comments.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Modify the mycode provider to output MCP to config.jsonc
    let config_path = ws.active_root_dir().join("local.config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let modified = config.replace(
        "target = \"{{ dir.workspace }}/.mycode/mcp.json\"",
        "target = \"{{ dir.workspace }}/config.jsonc\"",
    );
    fs::write(&config_path, &modified).expect("failed to write modified config");

    // Pre-populate config.jsonc with existing content including comments
    let existing_config = r#"{
  // This is the model configuration
  "model": "claude-3",
  /* Multi-line comment
     explaining temperature */
  "temperature": 0.5,
  "apiKey": "sk-123" // API key for authentication
}"#;
    fs::write(ws.root().join("config.jsonc"), existing_config).unwrap();

    // Deploy
    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Verify the merged content
    let merged = ws.read_file("config.jsonc");
    assert!(
        merged.contains("// This is the model configuration"),
        "single-line comment should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("Multi-line comment"),
        "multi-line comment should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("// API key for authentication"),
        "inline comment should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("\"model\": \"claude-3\""),
        "existing model key should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("\"mcpServers\""),
        "mcpServers should be added; got:\n{merged}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// TOML merge
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deploy_mcp_to_existing_toml_preserves_other_sections() {
    // Scenario: User has a config.toml with model and other sections.
    // Deploy should merge MCP servers while preserving existing sections.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Modify the mycode provider to output MCP to app-config.toml with a TOML template
    let config_path = ws.active_root_dir().join("local.config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let modified = config.replace(
        "target = \"{{ dir.workspace }}/.mycode/mcp.json\"",
        "target = \"{{ dir.workspace }}/app-config.toml\"",
    );
    // Also change the template to use a custom TOML template
    let modified = modified.replace(
        "template = \"{{ dir.application }}/templates/mycode/mcp.hbs\"",
        "template = \"{{ dir.workspace }}/mcp-toml.hbs\"",
    );
    fs::write(&config_path, &modified).expect("failed to write modified config");

    // Create a custom TOML template for MCP
    let toml_template = r#"
[[mcp_servers]]
name = "test-server"
command = "echo"
args = ["hello"]
"#;
    fs::write(ws.root().join("mcp-toml.hbs"), toml_template).unwrap();

    // Pre-populate app-config.toml with existing content
    let existing_config = r#"
[model]
name = "gpt-4"
temperature = 0.7

[api]
key = "sk-123"
base_url = "https://api.example.com"

# Custom user configuration
[custom]
enabled = true
"#;
    fs::write(ws.root().join("app-config.toml"), existing_config).unwrap();

    // Deploy
    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Verify the merged content
    let merged = ws.read_file("app-config.toml");
    assert!(
        merged.contains("[model]"),
        "model section should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("name = \"gpt-4\""),
        "model name should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("[api]"),
        "api section should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("key = \"sk-123\""),
        "api key should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("# Custom user configuration"),
        "comment should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("[custom]"),
        "custom section should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("[[mcp_servers]]"),
        "mcp_servers should be added; got:\n{merged}"
    );
    assert!(
        merged.contains("name = \"test-server\""),
        "test-server should be added; got:\n{merged}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Malformed file handling
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deploy_to_malformed_existing_file_skips_with_warning() {
    // Scenario: User has a malformed JSON file.
    // Deploy should skip merging and warn the user.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Modify the mycode provider to output MCP to settings.json
    let config_path = ws.active_root_dir().join("local.config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let modified = config.replace(
        "target = \"{{ dir.workspace }}/.mycode/mcp.json\"",
        "target = \"{{ dir.workspace }}/settings.json\"",
    );
    fs::write(&config_path, &modified).expect("failed to write modified config");

    // Pre-populate settings.json with malformed JSON
    let malformed_json = r#"{
  "model": "gpt-4",
  "temperature": 0.7,
  // Missing closing brace
"#;
    fs::write(ws.root().join("settings.json"), malformed_json).unwrap();

    // Deploy should succeed but skip the merge
    let result = ws.run_command(&["deploy", "--offline", "--no-gitignore"]);
    result.assert_success();

    // Verify stderr contains a warning about the malformed file
    assert!(
        result.stderr.contains("Skipping merge") || result.stderr.contains("malformed"),
        "stderr should contain warning about malformed file; got:\n{}",
        result.stderr
    );

    // Verify the file was not modified (still malformed)
    let content = ws.read_file("settings.json");
    assert_eq!(
        content, malformed_json,
        "malformed file should not be modified; got:\n{content}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Non-existent file (no merge)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deploy_to_nonexistent_file_writes_directly() {
    // Scenario: Target file does not exist.
    // Deploy should write the rendered content directly without merge.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Modify the mycode provider to output MCP to settings.json
    let config_path = ws.active_root_dir().join("local.config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let modified = config.replace(
        "target = \"{{ dir.workspace }}/.mycode/mcp.json\"",
        "target = \"{{ dir.workspace }}/settings.json\"",
    );
    fs::write(&config_path, &modified).expect("failed to write modified config");

    // Do NOT create settings.json - it should not exist

    // Deploy
    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Verify the file was created with the rendered content
    let content = ws.read_file("settings.json");
    assert!(
        content.contains("\"mcpServers\""),
        "mcpServers should be present; got:\n{content}"
    );
}

#[test]
fn deploy_mcp_to_existing_yaml_preserves_other_sections() {
    // Scenario: User has a config.yaml with model and other sections.
    // Deploy should merge MCP servers while preserving existing sections.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Modify the mycode provider to output MCP to app-config.yaml with a YAML template
    let config_path = ws.active_root_dir().join("local.config.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let modified = config.replace(
        "target = \"{{ dir.workspace }}/.mycode/mcp.json\"",
        "target = \"{{ dir.workspace }}/app-config.yaml\"",
    );
    // Also change the template to use a custom YAML template
    let modified = modified.replace(
        "template = \"{{ dir.application }}/templates/mycode/mcp.hbs\"",
        "template = \"{{ dir.workspace }}/mcp-yaml.hbs\"",
    );
    fs::write(&config_path, &modified).expect("failed to write modified config");

    // Create a custom YAML template for MCP
    let yaml_template = r#"mcpServers:
  test-server:
    command: echo
    args:
      - hello
"#;
    fs::write(ws.root().join("mcp-yaml.hbs"), yaml_template).unwrap();

    // Pre-populate app-config.yaml with existing content
    let existing_config = r#"model:
  name: gpt-4
  temperature: 0.7

api:
  key: sk-123
  base_url: https://api.example.com

# Custom user configuration
custom:
  enabled: true
"#;
    fs::write(ws.root().join("app-config.yaml"), existing_config).unwrap();

    // Deploy
    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    // Verify the merged content
    let merged = ws.read_file("app-config.yaml");
    assert!(
        merged.contains("model:"),
        "model section should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("name: gpt-4"),
        "model name should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("api:"),
        "api section should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("key: sk-123"),
        "api key should be preserved; got:\n{merged}"
    );
    // Note: serde_yaml doesn't preserve comments during parse/serialize
    assert!(
        merged.contains("custom:"),
        "custom section should be preserved; got:\n{merged}"
    );
    assert!(
        merged.contains("mcpServers:"),
        "mcpServers should be added; got:\n{merged}"
    );
    assert!(
        merged.contains("test-server:"),
        "test-server should be added; got:\n{merged}"
    );
}
