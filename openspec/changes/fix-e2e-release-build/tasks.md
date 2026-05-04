## 1. Build Infrastructure

- [ ] 1.1 In `mise.toml`, change the `tests:e2e` task's `depends` entry from `build` to `build-release`
- [ ] 1.2 In `tests/e2e/helpers.ts` line 8, change the binary path from `../../target/debug/dotagents` to `../../target/release/dotagents`

## 2. Root Directory References

- [ ] 2.1 In all files matching `tests/e2e/*.test.ts`, globally replace every occurrence of `.dotagents-debug` with `.dotagents` (57 occurrences total)
- [ ] 2.2 Verify no `.dotagents-debug` string remains anywhere under `tests/e2e/` after the replace

## 3. Overwrite Test

- [ ] 3.1 In `tests/e2e/init.test.ts`, remove the `test.skip` (or equivalent skip annotation) from the overwrite test whose skip reason references the debug binary defaulting `force = true`
- [ ] 3.2 Confirm the unskipped test asserts that `dotagents init` fails without `--force` on an existing root and succeeds with `--force`

## 4. Gemini Safety Audit

- [ ] 4.1 Read every deploy test in `tests/e2e/` that does NOT call `initWithLocalProvider()`; list any that do not also explicitly patch out or disable the gemini provider
- [ ] 4.2 For each unsafe deploy test identified in 4.1, add the gemini-removal step (mirror the pattern in `workflow.test.ts` line 178) before the deploy invocation

## 5. Documentation

- [ ] 5.1 In `tests/e2e/helpers.ts`, add a comment directly above `initWithLocalProvider()` stating it is the canonical setup for deploy tests because it activates only a local provider and avoids CI-unsafe providers such as gemini

## 6. Verification

- [ ] 6.1 Run `mise check` and confirm exit code 0 (cargo fmt + clippy)
- [ ] 6.2 Run `mise tests` and confirm exit code 0 (unit + integration + e2e)
