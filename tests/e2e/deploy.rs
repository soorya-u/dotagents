//! Tests for the `deploy` command.
//!
//! Groups covered
//! ──────────────
//!  4.  Output structure        – directories and files created by deploy
//!  5.  Rendered file contents  – frontmatter stripped, body present
//!  6.  Variable interpolation  – `{{ var.* }}` and `{{ env.* }}` expanded
//!  7.  Custom config scenarios – skills, disabled features, extra commands
//!  8.  Error handling          – bad config, missing workspace

use super::{LOCAL_CONFIG_COMMANDS_DISABLED, LOCAL_CONFIG_WITH_SKILL_PROVIDER_ONLY, TestWorkspace};

// ═════════════════════════════════════════════════════════════════════════════
// Group 4 – output structure
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn deploy_fails_without_workspace() {
    let ws = TestWorkspace::new();
    ws.run(&["deploy"]).assert_failure();
}

#[test]
fn deploy_succeeds_after_init() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["deploy"]).assert_success();
}

#[test]
fn deploy_creates_mycode_output_directory() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    assert!(
        ws.dir_exists(".mycode"),
        ".mycode/ should be created by deploy"
    );
}

#[test]
fn deploy_creates_commands_output_subdirectory() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    assert!(
        ws.dir_exists(".mycode/commands"),
        ".mycode/commands/ should be created by deploy"
    );
}

#[test]
fn deploy_creates_hello_command_output_file() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    // The command frontmatter declares `name: hello`; the target path template
    // uses `{{ command.name }}`, so the output file must be hello.md.
    assert!(ws.file_exists(".mycode/commands/hello.md"));
}

#[test]
fn deploy_creates_instructions_output_file() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    assert!(ws.file_exists(".mycode/instructions.md"));
}

#[test]
fn deploy_creates_mcp_output_file() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    assert!(ws.file_exists(".mycode/mcp.json"));
}

#[test]
fn deploy_creates_all_expected_output_files_in_one_pass() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    for rel in &[
        ".mycode/commands/hello.md",
        ".mycode/instructions.md",
        ".mycode/mcp.json",
    ] {
        assert!(ws.file_exists(rel), "expected deployed file: {rel}");
    }
}

#[test]
fn deploy_creates_nested_output_directories_automatically() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    assert!(
        !ws.dir_exists(".mycode"),
        ".mycode/ must not exist before deploy"
    );
    ws.run(&["deploy"]).assert_success();
    assert!(
        ws.dir_exists(".mycode/commands"),
        "deploy should create nested output dirs automatically"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 5 – rendered file contents
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn deploy_command_output_does_not_start_with_frontmatter() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    // command.hbs renders only `{{command.content}}`, which is the parsed body
    // *without* the YAML block.
    let content = ws.read(".mycode/commands/hello.md");
    assert!(
        !content.trim_start().starts_with("---"),
        "deployed command must not start with YAML frontmatter delimiters"
    );
}

#[test]
fn deploy_command_output_contains_source_body_text() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    let content = ws.read(".mycode/commands/hello.md");
    assert!(
        content.contains("Hello"),
        "deployed command should contain the heading from the source body"
    );
}

#[test]
fn deploy_command_filename_matches_frontmatter_name_field() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    // Target: `{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md`
    // Frontmatter: `name: hello`  →  output file: hello.md
    assert!(
        ws.file_exists(".mycode/commands/hello.md"),
        "output filename must be derived from the 'name' field in the command frontmatter"
    );
}

#[test]
fn deploy_mcp_output_is_valid_json() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    let content = ws.read(".mycode/mcp.json");
    let value: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("mcp.json is not valid JSON: {e}\ncontent:\n{content}"));
    assert!(value.is_object(), "mcp.json root must be a JSON object");
}

#[test]
fn deploy_mcp_output_has_mcp_servers_key() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    assert!(ws.read(".mycode/mcp.json").contains("mcpServers"));
}

#[test]
fn deploy_mcp_output_contains_both_configured_servers() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    let value: serde_json::Value = serde_json::from_str(&ws.read(".mycode/mcp.json")).unwrap();
    let servers = &value["mcpServers"];
    assert!(servers.is_object(), "'mcpServers' should be a JSON object");
    assert!(
        servers.get("server-stdio").is_some(),
        "'server-stdio' should be present"
    );
    assert!(
        servers.get("server-mcp").is_some(),
        "'server-mcp' should be present"
    );
}

#[test]
fn deploy_mcp_stdio_server_type_is_converted_to_local() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    // mcp.hbs: `{{#ifEq this.type "stdio"}}"local"{{else}}"{{this.type}}"{{/ifEq}}`
    // Parse the JSON and navigate directly to the field to avoid a brittle
    // substring match that could hit unrelated content (e.g. URLs).
    let mcp: serde_json::Value =
        serde_json::from_str(&ws.read(".mycode/mcp.json")).expect("mcp.json must be valid JSON");
    assert_eq!(
        mcp["mcpServers"]["server-stdio"]["type"],
        serde_json::json!("local"),
        "the stdio server type should be rendered as 'local'"
    );
}

#[test]
fn deploy_instructions_output_has_no_unrendered_handlebars() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    let content = ws.read(".mycode/instructions.md");
    assert!(
        !content.contains("{{"),
        "all Handlebars expressions should be rendered in deployed instructions"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 6 – variable interpolation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn deploy_instructions_interpolates_provider_variable_agent_name() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    // INSTRUCTIONS.md: `{{ var.agent_name }}`
    // local.config.toml: `variables = {agent_name = "Mycode"}`
    assert!(
        ws.read(".mycode/instructions.md").contains("Mycode"),
        "deployed instructions should contain the interpolated agent_name 'Mycode'"
    );
}

#[test]
fn deploy_instructions_interpolates_env_variable_app_name() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    // .env: APP_NAME=dotagents  (key lowercased → env.app_name)
    // INSTRUCTIONS.md: `{{ env.app_name }}`
    assert!(
        ws.read(".mycode/instructions.md").contains("dotagents"),
        "deployed instructions should contain the env variable value 'dotagents'"
    );
}

#[test]
fn deploy_custom_instructions_source_variable_is_rendered() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    // Override with a minimal template that uses both a var and an env var.
    ws.write_in_root_dir(
        "INSTRUCTIONS.md",
        "# Agent: {{ var.agent_name }}\nBuilt by {{ env.app_name }}.",
    );
    ws.run(&["deploy"]).assert_success();
    let content = ws.read(".mycode/instructions.md");
    assert!(
        content.contains("Mycode"),
        "var.agent_name should be substituted"
    );
    assert!(
        content.contains("dotagents"),
        "env.app_name should be substituted"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 7 – custom config scenarios
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn deploy_fails_when_skills_listed_as_feature_in_config() {
    // "skills" is not a valid feature name; the validator must reject it.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands", "instructions", "mcp", "skills"]

[targets]
custom = ["mycode"]

[providers.custom.mycode.commands]
template = "{{ dir.application }}/templates/mycode/command.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
"#,
    );
    let result = ws.run(&["deploy"]);
    result.assert_failure();
    let stderr_lc = result.stderr.to_lowercase();
    assert!(
        stderr_lc.contains("skills") || stderr_lc.contains("invalid"),
        "error should mention the invalid feature name; stderr: {}",
        result.stderr
    );
}

#[test]
fn deploy_succeeds_with_inert_skills_provider_config() {
    // A skills provider section is harmless when "skills" is absent from
    // `features`; the deploy pipeline skips the feature.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir("local.config.toml", LOCAL_CONFIG_WITH_SKILL_PROVIDER_ONLY);
    ws.run(&["deploy"]).assert_success();
    // Other features must still deploy correctly.
    assert!(ws.file_exists(".mycode/commands/hello.md"));
    assert!(ws.file_exists(".mycode/instructions.md"));
    assert!(ws.file_exists(".mycode/mcp.json"));
}

#[test]
fn deploy_does_not_create_skills_output_when_feature_not_in_features_list() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir("local.config.toml", LOCAL_CONFIG_WITH_SKILL_PROVIDER_ONLY);
    ws.run(&["deploy"]).assert_success();
    assert!(
        !ws.dir_exists(".mycode/skills"),
        ".mycode/skills/ should not be created when 'skills' is absent from features"
    );
}

#[test]
fn deploy_disabled_provider_feature_skips_output() {
    // `disabled = true` on the commands provider must prevent any command files
    // from being written.  Instructions and MCP should still deploy.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir("local.config.toml", LOCAL_CONFIG_COMMANDS_DISABLED);
    ws.run(&["deploy"]).assert_success();
    assert!(
        !ws.dir_exists(".mycode/commands"),
        ".mycode/commands/ should not exist when the commands provider is disabled"
    );
    assert!(ws.file_exists(".mycode/instructions.md"));
    assert!(ws.file_exists(".mycode/mcp.json"));
}

#[test]
fn deploy_multiple_command_files_are_all_deployed() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "commands/greet.md",
        "---\nname: greet\ndescription: A second command.\n---\n\n# Greet\n\nSay hello.\n",
    );
    ws.run(&["deploy"]).assert_success();
    assert!(ws.file_exists(".mycode/commands/hello.md"));
    assert!(
        ws.file_exists(".mycode/commands/greet.md"),
        "the second command should also be deployed"
    );
}

#[test]
fn deploy_second_command_contains_correct_body_and_no_frontmatter() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "commands/greet.md",
        "---\nname: greet\ndescription: A second command.\n---\n\n# Greet Command\n\nSay hello.\n",
    );
    ws.run(&["deploy"]).assert_success();
    let content = ws.read(".mycode/commands/greet.md");
    assert!(content.contains("Greet Command"));
    assert!(!content.trim_start().starts_with("---"));
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 8 – error handling
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn deploy_fails_with_invalid_toml_in_global_config() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir("config.toml", "this is not @#! valid toml !!!");
    ws.run(&["deploy"]).assert_failure();
}

#[test]
fn deploy_error_is_reported_on_stderr() {
    // When deploy fails (no init), the error message must go to stderr via
    // `log::error!` / `display_error`.
    let ws = TestWorkspace::new();
    let result = ws.run(&["deploy"]);
    result.assert_failure();
    assert!(
        !result.stderr.is_empty(),
        "error output should be written to stderr"
    );
}
