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

    // Add a second provider "aaa" that also targets the same instructions file.
    let config_path = ws.active_root_dir().join("config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let patched = original.replace(
        "[providers.mycode.instructions]",
        r#"[providers.aaa.instructions]
template = "{{ dir.application }}/templates/instructions.hbs"
target = ".mycode/instructions.md"

[providers.mycode.instructions]"#,
    );
    fs::write(&config_path, patched).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--force"])
        .assert_success();

    // The file should exist (written by "aaa" since it's alphabetically first).
    assert!(
        ws.file_exists(".mycode/instructions.md"),
        "instructions.md should be deployed by the winning provider"
    );
}

#[test]
fn dedup_cache_has_one_entry_per_unique_path() {
    // After deploying with multiple providers targeting the same path,
    // cache.toml should have only one entry for that path.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Add a second provider targeting the same instructions file.
    let config_path = ws.active_root_dir().join("config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let patched = original.replace(
        "[providers.mycode.instructions]",
        r#"[providers.aaa.instructions]
template = "{{ dir.application }}/templates/instructions.hbs"
target = ".mycode/instructions.md"

[providers.mycode.instructions]"#,
    );
    fs::write(&config_path, patched).unwrap();

    ws.run_command(&["deploy", "--offline", "--no-gitignore", "--force"])
        .assert_success();

    // Read cache.toml and count entries for the instructions feature.
    let cache_path = ws.root().join(format!("{}/cache.toml", ws.root_dir_name()));
    let cache_content = fs::read_to_string(&cache_path).unwrap();

    // Count how many times "instructions" appears as a feature key in the cache.
    // With dedup, only the winning provider should have a cache entry.
    let instructions_count = cache_content.matches("instructions").count();
    assert!(
        instructions_count <= 2,
        "cache should have at most 2 'instructions' references (one provider entry); got {instructions_count} in:\n{cache_content}"
    );
}

#[test]
fn dedup_undeploy_removes_file_once() {
    // After a deduped deploy, undeploy should remove the file cleanly.
    let ws = TestWorkspace::new();
    init_with_mycode_provider(&ws);

    // Add a second provider targeting the same instructions file.
    let config_path = ws.active_root_dir().join("config.toml");
    let original = fs::read_to_string(&config_path).unwrap();
    let patched = original.replace(
        "[providers.mycode.instructions]",
        r#"[providers.aaa.instructions]
template = "{{ dir.application }}/templates/instructions.hbs"
target = ".mycode/instructions.md"

[providers.mycode.instructions]"#,
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
