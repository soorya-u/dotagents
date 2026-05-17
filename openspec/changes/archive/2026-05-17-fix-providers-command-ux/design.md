## Context

`run_providers()` in `src/cli/providers.rs` handles the single `providers` subcommand (no `ls` sub-subcommand). Four issues exist: (1) an error message references a non-existent `providers ls` command; (2) `run_tui()` calls `select.interact().map_err(|e| anyhow!(...))` — when the user presses Escape, `cliclack` returns `Err`, which propagates all the way to `main.rs` and prints a fatal error box; (3) there are no `debug!()` calls, so `-v` is invisible; (4) the spinner starts unconditionally in TTY mode, even when `--quiet` should suppress it.

## Goals / Non-Goals

**Goals:**
- Escape in the TUI provider select returns exit 0 (not an error)
- `-v` / `--verbose` shows at least the registry URL being fetched and the cache path
- Spinner is suppressed when `is_tui_enabled()` returns false (CI/quiet mode)
- Error message references correct command name

**Non-Goals:**
- Redesigning the TUI provider browser — only the Escape path changes
- Adding full `--quiet` implementation across all commands (this proposal scopes to providers only)

## Decisions

1. **Escape exits cleanly via `Ok(true)`**: The `map_err` in `run_tui` is replaced with an `unwrap_or` that returns a sentinel (empty string or the first provider slug). Alternatively, check `e.kind()` — but `cliclack` error kinds are not stable. Simplest: catch the error from `interact()` and return `Ok(true)` directly.

2. **`debug!()` for fetch diagnostics**: Add `debug!("Fetching provider registry from {}", url)` before the HTTP call, and `debug!("Registry cached at {}", path.display())` after caching. These appear with `-v` / `-vv`.

3. **Spinner gated on `is_tui_enabled()`**: Move the spinner start into the `is_tty() && is_tui_enabled()` branch. This is already the TTY branch, so the change is: replace `is_tty()` with `is_tty() && !opts.quiet` (or check global quiet flag if accessible). Given `--quiet` is a global option, the simplest approach is to check `is_tui_enabled()` which already incorporates the CI check; TTY-with-quiet can be addressed in the unified-logging overhaul.

## Risks / Trade-offs

- **Swallowing TUI errors**: Returning `Ok(true)` on any `interact()` error masks genuine TUI failures (not just Escape). Mitigation: the providers select is read-only; no state is mutated, so a failed interaction is always safe to treat as a clean exit.
