## Context

`is_tty()` in `src/utils/tty.rs` is the single gate for all interactive prompts. It checks `stdin.is_terminal() && stdout.is_terminal()` using `std::io::IsTerminal`. Every prompt function (`prompt_offline`, `prompt_gitignore_update`, `prompt_confirm_undeploy`, etc.) calls `is_tty()` and returns a silent default when it returns `false`.

Two call sites bypass `is_tty()` and call `is_terminal()` directly:
- `is_tui_mode()` in `src/cli/init.rs:52`
- Several branches in `src/cli/config.rs:43,54,66,78`

These sites would ignore CI mode unless fixed.

The non-TTY fallbacks are already defined and correct. The only missing piece is a way to force `is_tty()` to return `false` even when the process is attached to a PTY.

## Goals / Non-Goals

**Goals:**
- `--ci` global flag and `DOTAGENTS_CI=true` env var both force `is_tty()` to `false` for the entire process lifetime.
- Fix the two direct `is_terminal()` call sites to use `is_tty()` so CI mode covers all interactive paths.
- Zero changes to non-TTY fallback values — this design does not re-litigate defaults.

**Non-Goals:**
- Auto-detecting the platform `CI` env var (intentionally excluded — explicit opt-in only).
- Changing log verbosity or output format — `--ci` is orthogonal to `--quiet`/`--verbose`.
- Adding new non-TTY fallback behavior — existing defaults are sufficient.

## Decisions

### D1: Global `OnceLock<bool>` for CI state

`is_tty()` is a free function called deep in prompt helpers with no shared context object. Threading a `bool` parameter through every call site would require changing every prompt function and every caller.

**Decision:** Store CI mode in a `static OnceLock<bool>` in `src/utils/tty.rs`, initialized once at startup before any prompt runs. `is_tty()` reads it and short-circuits to `false` if set.

*Alternative considered:* Pass `AppConfig` or a context struct to every prompt — rejected as excessive coupling for a single flag.

### D2: Initialization point

**Decision:** Initialize CI mode in `src/cli/runner.rs::run()` immediately after options are parsed, before dispatching to any subcommand. The env var `DOTAGENTS_CI` is checked there too; truthy values are `"true"`, `"1"`, `"yes"` (case-insensitive).

*Alternative considered:* Initialize in `main.rs` — same effect, but `runner.rs` already owns option dispatch and is the natural home for pre-dispatch setup (logging is initialized there too).

### D3: `--ci` flag placement

**Decision:** Add `ci: bool` to the global `Options` struct alongside `quiet` and `verbosity`, with `#[clap(long, global = true)]`. This makes `--ci` valid before any subcommand, consistent with `--quiet`.

### D4: Fix direct `is_terminal()` call sites

**Decision:** Replace all direct `std::io::stdin().is_terminal()` / `std::io::stdout().is_terminal()` calls in `init.rs` and `config.rs` with `is_tty()`. This ensures CI mode affects the init wizard gate and config edit gate without special-casing them.

## Risks / Trade-offs

- **Test isolation:** `OnceLock` is initialized once per process. Unit tests that call `is_tty()` in the same process could be affected if any test sets CI mode. Mitigation: expose a `#[cfg(test)]` reset helper, or ensure no test exercises CI mode via the lock (use direct flag passing in tests instead).
- **`DOTAGENTS_CI` collisions:** Users who have `DOTAGENTS_CI` set for an unrelated reason will get unexpected non-TTY behavior. Mitigation: document the env var clearly; the explicit name (not `CI`) makes accidental collision unlikely.
- **Undeploy auto-proceed:** CI mode auto-proceeds with file deletion (existing non-TTY default). Users must pass `--force` to also delete manually-edited files. This is intentional but worth documenting.

## Migration Plan

No migration needed — `--ci` is additive. Existing behavior is unchanged when the flag/env var is absent.
