## Why

When `skills add` is run in CI (non-TTY), the subprocess call to the skills package manager spawns an interactive menu that blocks forever because no `--yes` flag is passed. This makes the command unusable in automated pipelines.

## What Changes

- `PackageRunner::args()` gains a `ci: bool` parameter; when `true`, appends `--yes` to the argv list for all four runner variants (npm/pnpm/yarn/bun)
- `skills add` call site passes `!is_tui_enabled()` as the `ci` argument
- Unit tests added for all four `PackageRunner::args(_, true)` variants
- E2e test added: `skills add` in CI mode (`--ci` flag) does not hang (exits within timeout)

## Capabilities

### New Capabilities
- `skills-add-ci-mode`: `skills add` passes `--yes` to the downstream package runner when operating in CI/non-TTY mode, preventing interactive hangs

### Modified Capabilities
- `skills-add`: Extends the `PackageRunner::args` call signature to accept a `ci` flag

## Impact

- `src/core/config/common.rs` — `PackageRunner::args(&self, skill_name: &str, ci: bool)` signature change; all four match arms append `"--yes".into()` when `ci` is true
- `src/cli/skills.rs` — call site updated to `runner.args(&opts.name, !is_tui_enabled())`
- Unit tests in `src/core/config/common.rs` — four new test cases for `args(_, true)` variants
- E2e test in `tests/e2e/skills.test.ts` — one new case for `skills add` CI non-hang behavior
