## Context

Dotagents uses two logging systems side by side: the `log` crate (backed by `simplelog::TermLogger`) for background operations in schema, template, and config code, and `cliclack::log::*` for foreground user-facing output in CLI commands. TTY detection (`is_tty()` → `stdin().is_terminal() && stdout().is_terminal()`) is called independently at each callsite, and manual `if is_tty()` guards branch between the two systems throughout `add.rs`, `rm.rs`, and `ls.rs`. In TTY mode, `log::warn!()` calls still go through simplelog's plain formatter, which visually clashes with cliclack's rendered output.

## Goals / Non-Goals

**Goals:**
- Single call style for all log output — one macro regardless of TTY mode
- TTY mode: all log output goes through cliclack; simplelog is never initialised
- Non-TTY mode: all log output goes through simplelog, identical to current behavior
- `OnceLock<LogConfig>` determines routing at startup; no per-callsite `is_tty()` calls
- `src/prelude.rs` eliminates per-file logging and anyhow import boilerplate
- Fatal errors in TTY mode close with `cliclack::outro_cancel()` for clean terminal state

**Non-Goals:**
- Wrapping `cliclack::intro()`, `outro()`, `confirm()`, `spinner()`, `select()`, `multiselect()` — these are interactive primitives, not logging
- Wrapping `bail!` / `anyhow!` error creation — display is handled at `main.rs`, not at the point of error creation
- Removing the `log` or `simplelog` crates — they remain as the non-TTY backend

## Decisions

### D1 — Macros over wrapper functions

**Decision**: Use `macro_rules!` macros rather than regular functions.

**Rationale**: The `log` crate's macros are lazy — format arguments are only evaluated if the level is active. Functions evaluate arguments eagerly before the call, adding overhead in hot paths (e.g., `template_cache.rs` runs on every cache read). Macros preserve this property.

**Alternative considered**: Closure-based functions (`logs::debug(|| format!(...))`) — lazy but ugly callsites that diverge from the familiar `log::debug!()` style.

### D2 — Shadow `log` crate macro names

**Decision**: Name macros `error!`, `warn!`, `info!`, `debug!`, `trace!` — identical to the `log` crate. Files remove `use log::*` and add `use crate::prelude::*`.

**Rationale**: Zero cognitive overhead for callsite authors; minimal diff when sweeping existing callsites. Since we're doing a full sweep anyway, the namespace conflict is resolved cleanly.

**Alternative considered**: Distinct names (`app_warn!`, `log_step!`) — safe during migration but uglier and unnecessary given the full sweep.

### D3 — `OnceLock<LogConfig>` for mode caching

**Decision**: Store `LogConfig { is_tty: bool, level: LevelFilter }` in a `static OnceLock`, populated once inside `set_log_config()`.

**Rationale**: TTY mode is an immutable property of the process run. `OnceLock` models this intent correctly, avoids repeated syscalls, and makes the mode available to macro bodies via `$crate::utils::logs::log_config()` without passing context through every call.

**Alternative considered**: Call `is_tty()` inside every macro — correct but wasteful and loses the "decided once" semantic.

### D4 — Non-TTY delegates to `::log::` macros internally

**Decision**: In the non-TTY branch of each macro, call `::log::warn!(...)` etc. so simplelog (the registered `log::Log` backend) handles formatting, filtering, and output.

**Rationale**: Preserves all existing simplelog behavior (time/location/level formatting, module filtering) without reimplementing it. simplelog is still initialised in `set_log_config()` for the non-TTY path.

### D5 — Routing table

| Macro | TTY (always) | TTY (gated) | Non-TTY |
|---|---|---|---|
| `error!` | `cliclack::log::error()` | — | `::log::error!` |
| `warn!` | `cliclack::log::warning()` | — | `::log::warn!` |
| `info!` | — | `-v` → `cliclack::log::info()` | `::log::info!` |
| `debug!` | — | `-vv` → `cliclack::log::remark()` | `::log::debug!` |
| `trace!` | — | `-vvv` → `cliclack::log::remark()` | `::log::trace!` |
| `success!` | `cliclack::log::success()` | — | `::log::info!` (-v) |
| `step!` | `cliclack::log::step()` | — | `::log::debug!` (-vv) |

`--quiet` sets `LevelFilter::Error` and affects only the non-TTY simplelog backend. In TTY mode `warn!` and `error!` are always rendered.

### D6 — Module visibility for `$crate::` macro paths

**Decision**: Widen `mod utils` in `main.rs` to `pub(crate)` and `mod logs` in `utils/mod.rs` to `pub(crate)`. Expose `pub(crate) fn log_config()` and `pub(crate) static LOG_CONFIG`.

**Rationale**: `#[macro_export]` macros use `$crate::utils::logs::log_config()` to access the singleton. Both intermediate modules must be crate-visible for this path to resolve. This is the minimum change — no other modules are widened.

### D7 — Broad prelude

**Decision**: `src/prelude.rs` re-exports `anyhow::{Context, Result, anyhow, bail}` in addition to the logging macros.

**Rationale**: Almost every file already imports these individually. A broad prelude reduces per-file boilerplate and is consistent with how major Rust crates (tokio, sqlx, axum) ship preludes.

## Risks / Trade-offs

**Macro name shadowing** → If any file imports both `use log::warn` and `use crate::prelude::*`, the compiler will error on ambiguity. Mitigation: the callsite sweep removes all `use log::*` / `use log::warn` imports before adding the prelude.

**TTY mode drops the `log::Log` backend** → In TTY mode simplelog is not initialised, so `log::warn!()` calls from third-party crates or unforeseen paths silently do nothing. Mitigation: all first-party callsites use the new macros; the existing `add_filter_allow("dotagents")` already suppressed third-party logs anyway.

**`cliclack::log::*` return `Result`** → These calls can fail if the terminal is in an unexpected state. Mitigation: all cliclack log calls use `.ok()` to swallow errors, consistent with existing cliclack usage in the codebase.

## Migration Plan

1. Implement `LogConfig` + `OnceLock` + `log_config()` in `logs.rs`
2. Implement all seven macros
3. Widen module visibility, create `prelude.rs`
4. Update `display_error()` and `main.rs` fatal error path
5. Sweep all callsites — remove `use log::*`, add `use crate::prelude::*`
6. Remove manual `if use_interactive` guards in `add.rs`, `rm.rs`, `ls.rs`
7. Run `mise check` + `mise test-all`; smoke-test TTY and non-TTY paths manually

Rollback: revert `logs.rs` and `prelude.rs`; restore `use log::*` imports. No data migration involved.

## Open Questions

None — all decisions resolved during exploration.
