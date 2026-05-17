## Purpose

Implementation fixes and e2e tests for `--quiet` and `--verbose` flags on the providers command.

## Requirements

### Requirement: Fix and test --quiet flag for providers command
The `--quiet` flag suppresses provider listing output. The global `Options::quiet` flag is passed to `run_providers()` and gates all `println!()` output.

#### Scenario: --quiet suppresses provider listing (TC-PROV-09)
- **WHEN** `providers --ci --quiet` is run
- **THEN** exit code is 0, stdout is empty (no provider listing printed)

### Requirement: Fix and test --verbose flag for providers command
Diagnostic information (cache status, fetch URL) is emitted at debug level via `debug!()` calls in the fetch/cache pipeline, visible at `-v` verbosity.

#### Scenario: --verbose adds diagnostic output (TC-PROV-10)
- **WHEN** `providers --ci -v --offline` is run with a seeded cache
- **THEN** exit code is 0, stderr contains debug-level output with cache or fetch diagnostic information
