## Purpose

CI workflow that runs on every pull request and push to main, enforcing formatting, linting, tests, cross-platform compilation, and publish readiness.

## Requirements

> **Note:** This proposal is independent of all other open proposals (`add-release-pipeline`, `fix-mock-files`, `fix-init-dir-timing`, `fix-e2e-release-build`, `fix-error-handling`). The `implement-skills` proposal is archived at `openspec/changes/archive/2026-04-25-implement-skills/` and is not a dependency or prerequisite.

### Requirement: CI runs on every pull request and push to main
The CI workflow SHALL trigger automatically on every pull request (any branch) and on every push to the `main` branch.

#### Scenario: PR opened triggers CI
- **WHEN** a pull request is opened or updated against any branch
- **THEN** all five CI jobs start automatically within GitHub Actions

#### Scenario: Push to main triggers CI
- **WHEN** a commit is pushed directly to `main`
- **THEN** all five CI jobs start automatically

### Requirement: Formatting is enforced as a hard gate
The CI workflow SHALL fail if Rust source is not formatted per `rustfmt` or if TypeScript e2e source has biome violations. Formatting MUST be checked read-only — CI SHALL NOT auto-fix code.

#### Scenario: Unformatted Rust fails CI
- **WHEN** a PR contains Rust source not conforming to `rustfmt` output
- **THEN** the `fmt` job exits non-zero and the PR is blocked

#### Scenario: Biome violation fails CI
- **WHEN** a PR contains TypeScript in `tests/e2e/` that fails `biome check`
- **THEN** the `fmt` job exits non-zero and the PR is blocked

#### Scenario: Correctly formatted code passes
- **WHEN** all Rust and TypeScript source passes their respective format checks
- **THEN** the `fmt` job exits zero

### Requirement: Clippy warnings are treated as errors
The CI workflow SHALL fail if `cargo clippy` reports any warning when run with `-- -D warnings`.

#### Scenario: Clippy warning blocks PR
- **WHEN** a PR introduces a Rust pattern that triggers a clippy lint
- **THEN** the `clippy` job exits non-zero and the PR is blocked

#### Scenario: Clean clippy output passes
- **WHEN** no clippy warnings are present
- **THEN** the `clippy` job exits zero

### Requirement: Unit and integration tests must pass
The CI workflow SHALL run `cargo test --bin dotagents` and `cargo test --test integration` and fail if either exits non-zero. E2E tests SHALL NOT run in PR CI.

#### Scenario: Failing unit test blocks PR
- **WHEN** a PR causes any unit test in the binary to fail
- **THEN** the `test` job exits non-zero and the PR is blocked

#### Scenario: Failing integration test blocks PR
- **WHEN** a PR causes any integration test to fail
- **THEN** the `test` job exits non-zero and the PR is blocked

#### Scenario: E2E tests are not run
- **WHEN** CI runs on a PR
- **THEN** no e2e test process is started (tui-test is not invoked)

### Requirement: Binary must compile on all four target platforms
The CI workflow SHALL verify the binary compiles (via `cargo check`) on `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-musl`, `x86_64-apple-darwin`, and `aarch64-apple-darwin`. A failure on one platform SHALL NOT cancel checks on others.

#### Scenario: Platform compile error is caught
- **WHEN** a PR introduces platform-specific code that fails to compile on one target
- **THEN** the `check` job leg for that target exits non-zero
- **THEN** the remaining three platform legs continue running

#### Scenario: All platforms compile successfully
- **WHEN** the code compiles cleanly on all four targets
- **THEN** all four `check` matrix legs exit zero

### Requirement: Crate must be publishable on every PR
The CI workflow SHALL run `cargo publish --dry-run --locked` and fail if the crate cannot be published, catching missing files or invalid metadata before a real release.

#### Scenario: Missing file in published crate fails CI
- **WHEN** a file referenced in `Cargo.toml` is absent or excluded from the publish set
- **THEN** the `check-publish` job exits non-zero

#### Scenario: Valid crate metadata passes
- **WHEN** `Cargo.toml` metadata is complete and all included files are present
- **THEN** the `check-publish` job exits zero
