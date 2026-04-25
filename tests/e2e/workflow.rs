//! Full end-to-end workflow tests.
//!
//! These tests walk through multi-step scenarios from init to deploy and
//! verify the complete output tree, rendered content, and update behaviour.

use super::{LOCAL_CONFIG_WITH_SKILL_PROVIDER_ONLY, TestWorkspace};

#[test]
fn full_workflow_init_deploy_produces_complete_output_tree() {
    let ws = TestWorkspace::new();

    // ── Step 1: init ──────────────────────────────────────────────────────
    ws.run(&["init"]).assert_success();
    let d = ws.root_dir_name();
    assert!(ws.file_exists(format!("{d}/config.toml")));
    assert!(ws.file_exists(format!("{d}/local.config.toml")));
    assert!(ws.file_exists(format!("{d}/commands/hello.md")));
    assert!(ws.file_exists(format!("{d}/skills/hello-skill/SKILL.md")));
    assert!(ws.file_exists(format!("{d}/INSTRUCTIONS.md")));
    assert!(ws.file_exists(format!("{d}/mcp.jsonc")));

    // ── Step 2: deploy ────────────────────────────────────────────────────
    ws.run(&["deploy"]).assert_success();
    assert!(ws.file_exists(".mycode/commands/hello.md"));
    assert!(ws.file_exists(".mycode/instructions.md"));
    assert!(ws.file_exists(".mycode/mcp.json"));
    assert!(ws.file_exists(".mycode/skills/hello-skill/SKILLS.md"));

    // ── Step 3: spot-check rendered content ──────────────────────────────
    let cmd = ws.read(".mycode/commands/hello.md");
    assert!(
        !cmd.trim_start().starts_with("---"),
        "no frontmatter in deployed command"
    );
    assert!(cmd.contains("Hello"), "deployed command body present");

    let ins = ws.read(".mycode/instructions.md");
    assert!(
        !ins.contains("{{"),
        "all template vars rendered in instructions"
    );
    assert!(ins.contains("Mycode"), "agent_name interpolated");
    assert!(ins.contains("dotagents"), "env.app_name interpolated");

    let mcp: serde_json::Value =
        serde_json::from_str(&ws.read(".mycode/mcp.json")).expect("mcp.json must be valid JSON");
    assert!(mcp["mcpServers"].is_object());
}

#[test]
fn full_workflow_with_inert_skill_provider_config() {
    // A skills provider section in local.config.toml is harmless when "skills"
    // is absent from `features`.  All other features must deploy correctly.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir("local.config.toml", LOCAL_CONFIG_WITH_SKILL_PROVIDER_ONLY);
    ws.run(&["deploy"]).assert_success();

    assert!(ws.file_exists(".mycode/commands/hello.md"));
    assert!(ws.file_exists(".mycode/instructions.md"));
    assert!(ws.file_exists(".mycode/mcp.json"));
    assert!(
        !ws.dir_exists(".mycode/skills"),
        "skills output must be absent when 'skills' is not in features"
    );
}

#[test]
fn full_workflow_init_modify_source_redeploy_picks_up_change() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["deploy"]).assert_success();

    // Replace the instructions source.
    ws.write_in_root_dir(
        "INSTRUCTIONS.md",
        "# My Custom Instructions\n\nEdited content.",
    );
    ws.run(&["deploy"]).assert_success();

    let ins = ws.read(".mycode/instructions.md");
    assert!(
        ins.contains("My Custom Instructions"),
        "re-deploy should pick up the modified source; got: {ins}"
    );
}
