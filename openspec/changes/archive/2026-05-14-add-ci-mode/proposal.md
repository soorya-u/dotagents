## Why

Some CI environments (Docker with `-t`, certain GitHub Actions runners) allocate a PTY, causing `std::io::IsTerminal` to return `true` even when no human is present. This causes dotagents to launch interactive prompts that hang indefinitely, blocking pipelines. Users need an explicit, reliable way to force non-TTY behavior regardless of PTY state.

## What Changes

- Add a global `--ci` flag to the `Options` struct (alongside `--quiet` and `--verbose`).
- Respect a `DOTAGENTS_CI=true` environment variable as an equivalent override.
- When either is set, `is_tty()` returns `false` for the entire process, suppressing all interactive prompts.
- `is_tui_mode()` in `init.rs` and the direct `is_terminal()` call in `config.rs` are updated to route through `is_tty()` so CI mode affects them too.
- No change to logging — `--ci` is orthogonal to `--quiet`/`--verbose`.
- No auto-detection of the platform `CI` environment variable; only the explicit `DOTAGENTS_CI=true` is respected.

## Capabilities

### New Capabilities

- `ci-mode`: Global `--ci` flag and `DOTAGENTS_CI=true` env var that forces `is_tty()` to return `false` for the entire process, ensuring all interactive prompts use their non-TTY defaults without hanging.

### Modified Capabilities

<!-- No existing spec-level requirements are changing — the non-TTY fallback behavior
     for each prompt is already specified. This change only adds a mechanism to activate
     those existing fallbacks explicitly. -->

## Impact

- `src/cli/options.rs` — new `--ci` global flag on `Options`
- `src/utils/tty.rs` — `is_tty()` checks CI mode before the terminal test
- `src/cli/init.rs` — `is_tui_mode()` calls `std::io::stdin().is_terminal()` directly; must use `is_tty()` instead
- `src/cli/config.rs` — direct `std::io::stdin().is_terminal()` calls must use `is_tty()` instead
- `src/main.rs` / `src/cli/runner.rs` — CI mode initialised from flag + env var at startup, stored in a `OnceLock<bool>`
- No new dependencies required
