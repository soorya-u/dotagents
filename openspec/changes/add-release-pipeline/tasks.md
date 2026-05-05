## 1. Supporting files

- [x] 1.1 Create `rust-toolchain.toml` at repo root with `[toolchain]` section pinning `channel = "1.92"`
- [x] 1.2 Create `cliff.toml` at repo root configuring git-cliff: `[git] conventional_commits = true`, sections for `feat` (Features), `fix` (Bug Fixes), `chore` (Miscellaneous) — exclude `chore: release v*` commits from the log body
- [x] 1.3 Add `CHANGELOG.md` as an empty placeholder file so git-cliff can prepend to it on first run

## 2. Configure GitHub secrets

- [ ] 2.1 Generate a crates.io API token at crates.io → Account Settings → API Tokens and add it to repo secrets as `CRATES_IO_TOKEN`
- [ ] 2.2 Generate an npm Automation token at npmjs.com → Access Tokens and add it to repo secrets as `NPM_TOKEN`
- [ ] 2.3 Create a fine-grained GitHub PAT with Contents read+write on `soorya-u/homebrew-dotagents` and `soorya-u/scoop-dotagents` and add it to repo secrets as `RELEASE_PAT`

## 3. Stage 1 — release-prep.yml

- [x] 3.1 Create `.github/workflows/release-prep.yml` with `on: workflow_dispatch` trigger and a single `version` input (required, description: "Version to release e.g. 0.2.0")
- [x] 3.2 Add step to bump version: use `sed` to replace `^version = ".*"` in `Cargo.toml` with `version = "${{ inputs.version }}"`, then run `cargo update --workspace` to sync `Cargo.lock`
- [x] 3.3 Add step to generate CHANGELOG: `cargo install git-cliff --locked` then `git-cliff --tag v${{ inputs.version }} --prepend CHANGELOG.md`
- [x] 3.4 Add step to generate shell completions: build debug binary with `cargo build --locked`, then run `./target/debug/dotagents gen-completions --shell <shell> --to completions/` for each of bash, elvish, fish, powershell, zsh, then `zip -r completions.zip completions/`
- [x] 3.5 Add step to open PR using `peter-evans/create-pull-request@v6` with title `"chore: release v${{ inputs.version }}"`, branch `bot/release-v${{ inputs.version }}`, committing `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `completions.zip`
- [x] 3.6 After the PR is merged (handled externally), add a step in a post-merge job or document that the developer must run: create annotated tag `v${{ inputs.version }}` and push it (`git tag -a v$VERSION -m "Release v$VERSION" && git push origin v$VERSION`) — this tag push is what triggers Stage 2

## 4. Stage 2 — release.yml (preflight + build)

- [x] 4.1 Create `.github/workflows/release.yml` with `on: push: tags: ['v*.*.*']` trigger
- [x] 4.2 Add `preflight` job (`runs-on: ubuntu-latest`): `actions/checkout@v4`, `dtolnay/rust-toolchain@1.92`, `Swatinem/rust-cache@v2`, then steps: `cargo fmt --all -- --check`, `cargo clippy --locked -- -D warnings`, `cargo test --locked`
- [x] 4.3 Extract version from tag ref in `preflight`: `VERSION=$(echo "${{ github.ref_name }}" | sed 's/^v//')` and verify it matches `Cargo.toml` via `grep -q "^version = \"$VERSION\"" Cargo.toml`
- [x] 4.4 Add `build-release` job (`needs: [preflight]`, `strategy.fail-fast: false`) with matrix: `linux-x64-musl/ubuntu-latest/x86_64-unknown-linux-musl`, `linux-arm64-musl/ubuntu-latest/aarch64-unknown-linux-musl`, `macos-arm64/macos-latest/aarch64-apple-darwin`, `macos-x86/macos-latest/x86_64-apple-darwin`, `windows-x64/windows-latest/x86_64-pc-windows-msvc`
- [x] 4.5 In `build-release`: add `cargo install cross --locked` step gated on `matrix.build_name == 'linux-arm64-musl'`; use `cross` for that leg and `cargo` for all others
- [x] 4.6 In `build-release`: build with `cargo build --release --locked --verbose --target ${{ matrix.target }}`
- [x] 4.7 In `build-release`: upload binary to GitHub release via `softprops/action-gh-release@v2` with asset name `dotagents-${{ matrix.build_name }}` (append `.exe` for windows leg)

## 5. Stage 2 — release.yml (e2e + publish)

- [x] 5.1 Add `e2e` job (`needs: build-release`, `runs-on: ubuntu-latest`): download `dotagents-linux-x64-musl` artifact from the release, `chmod +x`, place in `target/release/dotagents`, set up `oven-sh/setup-bun@v2`, run `cd tests/e2e && bunx tui-test`
- [x] 5.2 Add `publish-cargo` job (`needs: e2e`, `runs-on: ubuntu-latest`): `dtolnay/rust-toolchain@1.92`, `cargo publish --locked` with `CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}`
- [x] 5.3 Add `publish-npm` job (`needs: e2e`, `runs-on: ubuntu-latest`): write `package.json` for each platform package (`@dotagents/linux-x64` etc.) containing the binary, then write root `dotagents` shim `package.json` with `optionalDependencies` for each platform package and a `postinstall` script; publish platform packages first, then root; use `NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}`

## 6. Stage 2 — release.yml (Homebrew + Scoop)

- [x] 6.1 Add `update-homebrew` job (`needs: [build-release, e2e]`, `runs-on: ubuntu-latest`): download `dotagents-macos-arm64` and `dotagents-macos-x86` from the release; compute `sha256sum` for each; clone `soorya-u/homebrew-dotagents` using `RELEASE_PAT`; set `git config user.name "github-actions[bot]"` and `git config user.email "41898282+github-actions[bot]@users.noreply.github.com"`; patch `Formula/dotagents.rb` replacing version string and both sha256 fields; commit and push to `main`
- [x] 6.2 Add `update-scoop` job (`needs: [build-release, e2e]`, `runs-on: ubuntu-latest`): download `dotagents-windows-x64.exe` from the release; compute `sha256sum`; clone `soorya-u/scoop-dotagents` using `RELEASE_PAT`; set `git config user.name "github-actions[bot]"` and `git config user.email "41898282+github-actions[bot]@users.noreply.github.com"`; patch `bucket/dotagents.json` using `jq` to update `version` and `architecture["64bit"].hash`; commit and push to `main`

## 7. Verification

- [ ] 7.1 Trigger `release-prep` with version `0.1.0` via workflow_dispatch and verify the PR opens with correct title and all four files changed
- [ ] 7.2 Merge the PR, push annotated tag `v0.1.0`, and verify Stage 2 starts with jobs: `preflight`, `build-release` (×5), `e2e`, `publish-cargo`, `publish-npm`, `update-homebrew`, `update-scoop`
- [ ] 7.3 Confirm the GitHub release has five binary assets and `completions.zip` attached
- [ ] 7.4 Confirm `dotagents 0.1.0` appears on crates.io and `npm info dotagents` returns version `0.1.0`
- [ ] 7.5 Confirm `brew tap soorya-u/dotagents && brew install dotagents` installs and runs `dotagents --version` correctly
- [ ] 7.6 Confirm `scoop bucket add dotagents https://github.com/soorya-u/scoop-dotagents && scoop install dotagents` installs correctly on Windows
