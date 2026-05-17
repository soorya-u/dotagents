## 1. Core fix

- [x] 1.1 In `src/core/config/common.rs`, change `PackageRunner::args` signature to `args(&self, skill_name: &str, ci: bool)` and append `"--yes".into()` to each match arm's vec when `ci` is true
- [x] 1.2 In `src/cli/skills.rs`, update the call site to `runner.args(&opts.name, !is_tui_enabled())`

## 2. Unit tests

- [x] 2.1 Add unit test: npm runner CI mode appends `--yes` as last element (covers `args("x", true)` for `Npm`)
- [x] 2.2 Add unit test: pnpm runner CI mode appends `--yes` as last element (covers `args("x", true)` for `Pnpm`)
- [x] 2.3 Add unit test: yarn runner CI mode appends `--yes` as last element (covers `args("x", true)` for `Yarn`)
- [x] 2.4 Add unit test: bun runner CI mode appends `--yes` as last element (covers `args("x", true)` for `Bun`)
- [x] 2.5 Add unit test: interactive mode (`ci=false`) does NOT include `--yes` for any runner variant

## 3. E2e test

- [x] 3.1 Add e2e test in `tests/e2e/skills.test.ts`: run `skills add <name> --ci` (with a mocked/unavailable registry endpoint), assert the process exits within the timeout without hanging (non-zero exit OK; hang would mean `--yes` is missing)

## 4. Verification

- [x] 4.1 Run `mise check` (fmt + clippy) — must exit 0
- [x] 4.2 Run `mise tests` (unit + integration + e2e) — must exit 0
