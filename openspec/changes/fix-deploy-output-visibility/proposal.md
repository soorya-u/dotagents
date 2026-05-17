## Why

Two deploy output issues affect CI usability: (1) `print_deploy_summary()` returns immediately in non-TTY mode — CI pipelines see no output about what was written or skipped; (2) when no providers are configured, `deploy` exits silently with `"✓ Nothing deployed"` in TTY and nothing at all in CI, leaving users unaware that their configuration has no providers.

## What Changes

- `print_deploy_summary()` in `src/cli/ui/deploy.rs` removes the early `!is_tui_enabled()` return — in non-TTY/CI mode it uses `println!()` to emit the summary line (e.g., `"2 written, 1 skipped"` or `"Nothing deployed"`)
- Before calling `print_deploy_summary()`, `deploy()` checks if zero providers are configured across all features; if so, it emits a `warn!()` log line: `"No providers configured — nothing to deploy. Add providers to config.toml."` — visible in both TTY and CI
- Unit tests added for CI-mode summary output
- E2e test added: `deploy --ci` with providers configured produces summary on stdout; `deploy --ci` with no providers emits the warning

## Capabilities

### New Capabilities

### Modified Capabilities
- `deploy-outro`: Deploy summary is printed in both TTY and CI/non-TTY mode; missing-providers condition emits a warning

## Impact

- `src/cli/ui/deploy.rs` — remove `!is_tui_enabled()` early return in `print_deploy_summary()`; use `println!()` for non-TTY output
- `src/cli/deploy.rs` — add check before `print_deploy_summary()`: if total providers across all features is zero, emit `warn!()`
- Unit tests in `src/cli/ui/deploy.rs`
- `tests/e2e/deploy.test.ts` — new CI-mode summary and no-providers warning tests
