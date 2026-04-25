//! Idempotency and re-deploy tests.
//!
//! Verifies that running `deploy` twice produces identical output and that
//! modifying a source file causes the corresponding output to update.

use super::TestWorkspace;

#[test]
fn deploy_twice_produces_identical_command_output() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    let first = ws.read(".mycode/commands/hello.md");
    ws.run(&["deploy"]).assert_success();
    assert_eq!(
        first,
        ws.read(".mycode/commands/hello.md"),
        "command output should be byte-for-byte identical across consecutive deploys"
    );
}

#[test]
fn deploy_twice_produces_identical_instructions_output() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    let first = ws.read(".mycode/instructions.md");
    ws.run(&["deploy"]).assert_success();
    assert_eq!(
        first,
        ws.read(".mycode/instructions.md"),
        "instructions output should be identical across consecutive deploys"
    );
}

#[test]
fn deploy_twice_produces_identical_mcp_output() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    let first = ws.read(".mycode/mcp.json");
    ws.run(&["deploy"]).assert_success();
    assert_eq!(
        first,
        ws.read(".mycode/mcp.json"),
        "mcp.json output should be identical across consecutive deploys"
    );
}

#[test]
fn deploy_reflects_updated_command_source_on_redeploy() {
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    let original = ws.read(".mycode/commands/hello.md");

    // Overwrite the source command with new content.
    ws.write_in_root_dir(
        "commands/hello.md",
        "---\nname: hello\ndescription: Updated.\n---\n\n# Updated Hello\n\nNew body.\n",
    );
    ws.run(&["deploy"]).assert_success();

    let redeployed = ws.read(".mycode/commands/hello.md");
    assert_ne!(
        original, redeployed,
        "output must change after the source is edited"
    );
    assert!(
        redeployed.contains("Updated Hello"),
        "the redeployed file should reflect the new source body"
    );
}

#[test]
fn init_force_then_deploy_still_succeeds() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();

    #[cfg(not(debug_assertions))]
    ws.run(&["init", "--force"]).assert_success();
    #[cfg(debug_assertions)]
    ws.run(&["init"]).assert_success(); // force=true by default in debug

    ws.run(&["deploy"]).assert_success();
    assert!(ws.file_exists(".mycode/commands/hello.md"));
    assert!(ws.file_exists(".mycode/instructions.md"));
    assert!(ws.file_exists(".mycode/mcp.json"));
}
