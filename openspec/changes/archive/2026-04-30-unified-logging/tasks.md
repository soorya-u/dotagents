## 1. LogConfig Infrastructure (`src/utils/logs.rs`)

- [x] 1.1 Add `pub(crate) struct LogConfig { pub(crate) is_tty: bool, pub(crate) level: log::LevelFilter }` 
- [x] 1.2 Add `pub(crate) static LOG_CONFIG: OnceLock<LogConfig> = OnceLock::new()` (import `std::sync::OnceLock`)
- [x] 1.3 Add `pub(crate) fn log_config() -> Option<&'static LogConfig>` returning `LOG_CONFIG.get()`
- [x] 1.4 Update `set_log_config(quiet, verbosity)`: call `is_tty()` once, compute `level`, populate `LOG_CONFIG` via `get_or_init`, then branch — non-TTY: init simplelog as today; TTY: skip simplelog init entirely
- [x] 1.5 Add unit tests for `LogConfig` construction and `log_config()` accessor in `mod tests`

## 2. Logging Macros (`src/utils/logs.rs`)

- [x] 2.1 Implement `error!` — TTY always: `cliclack::log::error(format!(...)).ok()`; non-TTY: `::log::error!(...)`
- [x] 2.2 Implement `warn!` — TTY always: `cliclack::log::warning(format!(...)).ok()`; non-TTY: `::log::warn!(...)`
- [x] 2.3 Implement `info!` — TTY if `level >= LevelFilter::Info`: `cliclack::log::info(format!(...)).ok()`; non-TTY: `::log::info!(...)`
- [x] 2.4 Implement `debug!` — TTY if `level >= LevelFilter::Debug`: `cliclack::log::remark(format!(...)).ok()`; non-TTY: `::log::debug!(...)`
- [x] 2.5 Implement `trace!` — TTY if `level >= LevelFilter::Trace`: `cliclack::log::remark(format!(...)).ok()`; non-TTY: `::log::trace!(...)`
- [x] 2.6 Implement `success!` — TTY always: `cliclack::log::success(format!(...)).ok()`; non-TTY: `::log::info!(...)`
- [x] 2.7 Implement `step!` — TTY always: `cliclack::log::step(format!(...)).ok()`; non-TTY: `::log::debug!(...)`
- [x] 2.8 Annotate all seven macros with `#[macro_export]`

## 3. Module Visibility for Macro Path Resolution

- [x] 3.1 Change `mod utils;` to `pub(crate) mod utils;` in `src/main.rs`
- [x] 3.2 Change `mod logs;` to `pub(crate) mod logs;` in `src/utils/mod.rs`

## 4. Prelude (`src/prelude.rs`)

- [x] 4.1 Create `src/prelude.rs` with `pub use anyhow::{Context, Result, anyhow, bail};`
- [x] 4.2 Add `pub use crate::{error, warn, info, debug, trace, success, step};` to re-export all seven macros
- [x] 4.3 Declare `mod prelude;` in `src/main.rs`

## 5. Fatal Error Display

- [x] 5.1 Add `use crate::prelude::*` to `src/utils/error.rs`; replace `log::error!("{}", error_message)` with `error!("{}", error_message)`
- [x] 5.2 In `src/main.rs` `Err(e)` branch: after `utils::display_error(e)`, call `cliclack::outro_cancel("Fatal error").ok()` when `utils::logs::log_config().is_some_and(|c| c.is_tty)` before `process::exit(1)`

## 6. Callsite Sweep — Background Log Macros

- [x] 6.1 `src/schema/config/cache.rs` — remove `use log::debug`, add `use crate::prelude::*`
- [x] 6.2 `src/schema/features/skill.rs` — remove `use log::warn`, add `use crate::prelude::*`
- [x] 6.3 `src/templates/renderer.rs` — remove `use log::warn`, add `use crate::prelude::*`
- [x] 6.4 `src/templates/template_cache.rs` — remove `use log::debug`, add `use crate::prelude::*`
- [x] 6.5 `src/templates/registry_resolver.rs` — remove `use log::warn`, add `use crate::prelude::*`
- [x] 6.6 `src/cli/deploy.rs` — remove `use log::warn`, add `use crate::prelude::*`
- [x] 6.7 `src/cli/init.rs` — remove `use log::*` / individual log imports, add `use crate::prelude::*`; replace `log::warn!(...)` / `log::info!(...)` with bare macro calls

## 7. Callsite Sweep — Foreground Interactive Callsites

- [x] 7.1 `src/cli/add.rs` — replace `if use_interactive { cliclack::log::success(...) } else { println!(...) }` blocks with `success!(...)` calls; remove `use_interactive` variable if unused; add `use crate::prelude::*`
- [x] 7.2 `src/cli/rm.rs` — replace `cliclack::log::success(...)` with `success!(...)`; add `use crate::prelude::*`
- [x] 7.3 `src/cli/ui/ls.rs` — replace `cliclack::log::step(...)`, `cliclack::log::info(...)`, `cliclack::log::success(...)` with `step!(...)`, `info!(...)`, `success!(...)`; add `use crate::prelude::*`

## 8. Verification

- [x] 8.1 Run `mise check` (cargo fmt + clippy) and fix any warnings
- [x] 8.2 Run `mise test-all` (unit + integration + e2e) — all must pass
- [ ] 8.3 Smoke-test TTY path: `cargo run -- add command test-cmd` in a terminal — confirm cliclack styled output
- [ ] 8.4 Smoke-test non-TTY path: `cargo run -- add command test-cmd 2>&1 | cat` — confirm plain simplelog output
