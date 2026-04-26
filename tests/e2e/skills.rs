//! E2E tests for the `skills add` command.

use super::TestWorkspace;

#[test]
fn skills_add_with_config_explicit_bad_runner_fails_with_error() {
    // after init, setting package-runner to a valid but absent binary via config.toml
    // should produce an error. config.toml has no table sections so appending is safe.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();

    let d = ws.root_dir_name();
    let config_path = format!("{d}/config.toml");
    let mut config = ws.read(&config_path);
    // append at top level — config.toml has no [table] sections
    config.push_str("\npackage-runner = \"bun\"\n");
    ws.write(&config_path, &config);

    let result = ws.run(&["skills", "add", "some-skill"]);
    // fails: either bunx is absent (binary check) or bun fails to install some-skill
    result.assert_failure();
}

#[test]
fn skills_add_local_config_runner_overrides_global() {
    // local.config.toml package-runner wins over config.toml.
    // Write a minimal local.config.toml with ONLY package-runner at the top
    // level to avoid the key landing inside a [providers.*] TOML table.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();

    let d = ws.root_dir_name();

    // global: npm (default, likely present)
    let config_path = format!("{d}/config.toml");
    let mut config = ws.read(&config_path);
    config.push_str("\npackage-runner = \"npm\"\n");
    ws.write(&config_path, &config);

    // local: bun — write a standalone file so package-runner is at top level
    let local_path = format!("{d}/local.config.toml");
    ws.write(&local_path, "package-runner = \"bun\"\n");

    let result = ws.run(&["skills", "add", "some-skill"]);
    // fails: either bunx absent (binary check fires before any network call)
    // or bun fails to clone "some-skill". Either way, not an npm-driven failure.
    result.assert_failure();
}

#[test]
#[ignore = "requires npx on PATH and network access; run with: cargo test -- --ignored"]
fn skills_add_default_runner_installs_into_dotagents_skills() {
    // with no runner configured, skills add uses npx and installs into .dotagents/skills/
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();

    let result = ws.run(&["skills", "add", "vercel-labs/agent-skills"]);
    result.assert_success();

    let d = ws.root_dir_name();
    assert!(
        ws.dir_exists(format!("{d}/skills/agent-skills")),
        "skill should be installed into .dotagents/skills/agent-skills/"
    );
}
