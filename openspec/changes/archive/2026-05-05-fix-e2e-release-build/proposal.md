## Why

The e2e test suite runs the debug binary, which uses `.dotagents-debug` as its root directory instead of `.dotagents`. This means tests currently verify debug-build behavior rather than the release binary shipped to users, and any test that hardcodes `.dotagents-debug` will fail as soon as the suite is pointed at the release binary. Additionally, certain deploy tests do not use `initWithLocalProvider()` and may include the gemini provider, which requires a local cache file unavailable in CI.

## What Changes

- `tests/e2e/helpers.ts`: binary path changed from `../../target/debug/dotagents` to `../../target/release/dotagents`
- `tests/e2e/*.test.ts`: all 57 occurrences of `.dotagents-debug` replaced with `.dotagents`
- `mise.toml`: `tests:e2e` task dependency changed from `build` to `build-release`
- `tests/e2e/init.test.ts`: overwrite test unskipped (skip reason was debug binary's force-default; release binary requires explicit `--force`)
- `tests/e2e/helpers.ts`: explanatory comment added to `initWithLocalProvider()` marking it as the canonical deploy-test helper
- All deploy tests audited; any that don't use `initWithLocalProvider()` and don't explicitly remove gemini from the active targets are fixed

## Capabilities

### New Capabilities

- `e2e-test-build`: Requirements covering which binary the e2e suite must run, which root directory name tests must reference, and which provider setup is required for CI-safe deploy tests

### Modified Capabilities

<!-- No existing spec requirements change behavior — the tui-test-e2e-suite spec does not prescribe the binary target or root dir name, so no delta spec is needed for it. -->

## Impact

- `tests/e2e/helpers.ts` — binary path and `initWithLocalProvider()` comment
- `tests/e2e/*.test.ts` — global string replacement of `.dotagents-debug` → `.dotagents` (57 occurrences)
- `tests/e2e/init.test.ts` — unskip one test case
- `mise.toml` — task dependency update (`build` → `build-release`)
- CI pipelines that run `mise tests` are not affected in interface, only in which binary gets built before the suite
