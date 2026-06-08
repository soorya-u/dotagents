//! Integration tests for provider deduplication when multiple providers target the same file.

use super::{TestWorkspace, init_with_mycode_provider};
use std::fs;

// ─────────────────────────────────────────────────────────────────────────────
// Dedup: multiple providers targeting same path
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn dedup_single_path_multiple_providers_only_winner_writes() {
    // Configure two providers (mycode + a second inline provider) both targeting
    // the same instructions file. Only the alphabetically first provider should write.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Add a second provider "aaa" that also targets the same instructions file
    // with a distinct agent_name so we can identify the winner by file content.
    let config_path = ws.active_root_dir().join("local.config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let mut patched = original.replace(
        "\n[providers.mycode.instructions]\ntemplate = \"{{ dir.application }}/templates/mycode/instructions.hbs\"\ntarget = \"{{ dir.workspace }}/.mycode/instructions.md\"\nvariables = {agent_name = \"Mycode\"}",
        "\n[providers.aaa.instructions]\ntemplate = \"{{ dir.application }}/templates/mycode/instructions.hbs\"\ntarget = \"{{ dir.workspace }}/.mycode/instructions.md\"\nvariables = {agent_name = \"aaa-provider\"}\n\n[providers.mycode.instructions]\ntemplate = \"{{ dir.application }}/templates/mycode/instructions.hbs\"\ntarget = \"{{ dir.workspace }}/.mycode/instructions.md\"\nvariables = {agent_name = \"mycode-provider\"}",
    );
    // Enable template mode so provider variables are substituted.
    patched.push_str("\n[feature-maps.instruction]\nmode = \"template\"\n");
    patched.push_str("[feature-maps.command]\nmode = \"template\"\n");
    fs::write(&config_path, patched).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--force"])
        .assert_success();

    // "aaa" is alphabetically first, so its agent_name should appear in the file.
    let content = ws.read_file(".mycode/instructions.md");
    assert!(
        content.contains("aaa-provider"),
        "instructions.md should contain winner's agent_name 'aaa-provider', got:\n{content}"
    );
    assert!(
        !content.contains("mycode-provider"),
        "instructions.md should NOT contain loser's agent_name 'mycode-provider', got:\n{content}"
    );
}

#[test]
fn dedup_cache_has_one_entry_per_unique_path() {
    // After deploying with multiple providers targeting the same path,
    // cache.toml should have only one entry for that path.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Add a second provider targeting the same instructions file.
    let config_path = ws.active_root_dir().join("local.config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let patched = original.replace(
        "\n[providers.mycode.instructions]\ntemplate = \"{{ dir.application }}/templates/mycode/instructions.hbs\"\ntarget = \"{{ dir.workspace }}/.mycode/instructions.md\"\nvariables = {agent_name = \"Mycode\"}",
        "\n[providers.aaa.instructions]\ntemplate = \"{{ dir.application }}/templates/mycode/instructions.hbs\"\ntarget = \"{{ dir.workspace }}/.mycode/instructions.md\"\nvariables = {agent_name = \"aaa-provider\"}\n\n[providers.mycode.instructions]\ntemplate = \"{{ dir.application }}/templates/mycode/instructions.hbs\"\ntarget = \"{{ dir.workspace }}/.mycode/instructions.md\"\nvariables = {agent_name = \"mycode-provider\"}",
    );
    fs::write(&config_path, patched).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--force"])
        .assert_success();

    // Read cache.toml and parse it to verify exactly one entry for the instructions feature.
    let cache_path = ws.root().join(format!("{}/cache.toml", ws.root_dir_name()));
    let cache_content = fs::read_to_string(&cache_path).unwrap();
    let cache: toml::Value =
        toml::from_str(&cache_content).expect("cache.toml should be valid TOML");

    // Cache structure: [providers.<name>.instruction.<item_key>]
    // Count how many top-level provider keys contain an "instruction" sub-table.
    let instruction_count = cache
        .get("providers")
        .and_then(|p| p.as_table())
        .map(|providers| {
            providers
                .values()
                .filter(|provider_table| provider_table.get("instruction").is_some())
                .count()
        })
        .unwrap_or(0);
    assert_eq!(
        instruction_count, 1,
        "cache should have exactly 1 provider with 'instruction' entry; got {instruction_count} in:\n{cache_content}"
    );
}

#[test]
fn dedup_undeploy_removes_file_once() {
    // After a deduped deploy, undeploy should remove the file cleanly.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Add a second provider targeting the same instructions file.
    let config_path = ws.active_root_dir().join("local.config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let patched = original.replace(
        "\n[providers.mycode.instructions]\ntemplate = \"{{ dir.application }}/templates/mycode/instructions.hbs\"\ntarget = \"{{ dir.workspace }}/.mycode/instructions.md\"\nvariables = {agent_name = \"Mycode\"}",
        "\n[providers.aaa.instructions]\ntemplate = \"{{ dir.application }}/templates/mycode/instructions.hbs\"\ntarget = \"{{ dir.workspace }}/.mycode/instructions.md\"\nvariables = {agent_name = \"aaa-provider\"}\n\n[providers.mycode.instructions]\ntemplate = \"{{ dir.application }}/templates/mycode/instructions.hbs\"\ntarget = \"{{ dir.workspace }}/.mycode/instructions.md\"\nvariables = {agent_name = \"mycode-provider\"}",
    );
    fs::write(&config_path, patched).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--force"])
        .assert_success();

    assert!(
        ws.file_exists(".mycode/instructions.md"),
        "instructions.md should exist after deploy"
    );

    ws.run_command(&["undeploy", "--force"]).assert_success();

    assert!(
        !ws.file_exists(".mycode/instructions.md"),
        "instructions.md should be removed after undeploy"
    );
}
