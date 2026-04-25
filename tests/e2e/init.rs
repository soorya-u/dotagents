//! Tests for the `init` command.
//!
//! Groups covered
//! ──────────────
//!  1.  File-tree creation  – every file and directory that `init` must produce
//!  2.  File content        – key fields present and correct in each created file
//!  3.  Behavioural flags   – --no-command, --no-skill, --no-mcp,
//!                            --no-instruction, --force

use super::TestWorkspace;

// ═════════════════════════════════════════════════════════════════════════════
// Group 1 – file-tree creation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn init_creates_root_directory() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    assert!(
        ws.root_dir().is_dir(),
        "root dir '{}' should exist after init",
        ws.root_dir().display()
    );
}

#[test]
fn init_creates_core_config_files() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let d = ws.root_dir_name();
    for rel in &[
        format!("{d}/config.toml"),
        format!("{d}/local.config.toml"),
        format!("{d}/.env"),
        format!("{d}/.env.example"),
        format!("{d}/.gitignore"),
    ] {
        assert!(ws.file_exists(rel), "expected file: {rel}");
    }
}

#[test]
fn init_creates_feature_source_files() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let d = ws.root_dir_name();
    assert!(ws.file_exists(format!("{d}/INSTRUCTIONS.md")));
    assert!(ws.file_exists(format!("{d}/mcp.jsonc")));
    assert!(ws.file_exists(format!("{d}/commands/hello.md")));
    assert!(ws.file_exists(format!("{d}/skills/hello-skill.md")));
}

#[test]
fn init_creates_all_mycode_template_files() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let d = ws.root_dir_name();
    for rel in &[
        format!("{d}/templates/mycode/command.hbs"),
        format!("{d}/templates/mycode/skill.hbs"),
        format!("{d}/templates/mycode/instructions.hbs"),
        format!("{d}/templates/mycode/mcp.hbs"),
    ] {
        assert!(ws.file_exists(rel), "expected template file: {rel}");
    }
}

#[test]
fn init_creates_subdirectory_structure() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let d = ws.root_dir_name();
    assert!(ws.dir_exists(format!("{d}/commands")));
    assert!(ws.dir_exists(format!("{d}/skills")));
    assert!(ws.dir_exists(format!("{d}/templates")));
    assert!(ws.dir_exists(format!("{d}/templates/mycode")));
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 2 – file content
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn init_config_toml_has_features_and_targets_sections() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let content = ws.read(format!("{}/config.toml", ws.root_dir_name()));
    assert!(
        content.contains("features"),
        "config.toml should declare features"
    );
    assert!(
        content.contains("targets"),
        "config.toml should declare targets"
    );
}

#[test]
fn init_config_toml_enables_known_features() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let content = ws.read(format!("{}/config.toml", ws.root_dir_name()));
    for feature in &["commands", "instructions", "mcp"] {
        assert!(
            content.contains(feature),
            "config.toml should list the '{feature}' feature"
        );
    }
}

#[test]
fn init_env_file_sets_app_name() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let content = ws.read(format!("{}/.env", ws.root_dir_name()));
    assert!(
        content.contains("APP_NAME"),
        ".env should define APP_NAME; got: {content:?}"
    );
}

#[test]
fn init_env_and_env_example_are_identical() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let d = ws.root_dir_name();
    assert_eq!(
        ws.read(format!("{d}/.env")),
        ws.read(format!("{d}/.env.example")),
        ".env and .env.example should have identical contents"
    );
}

#[test]
fn init_gitignore_excludes_local_config_and_env() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let content = ws.read(format!("{}/.gitignore", ws.root_dir_name()));
    assert!(
        content.contains("local.config.toml"),
        ".gitignore should exclude local.config.toml"
    );
    assert!(
        content.contains(".env"),
        ".gitignore should exclude .env; got: {content:?}"
    );
}

#[test]
fn init_command_file_has_yaml_frontmatter_with_name_and_description() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let content = ws.read(format!("{}/commands/hello.md", ws.root_dir_name()));
    assert!(
        content.starts_with("---"),
        "command file should start with YAML frontmatter"
    );
    assert!(
        content.contains("name:"),
        "frontmatter should include a 'name' field"
    );
    assert!(
        content.contains("description:"),
        "frontmatter should include a 'description' field"
    );
}

#[test]
fn init_skill_file_has_yaml_frontmatter_with_name_and_description() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let content = ws.read(format!("{}/skills/hello-skill.md", ws.root_dir_name()));
    assert!(
        content.starts_with("---"),
        "skill file should start with YAML frontmatter"
    );
    assert!(
        content.contains("name:"),
        "frontmatter should include a 'name' field"
    );
    assert!(
        content.contains("description:"),
        "frontmatter should include a 'description' field"
    );
}

#[test]
fn init_mcp_file_declares_servers() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let content = ws.read(format!("{}/mcp.jsonc", ws.root_dir_name()));
    assert!(
        content.contains("servers"),
        "mcp.jsonc should contain a 'servers' key"
    );
    assert!(
        content.contains("stdio") || content.contains("server-stdio"),
        "mcp.jsonc should reference a stdio server"
    );
    assert!(
        content.contains("http") || content.contains("server-mcp"),
        "mcp.jsonc should reference an http server"
    );
}

#[test]
fn init_instructions_md_contains_unrendered_handlebars_variables() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    // The source file is a template – it must still contain raw Handlebars syntax.
    let content = ws.read(format!("{}/INSTRUCTIONS.md", ws.root_dir_name()));
    assert!(
        content.contains("{{"),
        "INSTRUCTIONS.md source should contain Handlebars expressions before rendering"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 3 – behavioural flags
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn init_no_command_produces_no_command_files() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "--no-command"]).assert_success();
    let d = ws.root_dir_name();
    // The commands/ directory is only created implicitly when a file is written
    // into it.  With --no-command no file is written, so the dir may not exist.
    assert!(
        ws.dir_entries(format!("{d}/commands")).is_empty(),
        "no command files should be present when --no-command is used"
    );
}

#[test]
fn init_no_skill_produces_no_skill_files() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "--no-skill"]).assert_success();
    let d = ws.root_dir_name();
    assert!(
        ws.dir_entries(format!("{d}/skills")).is_empty(),
        "no skill files should be present when --no-skill is used"
    );
}

#[test]
fn init_no_mcp_skips_mcp_jsonc() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "--no-mcp"]).assert_success();
    assert!(
        !ws.file_exists(format!("{}/mcp.jsonc", ws.root_dir_name())),
        "mcp.jsonc should not be created when --no-mcp is used"
    );
}

#[test]
fn init_no_instruction_skips_instructions_md() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "--no-instruction"]).assert_success();
    assert!(
        !ws.file_exists(format!("{}/INSTRUCTIONS.md", ws.root_dir_name())),
        "INSTRUCTIONS.md should not be created when --no-instruction is used"
    );
}

#[test]
fn init_all_no_flags_keeps_config_and_templates_but_no_feature_files() {
    let ws = TestWorkspace::new();
    ws.run(&[
        "init",
        "--no-command",
        "--no-skill",
        "--no-mcp",
        "--no-instruction",
    ])
    .assert_success();
    let d = ws.root_dir_name();

    // Core config files are always written.
    assert!(ws.file_exists(format!("{d}/config.toml")));
    assert!(ws.file_exists(format!("{d}/local.config.toml")));
    assert!(ws.file_exists(format!("{d}/.env")));
    assert!(ws.file_exists(format!("{d}/.gitignore")));

    // Feature files must be absent.
    assert!(!ws.file_exists(format!("{d}/INSTRUCTIONS.md")));
    assert!(!ws.file_exists(format!("{d}/mcp.jsonc")));
    assert!(ws.dir_entries(format!("{d}/commands")).is_empty());
    assert!(ws.dir_entries(format!("{d}/skills")).is_empty());

    // Template files are not feature-gated; they must still exist.
    assert!(ws.file_exists(format!("{d}/templates/mycode/command.hbs")));
    assert!(ws.file_exists(format!("{d}/templates/mycode/skill.hbs")));
}

#[test]
fn init_fails_without_force_when_root_dir_already_exists() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();

    // Release builds default --force to false → second init must fail.
    // Debug builds default --force to true  → second init should succeed.
    #[cfg(not(debug_assertions))]
    ws.run(&["init"]).assert_failure();

    #[cfg(debug_assertions)]
    ws.run(&["init"]).assert_success();
}

#[test]
fn init_force_recreates_root_dir_and_restores_config() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();

    // Corrupt config.toml so we can verify it gets replaced.
    ws.write_in_root_dir("config.toml", "# intentionally corrupted");

    #[cfg(not(debug_assertions))]
    {
        ws.run(&["init", "--force"]).assert_success();
        let restored = ws.read(format!("{}/config.toml", ws.root_dir_name()));
        assert!(
            restored.contains("targets"),
            "config.toml should be fully restored after --force init"
        );
    }

    // Debug builds have --force=true by default.
    #[cfg(debug_assertions)]
    {
        ws.run(&["init"]).assert_success();
        let restored = ws.read(format!("{}/config.toml", ws.root_dir_name()));
        assert!(
            restored.contains("targets"),
            "config.toml should be fully restored by force init"
        );
    }
}
