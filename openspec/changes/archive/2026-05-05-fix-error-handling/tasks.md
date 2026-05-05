## 1. Discover Call Sites

- [x] 1.1 Run `grep -rn "get_templater()" src/` to list every call site that will need updating after the signature change

## 2. Make Templater Initialization Fallible

- [x] 2.1 In `src/templates/templater.rs`, change `Templater::new()` to propagate `load_default_variables()` with `?` instead of `.expect("failed to load global variables")`
- [x] 2.2 Replace `static TEMPLATER: LazyLock<Templater>` with `static TEMPLATER: OnceLock<Templater>` and update the import (`use std::sync::OnceLock`)
- [x] 2.3 Rewrite `get_templater()` to use a manual init pattern with `OnceLock` (note: `get_or_try_init` is unstable) and return `Result<&'static Templater>`

## 3. Update Call Sites

- [x] 3.1 In `src/cli/deploy.rs`, add `.context("failed to initialise templater")?` after `get_templater()` so the error propagates to `main.rs`
- [x] 3.2 In `src/cli/skills.rs`, add `?` after `get_templater()` so the error propagates to `main.rs`
- [x] 3.3 Check every other call site found in task 1.1 — no other call sites existed (app.rs imports but doesn't call `get_templater()`)

## 4. Verify and Lint

- [x] 4.1 Run `mise check` (`cargo fmt` + `cargo clippy`) — passes clean
- [x] 4.2 Run `mise tests` (unit + integration + e2e) — all pass

## 5. Add E2E Test

- [x] 5.1 Fix existing e2e test assertion in `tests/e2e/errors.test.ts` — was checking for `■`/`Fatal error` which only appear in TTY mode; changed to `[ERROR]` for non-TTY. Test already existed.
- [x] 5.2 Run `mise tests:e2e` — error-specific tests all pass
