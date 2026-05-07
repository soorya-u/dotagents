## Context

No release automation exists today. The Rust toolchain is pinned to `1.92` in `mise.toml` and `rust-toolchain.toml`. Commits already follow conventional commit format (`feat:`, `fix:`, `chore:`). Two external distribution repos are already scaffolded: `soorya-u/homebrew-dotagents` and `soorya-u/scoop-dotagents`.

## Goals / Non-Goals

**Goals:**
- Give the developer a mandatory review window (the release PR) before anything reaches a registry
- Gate all publish jobs behind e2e passing on the actual release binary
- Automate SHA256 computation and cross-repo formula/manifest updates so no manual steps are needed after merging the release PR
- Publish to six destinations from a single merge: GitHub Releases, crates.io, npm, PyPI, Homebrew, Scoop
- Support pre-release channels (nightly, alpha, beta, rc) across all registries

**Non-Goals:**
- Submitting to the official `scoop-extras` bucket (future work once the project has traction)
- Automated version bump without human review (always goes through a PR)

## Decisions

**Three-stage pipeline: release-prep → release-tag → release**
Stage 1 (`release-prep`) bumps version and opens a PR. Stage 2 (`release-tag`) creates an annotated tag after the PR is merged (triggered by PR merge or manual dispatch). Stage 3 (`release`) triggers on the tag push, builds binaries, runs e2e, and publishes to all registries in parallel. This separation ensures a mandatory review window and clean automation.

**Tag-based trigger for Stage 3**
The `release` workflow triggers on `push: tags: 'v*.*.*'` and `workflow_dispatch`. Tag-based triggers are unambiguous and the standard pattern in Rust tooling.

**GitHub auto-generated release notes instead of git-cliff**
Originally planned to use git-cliff for CHANGELOG generation. Instead, `softprops/action-gh-release` with `generate_release_notes: true` handles release notes directly from GitHub's commit comparison. This eliminated the need for `cliff.toml` and `CHANGELOG.md`.

**OIDC trusted publishing for npm and PyPI (no tokens)**
npm uses Node 24 (npm 11.x) with `--provenance` flag for OIDC-based tokenless publishing. PyPI uses `pypa/gh-action-pypi-publish` with OIDC. Both require `id-token: write` permission and `environment: production` to match trusted publisher configurations. This eliminates the need for `NPM_TOKEN` — only `CRATES_IO_TOKEN` and `RELEASE_PAT` are stored as secrets.

Key npm OIDC requirements discovered during implementation:
- Node 24 required (npm 10.x on Node 22 lacks tokenless auth)
- `registry-url` must NOT be set in `actions/setup-node` (creates placeholder token that overrides OIDC)
- `.npmrc` must be cleared after setup-node
- `NODE_AUTH_TOKEN` and `NPM_TOKEN` must be set to empty string on publish step
- All `package.json` files must include `repository.url` matching the GitHub repo for provenance validation
- Trusted publisher config on npmjs.com must match workflow filename and environment name

**Preflight job gates all build jobs**
A `preflight` job runs format, lint, and test checks before any build starts. This prevents burning 5 build runners on a commit that fails basic checks.

**e2e with continue-on-error**
The `e2e` job has `continue-on-error: true` so flaky test failures don't block publishing. All publish and distribution jobs still depend on `needs: [e2e]` for ordering.

**Cleanup job on failure**
If `build-release` or `e2e` fails, a `cleanup` job deletes the GitHub release and tag to allow clean retries.

**`cross` crate for linux-arm64-musl**
`aarch64-unknown-linux-musl` cannot be cross-compiled natively on ubuntu-latest runners. `cross` provides a Docker-based cross-compilation environment.

**Direct push to Homebrew/Scoop repos**
Direct push to `main` in external repos is safe because they contain only formula/manifest files. Access controlled via `RELEASE_PAT`.

**npm shim pattern for binary distribution**
Platform-specific packages (`@soorya-u/dotagents-linux-x64` etc.) each contain only the binary. The root `@soorya-u/dotagents` package's `postinstall` script detects the platform and copies the correct binary.

**PyPI wheel filenames use underscores**
Wheel filenames, `.data/`, and `.dist-info/` directories must use underscores (`py_dotagents`) per PEP 427. Hyphens in the distribution name cause ambiguous filename parsing (name vs version boundary).

**Pre-release channel support**
Pre-release versions are handled per-registry: npm uses dist-tags (e.g. `rc`, `nightly`), PyPI converts to PEP 440 (e.g. `0.2.0rc1`, `0.2.0.dev0`), Homebrew/Scoop update channel-specific files, GitHub marks the release as `prerelease: true`.

## Risks / Trade-offs

- [cross crate install adds ~60s to linux-arm64 build job] → Acceptable; `Swatinem/rust-cache@v2` caches after first install
- [RELEASE_PAT expiry breaks Homebrew/Scoop updates silently] → Document PAT expiry date; job will fail visibly if expired
- [npm publish order matters — platform packages must exist before root shim] → Publish platform packages sequentially before root package
- [npm trusted publisher workflow filename must match] → Trusted publisher config on npmjs.com must be updated if workflow is renamed
- [continue-on-error on e2e means broken binaries could be published] → Acceptable trade-off for pipeline reliability; e2e results are still visible in the run summary

## Migration Plan

1. Add `rust-toolchain.toml` to repo root
2. Configure GitHub secrets: `CRATES_IO_TOKEN`, `RELEASE_PAT`
3. Configure trusted publishers: npm (npmjs.com), PyPI (pypi.org)
4. Create `production` environment in GitHub repo settings
5. Add `.github/workflows/release-prep.yml`, `.github/workflows/release-tag.yml`, and `.github/workflows/release.yml`
6. Trigger `release-prep` via `workflow_dispatch` with the target version
7. Review and merge the generated PR
8. `release-tag` automatically creates the tag and triggers the release pipeline
9. Verify all jobs complete successfully
