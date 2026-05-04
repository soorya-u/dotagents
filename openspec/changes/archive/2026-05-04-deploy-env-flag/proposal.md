## Why

The `deploy` command always loads environment variables from a single hardcoded location (`.dotagents/.env`), making it impossible to switch between environments (dev/staging/prod) or reference `.env` files stored outside the `.dotagents/` directory without editing that file in place.

## What Changes

- Add an `--env <path>` flag to the `deploy` command, repeatable for multiple files
- When `--env` is provided, the specified files replace `.dotagents/.env` entirely (default is not loaded)
- When `--env` is not provided, behaviour is unchanged (`.dotagents/.env` loaded, silently ignored if missing)
- Multiple `--env` files are merged left-to-right; later files override earlier ones on duplicate keys
- A specified `--env` file that does not exist is a hard error

## Capabilities

### New Capabilities

- `deploy-env-flag`: `--env` flag on the `deploy` command allowing users to specify one or more custom `.env` files that replace the default `.dotagents/.env` when loading `env.*` template variables

### Modified Capabilities

<!-- No existing spec-level behaviour changes -->

## Impact

- `src/cli/options.rs` — `DeployOptions` gains an `env: Vec<PathBuf>` field
- `src/templates/variables.rs` — `get_env_variables()` reads from a `OnceLock<Vec<PathBuf>>` instead of always using the application dir default; new `set_env_paths()` setter
- `src/cli/deploy.rs` — calls `set_env_paths(options.env)` before `get_templater()` fires
- No changes to templating pipeline, config loading, or other commands
