## Tasks

### 1. Add `Blank` variant and rename `WithCustomProvider` → `Advanced` in `InitTemplate`
- Update `src/cli/options.rs`: add `Blank`, rename `WithCustomProvider` → `Advanced`, update `ValueEnum` derive labels.
- Update `src/cli/init.rs`: handle `Blank` in `build_config_content` (skip local config), update skip conditions for `.env`/`local.config.toml`/mycode templates, change default to `Blank`.
- Update `src/cli/ui/init.rs`: add `Blank` to TUI selector, update descriptions, rename `WithCustomProvider` → `Advanced`.
- **Test with tui-devtools**: run `dotagents init` interactively, verify 3 options appear with correct labels and each produces the right file set.

### 2. Update unit tests
- Update `build_config_content` tests in `src/cli/init.rs` for `Blank` variant.
- Add test: `Blank` produces empty local config string.
- Add test: `Starter` global and local are identical.
- Add test: `Advanced` local includes mycode provider block.
- Update `InitTemplate` equality tests for renamed variant.
- Run `cargo test` — all init-related tests pass.

### 3. Update `init-templates` spec
- Rewrite `openspec/specs/init-templates/spec.md` from 2-template to 3-template requirements.
- Add scenarios for each template's file set.
- Update default template scenario (`Blank` is now default).
- Update `--template` flag scenarios (add `blank`, rename `with-custom-provider` → `advanced`).

### 4. Update e2e tests
- Update existing e2e tests in `tests/e2e/` that reference `WithCustomProvider` or `starter` default.
- Add e2e test for `Blank` template file set.
- Add e2e test for `--template blank` flag.
- Run `mise tests:e2e` — all pass.

### 5. Verification
- `mise check` — cargo fmt + clippy exit 0.
- `mise tests` — all suites exit 0.
