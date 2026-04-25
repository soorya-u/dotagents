//! Smoke tests for the global --verbose / --quiet flags.

use super::TestWorkspace;

#[test]
fn init_with_verbose_flag_succeeds() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "-v"]).assert_success();
    assert!(ws.active_root_dir().exists());
}

#[test]
fn deploy_with_verbose_flag_succeeds() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init"]).assert_success();
    ws.run_command(&["deploy", "-v"]).assert_success();
    assert!(ws.root().join(".mycode").is_dir());
}

#[test]
fn init_and_deploy_with_quiet_flag_succeed() {
    let ws = TestWorkspace::new();
    ws.run_command(&["init", "--quiet"]).assert_success();
    assert!(ws.active_root_dir().exists());
    ws.run_command(&["deploy", "--quiet"]).assert_success();
}
