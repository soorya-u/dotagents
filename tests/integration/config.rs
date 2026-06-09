//! Integration tests for AppConfig merge scenarios.

use super::{TestWorkspace, init_with_mycode_provider};
use std::fs;

// ─────────────────────────────────────────────────────────────────────────────
// Feature override
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn feature_subset_limits_deployed_artifacts() {
    // local config restricts features to commands-only; instructions and mcp should not deploy
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Rewrite local config with only commands feature enabled.
    let config_path = ws.active_root_dir().join("local.config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let restricted = original.replace(
        r#"features = [
    "command",
    "instruction",
    "mcp",
    "skill",
]"#,
        r#"features = ["command"]"#,
    );
    fs::write(&config_path, restricted).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    assert!(
        ws.file_exists(".mycode/commands/hello.md"),
        "commands should be deployed when feature is enabled"
    );
    assert!(
        !ws.file_exists(".mycode/instructions.md"),
        "instructions.md should not be deployed when instructions feature is absent"
    );
    assert!(
        !ws.file_exists(".mycode/mcp.json"),
        "mcp.json should not be deployed when mcp feature is absent"
    );
    assert!(
        !ws.file_exists(".mycode/skills/hello-skill/SKILL.md"),
        "skills should not be deployed when skills feature is absent"
    );
}

#[test]
fn features_all_enabled_deploys_all_artifacts() {
    // default advanced enables all features; all output files should appear
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    assert!(
        ws.file_exists(".mycode/commands/hello.md"),
        "commands deployed"
    );
    assert!(
        ws.file_exists(".mycode/instructions.md"),
        "instructions deployed"
    );
    assert!(ws.file_exists(".mycode/mcp.json"), "mcp deployed");
    assert!(
        ws.file_exists(".mycode/skills/hello-skill/SKILL.md"),
        "skill deployed"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Provider disabled flag
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn provider_disabled_true_skips_that_feature_output() {
    // setting disabled = true on the mycode.instructions provider prevents instructions deployment
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    let config_path = ws.active_root_dir().join("local.config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    // Insert disabled = true into the instructions provider section.
    let patched = original.replace(
        "[providers.mycode.instructions]",
        "[providers.mycode.instructions]\ndisabled = true",
    );
    fs::write(&config_path, patched).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    assert!(
        !ws.file_exists(".mycode/instructions.md"),
        "disabled provider should not produce output"
    );
    assert!(
        ws.file_exists(".mycode/commands/hello.md"),
        "non-disabled provider should still produce output"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Variable deep-merge
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn local_provider_variables_override_global_variables() {
    // The mycode provider sets agent_name = "Mycode"; global sets agent_name = "my agent".
    // Provider-level variables should win, so instructions output should contain "Mycode".
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Enable template mode for instruction/command so variables are substituted.
    let config_path = ws.active_root_dir().join("local.config.toml");
    let mut original = fs::read_to_string(&config_path).unwrap();
    original.push_str("\n[feature-maps.instruction]\nmode = \"template\"\n");
    original.push_str("[feature-maps.command]\nmode = \"template\"\n");
    fs::write(&config_path, original).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let instructions = ws.read_file(".mycode/instructions.md");
    assert!(
        instructions.contains("Mycode"),
        "provider-level agent_name should appear in output; got: {instructions}"
    );
    assert!(
        !instructions.contains("my agent"),
        "global agent_name should be overridden by provider-level variable; got: {instructions}"
    );
}

#[test]
fn global_variable_used_when_no_provider_override() {
    // When no provider-level agent_name variable exists, global variable is used.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    let config_path = ws.active_root_dir().join("local.config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    // Remove provider-level variable overrides from all sections.
    let mut patched = original
        .replace("variables = {agent_name = \"Mycode\"}", "")
        .replace(
            "variables = { \"agent_name\" = \"my agent\" }",
            "variables = { \"agent_name\" = \"GlobalAgent\" }",
        );
    // Enable template mode for instruction so variables are substituted.
    patched.push_str("\n[feature-maps.instruction]\nmode = \"template\"\n");
    patched.push_str("[feature-maps.command]\nmode = \"template\"\n");
    fs::write(&config_path, patched).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    let instructions = ws.read_file(".mycode/instructions.md");
    assert!(
        instructions.contains("GlobalAgent"),
        "global agent_name should be used when no provider override; got: {instructions}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Targets behaviour
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn empty_targets_deploys_only_inline_providers() {
    // targets = [] means no remote registry fetch; inline mycode provider still deploys
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws); // already patches targets = []

    ws.run_command(&["deploy", "--offline", "--no-gitignore"])
        .assert_success();

    assert!(
        ws.dir_exists(".mycode"),
        ".mycode/ should be created by the inline mycode provider"
    );
}
