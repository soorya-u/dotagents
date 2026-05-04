## Why

Every pull request to `dotagents` currently merges without any automated check — no formatting, no lint, no tests. The only CI workflow in the repo (`generate-registry.yml`) runs only on template changes, not on code changes. This means broken Rust code, clippy warnings, and failing tests can all land on `main` silently.

## What Changes

- Add `.github/workflows/ci.yml` with five parallel jobs that gate every PR and push to `main`:
  - **fmt** — enforces `cargo fmt` (read-only check, not auto-fix) and `biome check` on the TypeScript e2e suite
  - **clippy** — runs `cargo clippy --locked -- -D warnings` (warnings are hard errors)
  - **test** — runs unit tests (`cargo test --bin dotagents`) and integration tests (`cargo test --test integration`); e2e is excluded from PR CI and reserved for the release pipeline
  - **check** — compiles the binary for all four target platforms (Linux musl, macOS x86, macOS arm64, Windows msvc) to catch platform-specific compile errors without running tests on every PR
  - **check-publish** — runs `cargo publish --dry-run` to catch missing files or bad Cargo metadata before a real release

## Capabilities

### New Capabilities

- `pr-ci`: Defines the automated checks that must pass on every pull request before code reaches `main`.

### Modified Capabilities

*(none)*

## Impact

- `.github/workflows/ci.yml` — new file
- No changes to source code, `Cargo.toml`, or `mise.toml`
- Requires no new secrets — `GITHUB_TOKEN` is auto-provided
