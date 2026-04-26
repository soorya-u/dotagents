//! Tests for remote template fetching behaviour.
//!
//! Groups covered
//! ──────────────
//!  1.  URL rejection (no network) – http://, untrusted https:// blocked before
//!      any request; error messages are descriptive
//!  2.  Local template unaffected   – existing local-path logic unchanged after
//!      the remote branch was introduced
//!  3.  Live network (ignored)      – trusted URL fetched and rendered correctly;
//!      run with `cargo test -- --ignored` when a network is available

use super::TestWorkspace;

// ═════════════════════════════════════════════════════════════════════════════
// Group 1 – URL rejection (no network required)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn deploy_plain_http_url_fails_with_https_error() {
    // A plain http:// template URL must fail with a message telling the user to
    // use HTTPS; it must NOT produce a confusing "file not found" error.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["mycode"]

[providers.mycode.commands]
template = "http://dotagents.soorya-u.dev/templates/mycode/command.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
"#,
    );
    let result = ws.run(&["deploy"]);
    result.assert_failure();
    assert!(
        result.stderr.to_lowercase().contains("https")
            || result.stderr.to_lowercase().contains("non-https"),
        "error should mention HTTPS; stderr:\n{}",
        result.stderr
    );
}

#[test]
fn deploy_untrusted_https_url_fails_with_domain_error() {
    // An https:// URL from a domain other than dotagents.soorya-u.dev must fail
    // before any network request is made.  The error must name the trusted
    // domain so the user knows what is allowed.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["mycode"]

[providers.mycode.commands]
template = "https://example.com/template.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
"#,
    );
    let result = ws.run(&["deploy"]);
    result.assert_failure();
    assert!(
        result.stderr.contains("dotagents.soorya-u.dev")
            || result.stderr.to_lowercase().contains("untrusted"),
        "error should mention the trusted domain; stderr:\n{}",
        result.stderr
    );
}

#[test]
fn deploy_http_url_does_not_create_output_file() {
    // When a template URL is rejected, no partial output file should appear.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["mycode"]

[providers.mycode.commands]
template = "http://dotagents.soorya-u.dev/templates/mycode/command.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
"#,
    );
    ws.run(&["deploy"]).assert_failure();
    assert!(
        !ws.dir_exists(".mycode/commands"),
        ".mycode/commands/ must not be created when the template URL is rejected"
    );
}

#[test]
fn deploy_untrusted_url_does_not_create_output_file() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["mycode"]

[providers.mycode.commands]
template = "https://example.com/template.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
"#,
    );
    ws.run(&["deploy"]).assert_failure();
    assert!(
        !ws.dir_exists(".mycode/commands"),
        ".mycode/commands/ must not be created when the template URL is rejected"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 2 – local template path unaffected by the remote branch
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn deploy_local_template_path_still_works_after_remote_branch() {
    // Existing local-path templates must be completely unaffected by the new
    // URL-detection branch inside renderer.rs.
    let ws = TestWorkspace::new();
    ws.init_and_deploy();
    // The default init scaffolds a `mycode` provider that uses local .hbs
    // templates; all three output files must still be present.
    assert!(
        ws.file_exists(".mycode/commands/hello.md"),
        "local command template should still deploy"
    );
    assert!(
        ws.file_exists(".mycode/instructions.md"),
        "local instructions template should still deploy"
    );
    assert!(
        ws.file_exists(".mycode/mcp.json"),
        "local mcp template should still deploy"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 3 – live network (run with `cargo test -- --ignored`)
// ═════════════════════════════════════════════════════════════════════════════
//
// Group 4 – --offline flag (no network required)
// ═════════════════════════════════════════════════════════════════════════════

// --offline succeeds when every provider is fully configured (cache never consulted)
#[test]
fn deploy_offline_succeeds_when_all_providers_fully_configured() {
    // The `with-custom-provider` init config wires up `mycode` with explicit
    // template+target for all features.  Because the resolver skips fully-configured
    // providers, --offline does not consult the template cache at all and deploy must succeed.
    let ws = TestWorkspace::new();
    ws.run(&["init", "--template", "with-custom-provider"])
        .assert_success();
    ws.run(&["deploy", "--offline"]).assert_success();
    assert!(
        ws.file_exists(".mycode/commands/hello.md"),
        "fully-configured provider should still deploy normally under --offline"
    );
}

// --offline on a cold cache produces a hard error for any provider needing auto-resolution
#[test]
fn deploy_offline_cold_cache_fails_with_clear_error() {
    // Override config to target a provider that has no [providers.*] block so the
    // resolver must consult the cache.  With --offline and a cold cache the deploy
    // must fail rather than silently skip the provider.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["unknown-provider-that-will-never-be-cached"]
"#,
    );
    let result = ws.run(&["deploy", "--offline"]);
    result.assert_failure();
    assert!(
        result.stderr.contains("--offline") || result.stderr.to_lowercase().contains("offline"),
        "error must mention --offline so the user knows how to fix it; stderr:\n{}",
        result.stderr
    );
}

// --offline cold-cache error message instructs the user to warm the cache first
#[test]
fn deploy_offline_cold_cache_error_directs_user_to_warm_cache() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["unknown-provider-that-will-never-be-cached"]
"#,
    );
    let result = ws.run(&["deploy", "--offline"]);
    result.assert_failure();
    // The resolver emits: "Run without --offline first to populate the cache."
    let stderr_lc = result.stderr.to_lowercase();
    assert!(
        stderr_lc.contains("cache") || stderr_lc.contains("populate"),
        "error should tell the user to populate the cache; stderr:\n{}",
        result.stderr
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Group 5 – registry auto-resolution and --no-cache
//           (live network — run with `cargo test -- --ignored`)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "requires live network connection to dotagents.soorya-u.dev"]
fn deploy_trusted_remote_template_url_fetches_and_renders() {
    // A trusted https://dotagents.soorya-u.dev URL should be fetched at deploy
    // time and the response body used as the template.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["mycode"]

[providers.mycode.commands]
template = "https://dotagents.soorya-u.dev/templates/claude/command.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
"#,
    );
    ws.run(&["deploy"]).assert_success();
    assert!(
        ws.file_exists(".mycode/commands/hello.md"),
        "remote template should be fetched and render the command output file"
    );
    let content = ws.read(".mycode/commands/hello.md");
    assert!(
        !content.is_empty(),
        "rendered output from remote template must not be empty"
    );
}

#[test]
#[ignore = "requires live network connection to dotagents.soorya-u.dev"]
fn deploy_trusted_remote_template_not_found_fails_with_404_in_error() {
    // A trusted URL that returns 404 must stop deploy with an error message
    // that contains "404" so the user can identify the misconfiguration.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["mycode"]

[providers.mycode.commands]
template = "https://dotagents.soorya-u.dev/templates/nonexistent/missing.hbs"
target = "{{ dir.workspace }}/.mycode/commands/{{ command.name }}.md"
"#,
    );
    let result = ws.run(&["deploy"]);
    result.assert_failure();
    assert!(
        result.stderr.contains("404"),
        "error should contain the HTTP status code; stderr:\n{}",
        result.stderr
    );
}

// provider in targets with no [providers.*] block → template/target auto-resolved from registry
#[test]
#[ignore = "requires live network connection to dotagents.soorya-u.dev"]
fn deploy_auto_resolves_template_and_target_for_known_provider() {
    // A provider listed in `targets` with no [providers.*] config should have its
    // template URL and target path fetched from registry.json → provider.toml and
    // the feature should be rendered into the expected output directory.
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["claude"]
"#,
    );
    ws.run(&["deploy"]).assert_success();
    assert!(
        ws.dir_exists(".claude/commands"),
        "auto-resolved claude provider should deploy commands to .claude/commands/"
    );
}

// first deploy (online) warms the cache; second deploy (--offline) uses cached files
#[test]
#[ignore = "requires live network connection to dotagents.soorya-u.dev"]
fn deploy_offline_with_warm_cache_succeeds_without_network() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["claude"]
"#,
    );
    // First online deploy seeds the template-source cache.
    ws.run(&["deploy"]).assert_success();
    // --offline resolves from the now-warm cache; no network request is made.
    ws.run(&["deploy", "--offline"]).assert_success();
    assert!(
        ws.dir_exists(".claude/commands"),
        "warm-cache offline deploy should still produce output"
    );
}

// --no-cache bypasses checksum check and re-downloads all template files
#[test]
#[ignore = "requires live network connection to dotagents.soorya-u.dev"]
fn deploy_no_cache_forces_re_download_even_when_cached() {
    let ws = TestWorkspace::new();
    ws.run(&["init"]).assert_success();
    ws.write_in_root_dir(
        "local.config.toml",
        r#"schema = "https://dotagents.soorya-u.dev/schemas/config.schema.json"
features = ["commands"]
targets = ["claude"]
"#,
    );
    // Seed the cache with a first online deploy.
    ws.run(&["deploy"]).assert_success();
    // --no-cache must re-download everything and still succeed.
    ws.run(&["deploy", "--no-cache"]).assert_success();
    assert!(
        ws.dir_exists(".claude/commands"),
        "--no-cache deploy should still produce output after re-downloading templates"
    );
}
