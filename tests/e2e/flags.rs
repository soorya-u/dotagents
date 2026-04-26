//! Tests for the `--verbose` / `-v` and `--quiet` / `-q` global flags.
//!
//! These flags affect log-level filtering (via `simplelog`).  All tests verify
//! that the flags do not break normal operation and that quiet mode produces
//! no stdout output (since all log levels in quiet mode go to stderr or are
//! suppressed entirely).

use super::TestWorkspace;

// ─────────────────────────────────────────────────────────────────────────────
// Verbose
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn verbose_flag_does_not_break_init() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "-v"]).assert_success();
    assert!(ws.root_dir().is_dir());
}

#[test]
fn very_verbose_flag_does_not_break_init() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "-vvv"]).assert_success();
    assert!(ws.root_dir().is_dir());
}

#[test]
fn verbose_flag_does_not_break_deploy() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "--template", "with-custom-provider"])
        .assert_success();
    ws.run(&["deploy", "-v"]).assert_success();
    assert!(ws.file_exists(".mycode/commands/hello.md"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Quiet
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn quiet_flag_does_not_break_init() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "--quiet"]).assert_success();
    assert!(ws.root_dir().is_dir());
}

#[test]
fn quiet_flag_does_not_break_deploy() {
    let ws = TestWorkspace::new();
    ws.run(&["init", "--template", "with-custom-provider"])
        .assert_success();
    ws.run(&["deploy", "--quiet"]).assert_success();
    assert!(ws.file_exists(".mycode/commands/hello.md"));
}

#[test]
fn quiet_init_produces_no_stdout() {
    // simplelog sends Warn/Error to stderr and Info/Debug to stdout.
    // In quiet mode only Error is allowed, so stdout must be silent.
    let ws = TestWorkspace::new();
    let result = ws.run(&["init", "--quiet"]);
    result.assert_success();
    assert!(
        result.stdout.is_empty(),
        "quiet init should produce no stdout; got: {:?}",
        result.stdout
    );
}

#[test]
fn quiet_deploy_produces_no_stdout() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    let result = ws.run(&["deploy", "--quiet"]);
    result.assert_success();
    assert!(
        result.stdout.is_empty(),
        "quiet deploy should produce no stdout; got: {:?}",
        result.stdout
    );
}
