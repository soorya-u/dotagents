## 1. Core Implementation

- [x] 1.1 Add `ENV_PATHS: OnceLock<Vec<PathBuf>>` global and `set_env_paths(paths: Vec<PathBuf>)` setter in `src/templates/variables.rs`
- [x] 1.2 Update `get_env_variables()` in `src/templates/variables.rs` to read from `ENV_PATHS`: if set and non-empty load those files (hard error on missing), otherwise fall back to default `.dotagents/.env` (silent ignore)
- [x] 1.3 Merge multiple env files left-to-right in `get_env_variables()`, with later files overriding earlier ones on duplicate keys
- [x] 1.4 Add `#[clap(long)] pub env: Vec<PathBuf>` field to `DeployOptions` in `src/cli/options.rs`
- [x] 1.5 Call `set_env_paths(options.env)` in `src/cli/deploy.rs` immediately after options are received, before any call to `get_templater()`

## 2. Unit Tests

- [x] 2.1 Add unit test in `src/templates/variables.rs`: `get_env_variables()` returns empty `env` map when lock is unset and no default `.env` exists
- [x] 2.2 Add unit test: loading a single file populates `env.*` keys lowercased
- [x] 2.3 Add unit test: loading two files with a duplicate key — later file wins
- [x] 2.4 Add unit test: a path in the lock that does not exist returns an `Err`

## 3. E2E Tests

- [x] 3.1 Run `tui-devtools` as a daemon and walk through each affected flow in a temp workspace: `deploy --env <file>`, `deploy --env <a> --env <b>`, `deploy --env <missing>`, and plain `deploy` — record exact terminal output (symbols, spacing, error text) to use as assertion anchors
- [x] 3.2 Add e2e test in `tests/e2e/`: `deploy --env ./custom.env` loads vars from the custom file and not from `.dotagents/.env`
- [x] 3.3 Add e2e test: `deploy --env ./a.env --env ./b.env` merges correctly with last-wins precedence
- [x] 3.4 Add e2e test: `deploy --env ./nonexistent.env` exits non-zero with an error message referencing the missing path
- [x] 3.5 Add e2e test: `deploy` with no `--env` flag and a missing `.dotagents/.env` succeeds silently (existing behaviour unchanged)

## 4. Verification

- [x] 4.1 Run `mise check` (cargo fmt + clippy) and fix any issues
- [x] 4.2 Run `mise tests` (unit + integration + e2e) and fix any failures
