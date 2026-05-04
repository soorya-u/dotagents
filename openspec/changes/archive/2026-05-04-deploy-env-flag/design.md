## Context

`Templater` is a `LazyLock` global (`TEMPLATER: LazyLock<Templater>`) initialised on first call to `get_templater()`. During initialisation, `load_default_variables()` calls `get_env_variables()`, which resolves the env file path as `get_application_dir().join(".env")` — hardcoded, no way to inject a custom path from the CLI layer.

`deploy.rs` calls `get_templater()` after CLI options are parsed, which means there is a window between option parsing and the LazyLock firing where we can communicate the custom paths.

There is already a precedent in the codebase for this pattern: `WORKSPACE_DIR` uses a `OnceLock` that is set once before first use (see `utils/path.rs`).

## Goals / Non-Goals

**Goals:**
- Allow users to supply one or more custom `.env` files via `--env <path>` on the `deploy` subcommand
- When `--env` is provided, replace `.dotagents/.env` entirely (no implicit default loading)
- Merge multiple files left-to-right (later files win on duplicate keys)
- Hard-error if a specified file does not exist
- Keep env vars available to both config rendering and feature rendering (same as today)

**Non-Goals:**
- Applying `--env` to other subcommands (`init`, `gen-completions`)
- Supporting environment variable overrides inline (`--env KEY=VALUE`)
- Layering on top of `.dotagents/.env` (additive mode)

## Decisions

### Use a `OnceLock<Vec<PathBuf>>` in `variables.rs`

**Decision**: Introduce `ENV_PATHS: OnceLock<Vec<PathBuf>>` in `variables.rs`. `set_env_paths(paths)` writes to it once. `get_env_variables()` reads it: if set and non-empty, load those files; if unset or empty, fall back to the default `.dotagents/.env` (existing behaviour).

**Alternatives considered**:
- *Thread paths through `Templater::new()`*: Would require changing `Templater`'s constructor signature, updating the `LazyLock` closure, and plumbing the paths through `AppConfig::from_application`. Much higher surface area.
- *Move env loading out of the LazyLock entirely*: Would require the renderer to accept env vars as a separate argument at every call site. Breaks the existing `globals`-based model and touches many files.
- *Re-initialise the LazyLock*: Not possible in stable Rust without `unsafe`.

The `OnceLock` approach is the minimal, surgical change: one new global, one setter, one modified reader — no pipeline changes.

### Path resolution relative to CWD

**Decision**: Paths passed to `--env` are resolved relative to the current working directory (standard CLI behaviour), not relative to `.dotagents/`.

**Rationale**: Users invoking `dotagents deploy --env ./envs/prod.env` expect shell-standard path semantics. Resolving against the `.dotagents/` dir would be surprising and undocumented.

### Hard error on missing explicitly-specified file

**Decision**: If a path is supplied via `--env` and the file does not exist, `get_env_variables()` returns an `Err`. The default `.dotagents/.env` path retains its silent-ignore behaviour.

**Rationale**: Silently loading nothing when the user mistyped a path produces confusing template output with empty `env.*` variables. An explicit path implies intent; a missing file is almost certainly a mistake.

### Left-to-right merge, later files win

**Decision**: When multiple `--env` files are given, they are loaded in order and duplicate keys are overwritten by later entries.

**Rationale**: Matches the convention of docker-compose env files, dotenv-flow, and most dotenv tooling. Enables a natural `--env base.env --env prod.env` layering pattern.

## Risks / Trade-offs

- **`OnceLock` test interference**: Like `WORKSPACE_DIR`, the `OnceLock` is process-global. Tests that call `set_env_paths` will affect any test running in the same process. Mitigation: unit tests for `get_env_variables` should not call `set_env_paths` and instead test the function with the lock unset (its default state); e2e tests run in isolated subprocesses so are unaffected.
- **`set_env_paths` called after `get_templater()`**: If a caller invokes `get_templater()` before `set_env_paths()`, env vars silently use the default path. Mitigation: `deploy.rs` must call `set_env_paths` immediately after parsing options, before any templater use — this is enforced by code order, not a runtime check.

## Open Questions

None — all decisions resolved during exploration.
