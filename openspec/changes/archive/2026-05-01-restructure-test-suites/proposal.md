## Why

The current `tests/integration/` and `tests/e2e/` suites both spawn the compiled binary and assert on exit codes or file contents — there is no test layer that exercises Rust logic directly, and the interactive TUI path (cliclack prompts in `init`, `add`, `rm`, `deploy`) is entirely untested because all existing tests run in non-TTY mode. Restructuring into three clear layers (unit, integration, e2e) closes both gaps and makes failures point to the exact broken component.

## What Changes

- **Replace** `tests/integration/` binary-spawning smoke tests with Rust function-level integration tests that call library code directly (no binary spawn, no filesystem-heavy setup).
- **Replace** `tests/e2e/` Rust binary tests with a TypeScript test suite using `@microsoft/tui-test`, covering both non-interactive CLI flows and interactive TUI flows (the currently untested cliclack prompt paths).
- **Add** a discovery phase: before writing tui-tests, an agent uses `tui-devtools` (for TTY flows) and bare binary invocation (for CLI flows) to observe and record actual runtime behaviour across all user flows, then the tui-tests are written from those observations.
- **Add** `bun` and `npm:tui-devtools` as managed tools in `mise.toml`; `@microsoft/tui-test` as a pinned dependency in `tests/e2e/package.json`.
- **Add** mise tasks: `e2e-install`, updated `test-e2e`, updated `test-all`.

## Capabilities

### New Capabilities

- `rust-integration-tests`: Logic-level Rust tests in `tests/integration/` that call library functions directly — config merging, template rendering, feature parsing, cache logic, gitignore management — without spawning a binary process.
- `tui-test-e2e-suite`: TypeScript e2e test suite in `tests/e2e/` using `@microsoft/tui-test`. Covers three tiers: CLI flows (flag-driven, non-interactive), TUI flows (interactive cliclack prompts navigated via tui-test input API), and journey flows (multi-command user stories). Filesystem side-effect assertions use Node.js `fs`.
- `mise-js-toolchain`: `bun` runtime and `npm:tui-devtools` CLI managed as mise tools; `@microsoft/tui-test` as a pinned `bun.lockb`-tracked dev dependency in `tests/e2e/package.json`. All JS tooling is installed and versioned through mise with no separate setup step required.

### Modified Capabilities

(none — no existing spec-level requirements are changing)

## Impact

- `mise.toml`: adds `bun = "latest"`, `"npm:tui-devtools" = "latest"`, new tasks `e2e-install`, updated `test-e2e` and `test-all`.
- `tests/integration/`: current Rust binary-invocation tests removed; replaced with logic-level Rust tests organised by module (`config.rs`, `render.rs`, `features.rs`, `cache.rs`, `gitignore.rs`).
- `tests/e2e/`: current Rust binary-invocation tests removed; replaced with TypeScript tui-test files plus `package.json` and `bun.lockb`.
- `Cargo.toml`: `tests/e2e` target removed (no longer a Rust test crate); `tests/integration` target retained as a Rust integration test crate.
- `.gitignore`: `tests/e2e/node_modules/` added.
- CI: must run `mise install` before test steps to provision bun and tui-devtools.
