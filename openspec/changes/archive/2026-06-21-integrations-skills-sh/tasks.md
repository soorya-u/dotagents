## 1. Module scaffolding

- [x] 1.1 Create `src/integrations/mod.rs` declaring `pub(crate) mod skills_sh;` and wire it into `src/main.rs` (or the appropriate `mod` declaration site)
- [x] 1.2 Move `PackageRunner` enum + `binary()` + `args()` from `src/core/config/common.rs` to `src/integrations/skills_sh.rs`; update `args()` to emit `--agent openclaw --copy` (and `--yes` when ci)
- [x] 1.3 Remove the old `PackageRunner` from `src/core/config/common.rs` and fix all imports/tests that referenced it there

## 2. Config schema changes

- [x] 2.1 Define `IntegrationsConfig` and `SkillsShConfig` structs (in `src/core/config/common.rs` or a new `src/core/config/integrations.rs`); `SkillsShConfig` carries `package_runner: Option<PackageRunner>`
- [x] 2.2 Add `integrations: Option<IntegrationsConfig>` to `GlobalConfig` and `LocalConfig`; remove the top-level `package_runner` field from both
- [x] 2.3 Update `AppConfig::from_application` to merge `integrations.skills-sh.package-runner` from local over global (same priority semantics as before)
- [x] 2.4 Update all `GlobalConfig::new()` / `LocalConfig::new()` / `with_*` constructors to drop `package_runner` and initialize `integrations: None`
- [x] 2.5 Update unit tests in `global.rs`, `local.rs`, `app.rs`, `common.rs` to use the new `[integrations.skills-sh]` path; remove tests for the top-level field
- [x] 2.6 Verify `serde` round-trips `[integrations.skills-sh]` correctly with kebab-case rename; add a test that top-level `package-runner` is no longer accepted

## 3. Integrations module — add

- [x] 3.1 Implement `integrations::skills_sh::add(opts)` in `src/integrations/skills_sh.rs`: resolve runner (CLI flag > local config > global config > npm default), validate explicit runner against PATH, spawn the skills CLI with `current_dir(application_dir)`, `--agent openclaw`, `--copy`, and `--yes` when non-TTY
- [x] 3.2 Drop the `CLAUDE_CONFIG_DIR` env var from the spawn; ensure no env var redirect is set
- [x] 3.3 Add a post-install assertion that `<application_dir>/skills/<expected-name>/SKILL.md` exists after the subprocess returns; error clearly if not (openclaw coupling mitigation)
- [x] 3.4 Add unit tests for `add()` arg construction (runner variants, ci flag, openclaw+copy flags)

## 4. Integrations module — remove + lockfile reader

- [x] 4.1 Implement a read-only `read_lockfile(application_dir) -> Option<LockfileData>` that parses `<application_dir>/skills-lock.json`; returns `None` + warning on missing/malformed file
- [x] 4.2 Implement `is_external_skill(name, application_dir) -> bool` using the lockfile reader (present in `skills` map = external)
- [x] 4.3 Implement `integrations::skills_sh::remove(name, application_dir)`: spawn `npx skills remove <name> --agent openclaw --yes` with `current_dir(application_dir)`; return success/failure but do NOT edit the lockfile
- [x] 4.4 Add unit tests for the lockfile reader (present, absent, missing file, malformed JSON)

## 5. CLI wiring — add

- [x] 5.1 Replace `src/cli/skills.rs::add()` body with a delegation call to `integrations::skills_sh::add(opts)`; remove the old spawn logic and `CLAUDE_CONFIG_DIR` handling
- [x] 5.2 Update `src/cli/runner.rs` dispatch (no signature change expected, verify it still calls `skills::add`)
- [x] 5.3 Update `src/cli/options.rs` `--runner` flag help text to reference `[integrations.skills-sh]` config location

## 6. CLI wiring — rm provenance branch

- [x] 6.1 In `src/cli/skills.rs::rm_skill()`, after the existing-not-found check and before the confirm prompt, read provenance via `integrations::skills_sh::is_external_skill(name, application_dir)`
- [x] 6.2 If external: after confirm/force logic, delegate file removal to `integrations::skills_sh::remove(name, application_dir)`; then run the existing undeploy cleanup (cache + gitignore fence) regardless of subprocess success
- [x] 6.3 If local: keep the existing `fs::remove_dir_all` + undeploy cleanup path unchanged
- [x] 6.4 Ensure the confirm prompt and `--force`/non-TTY behavior apply to both external and local paths

## 7. Config command display

- [x] 7.1 Update `src/cli/config.rs` to read `package_runner` from `config.integrations.skills-sh.package-runner` instead of the top-level field; update the displayed label if needed
- [x] 7.2 Update `src/templates/remote.rs` and any other sites that construct `AppConfig`/`GlobalConfig`/`LocalConfig` with `package_runner: None` to use `integrations: None`

## 8. Manual tui-devtools discovery (required before e2e)

- [x] 8.1 Run `tui-devtools` daemon and drive the `skills add` flow end-to-end in an isolated temp workspace; record exact terminal output (success and error paths)
- [x] 8.2 Drive the `skills rm` flow for both an external skill (lockfile present) and a local skill (lockfile absent); record the confirm prompt, delegation messages, and cleanup output
- [x] 8.3 Record observations for use in e2e test assertions (do not write assertions from source assumptions)

## 9. Unit tests

- [x] 9.1 Add unit tests in `src/integrations/skills_sh.rs` for `PackageRunner::args` with openclaw+copy flags (all four runners, ci on/off)
- [x] 9.2 Add unit tests for the lockfile reader (present/absent/missing/malformed)
- [x] 9.3 Add unit tests for `is_external_skill` true/false cases
- [x] 9.4 Add unit tests for the config merge of `[integrations.skills-sh].package-runner` (local over global, CLI flag over config, absent = None)

## 10. E2E tests

- [x] 10.1 Add/update `tests/e2e/skills.test.ts` for `skills add`: assert skill files land in `.dotagents-debug/skills/<name>/SKILL.md` (real files, not symlinks), no `.claude/skills/` created, lockfile created at `.dotagents-debug/skills-lock.json`
- [x] 10.2 Add e2e test for `skills add` with `--runner` flag (CLI path)
- [x] 10.3 Add e2e test for `skills rm` external path: install a skill, then rm it; assert subprocess ran, files gone, deployed output cleaned, command exits 0
- [x] 10.4 Add e2e test for `skills rm` local path: `skills new` a skill, deploy, then rm it; assert `fs::remove_dir_all` path, deployed output cleaned, no subprocess spawned
- [x] 10.5 Add e2e test for `[integrations.skills-sh]` config: scaffold config with the new table, run `skills add`, assert the configured runner is used
- [x] 10.6 Add e2e test that top-level `package-runner` in config is no longer honored (breaking change confirmation)

## 11. Verification

- [x] 11.1 Run `mise check` (cargo fmt + cargo clippy) and resolve any issues
- [x] 11.2 Run `mise tests` (unit + integration + e2e) and resolve any failures
- [x] 11.3 Both commands exit 0
