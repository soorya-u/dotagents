## Context

The repo has one existing workflow (`generate-registry.yml`) that runs on pushes to `main` affecting `public/v1/templates/**`. No workflow runs on pull requests or on general code changes. The local developer workflow uses `mise check` (fmt + clippy) and `mise tests` (unit + integration + e2e), but nothing enforces this in CI. The Rust toolchain is pinned to `1.92` via `mise.toml`; the TypeScript e2e suite uses Bun `1.3`.

## Goals / Non-Goals

**Goals:**
- Block merging PRs that have fmt violations, clippy warnings, failing unit/integration tests, or platform compile errors
- Keep PR CI fast — all jobs run in parallel, no sequential dependencies
- Pin the exact same Rust version in CI as in local dev (`mise.toml`)
- Verify the crate is publishable on every PR via `--dry-run`

**Non-Goals:**
- Running e2e tests on PRs (slow, requires release binary — reserved for release pipeline)
- Auto-fixing formatting (CI checks only, never mutates)
- Setting up branch protection rules (done manually in GitHub settings)
- Caching Bun packages (minimal gain for `bunx biome check`)

## Decisions

**`dtolnay/rust-toolchain@1.92` over `rustup update`**
The reference project uses `rustup update` (latest stable). We pin to `1.92` to match `mise.toml` exactly — using latest stable in CI would mean CI and local could silently diverge. `dtolnay/rust-toolchain` is the standard action for pinning.

**`cargo check` (not `cargo build`) for the cross-platform matrix**
`cargo build` produces a binary but takes significantly longer. `cargo check` performs full type-checking and catches all compile errors without linking — sufficient to validate platform compatibility. Tests only run on ubuntu-latest where we have a consistent environment.

**`--locked` on all cargo commands**
Ensures `Cargo.lock` is always respected in CI. Without `--locked`, Cargo may silently update dependencies on a cache miss, making CI non-deterministic.

**`fail-fast: false` on the platform matrix**
With `fail-fast: true` (default), a Windows compile failure would cancel the macOS jobs mid-run, hiding additional errors. Seeing all four platform results on every run is more useful for diagnosis.

**Five separate jobs over one job with sequential steps**
Independent jobs run in parallel on GitHub Actions. A fmt failure doesn't block the test job from running — developers see all failures at once rather than fixing one gate at a time.

**`cargo publish --dry-run` as a separate job**
Catches stale `include`/`exclude` patterns in `Cargo.toml` and missing files before they become a real publish failure. Costs one extra runner but saves a broken release.

## Risks / Trade-offs

- [musl target on ubuntu-latest requires `musl-tools`] → Add `sudo apt-get install -y musl-tools` step before `cargo check` in the ubuntu musl matrix leg
- [check-publish may fail if `Cargo.lock` is excluded from the published crate] → Verify `Cargo.toml` publish settings; `--dry-run` will surface this immediately
- [Windows cargo check may be slower than other platforms] → Acceptable; `fail-fast: false` means it doesn't block other jobs
