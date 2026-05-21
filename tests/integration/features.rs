//! Integration tests for feature source-file format and deploy output.

use super::{TestWorkspace, init_with_mycode_provider};

// ─────────────────────────────────────────────────────────────────────────────
// CommandFeature source file format
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn add_command_source_has_yaml_frontmatter() {
    // `add command` should create a markdown file with YAML frontmatter.
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--features", "commands,instructions,mcp,skills"])
        .assert_success();
    ws.run_command(&[
        "commands",
        "new",
        "greet",
        "--description",
        "Greet the user",
    ])
    .assert_success();

    let d = ws.root_dir_name();
    let source = ws.read_file(format!("{d}/commands/greet.md"));

    assert!(
        source.starts_with("---"),
        "command source should start with YAML frontmatter; got:\n{source}"
    );
    // serde_yaml may quote values; extract the block between the --- delimiters.
    let frontmatter = source.splitn(3, "---").nth(1).unwrap_or("");
    assert!(
        frontmatter.contains("greet"),
        "frontmatter should include command name; got:\n{source}"
    );
    assert!(
        frontmatter.contains("Greet the user"),
        "frontmatter should include description; got:\n{source}"
    );
}

#[test]
fn add_command_with_category_and_tags_includes_them_in_source() {
    // Flags --category and --tags should appear in the generated frontmatter.
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--features", "commands,instructions,mcp,skills"])
        .assert_success();
    ws.run_command(&[
        "commands",
        "new",
        "tag-cmd",
        "--description",
        "Tagged",
        "--category",
        "Utilities",
        "--tags",
        "tag1,tag2",
    ])
    .assert_success();

    let d = ws.root_dir_name();
    let source = ws.read_file(format!("{d}/commands/tag-cmd.md"));
    assert!(
        source.contains("Utilities"),
        "category should appear in source frontmatter; got:\n{source}"
    );
    assert!(
        source.contains("tag1"),
        "tags should appear in source frontmatter; got:\n{source}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SkillFeature source file format
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn add_skill_source_has_expected_metadata() {
    // `add skill` should create SKILL.md with YAML frontmatter inside a subdirectory.
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--features", "commands,instructions,mcp,skills"])
        .assert_success();
    ws.run_command(&["skills", "new", "my-skill", "--description", "A test skill"])
        .assert_success();

    let d = ws.root_dir_name();
    let source = ws.read_file(format!("{d}/skills/my-skill/SKILL.md"));

    assert!(
        source.starts_with("---"),
        "skill source should start with YAML frontmatter; got:\n{source}"
    );
    // serde_yaml may quote values; extract the block between the --- delimiters.
    let frontmatter = source.splitn(3, "---").nth(1).unwrap_or("");
    assert!(
        frontmatter.contains("my-skill"),
        "frontmatter should include skill name; got:\n{source}"
    );
    assert!(
        frontmatter.contains("A test skill"),
        "frontmatter should include description; got:\n{source}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// McpFeature source file format
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn mcp_source_file_is_non_empty() {
    // `init` creates a non-empty mcp.jsonc source file.
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--features", "commands,instructions,mcp,skills"])
        .assert_success();

    let d = ws.root_dir_name();
    let source = ws.read_file(format!("{d}/mcp.jsonc"));

    assert!(
        !source.is_empty(),
        "mcp.jsonc source should be non-empty after init"
    );
}

#[test]
fn mcp_source_supports_expanded_server_fields() {
    // Expanded MCP source fields should deploy through the local provider.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);
    let d = ws.root_dir_name();
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
              "headers": {"Authorization": "Bearer token"},
              "enabledTools": ["search"],
              "disabled": true
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

    let content = ws.read_file(".mycode/mcp.json");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("deployed mcp.json should be valid JSON");
    assert_eq!(
        parsed["mcpServers"]["sse-server"]["type"],
        serde_json::json!("sse")
    );
    assert_eq!(
        parsed["mcpServers"]["stdio-server"]["tools"],
        serde_json::json!(["read"])
    );
    assert_eq!(
        parsed["mcpServers"]["stdio-server"]["disabledTools"],
        serde_json::json!(["delete"])
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Deploy output — one file per feature item
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deploy_creates_one_output_file_per_command() {
    // Each command source file produces one deployed output file.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Add a second command beyond the scaffold's hello.md.
    ws.run_command(&["commands", "new", "greet", "--description", "Greet user"])
        .assert_success();

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let files = ws.list_files(".mycode/commands");
    assert!(
        files.len() >= 2,
        "each source command should produce an output file; found: {files:?}"
    );
    assert!(
        files.iter().any(|f| f == "hello.md"),
        "hello.md should be in output; found: {files:?}"
    );
    assert!(
        files.iter().any(|f| f == "greet.md"),
        "greet.md should be in output; found: {files:?}"
    );
}

#[test]
fn deploy_creates_skill_output_under_skill_name_directory() {
    // Each skill is deployed to `.mycode/skills/<name>/SKILL.md`.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    assert!(
        ws.file_exists(".mycode/skills/hello-skill/SKILL.md"),
        "skill output should be at .mycode/skills/<name>/SKILL.md"
    );
}

#[test]
fn deployed_command_output_name_matches_source_name() {
    // The output filename corresponds to the command's name field.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["commands", "new", "custom-cmd", "--description", "Custom"])
        .assert_success();

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    assert!(
        ws.file_exists(".mycode/commands/custom-cmd.md"),
        "deployed command file should carry the command name; files: {:?}",
        ws.list_files(".mycode/commands")
    );
}
