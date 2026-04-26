//! Smoke tests for the `deploy` command.

use std::fs;

use super::TestWorkspace;

#[test]
fn deploy_without_init_fails() {
    let ws = TestWorkspace::new();
    ws.run_command(&["deploy"]).assert_failure();
}

#[test]
fn deploy_after_init_succeeds() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["deploy"]).assert_success();
}

#[test]
fn deploy_creates_mycode_output_directory() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--template", "with-custom-provider"])
        .assert_success();
    ws.run_command(&["deploy"]).assert_success();

    // The default local.config.toml targets the `mycode` custom provider,
    // which writes into `.mycode/`.
    assert!(
        ws.root().join(".mycode").is_dir(),
        ".mycode/ directory should be created during deploy"
    );
}

#[test]
fn deploy_creates_command_output_files() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--template", "with-custom-provider"])
        .assert_success();
    ws.run_command(&["deploy"]).assert_success();

    let commands_dir = ws.root().join(".mycode/commands");
    assert!(
        commands_dir.is_dir(),
        ".mycode/commands/ should be created during deploy"
    );
    let files = std::fs::read_dir(&commands_dir)
        .expect("failed to read .mycode/commands/")
        .filter_map(|e| e.ok().map(|d| d.file_name().to_string_lossy().to_string()))
        .collect::<Vec<_>>();
    assert!(!files.is_empty(), "at least one command should be deployed");
}

// ─────────────────────────────────────────────────────────────────────────────
// Remote template URL smoke tests (no network required)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn deploy_fails_with_plain_http_template_url() {
    // http:// is rejected before any network request is made.
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    fs::write(
        ws.active_root_dir().join("local.config.toml"),
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["mycode"]

[providers.mycode.commands]
template = "http://dotagents.soorya-u.dev/templates/mycode/command.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
"#,
    )
    .expect("failed to write local.config.toml");
    ws.run_command(&["deploy"]).assert_failure();
}

#[test]
fn deploy_fails_with_untrusted_https_template_url() {
    // https:// from a domain other than dotagents.soorya-u.dev is rejected
    // without making any network request.
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    fs::write(
        ws.active_root_dir().join("local.config.toml"),
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["mycode"]

[providers.mycode.commands]
template = "https://example.com/template.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
"#,
    )
    .expect("failed to write local.config.toml");
    ws.run_command(&["deploy"]).assert_failure();
}

#[test]
fn init_and_deploy_end_to_end() {
    let ws = TestWorkspace::new();

    ws.run_command(&["init", "--template", "with-custom-provider"])
        .assert_success();
    assert!(ws.active_root_dir().exists());

    let d = ws.root_dir_name();
    assert!(ws.root().join(format!("{d}/config.toml")).exists());
    assert!(ws.root().join(format!("{d}/commands")).is_dir());
    assert!(ws.root().join(format!("{d}/templates")).is_dir());

    ws.run_command(&["deploy"]).assert_success();

    assert!(
        ws.root().join(".mycode").is_dir(),
        "provider output directory should exist after deploy"
    );
}
