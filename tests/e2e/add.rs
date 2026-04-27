//! End-to-end tests for `dotagents add command` and `dotagents add skill`.
//!
//! Groups covered
//! ──────────────
//!  1.  Workspace discovery  – fails without .dotagents/
//!  2.  Command creation     – file created, frontmatter correct, starter body present
//!  3.  Skill creation       – directory + SKILL.md created, frontmatter correct, starter body
//!  4.  Flags                – --description, --category, --tags, --license, --compatibility
//!  5.  --force              – overwrites existing file
//!  6.  Error cases          – duplicate without --force

use super::TestWorkspace;

// ═════════════════════════════════════════════════════════════════════════════
// Group 1 – workspace discovery
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn add_command_without_workspace_exits_nonzero() {
    let ws = TestWorkspace::new();
    ws.run(&["add", "command", "x"]).assert_failure();
}

#[test]
fn add_skill_without_workspace_exits_nonzero() {
    let ws = TestWorkspace::new();
    ws.run(&["add", "skill", "x"]).assert_failure();
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 2 – command creation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn add_command_creates_md_file() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "new-cmd"]).assert_success();
    let d = ws.root_dir_name();
    assert!(
        ws.file_exists(format!("{d}/commands/new-cmd.md")),
        "command file should exist"
    );
}

#[test]
fn add_command_file_starts_with_frontmatter() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "fmtest"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/fmtest.md"));
    assert!(
        content.trim_start().starts_with("---"),
        "command file should begin with YAML frontmatter; content:\n{content}"
    );
}

#[test]
fn add_command_frontmatter_contains_name() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "named-cmd"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/named-cmd.md"));
    assert!(
        content.contains("named-cmd"),
        "frontmatter should contain the command name; content:\n{content}"
    );
}

#[test]
fn add_command_body_contains_starter_heading_with_name() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "starter-test"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/starter-test.md"));
    assert!(
        content.contains("# starter-test"),
        "starter body should contain '# starter-test' heading; content:\n{content}"
    );
}

#[test]
fn add_command_body_contains_steps_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "steps-test"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/steps-test.md"));
    assert!(
        content.contains("## Steps"),
        "starter body should contain a Steps section; content:\n{content}"
    );
}

#[test]
fn add_command_body_contains_when_to_use_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "when-test"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/when-test.md"));
    assert!(
        content.contains("## When to use"),
        "starter body should contain a 'When to use' section; content:\n{content}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 3 – skill creation
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn add_skill_creates_skill_md_file() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "my-new-skill"]).assert_success();
    let d = ws.root_dir_name();
    assert!(
        ws.file_exists(format!("{d}/skills/my-new-skill/SKILL.md")),
        "SKILL.md should be created inside the skill directory"
    );
}

#[test]
fn add_skill_creates_named_directory() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "dir-skill"]).assert_success();
    let d = ws.root_dir_name();
    assert!(
        ws.dir_exists(format!("{d}/skills/dir-skill")),
        "a directory named after the skill should be created"
    );
}

#[test]
fn add_skill_file_starts_with_frontmatter() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "fm-skill"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/fm-skill/SKILL.md"));
    assert!(
        content.trim_start().starts_with("---"),
        "SKILL.md should begin with YAML frontmatter; content:\n{content}"
    );
}

#[test]
fn add_skill_frontmatter_contains_name() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "named-skill"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/named-skill/SKILL.md"));
    assert!(
        content.contains("named-skill"),
        "frontmatter should contain the skill name; content:\n{content}"
    );
}

#[test]
fn add_skill_frontmatter_contains_metadata_version() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "versioned-skill"])
        .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/versioned-skill/SKILL.md"));
    assert!(
        content.contains("version:") && content.contains("1.0"),
        "SKILL.md should contain metadata.version = '1.0'; content:\n{content}"
    );
}

#[test]
fn add_skill_body_contains_starter_heading_with_name() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "heading-skill"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/heading-skill/SKILL.md"));
    assert!(
        content.contains("# heading-skill"),
        "starter body should contain '# heading-skill'; content:\n{content}"
    );
}

#[test]
fn add_skill_body_contains_instructions_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "instruct-skill"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/instruct-skill/SKILL.md"));
    assert!(
        content.contains("## Instructions"),
        "skill starter body should contain an Instructions section; content:\n{content}"
    );
}

#[test]
fn add_skill_body_contains_when_to_use_section() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "when-skill"]).assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/when-skill/SKILL.md"));
    assert!(
        content.contains("## When to use"),
        "skill starter body should contain a 'When to use' section; content:\n{content}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 4 – flags populate frontmatter
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn add_command_description_flag_appears_in_frontmatter() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "command",
        "desc-cmd",
        "--description",
        "My custom description",
    ])
    .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/desc-cmd.md"));
    assert!(
        content.contains("My custom description"),
        "description flag value should appear in frontmatter; content:\n{content}"
    );
}

#[test]
fn add_command_category_flag_appears_in_frontmatter() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "command",
        "cat-cmd",
        "--description",
        "x",
        "--category",
        "Workflow",
    ])
    .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/cat-cmd.md"));
    assert!(
        content.contains("category: Workflow"),
        "category flag should appear in frontmatter; content:\n{content}"
    );
}

#[test]
fn add_command_tags_flag_appears_in_frontmatter() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "command",
        "tag-cmd",
        "--description",
        "x",
        "--tags",
        "foo,bar",
    ])
    .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/tag-cmd.md"));
    assert!(
        content.contains("foo") && content.contains("bar"),
        "tags flag values should appear in frontmatter; content:\n{content}"
    );
}

#[test]
fn add_skill_description_flag_appears_in_frontmatter() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "skill",
        "desc-skill",
        "--description",
        "My skill description",
    ])
    .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/desc-skill/SKILL.md"));
    assert!(
        content.contains("My skill description"),
        "description should appear in SKILL.md; content:\n{content}"
    );
}

#[test]
fn add_skill_license_flag_appears_in_frontmatter() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "skill",
        "lic-skill",
        "--description",
        "x",
        "--license",
        "MIT",
    ])
    .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/lic-skill/SKILL.md"));
    assert!(
        content.contains("MIT"),
        "license should appear in SKILL.md; content:\n{content}"
    );
}

#[test]
fn add_skill_compatibility_flag_appears_in_frontmatter() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "skill",
        "compat-skill",
        "--description",
        "x",
        "--compatibility",
        "Claude Code",
    ])
    .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/compat-skill/SKILL.md"));
    assert!(
        content.contains("Claude Code"),
        "compatibility should appear in SKILL.md; content:\n{content}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 5 – --force
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn add_command_force_overwrites_existing_file() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "command",
        "overwrite-me",
        "--description",
        "original",
    ])
    .assert_success();
    ws.run(&[
        "add",
        "command",
        "overwrite-me",
        "--description",
        "replaced",
        "--force",
    ])
    .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/commands/overwrite-me.md"));
    assert!(
        content.contains("replaced"),
        "force should overwrite with new content; content:\n{content}"
    );
    assert!(
        !content.contains("original"),
        "old content should be gone after force overwrite; content:\n{content}"
    );
}

#[test]
fn add_skill_force_overwrites_existing_skill_md() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&[
        "add",
        "skill",
        "overwrite-skill",
        "--description",
        "original",
    ])
    .assert_success();
    ws.run(&[
        "add",
        "skill",
        "overwrite-skill",
        "--description",
        "replaced",
        "--force",
    ])
    .assert_success();
    let d = ws.root_dir_name();
    let content = ws.read(format!("{d}/skills/overwrite-skill/SKILL.md"));
    assert!(
        content.contains("replaced"),
        "force should overwrite SKILL.md; content:\n{content}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 6 – error cases
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn add_command_duplicate_without_force_exits_nonzero() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "dup-cmd", "--description", "first"])
        .assert_success();
    let result = ws.run(&["add", "command", "dup-cmd", "--description", "second"]);
    result.assert_failure();
}

#[test]
fn add_command_duplicate_error_mentions_force() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "command", "dup-cmd2", "--description", "first"])
        .assert_success();
    let result = ws.run(&["add", "command", "dup-cmd2", "--description", "second"]);
    result.assert_failure();
    assert!(
        result.stderr.contains("force") || result.stderr.contains("--force"),
        "error should mention --force; stderr: {}",
        result.stderr
    );
}

#[test]
fn add_skill_duplicate_without_force_exits_nonzero() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.run(&["add", "skill", "dup-skill", "--description", "first"])
        .assert_success();
    let result = ws.run(&["add", "skill", "dup-skill", "--description", "second"]);
    result.assert_failure();
}
