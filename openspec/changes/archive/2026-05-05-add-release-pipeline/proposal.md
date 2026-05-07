## Why

There is no automated release process for `dotagents` — every publish to crates.io, npm, PyPI, Homebrew, and Scoop would have to be done manually and in the right order, with no safety gate. A three-stage pipeline with a mandatory review PR before anything is published gives a review window, enforces e2e tests on the release binary, and automates the otherwise error-prone cross-registry publish sequence.

## What Changes

- Add `rust-toolchain.toml` — pins `channel = "1.92"` so the toolchain is explicit for all CI jobs
- Add `.github/workflows/release-prep.yml` — Stage 1: manual `workflow_dispatch` that bumps version and opens a review PR
- Add `.github/workflows/release-tag.yml` — Stage 2: creates annotated tag after release PR merge and triggers the release workflow
- Add `.github/workflows/release.yml` — Stage 3: triggers on tag push; creates GitHub release, builds binaries for 5 platforms, runs e2e, publishes to crates.io + npm + PyPI, updates Homebrew formula and Scoop manifest
- Add `scripts/ci/publish_npm.sh` — publishes npm platform packages and root shim via OIDC trusted publishing
- Add `scripts/ci/publish_pypi.sh` — builds PEP 427 wheels with PEP 440 version conversion
- Add `scripts/ci/update_homebrew.sh` — updates Homebrew formula with SHA256 and version
- Add `scripts/ci/update_scoop.sh` — updates Scoop manifest with SHA256 and version
- `soorya-u/homebrew-dotagents` repo — formula updated by pipeline on every release
- `soorya-u/scoop-dotagents` repo — manifest updated by pipeline on every release

## Capabilities

### New Capabilities

- `release-pipeline`: Defines the three-stage release process — what triggers each stage, which registries are published to, what gates publication, and which secrets/trusted publishers are required.

### Modified Capabilities

*(none)*

## Impact

- `.github/workflows/release-prep.yml` — new file
- `.github/workflows/release-tag.yml` — new file
- `.github/workflows/release.yml` — new file
- `scripts/ci/publish_npm.sh` — new file
- `scripts/ci/publish_pypi.sh` — new file
- `scripts/ci/update_homebrew.sh` — new file
- `scripts/ci/update_scoop.sh` — new file
- `rust-toolchain.toml` — new file at repo root
- External repos: `soorya-u/homebrew-dotagents`, `soorya-u/scoop-dotagents` — updated by pipeline
- Requires `CRATES_IO_TOKEN` and `RELEASE_PAT` secrets in GitHub repo settings
- Requires npm and PyPI trusted publisher configuration (OIDC, no tokens stored)
- Requires `production` environment in GitHub repo settings
- `GITHUB_TOKEN` is auto-provided and needs no setup
