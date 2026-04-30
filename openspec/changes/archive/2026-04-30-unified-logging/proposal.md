## Why

Dotagents runs in two distinct modes — interactive TUI (human in a terminal) and non-interactive CLI (agent or CI pipeline) — but both currently share the same simplelog backend, causing raw plain-text log lines to clash with cliclack's styled output in TTY mode, and leaving non-TTY callers with no output at all from cliclack-only callsites. Additionally, `log::` macros and `cliclack::log::*` functions are used inconsistently across the codebase with manual `if is_tty()` guards scattered at every callsite.

## What Changes

- A `LogConfig` struct stored in `OnceLock` captures TTY mode and log level once at startup, replacing per-callsite `is_tty()` calls
- Seven new macros (`error!`, `warn!`, `info!`, `debug!`, `trace!`, `success!`, `step!`) defined in `src/utils/logs.rs` route to cliclack in TTY mode and simplelog in non-TTY mode with lazy evaluation
- `success!` and `step!` are new semantic log levels with no `log` crate equivalent — in TTY they map to `cliclack::log::success` / `cliclack::log::step`; in non-TTY they degrade to `info` / `debug` level
- A new `src/prelude.rs` re-exports all logging macros plus `anyhow::{Context, Result, anyhow, bail}` — files add one `use crate::prelude::*` and drop all `use log::*` imports
- All manual `if use_interactive` log guards in `add.rs`, `rm.rs`, and `ls.rs` are removed
- `--quiet` is a non-TTY-only concept; TTY mode always renders `error!` and `warn!`
- `display_error()` uses the new `error!` macro; `main.rs` calls `cliclack::outro_cancel()` on fatal errors in TTY mode

## Capabilities

### New Capabilities

- `unified-logging`: A single set of macros that automatically route log output to cliclack (TTY) or simplelog (non-TTY) based on a startup-cached `LogConfig`. Introduces `success!` and `step!` as first-class semantic log levels. Provides a broad `prelude` module that eliminates per-file logging and anyhow import boilerplate.

### Modified Capabilities

## Impact

- New file: `src/prelude.rs`
- Updated: `src/utils/logs.rs` — `LogConfig`, `OnceLock`, seven macros, `log_config()` accessor
- Updated: `src/utils/mod.rs` — `pub(crate) mod logs` for macro path resolution
- Updated: `src/main.rs` — `pub(crate) mod utils`, `mod prelude`, `outro_cancel` on fatal error
- Updated: `src/utils/error.rs` — use `error!` macro
- Updated: `src/schema/config/cache.rs`, `src/schema/features/skill.rs`, `src/templates/renderer.rs`, `src/templates/template_cache.rs`, `src/templates/registry_resolver.rs`, `src/cli/deploy.rs`, `src/cli/init.rs` — replace `use log::*` with `use crate::prelude::*`
- Updated: `src/cli/add.rs`, `src/cli/rm.rs`, `src/cli/ui/ls.rs` — remove manual TTY guards, use `success!` / `step!`
