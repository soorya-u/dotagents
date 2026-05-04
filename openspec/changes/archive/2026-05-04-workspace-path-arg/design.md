## Context

Workspace resolution in dotagents is rooted in a `OnceLock<Result<PathBuf, String>>` in `src/utils/path.rs`. On first call, `get_workspace_dir()` walks up from `std::env::current_dir()` until it finds a directory containing `.dotagents/`. All downstream path helpers (`get_application_dir`, `get_commands_dir`, `get_skills_dir`) and template variables (`dir.workspace`, `dir.application`) derive from this lock.

`dotagents init` is the exception — it never calls `get_workspace_dir()`. It resolves the target directory by joining the hardcoded `ROOT_DIR` constant to the implicit CWD.

The constraint driving the design is that `OnceLock` can only be set once per process. Any workspace override must be injected before the first call to `get_workspace_dir()`.

## Goals / Non-Goals

**Goals:**
- `init [PATH]`, `deploy [PATH]`, `undeploy [PATH]` accept an optional positional argument.
- Omitting `PATH` preserves existing CWD-walk behaviour exactly.
- Both absolute and relative paths work; relative paths are resolved against CWD at parse-call time.
- `init PATH` creates `PATH` (and any missing parents) before scaffolding `.dotagents/` inside it.
- `deploy PATH` / `undeploy PATH` validate that `PATH/.dotagents/` exists before proceeding.
- `{{ dir.workspace }}` and `{{ dir.application }}` in templates reflect the overridden path.
- The interactive init wizard still runs when `PATH` is provided.

**Non-Goals:**
- Adding path control to `commands` or `skills` subcommands (tracked separately in issue #59).
- A global `--dir` flag on the top-level `Options` struct.
- Resetting or re-initialising the `WORKSPACE_DIR` lock between calls (single-process assumption).

## Decisions

### D1 — Positional `Option<PathBuf>` on each command's options struct

**Decision:** Add `pub dir: Option<PathBuf>` (no Clap `long`/`short` — positional) to `InitOptions`, `DeployOptions`, and `UndeployOptions`.

**Rationale:** Keeps the argument per-command (matching the issue's "first argument" framing) rather than a global flag. `Option` lets us distinguish "not provided" (walk from CWD) from "provided as `.`" and avoids binding to CWD at parse time.

**Alternatives considered:**
- Global `--dir` flag on `Options`: rejected — it doesn't fit the positional UX and complicates clap subcommand parsing.
- `PathBuf` with `default_value = "."`: rejected — evaluates at parse time, not runtime; hides the "was it provided?" signal.

### D2 — Pre-populate `WORKSPACE_DIR` via `override_workspace_dir()`

**Decision:** Add a `pub fn override_workspace_dir(path: PathBuf) -> Result<()>` function to `src/utils/path.rs` that validates the path contains `ROOT_DIR` and calls `WORKSPACE_DIR.set(Ok(path))`. Call it at the very top of `deploy()` and `undeploy()`, before `AppConfig::from_application`.

**Rationale:** The OnceLock is already the single source of truth for all path resolution. Pre-populating it is the minimal-change, zero-refactor approach. All callers downstream automatically pick up the override with no signature changes.

**Alternatives considered:**
- Threaded path parameter: would require changing every function in the call chain (`AppConfig::from_application`, `get_application_dir`, feature loaders, templater) — high blast radius for no gain.
- Temporarily set `std::env::current_dir`: side-effectful and not thread-safe; breaks parallel `rayon` rendering in deploy.

### D3 — Resolve relative paths at call time via `current_dir().join(path)`

**Decision:** Resolve `opts.dir` by joining it to `std::env::current_dir()` at the start of each command handler, then canonicalize when the path already exists (`deploy`/`undeploy`). For `init`, create the path with `fs::create_dir_all` before canonicalization.

**Rationale:** Resolving at call time (not parse time) is idiomatic Rust CLI practice and ensures the path reflects the real CWD when the process starts. `canonicalize` requires the path to exist, so `init` must create it first.

**Alternatives considered:**
- `PathBuf::from(path).canonicalize()` at parse: fails if path doesn't exist yet (breaks `init` with a new directory).

### D4 — `init` creates `PATH` with `fs::create_dir_all`

**Decision:** When `PATH` is provided to `init`, call `fs::create_dir_all(&workspace)` before the existing `fs::create_dir(&main_dir)` logic. Error if `create_dir_all` fails.

**Rationale:** The user expects `dotagents init ~/newproject` to "just work" without manually creating the directory first. This is consistent with how `git init <dir>` behaves.

## Risks / Trade-offs

- **OnceLock test isolation** → The `WORKSPACE_DIR` lock is process-global. Tests that call `override_workspace_dir` can bleed into other tests running in the same process. The existing CLAUDE.md caveat about `WORKSPACE_DIR` already flags this; new tests must use isolated temp dirs and avoid relying on the lock's state. Mitigation: integration tests for this feature should run as separate processes (e2e suite already does this).

- **Silent no-op on double-set** → `OnceLock::set` silently ignores a second set. If `override_workspace_dir` is called after `get_workspace_dir()` has already fired, the override is dropped. Mitigation: always call `override_workspace_dir` as the very first line of the command handler, before any other workspace-touching code.

- **No validation for `init` path** → For `init`, we cannot validate that `PATH/.dotagents` does NOT already exist at the override stage (that check happens later in `initialize_agents_dir` via the `--force` / TUI overwrite flow). This is fine — the existing logic already handles it.

## Open Questions

None. All design decisions were resolved during the explore session.
