## 1. Supporting files

- [x] 1.1 Create `rust-toolchain.toml` at repo root with `[toolchain]` section pinning `channel = "1.92"`

## 2. Configure secrets and trusted publishers

- [x] 2.1 Generate a crates.io API token and add to repo secrets as `CRATES_IO_TOKEN`
- [x] 2.2 Configure npm trusted publisher on npmjs.com for each scoped package — set repository, workflow filename (`release.yml`), and environment (`production`)
- [x] 2.3 Configure PyPI trusted publisher on pypi.org — set repository, workflow filename (`release.yml`), and environment (`production`)
- [x] 2.4 Create a fine-grained GitHub PAT with Contents read+write on `soorya-u/homebrew-dotagents` and `soorya-u/scoop-dotagents` and add to repo secrets as `RELEASE_PAT`
- [x] 2.5 Create `production` environment in GitHub repo settings

## 3. Stage 1 — release-prep.yml

- [x] 3.1 Create `.github/workflows/release-prep.yml` with `on: workflow_dispatch` trigger and a single `version` input
- [x] 3.2 Add step to bump version: use `sed` to replace `^version = ".*"` in `Cargo.toml`, then `cargo update --workspace` to sync `Cargo.lock`
- [x] 3.3 Add step to open PR using `peter-evans/create-pull-request@v6` with title `"chore: release v${{ inputs.version }}"`, branch `bot/release-v${{ inputs.version }}`, committing `Cargo.toml` and `Cargo.lock`

## 4. Stage 2 — release-tag.yml

- [x] 4.1 Create `.github/workflows/release-tag.yml` with triggers: PR merge on `main` (from `bot/release-v*` branches) and `workflow_dispatch` with version input
- [x] 4.2 Extract version from PR branch name or workflow input
- [x] 4.3 Create annotated tag `v{version}` and push to origin
- [x] 4.4 Trigger `release.yml` via `gh workflow run release.yml --ref v{version}`

## 5. Stage 3 — release.yml (preflight + build)

- [x] 5.1 Create `.github/workflows/release.yml` with `on: push: tags: ['v*.*.*']` and `workflow_dispatch` triggers
- [x] 5.2 Add `preflight` job: `actions/checkout@v6`, `dtolnay/rust-toolchain@1.92`, `Swatinem/rust-cache@v2`, verify tag matches Cargo.toml, `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, `cargo test --locked`
- [x] 5.3 Add `create-release` job (`needs: [preflight]`): generate shell completions, create GitHub Release with `softprops/action-gh-release@v3` and `generate_release_notes: true`
- [x] 5.4 Add `build-release` job (`needs: [create-release]`, matrix of 5 platforms): build binaries and upload to GitHub Release
- [x] 5.5 In `build-release`: use `cross` for `linux-arm64-musl` target, `cargo` for all others

## 6. Stage 3 — release.yml (e2e + publish)

- [x] 6.1 Add `e2e` job (`needs: [build-release]`, `continue-on-error: true`): download release binary, run e2e tests via `bunx @microsoft/tui-test`
- [x] 6.2 Add `publish-cargo` job (`needs: [e2e]`, `environment: production`): `cargo publish --locked` with `CARGO_REGISTRY_TOKEN`
- [x] 6.3 Add `publish-npm` job (`needs: [e2e]`, `environment: production`): Node 24, clear `.npmrc`, set `NODE_AUTH_TOKEN=""` and `NPM_TOKEN=""`, run `publish_npm.sh` with `--provenance` flag
- [x] 6.4 Create `scripts/ci/publish_npm.sh`: publish platform packages then root shim, all with `--provenance`, `repository.url` in all `package.json` files, pre-release dist-tags
- [x] 6.5 Add `publish-pypi` job (`needs: [e2e]`, `environment: production`): run `publish_pypi.sh` to build wheels, then `pypa/gh-action-pypi-publish@release/v1`
- [x] 6.6 Create `scripts/ci/publish_pypi.sh`: build platform-specific wheels with PEP 440 version conversion, underscore-based filenames (`py_dotagents`)

## 7. Stage 3 — release.yml (Homebrew + Scoop + cleanup)

- [x] 7.1 Add `update-homebrew` job (`needs: [e2e]`): download macOS binaries, compute SHA256, run `update_homebrew.sh` with `RELEASE_PAT`
- [x] 7.2 Add `update-scoop` job (`needs: [e2e]`): download Windows binary, compute SHA256, run `update_scoop.sh` with `RELEASE_PAT`
- [x] 7.3 Add `cleanup` job: delete release and tag if `build-release` or `e2e` fails

## 8. Verification

- [x] 8.1 Trigger `release-prep` and verify PR opens with correct title and version bump
- [x] 8.2 Merge PR and verify `release-tag` creates tag and triggers release
- [x] 8.3 Verify npm OIDC trusted publishing works (Node 24, no token)
- [x] 8.4 Verify PyPI OIDC trusted publishing works (wheel filenames with underscores)
- [ ] 8.5 Verify full end-to-end release with a real version (all registries succeed)
