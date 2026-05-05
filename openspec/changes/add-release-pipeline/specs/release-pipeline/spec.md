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

### Requirement: Stage 2 only triggers on an explicit version tag push
The `release` workflow SHALL trigger on `push: tags: 'v*.*.*'`. It MUST NOT trigger on branch pushes. The tag is created and pushed manually by the developer after the release PR is merged.

#### Scenario: Non-tag push does not trigger Stage 2
- **WHEN** a regular feature commit is pushed to `main`
- **THEN** the `release` workflow is not started at all

#### Scenario: Tag push triggers full pipeline
- **WHEN** the developer merges the `"chore: release v0.2.0"` PR and pushes the annotated tag `v0.2.0`
- **THEN** all Stage 2 jobs run: build-release, e2e, publish-cargo, publish-npm, update-homebrew, update-scoop

### Requirement: A preflight job gates all build and publish work
Stage 2 SHALL run a `preflight` job as the first job. All `build-release` matrix legs SHALL declare `needs: [preflight]` and MUST NOT start if `preflight` fails. `preflight` MUST run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test --locked`.

#### Scenario: Fmt violation aborts release
- **WHEN** the tagged commit contains unformatted Rust code
- **THEN** the `preflight` job exits non-zero
- **THEN** all `build-release` legs, `e2e`, and all publish jobs are skipped

#### Scenario: Clean commit proceeds to build
- **WHEN** `preflight` passes fmt, clippy, and tests
- **THEN** all five `build-release` matrix legs start in parallel

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
The `update-homebrew` and `update-scoop` jobs SHALL compute the SHA256 of the released binaries and push updated formula/manifest files to their respective repos. These jobs SHALL declare `needs: [build-release, e2e]` — they MUST NOT start if e2e fails, to avoid pushing a formula that points to a broken binary.

#### Scenario: Homebrew formula is updated
- **WHEN** `build-release` completes and macOS binaries are available on the GitHub release
- **THEN** `Formula/dotagents.rb` in `soorya-u/homebrew-dotagents` is updated with the new version and correct SHA256 for both `aarch64-apple-darwin` and `x86_64-apple-darwin`
- **THEN** the commit is pushed directly to `main` of `soorya-u/homebrew-dotagents`

#### Scenario: Scoop manifest is updated
- **WHEN** `build-release` completes and the Windows binary is available on the GitHub release
- **THEN** `bucket/dotagents.json` in `soorya-u/scoop-dotagents` is updated with the new version and correct SHA256 for `x86_64-pc-windows-msvc`
- **THEN** the commit is pushed directly to `main` of `soorya-u/scoop-dotagents`

### Requirement: All required secrets must be documented by name
The pipeline MUST document the following secrets by exact name so any developer can configure them. The pipeline MUST fail with a clear error (not silently skip) if a required secret is absent.

| Secret | Required by | How to obtain |
|---|---|---|
| `GITHUB_TOKEN` | All jobs | Auto-provided by GitHub Actions — no setup needed |
| `CRATES_IO_TOKEN` | `publish-cargo` | crates.io → Account Settings → API Tokens |
| `NPM_TOKEN` | `publish-npm` | npmjs.com → Access Tokens → Automation token |
| `RELEASE_PAT` | `update-homebrew`, `update-scoop` | Fine-grained PAT with Contents read+write on `soorya-u/homebrew-dotagents` and `soorya-u/scoop-dotagents` |

#### Scenario: Missing CRATES_IO_TOKEN fails publish-cargo visibly
- **WHEN** `CRATES_IO_TOKEN` is not set in repository secrets
- **THEN** the `publish-cargo` job fails with a cargo authentication error
- **THEN** the failure is visible in the GitHub Actions run summary

#### Scenario: Missing RELEASE_PAT fails Homebrew/Scoop update visibly
- **WHEN** `RELEASE_PAT` is not set or is expired
- **THEN** the `update-homebrew` and `update-scoop` jobs fail with a git push authentication error
- **THEN** the failure is visible in the GitHub Actions run summary
