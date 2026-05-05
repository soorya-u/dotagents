## Context

No release automation exists today. The Rust toolchain is pinned to `1.92` in `mise.toml` but not in a `rust-toolchain.toml`, so CI jobs that don't use mise would float to latest stable. Commits already follow conventional commit format (`feat:`, `fix:`, `chore:`), making CHANGELOG generation via `git-cliff` directly applicable. Two external distribution repos are already scaffolded: `soorya-u/homebrew-dotagents` (with placeholder SHA256 in `Formula/dotagents.rb`) and `soorya-u/scoop-dotagents` (with placeholder SHA256 in `bucket/dotagents.json`).

## Goals / Non-Goals

**Goals:**
- Give the developer a mandatory review window (the release PR) before anything reaches a registry
- Gate all publish jobs behind e2e passing on the actual release binary
- Automate SHA256 computation and cross-repo formula/manifest updates so no manual steps are needed after merging the release PR
- Publish to five destinations atomically from a single merge: GitHub Releases, crates.io, npm, Homebrew, Scoop

**Non-Goals:**
- Submitting to the official `scoop-extras` bucket (future work once the project has traction)
- Supporting Intel macOS (`x86_64-apple-darwin`) in npm packages — binary is built and uploaded to GitHub releases but the npm shim only auto-installs on platforms with a published package
- Automated version bump without human review (always goes through a PR)

## Decisions

**Two-stage over single tag-triggered pipeline**
A single tag-push pipeline (the common pattern) immediately starts building and publishing — there's no window to edit the CHANGELOG before it's attached to a GitHub release. The two-stage model separates "prepare" from "publish": Stage 1 is cheap (just files), Stage 2 only fires after a deliberate merge. Alternative considered: `release-plz` (auto-creates release PRs on every push to main) — rejected because Q4 specified manual dispatch, not automatic.

**Tag-based trigger for Stage 2**
Stage 1 pushes an annotated tag (`v$VERSION`) after the release PR is merged. Stage 2 triggers on `on: push: tags: 'v*.*.*'`. Alternative considered: filtering `push` to `main` by `startsWith(github.event.head_commit.message, 'chore: release v')` — rejected because commit message matching is fragile (squash merges can reword the title, and any manual push with a matching prefix fires the pipeline unexpectedly). The tag-based approach is the pattern used by widely-deployed Rust tooling (e.g. t3code, cargo-dist) and is unambiguous: a tag only exists if Stage 1 explicitly created it. Another alternative: a dedicated `release/*` branch — adds branch management overhead with no benefit since we already have the PR as the review gate.

**Preflight job gates all build jobs**
A `preflight` job runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test --locked` on `ubuntu-latest` before any `build-release` matrix leg starts. `build-release` declares `needs: [preflight]`. This prevents burning 5 build runners (including macOS and Windows which consume more minutes) on a commit that fails basic checks. Alternative: skip preflight and let the CI workflow catch it — cheaper in the happy path but wasteful on failures.

**Commits to Homebrew/Scoop repos attributed to `github-actions[bot]`**
The `update-homebrew` and `update-scoop` jobs set `git config user.name "github-actions[bot]"` and `git config user.email "41898282+github-actions[bot]@users.noreply.github.com"` before committing. GitHub recognizes that email and renders commits with the bot avatar. Authentication uses `RELEASE_PAT` (owned by the repo author) — authorship and auth are separate in git. Alternative: a GitHub App with auto-rotating tokens — more secure, no expiry concern, but requires one-time org-level setup. Current choice: `RELEASE_PAT` with documented expiry reminder, upgrade to GitHub App if PAT rotation becomes painful.

**`cross` crate for linux-arm64-musl**
`aarch64-unknown-linux-musl` cannot be cross-compiled natively on ubuntu-latest runners without QEMU or a cross-compilation toolchain. `cross` (cargo install cross) provides a Docker-based cross-compilation environment and is the standard approach for this target. Alternative: GitHub's ARM runners — more expensive and overkill for a binary build.

**e2e gates all publish jobs, runs on linux only**
e2e tests validate the release binary end-to-end. Running on the `linux-x64-musl` binary is sufficient to catch regressions before publishing. Running e2e on all 5 platforms would require PTY support on Windows and macOS runners — complex and slow. If the Linux binary passes e2e, all publish jobs proceed; failure aborts the entire release.

**Direct push to Homebrew/Scoop repos over opening PRs**
Opening a PR to the tap/bucket repo on each release would require a human to merge it before the formula is live — defeating the automation goal. Direct push to `main` in both external repos is safe because those repos contain only the formula/manifest (no source code, no risk of breaking changes). Access is controlled via a fine-grained `RELEASE_PAT` scoped to only those two repos.

**npm shim pattern for binary distribution**
Platform-specific packages (`@dotagents/linux-x64` etc.) each contain only the binary for their platform. The root `dotagents` package's `postinstall` script detects the platform and copies the correct binary to `node_modules/.bin/`. Alternative: `cargo-dist` — handles this automatically but is opinionated about the whole release flow and conflicts with the custom two-stage model. Manual shim gives full control.

## Risks / Trade-offs

- [cross crate install adds ~60s to linux-arm64 build job] → Acceptable; `Swatinem/rust-cache@v2` caches the `cross` binary after first install
- [RELEASE_PAT expiry breaks Homebrew/Scoop updates silently] → Document PAT expiry date in repo secrets description; the release job will fail visibly if the PAT is expired
- [npm publish order matters — platform packages must exist before root shim] → Publish platform packages sequentially before the root package in the publish-npm job script

## Migration Plan

1. Add `rust-toolchain.toml` and `cliff.toml` to repo root
2. Configure GitHub secrets: `CRATES_IO_TOKEN`, `NPM_TOKEN`, `RELEASE_PAT`
3. Add `.github/workflows/release-prep.yml` and `.github/workflows/release.yml`
4. Trigger Stage 1 via `workflow_dispatch` with version `0.1.0` for the first release
5. Review and merge the generated PR
6. Push annotated tag `v0.1.0` to trigger Stage 2
7. Verify Stage 2 completes all jobs successfully (preflight → build-release → e2e → publish-cargo, publish-npm, update-homebrew, update-scoop)

Rollback: if Stage 2 partially fails after tag creation, delete the tag (`git push --delete origin v{version}`), fix the issue, and re-trigger by pushing the tag again.
