//! Smoke tests for the `skills` subcommand.

use super::TestWorkspace;

#[test]
fn skills_add_help_exits_zero() {
    // subcommand is wired up and help is reachable
    let ws = TestWorkspace::new();
    let result = ws.run_command(&["skills", "add", "--help"]);
    result.assert_success();
    assert!(
        result.stdout.contains("add") || result.stderr.contains("add"),
        "help output should mention 'add'"
    );
}

#[test]
fn skills_add_outside_workspace_exits_nonzero() {
    // running from a directory with no .dotagents ancestor errors out
    let ws = TestWorkspace::new();
    let result = ws.run_command(&["skills", "add", "some-skill"]);
    result.assert_failure();
}

#[test]
fn skills_add_explicit_nonexistent_runner_exits_nonzero_with_helpful_error() {
    // --runner with a binary that isn't on PATH gives a friendly error
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    let result = ws.run_command(&[
        "skills",
        "add",
        "some-skill",
        "--runner",
        "totally-nonexistent-bin-xyz",
    ]);
    result.assert_failure();
    let combined = format!("{}{}", result.stdout, result.stderr);
    assert!(
        combined.contains("totally-nonexistent-bin-xyz") || combined.contains("package-runner"),
        "error should name the missing binary or reference package-runner; got: {combined}"
    );
}
