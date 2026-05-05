## ADDED Requirements

### Requirement: E2E suite runs the release binary
The e2e test suite SHALL invoke the release binary (`target/release/dotagents`). The `tests:e2e` mise task SHALL declare `build-release` as its dependency so the release binary is always compiled before the suite runs.

#### Scenario: Binary path resolves to release build
- **WHEN** a tui-test test spawns the Dotagents binary via the path defined in `tests/e2e/helpers.ts`
- **THEN** the process that runs is `target/release/dotagents`, not `target/debug/dotagents`

#### Scenario: Mise task builds release binary before running suite
- **WHEN** `mise tests:e2e` is invoked
- **THEN** `mise build-release` completes before any test process is spawned

### Requirement: No `.dotagents-debug` references in e2e test files
All e2e test files under `tests/e2e/` SHALL reference `.dotagents` as the root directory name. The string `.dotagents-debug` SHALL NOT appear in any file under `tests/e2e/`.

#### Scenario: Filesystem assertions use correct root directory name
- **WHEN** an e2e test checks for the existence of a file inside the Dotagents root directory
- **THEN** the path used in the assertion contains `.dotagents`, not `.dotagents-debug`

#### Scenario: Terminal output assertions use correct root directory name
- **WHEN** an e2e test uses `getByText` to assert on output that includes the root directory path
- **THEN** the expected string contains `.dotagents`, not `.dotagents-debug`

### Requirement: Overwrite test exercises release-binary behavior
The overwrite test in `tests/e2e/init.test.ts` SHALL be active (not skipped). It SHALL verify that the release binary requires an explicit `--force` flag to overwrite an existing root directory.

#### Scenario: Init without --force fails on existing root
- **WHEN** `dotagents init` runs in a workspace that already contains a `.dotagents/` directory and `--force` is not passed
- **THEN** the process exits non-zero and the existing directory is not modified

#### Scenario: Init with --force succeeds on existing root
- **WHEN** `dotagents init --force` runs in a workspace that already contains a `.dotagents/` directory
- **THEN** the process exits zero and the root directory is re-scaffolded

### Requirement: Deploy tests are CI-safe with respect to the gemini provider
Every deploy test in `tests/e2e/` SHALL either use `initWithLocalProvider()` from `helpers.ts` or explicitly remove the gemini provider from the active targets before running deploy. No deploy test SHALL rely on a gemini local cache file.

#### Scenario: Deploy test using initWithLocalProvider never activates gemini
- **WHEN** a deploy test calls `initWithLocalProvider()` to set up the workspace
- **THEN** only the local test provider is active and gemini is not present in the deploy target list

#### Scenario: Deploy test that does not use initWithLocalProvider patches out gemini
- **WHEN** a deploy test sets up its workspace without `initWithLocalProvider()`
- **THEN** it explicitly removes or disables the gemini provider from `config.toml` or `local.config.toml` before invoking deploy

### Requirement: initWithLocalProvider is documented as canonical deploy-test setup
The `initWithLocalProvider()` function in `tests/e2e/helpers.ts` SHALL have a comment explaining that it is the canonical helper for deploy tests because it sets up only a local provider and avoids CI-unsafe external providers such as gemini.

#### Scenario: Comment is present on initWithLocalProvider
- **WHEN** a developer reads `tests/e2e/helpers.ts`
- **THEN** `initWithLocalProvider()` has a comment stating it is the required setup for deploy tests and explaining why (no gemini dependency)
