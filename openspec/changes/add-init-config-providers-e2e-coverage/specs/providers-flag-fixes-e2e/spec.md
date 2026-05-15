## ADDED Requirements

### Requirement: Fix and test --quiet flag for providers command
The `--quiet` flag currently has no effect on the `providers` command because output uses `println!()` which bypasses the log framework. Fix the output path to respect the quiet flag, then add e2e test.

#### Scenario: --quiet suppresses provider listing (TC-PROV-09)
- **WHEN** `providers --ci --quiet` is run
- **THEN** exit code is 0, stdout is empty (no provider listing printed)

### Requirement: Fix and test --verbose flag for providers command
The `--verbose` flag currently adds no extra detail to the `providers` command output. Add diagnostic information (cache status, fetch URL) at debug level, then add e2e test.

#### Scenario: --verbose adds diagnostic output (TC-PROV-10)
- **WHEN** `providers --ci -v --offline` is run with a seeded cache
- **THEN** exit code is 0, stderr contains debug-level output with cache or fetch diagnostic information (e.g., cache path, "reading from cache", or similar)
