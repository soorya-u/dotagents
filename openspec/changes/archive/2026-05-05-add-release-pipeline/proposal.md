## Why

There is no automated release process for `dotagents` — every publish to crates.io, npm, Homebrew, and Scoop would have to be done manually and in the right order, with no safety gate. A two-stage pipeline with a mandatory review PR before anything is published gives a review window, enforces e2e tests on the release binary, and automates the otherwise error-prone cross-registry publish sequence.

## What Changes

- Add `cliff.toml` — git-cliff configuration to generate `CHANGELOG.md` from conventional commits (`feat:`, `fix:`, `chore:` sections)
- Add `rust-toolchain.toml` — pins `channel = "1.92"` so the toolchain is explicit for all CI jobs
- Add `.github/workflows/release-prep.yml` — Stage 1: manual `workflow_dispatch` that bumps version, generates CHANGELOG, bundles shell completions, and opens a review PR
- Add `.github/workflows/release.yml` — Stage 2: triggers on merge of `"chore: release v*"` PR; creates a git tag, builds binaries for 5 platforms, runs e2e, publishes to crates.io + npm, updates the Homebrew formula and Scoop manifest
- `soorya-u/homebrew-dotagents` repo (already scaffolded) — `Formula/dotagents.rb` updated by pipeline on every release
- `soorya-u/scoop-dotagents` repo (already scaffolded) — `bucket/dotagents.json` updated by pipeline on every release

## Capabilities

### New Capabilities

- `release-pipeline`: Defines the two-stage release process — what triggers each stage, which registries are published to, what gates publication, and which secrets are required.

### Modified Capabilities

*(none)*

## Impact

- `.github/workflows/release-prep.yml` — new file
- `.github/workflows/release.yml` — new file
- `cliff.toml` — new file at repo root
- `rust-toolchain.toml` — new file at repo root
- `CHANGELOG.md` — created on first release run
- External repos: `soorya-u/homebrew-dotagents`, `soorya-u/scoop-dotagents` — updated by pipeline
- Requires 3 secrets to be configured in GitHub repo settings: `CRATES_IO_TOKEN`, `NPM_TOKEN`, `RELEASE_PAT`
- `GITHUB_TOKEN` is auto-provided and needs no setup
