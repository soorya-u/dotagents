## Context

The e2e test suite (`tests/e2e/`) uses `@microsoft/tui-test` to exercise the Dotagents binary. The binary path is hardcoded in `tests/e2e/helpers.ts` as `../../target/debug/dotagents`. Because the debug binary is configured (via `cfg(debug_assertions)`) to use `.dotagents-debug` as its root directory instead of `.dotagents`, every test that checks filesystem paths references `.dotagents-debug`. The `tests:e2e` mise task declares `build` (debug) as its dependency, so the debug binary is always what gets exercised.

This has two concrete consequences:
1. Tests never exercise the release binary, which is what users actually run.
2. The 57 hardcoded `.dotagents-debug` path references would all be wrong if the binary path were switched to release without updating the path strings — and the skip on the overwrite test exists specifically because the debug binary defaults `force = true`, while the release binary requires an explicit `--force` flag.

A separate concern exists in `workflow.test.ts`, which patches out the gemini target after `init`. Other tests that exercise deploy are expected to use `initWithLocalProvider()` (from `helpers.ts`), which sets up only a local provider. If any deploy test bypasses that helper and does not explicitly remove gemini, it will fail in CI environments where the gemini provider requires a local cache file.

## Goals / Non-Goals

**Goals:**
- Switch the e2e suite to run the release binary
- Remove all `.dotagents-debug` references from test files
- Update the `tests:e2e` mise task to build the release binary
- Unskip the overwrite test that was blocked by debug-binary behavior
- Audit all deploy tests for gemini provider safety in CI
- Document `initWithLocalProvider()` as the canonical deploy-test setup

**Non-Goals:**
- Changing the debug/release root directory logic in the Rust source (`src/constants/dir.rs`)
- Adding new test flows beyond what is needed to fix the regression
- Modifying release build flags or LTO settings

## Decisions

**Decision: Switch binary path in `helpers.ts`, not via an environment variable.**
An env var would add indirection and require every CI step to set it. The binary path is a test infrastructure constant; a direct string change in one place is simpler and self-documenting.

*Alternative considered*: Parameterize via `process.env.DOTAGENTS_BIN`. Rejected because it adds configuration surface with no benefit — the release binary is always what the suite should test.

**Decision: Global string replace for `.dotagents-debug` → `.dotagents`.**
All 57 occurrences are mechanical: they appear in `existsSync`, `readFileSync`, and `getByText` assertions. A targeted per-file approach gives no benefit over a global replace and is harder to audit.

**Decision: Change `tests:e2e` dependency from `build` to `build-release` in `mise.toml`.**
The suite must always compile the release binary before running. This makes the dependency explicit and prevents the suite from running a stale debug build.

**Decision: Audit for gemini safety is code-review, not a new abstraction.**
Rather than introducing a new `safeInit()` wrapper, the fix is to verify that all deploy tests already call `initWithLocalProvider()` (or explicitly patch gemini out). Adding a comment to `initWithLocalProvider()` makes the intent clear for future test authors.

## Risks / Trade-offs

- [Longer CI time] Release builds are slower than debug builds (LTO, optimizations). → Acceptable: correctness beats speed; `mise tests:unit` and `mise tests:integration` still use the debug build.
- [Rebase conflict with `fix-init-dir-timing`] Both changes touch `init.test.ts`. → The conflict is mechanical (line-level); whoever merges second rebases and picks both changes.

## Migration Plan

1. Update `mise.toml` `tests:e2e` dependency.
2. Update binary path in `helpers.ts`.
3. Global-replace `.dotagents-debug` → `.dotagents` across all `tests/e2e/*.test.ts` files.
4. Unskip the overwrite test in `init.test.ts`.
5. Audit deploy tests; patch any that lack gemini safety.
6. Add explanatory comment to `initWithLocalProvider()` in `helpers.ts`.
7. Run `mise check && mise tests` — both must exit 0.

Rollback: revert the `helpers.ts` binary path and `mise.toml` dependency; the string replacements are forward-only (no `.dotagents-debug` should remain after the fix).

## Open Questions

None. The change is fully scoped and the affected files are identified.
