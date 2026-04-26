## 1. Config Schema — PackageRunner

- [x] 1.1 Add `PackageRunner` enum to `src/schema/config/common.rs` with variants `Npm`, `Pnpm`, `Yarn`, `Bun` and `#[serde(rename_all = "lowercase")]`
- [x] 1.2 Add `impl PackageRunner` with `binary() -> &str` and `args(skill_name: &str) -> Vec<String>` methods mapping each variant to its invocation
- [x] 1.3 Add `package_runner: Option<PackageRunner>` field to `GlobalConfig` in `src/schema/config/global.rs`
- [x] 1.4 Add `package_runner: Option<PackageRunner>` field to `LocalConfig` in `src/schema/config/local.rs`
- [x] 1.5 Add `package_runner: Option<PackageRunner>` field to `AppConfig` in `src/schema/config/app.rs`
- [x] 1.6 Add merge line in `AppConfig::from()`: `local.package_runner.clone().or_else(|| global.package_runner.clone())`
- [x] 1.7 Add unit tests for `PackageRunner` serialisation round-trip and `args()` output for all four variants

## 2. CLI Shape — Skills Subcommand

- [x] 2.1 Add `SkillsAddOptions` struct to `src/cli/options.rs` with `name: String` and `runner: Option<PackageRunner>` fields (clap derive)
- [x] 2.2 Add `SkillsAction` enum to `src/cli/options.rs` with a single `Add(SkillsAddOptions)` variant
- [x] 2.3 Add `Skills(SkillsAction)` variant to the `Action` enum in `src/cli/options.rs`
- [x] 2.4 Add dispatch arm `Action::Skills(SkillsAction::Add(opts)) => skills::add(opts)` in `src/cli/runner.rs`

## 3. Skills Add Handler

- [x] 3.1 Create `src/cli/skills.rs` with a `pub(crate) fn add(opts: SkillsAddOptions) -> Result<bool>` function
- [x] 3.2 Resolve `AppConfig` (reuse `AppConfig::from_application`) to get `package_runner`
- [x] 3.3 Merge runner: CLI flag (`opts.runner`) takes priority over `app_config.package_runner`; if both `None`, use `PackageRunner::Npm` as silent default
- [x] 3.4 When resolved runner is `Some` (explicit), check binary is on PATH using `std::process::Command::new(binary).arg("--version").output()` and map `ErrorKind::NotFound` to a friendly bail with config.toml hint
- [x] 3.5 Resolve absolute path to `.dotagents` dir via `get_application_dir()`
- [x] 3.6 Build `std::process::Command` using `runner.args(&opts.name)`, set env var `CLAUDE_CONFIG_DIR=<application_dir>`, inherit stdio
- [x] 3.7 Spawn and wait; propagate non-zero exit code as `Ok(false)` (consistent with deploy behaviour)
- [x] 3.8 Wire `src/cli/skills.rs` into `src/cli/mod.rs`

## 4. Unit Tests

- [x] 4.1 Unit test: `package_runner` field deserialises correctly from TOML in `GlobalConfig` and `LocalConfig` for all four values
- [x] 4.2 Unit test: `AppConfig::from()` merge — local runner wins over global; absent local falls back to global; both absent yields `None`
- [x] 4.3 Unit test: invalid `package-runner` value in TOML fails deserialization with a descriptive error
- [x] 4.4 Unit test: `PackageRunner::args()` returns correct argv for each variant (e.g. `pnpm dlx skills add foo`)

## 5. Integration Tests (tests/integration/skills.rs)

- [x] 5.1 Create `tests/integration/skills.rs` and wire it into `tests/integration/main.rs`
- [x] 5.2 Smoke test: `dotagents skills add --help` exits 0 and output contains "add" — verifies the subcommand is wired up
- [x] 5.3 Smoke test: `dotagents skills add <name>` outside any workspace exits non-zero with an error referencing the missing root directory
- [x] 5.4 Smoke test: `dotagents skills add <name> --runner totally-nonexistent-bin-xyz` after `init` exits non-zero and stderr names the missing binary and references `package-runner`

## 6. E2E Tests (tests/e2e/skills.rs)

- [x] 6.1 Create `tests/e2e/skills.rs` and wire it into `tests/e2e/main.rs`
- [x] 6.2 E2E test: after `init`, writing `package-runner = "bun"` into `config.toml` and running `skills add <name>` exits non-zero (config-driven runner validation path)
- [x] 6.3 E2E test: after `init`, writing `package-runner = "bun"` into `local.config.toml` with global set to `"npm"` confirms local > global merge and exits non-zero
- [x] 6.4 E2E test (marked `#[ignore]`): after `init`, `dotagents skills add vercel-labs/agent-skills` with default runner installs skill files into `.dotagents-debug/skills/agent-skills/` — requires `npx` on PATH and network; run manually or in CI with `cargo test -- --ignored`

## 7. Verification

- [x] 7.1 Run `mise check` (cargo fmt + clippy) and fix all warnings
- [x] 7.2 Run `mise test-all` and confirm all tests pass
