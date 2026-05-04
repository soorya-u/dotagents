## Context

The CLI exposes a `display_error()` function that renders errors as a formatted box with a cause chain. All subcommands are expected to return `anyhow::Result<bool>` so that `main.rs` can route failures through this path. However, `Templater` is initialized in a `LazyLock<Templater>` static, which requires an infallible closure. The original author worked around this by calling `.expect()` inside the closure. When `load_default_variables()` fails (most commonly: no workspace directory found), the expect fires and the process aborts with a raw Rust panic message rather than flowing through `display_error()`. The `undeploy` subcommand does not use `get_templater()` directly and therefore already surfaces the same workspace-not-found error correctly.

## Goals / Non-Goals

**Goals:**
- Make `Templater::new()` fully fallible — remove the `.expect()` inside it.
- Change `get_templater()` to return `Result<&'static Templater>` so callers can propagate initialization failures with `?`.
- Ensure that a missing workspace directory (and any other `Templater` initialization failure) reaches `display_error()` and exits with code 1, matching `undeploy` behavior.
- Add an e2e test confirming the formatted error output on deploy failure.

**Non-Goals:**
- Replacing `anyhow` with `thiserror`. The codebase is a CLI app, not a library. `anyhow` is correct here; the bug is a single misplaced `.expect()`, not an architectural error-type problem.
- Changing `undeploy` error handling (it already works correctly).
- Caching the error result across calls (if initialization fails, the process will exit at the first call site anyway).

## Decisions

### Keep `anyhow` as the error type

`thiserror` is appropriate for libraries that need to expose typed error variants to downstream consumers. `dotagents` is a CLI binary; callers (i.e. the OS) only receive an exit code. `anyhow` with `.context()` chains already produces informative error messages through `display_error()`. Switching to `thiserror` would add ceremony without benefit.

### Use `OnceLock<Templater>` instead of `LazyLock<Templater>`

`LazyLock` requires an infallible initializer (`FnOnce() -> T`). There is no way to surface a `Result` from inside it without `.unwrap()` / `.expect()`. `OnceLock<Templater>` supports a fallible initialization pattern: `get_or_try_init(|| Templater::new())` returns `Result<&Templater>`. This is the idiomatic Rust solution for fallible statics. The alternative — `LazyLock<Result<Templater>>` — would require unwrapping the inner `Result` at every call site and would leave the error value alive in the static forever; `OnceLock` avoids both problems.

### Propagate with `?`, not `.unwrap()`

All `get_templater()` call sites already operate inside `Result`-returning functions. Adding `?` after `get_templater()?` is sufficient; no intermediate error wrapping is needed beyond the `.context()` already inside `Templater::new()`.

## Risks / Trade-offs

- [Risk] `OnceLock::get_or_try_init` is stabilized in Rust 1.70+ — the project's MSRV must be at or above this. → Mitigation: verify `rust-toolchain.toml` or `Cargo.toml` edition before merging; the project already uses `LazyLock` (stable since 1.80) so MSRV is sufficient.
- [Risk] The comment on line 166 of `deploy.rs` ("Must be called before get_templater()") documents order-of-initialization sensitivity. Changing to `OnceLock` preserves this because `get_or_try_init` only fires on first call, and `set_env_paths` is still called before `get_templater()`. → Mitigation: keep the comment accurate; ensure no new call site calls `get_templater()` before `set_env_paths`.

## Migration Plan

1. Change `Templater::new()` to propagate `load_default_variables()` errors with `?` instead of `.expect()`.
2. Replace `static TEMPLATER: LazyLock<Templater>` with `static TEMPLATER: OnceLock<Templater>`.
3. Rewrite `get_templater()` to call `TEMPLATER.get_or_try_init(Templater::new)` and return `Result<&'static Templater>`.
4. Grep for all `get_templater()` call sites in `src/` and append `?` (or `.context(...)` if additional context improves the error message).
5. Run `mise check` and `mise tests` to confirm no regressions.
6. Add e2e test in `tests/e2e/` asserting exit code 1 and formatted error box (no "panicked at") when deploy runs with no workspace directory.

No rollback strategy needed — this is a pure error-path improvement with no protocol or file-format changes.

## Open Questions

- None. The approach is fully determined by the existing codebase patterns.
