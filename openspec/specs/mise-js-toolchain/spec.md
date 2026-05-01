# Spec: mise JS Toolchain

## Purpose

Defines how JavaScript/TypeScript tooling (bun, tui-devtools, tui-test) is declared and managed through mise so that developers and CI environments get a reproducible, zero-manual-step setup.

## Requirements

### Requirement: bun is managed as a mise tool
`mise.toml` SHALL declare `bun = "latest"` under `[tools]`. After `mise install`, `bun` SHALL be available on PATH for all mise tasks without any separate installation step.

#### Scenario: bun available after mise install
- **WHEN** `mise install` is run on a clean checkout
- **THEN** `bun --version` exits 0 and prints a version string

#### Scenario: bun used as the JS runtime for e2e tasks
- **WHEN** `mise test-e2e` is invoked
- **THEN** it uses the mise-managed bun binary, not any system-level node or bun installation

### Requirement: tui-devtools is managed as a mise npm backend tool
`mise.toml` SHALL declare `"npm:tui-devtools" = "latest"` under `[tools]`. After `mise install`, the `tui-devtools` CLI SHALL be available on PATH.

#### Scenario: tui-devtools available after mise install
- **WHEN** `mise install` is run
- **THEN** `tui-devtools --help` (or equivalent version/help command) exits 0

#### Scenario: No separate global npm install required
- **WHEN** a developer or CI environment runs `mise install`
- **THEN** tui-devtools is ready to use without running `npm install -g tui-devtools` manually

### Requirement: tui-test is a pinned local dev dependency
`tests/e2e/package.json` SHALL declare `@microsoft/tui-test` as a dev dependency. `tests/e2e/bun.lockb` SHALL be committed to the repository to pin the exact version. `tests/e2e/node_modules/` SHALL be listed in `.gitignore`.

#### Scenario: tui-test importable in test files after bun install
- **WHEN** `bun install` is run in `tests/e2e/`
- **THEN** `import { test, expect } from "@microsoft/tui-test"` resolves correctly in test files

#### Scenario: Lockfile ensures reproducible installs
- **WHEN** `bun install --frozen-lockfile` is run in CI
- **THEN** it installs exactly the version recorded in `bun.lockb` without fetching newer releases

#### Scenario: node_modules not committed
- **WHEN** `git status` is checked after `bun install`
- **THEN** `tests/e2e/node_modules/` does not appear as an untracked or staged path

### Requirement: mise tasks wire the full test pipeline
`mise.toml` SHALL define tasks such that `mise test-all` runs unit tests, integration tests, and e2e tests in sequence with no manual intermediate steps.

#### Scenario: e2e-install task installs bun deps
- **WHEN** `mise e2e-install` is run
- **THEN** `tests/e2e/node_modules/@microsoft/tui-test` is present

#### Scenario: test-e2e depends on build and e2e-install
- **WHEN** `mise test-e2e` is invoked from a clean state
- **THEN** mise first builds the binary and installs bun deps before running tui-test

#### Scenario: test-all runs all three suites
- **WHEN** `mise test-all` is invoked
- **THEN** it runs unit tests (`cargo test --bin dotagents`), integration tests (`cargo test --test integration`), and e2e tests (tui-test) and fails if any suite fails
