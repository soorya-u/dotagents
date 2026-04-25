//! Tests for the binary invoked with no subcommand.
//!
//! `runner.rs` calls `Options::command().print_help()` (stdout) then
//! `std::process::exit(0)` when no action is given.

use super::TestWorkspace;

#[test]
fn no_subcommand_exits_with_code_zero() {
    let ws = TestWorkspace::new();
    ws.run(&[]).assert_success();
}

#[test]
fn no_subcommand_prints_help_text() {
    let ws = TestWorkspace::new();
    let result = ws.run(&[]);
    // clap's `print_help()` writes to stdout; some versions write to stderr.
    let combined = format!("{}{}", result.stdout, result.stderr).to_lowercase();
    assert!(
        combined.contains("usage") && (combined.contains("init") || combined.contains("deploy")),
        "help text should include 'usage' and a known subcommand; got:\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr
    );
}
