//! Smoke tests for the `init` command.

use super::TestWorkspace;
use std::path::Path;

#[test]
fn init_creates_basic_structure() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();

    let root = ws.active_root_dir();
    assert!(root.exists(), "root dir should exist at {}", root.display());

    let d = ws.root_dir_name();
    assert!(ws.file_exists(Path::new(&format!("{d}/config.toml"))));
    assert!(ws.file_exists(Path::new(&format!("{d}/.env"))));
    assert!(ws.file_exists(Path::new(&format!("{d}/.gitignore"))));
    assert!(ws.file_exists(Path::new(&format!("{d}/INSTRUCTIONS.md"))));
    assert!(ws.file_exists(Path::new(&format!("{d}/mcp.jsonc"))));
}

#[test]
fn init_creates_commands_directory_with_sample_file() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();

    let d = ws.root_dir_name();
    let commands_path = format!("{d}/commands");
    assert!(ws.dir_exists(Path::new(&commands_path)));
    assert!(
        !ws.list_files(Path::new(&commands_path)).is_empty(),
        "commands/ should contain a sample file"
    );
}

#[test]
fn init_creates_skills_directory_with_sample_file() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();

    let d = ws.root_dir_name();
    let skills_path = format!("{d}/skills");
    assert!(ws.dir_exists(Path::new(&skills_path)));
    assert!(
        !ws.list_files(Path::new(&skills_path)).is_empty(),
        "skills/ should contain a sample file"
    );
}

#[test]
fn init_creates_templates_directory() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();

    let d = ws.root_dir_name();
    assert!(ws.dir_exists(Path::new(&format!("{d}/templates"))));
    assert!(ws.dir_exists(Path::new(&format!("{d}/templates/mycode"))));
}

#[test]
fn init_no_command_flag_leaves_commands_dir_empty() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--no-command"]).assert_success();

    let d = ws.root_dir_name();
    assert!(
        ws.list_files(Path::new(&format!("{d}/commands")))
            .is_empty(),
        "commands/ should be empty when --no-command is used"
    );
}

#[test]
fn init_no_instruction_flag_skips_instructions_file() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--no-instruction"])
        .assert_success();

    let d = ws.root_dir_name();
    assert!(
        !ws.file_exists(Path::new(&format!("{d}/INSTRUCTIONS.md"))),
        "INSTRUCTIONS.md should not exist when --no-instruction is used"
    );
}

#[test]
fn init_no_mcp_flag_skips_mcp_file() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--no-mcp"]).assert_success();

    let d = ws.root_dir_name();
    assert!(
        !ws.file_exists(Path::new(&format!("{d}/mcp.jsonc"))),
        "mcp.jsonc should not exist when --no-mcp is used"
    );
}

#[test]
fn init_no_skill_flag_leaves_skills_dir_empty() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--no-skill"]).assert_success();

    let d = ws.root_dir_name();
    assert!(
        ws.list_files(Path::new(&format!("{d}/skills"))).is_empty(),
        "skills/ should be empty when --no-skill is used"
    );
}

#[test]
fn init_combination_of_no_flags() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--no-command", "--no-skill", "--no-mcp"])
        .assert_success();

    let d = ws.root_dir_name();
    assert!(ws.dir_exists(Path::new(d)));
    assert!(
        ws.list_files(Path::new(&format!("{d}/commands")))
            .is_empty()
    );
    assert!(ws.list_files(Path::new(&format!("{d}/skills"))).is_empty());
    assert!(!ws.file_exists(Path::new(&format!("{d}/mcp.jsonc"))));
}

#[test]
fn init_fails_when_directory_already_exists_without_force() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();

    #[cfg(not(debug_assertions))]
    ws.run_command(&["init"]).assert_failure();

    #[cfg(debug_assertions)]
    ws.run_command(&["init"]).assert_success();
}

#[test]
fn init_force_overwrites_existing_directory() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();

    #[cfg(not(debug_assertions))]
    ws.run_command(&["init", "--force"]).assert_success();
    #[cfg(debug_assertions)]
    ws.run_command(&["init"]).assert_success();

    let d = ws.root_dir_name();
    assert!(ws.dir_exists(Path::new(d)));
    assert!(ws.file_exists(Path::new(&format!("{d}/config.toml"))));
}

#[test]
fn init_config_toml_has_expected_sections() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();

    let d = ws.root_dir_name();
    let content = ws.read_file(Path::new(&format!("{d}/config.toml")));
    // config.toml uses `features = [...]` (inline array), not a `[features]` section.
    assert!(content.contains("features"), "should declare features");
    assert!(content.contains("targets"), "should declare targets");
}

#[test]
fn init_env_file_is_readable_and_non_empty() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();

    let d = ws.root_dir_name();
    let content = ws.read_file(Path::new(&format!("{d}/.env")));
    assert!(!content.is_empty(), ".env file should not be empty");
}

#[test]
fn init_gitignore_excludes_local_config() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();

    let d = ws.root_dir_name();
    let content = ws.read_file(Path::new(&format!("{d}/.gitignore")));
    assert!(
        content.contains("local.config.toml"),
        ".gitignore should exclude local.config.toml"
    );
}
