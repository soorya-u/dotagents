## ADDED Requirements

### Requirement: Stage 1 creates a reviewable release PR via manual dispatch
A developer SHALL be able to trigger a release preparation by running the `release-prep` workflow with a version string input. The workflow MUST open a pull request containing the version bump, updated CHANGELOG, and bundled shell completions — and MUST NOT publish anything to any registry.

#### Scenario: Dispatch opens release PR
- **WHEN** a developer triggers `release-prep` workflow with input version `"0.2.0"`
- **THEN** `Cargo.toml` version is bumped to `0.2.0`
- **THEN** `CHANGELOG.md` is prepended with a new section for `v0.2.0` generated from conventional commits since the last tag
- **THEN** shell completions (bash, elvish, fish, powershell, zsh) are generated and zipped as `completions.zip`
- **THEN** a PR titled exactly `"chore: release v0.2.0"` is opened containing all changed files

#### Scenario: Stage 1 does not publish
- **WHEN** the `release-prep` workflow completes
- **THEN** no package is published to crates.io, npm, Homebrew, or Scoop

### Requirement: Stage 2 only triggers on release PR merges
The `release` workflow SHALL trigger on push to `main` but MUST exit early without doing any work if the merge commit title does not match the pattern `"chore: release v*"`.

#### Scenario: Non-release push is ignored
- **WHEN** a regular feature commit is pushed to `main`
- **THEN** the `release` workflow starts but exits in the first step without creating a tag or building any binary

#### Scenario: Release PR merge triggers full pipeline
- **WHEN** the `"chore: release v0.2.0"` PR is merged to `main`
- **THEN** all Stage 2 jobs run: tag, build-release, e2e, publish-cargo, publish-npm, update-homebrew, update-scoop

### Requirement: E2E tests must pass before any registry publish
All publish jobs (crates.io, npm) SHALL declare `needs: e2e` and MUST NOT start if the `e2e` job exits non-zero. The `e2e` job MUST use the actual release binary built in the `build-release` job.

#### Scenario: E2E failure aborts publish
- **WHEN** any e2e test fails on the release binary
- **THEN** the `e2e` job exits non-zero
- **THEN** `publish-cargo` and `publish-npm` jobs are skipped
- **THEN** no version is published to crates.io or npm

#### Scenario: E2E pass allows publish
- **WHEN** all e2e tests pass on the `linux-x64-musl` release binary
- **THEN** `publish-cargo` and `publish-npm` jobs start

### Requirement: Homebrew and Scoop are updated with correct SHA256 on every release
The `update-homebrew` and `update-scoop` jobs SHALL compute the SHA256 of the released binaries and push updated formula/manifest files to their respective repos. These jobs SHALL run after `build-release` and MUST NOT require e2e to pass first.

#### Scenario: Homebrew formula is updated
- **WHEN** `build-release` completes and macOS binaries are available on the GitHub release
- **THEN** `Formula/dotagents.rb` in `soorya-u/homebrew-dotagents` is updated with the new version and correct SHA256 for both `aarch64-apple-darwin` and `x86_64-apple-darwin`
- **THEN** the commit is pushed directly to `main` of `soorya-u/homebrew-dotagents`

#### Scenario: Scoop manifest is updated
- **WHEN** `build-release` completes and the Windows binary is available on the GitHub release
- **THEN** `bucket/dotagents.json` in `soorya-u/scoop-dotagents` is updated with the new version and correct SHA256 for `x86_64-pc-windows-msvc`
- **THEN** the commit is pushed directly to `main` of `soorya-u/scoop-dotagents`

### Requirement: All required secrets must be documented by name
The pipeline MUST document the four required secrets by exact name so any developer can configure them. The pipeline MUST fail with a clear error (not silently skip) if a required secret is absent.

#### Scenario: Missing CRATES_IO_TOKEN fails publish-cargo visibly
- **WHEN** `CRATES_IO_TOKEN` is not set in repository secrets
- **THEN** the `publish-cargo` job fails with a cargo authentication error
- **THEN** the failure is visible in the GitHub Actions run summary

#### Scenario: Missing RELEASE_PAT fails Homebrew/Scoop update visibly
- **WHEN** `RELEASE_PAT` is not set or is expired
- **THEN** the `update-homebrew` and `update-scoop` jobs fail with a git push authentication error
- **THEN** the failure is visible in the GitHub Actions run summary
