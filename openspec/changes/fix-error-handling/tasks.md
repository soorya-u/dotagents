## 1. Discover Call Sites

- [ ] 1.1 Run `grep -rn "get_templater()" src/` to list every call site that will need updating after the signature change

## 2. Make Templater Initialization Fallible

- [ ] 2.1 In `src/templates/templater.rs`, change `Templater::new()` to propagate `load_default_variables()` with `?` instead of `.expect("failed to load global variables")`
- [ ] 2.2 Replace `static TEMPLATER: LazyLock<Templater>` with `static TEMPLATER: OnceLock<Templater>` and update the import (`use std::sync::OnceLock`)
- [ ] 2.3 Rewrite `get_templater()` to use `TEMPLATER.get_or_try_init(Templater::new)` and return `Result<&'static Templater>`

## 3. Update Call Sites

- [ ] 3.1 In `src/cli/deploy.rs`, add `?` after `get_templater()` (or `.context("failed to initialise templater")`) so the error propagates to `main.rs`
- [ ] 3.2 In `src/cli/skills.rs`, add `?` after `get_templater()` so the error propagates to `main.rs`
- [ ] 3.3 Check every other call site found in task 1.1 and apply the same `?` / `.context()` treatment

## 4. Verify and Lint

- [ ] 4.1 Run `mise check` (`cargo fmt` + `cargo clippy`) and fix any warnings or formatting issues
- [ ] 4.2 Run `mise tests` (unit + integration + e2e) and confirm all suites pass

## 5. Add E2E Test

- [ ] 5.1 In `tests/e2e/` (in `errors.test.ts` or `deploy.test.ts`), add a test that runs `deploy` from a temp directory with no `.dotagents` ancestor, asserts exit code 1, asserts stderr contains the formatted error box text (e.g., `Fatal error` or `■`), and asserts stderr does NOT contain `panicked at`
- [ ] 5.2 Run `mise tests:e2e` to confirm the new test passes
