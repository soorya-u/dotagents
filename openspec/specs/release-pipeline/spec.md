## ADDED Requirements

### Requirement: Three-stage release flow — prep, tag, release
The release process uses three workflows: `release-prep` (Stage 1) opens a version-bump PR, `release-tag` (Stage 2) creates an annotated tag after the PR merges, and `release` (Stage 3) builds, tests, and publishes. This separation ensures a mandatory review window before anything reaches a registry.

#### Scenario: Dispatch opens release PR
- **WHEN** a developer triggers `release-prep` workflow with input version `"0.2.0"`
- **THEN** `Cargo.toml` version is bumped to `0.2.0`
- **THEN** `Cargo.lock` is updated via `cargo update --workspace`
- **THEN** a PR titled exactly `"chore: release v0.2.0"` is opened on branch `bot/release-v0.2.0`

#### Scenario: Stage 1 does not publish
- **WHEN** the `release-prep` workflow completes
- **THEN** no package is published to any registry

#### Scenario: Tag is created automatically after PR merge
- **WHEN** a PR from branch `bot/release-v*` is merged to `main`
- **THEN** `release-tag` creates an annotated tag `v{version}` and triggers the `release` workflow
- **WHEN** `release-tag` is triggered via `workflow_dispatch` with a version input
- **THEN** the same tag creation and release trigger occurs

### Requirement: Release workflow triggers on tag push or manual dispatch
The `release` workflow SHALL trigger on `push: tags: 'v*.*.*'` and `workflow_dispatch`. It MUST NOT trigger on branch pushes.

#### Scenario: Non-tag push does not trigger release
- **WHEN** a regular feature commit is pushed to `main`
- **THEN** the `release` workflow is not started at all

#### Scenario: Tag push triggers full pipeline
- **WHEN** the tag `v0.2.0` is pushed
- **THEN** all release jobs run: preflight, create-release, build-release, e2e, publish-cargo, publish-npm, publish-pypi, update-homebrew, update-scoop

### Requirement: A preflight job gates all build and publish work
The `preflight` job runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test --locked`. It also verifies the tag version matches `Cargo.toml`. The `create-release` job declares `needs: [preflight]`.

#### Scenario: Fmt violation aborts release
- **WHEN** the tagged commit contains unformatted Rust code
- **THEN** the `preflight` job exits non-zero
- **THEN** all downstream jobs are skipped

#### Scenario: Clean commit proceeds to build
- **WHEN** `preflight` passes fmt, clippy, and tests
- **THEN** `create-release` starts, followed by five `build-release` matrix legs in parallel

### Requirement: E2E tests gate all publish and distribution jobs
All publish jobs (`publish-cargo`, `publish-npm`, `publish-pypi`) and distribution jobs (`update-homebrew`, `update-scoop`) SHALL declare `needs: [e2e]`. The `e2e` job has `continue-on-error: true` so downstream jobs still run even if e2e has flaky failures, but the job result is visible for monitoring.

#### Scenario: E2E runs on release binary
- **WHEN** `build-release` completes
- **THEN** the `e2e` job downloads the `linux-x64-musl` binary from the GitHub release and runs the e2e test suite via `bunx @microsoft/tui-test`

### Requirement: Publish to crates.io using API token
The `publish-cargo` job uses `CARGO_REGISTRY_TOKEN` secret and runs `cargo publish --locked`. The job runs in the `production` environment.

### Requirement: Publish to npm using OIDC trusted publishing
The `publish-npm` job uses Node 24 (npm 11.x) with OIDC trusted publishing — no `NPM_TOKEN` required. The workflow clears any `.npmrc` created by `actions/setup-node` and sets `NODE_AUTH_TOKEN` and `NPM_TOKEN` to empty strings to prevent placeholder tokens from overriding OIDC. The `--provenance` flag is required on all `npm publish` commands. All `package.json` files MUST include `repository.url` matching the GitHub repo URL for provenance validation. The job runs in the `production` environment with `id-token: write` permission.

Platform-specific packages (`@soorya-u/dotagents-linux-x64`, etc.) are published first, then the root shim package (`@soorya-u/dotagents`). Pre-release versions (e.g. `0.2.0-rc`) are published under a named dist-tag (e.g. `rc`) instead of `latest`.

#### Scenario: npm OIDC publish succeeds
- **WHEN** the npm trusted publisher is configured on npmjs.com with the correct repository, workflow filename (`release.yml`), and environment (`production`)
- **THEN** `npm publish --access public --provenance --tag <dist-tag>` succeeds without any token

### Requirement: Publish to PyPI using OIDC trusted publishing
The `publish-pypi` job builds platform-specific wheels and publishes via `pypa/gh-action-pypi-publish@release/v1` using OIDC trusted publishing. Semver pre-release identifiers are converted to PEP 440 format (e.g. `0.2.0-rc.1` → `0.2.0rc1`, `0.2.0-nightly` → `0.2.0.dev0`). Wheel filenames MUST use underscores (e.g. `py_dotagents`), not hyphens, per PEP 427. The job runs in the `production` environment.

#### Scenario: PyPI publish succeeds
- **WHEN** the PyPI trusted publisher is configured with the correct repository, workflow filename, and environment
- **THEN** wheels for all five platforms are uploaded to PyPI

### Requirement: Homebrew and Scoop are updated with correct SHA256 on every release
The `update-homebrew` and `update-scoop` jobs compute SHA256 of released binaries and push updated formula/manifest files to their respective repos using `RELEASE_PAT`.

#### Scenario: Homebrew formula is updated
- **WHEN** `e2e` completes and macOS binaries are available on the GitHub release
- **THEN** the Homebrew formula in `soorya-u/homebrew-dotagents` is updated with the new version and correct SHA256 for both `aarch64-apple-darwin` and `x86_64-apple-darwin`

#### Scenario: Scoop manifest is updated
- **WHEN** `e2e` completes and the Windows binary is available on the GitHub release
- **THEN** the Scoop manifest in `soorya-u/scoop-dotagents` is updated with the new version and SHA256

### Requirement: Cleanup job deletes release on pre-publish failure
If `build-release` or `e2e` fails, the `cleanup` job deletes the GitHub release and tag to allow a clean retry.

### Requirement: Pre-release channel support
Pre-release versions (containing `-` in the semver string, e.g. `0.2.0-rc`, `0.2.0-nightly`) are handled across all registries:
- **GitHub**: release is marked as `prerelease: true`
- **npm**: published under a named dist-tag (e.g. `rc`, `nightly`) instead of `latest`
- **PyPI**: version converted to PEP 440 format
- **Homebrew/Scoop**: channel-specific formula/manifest files are updated

### Requirement: All required secrets and configuration must be documented
The pipeline requires the following configuration:

| Secret/Config | Required by | How to obtain |
|---|---|---|
| `GITHUB_TOKEN` | All jobs | Auto-provided by GitHub Actions |
| `CRATES_IO_TOKEN` | `publish-cargo` | crates.io → Account Settings → API Tokens |
| `RELEASE_PAT` | `update-homebrew`, `update-scoop` | Fine-grained PAT with Contents read+write on tap/bucket repos |
| npm trusted publisher | `publish-npm` | npmjs.com → package settings → Configure trusted publishers |
| PyPI trusted publisher | `publish-pypi` | pypi.org → project settings → Publishing |
| `production` environment | `publish-cargo`, `publish-npm`, `publish-pypi` | GitHub repo → Settings → Environments |
