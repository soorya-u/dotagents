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
