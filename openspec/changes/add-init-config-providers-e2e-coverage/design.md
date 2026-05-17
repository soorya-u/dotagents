## Context

The init, config, and providers e2e suites cover the main flows but manual testing of v0.1.0 identified 8 gaps. The init gaps are Clap validation errors (exclusive features, invalid values). The config gaps are missing-file graceful handling. The providers gaps include TUI coverage and two flags (`--quiet`, `--verbose`) that are currently non-functional for the `providers` command — these require implementation changes alongside tests.

The `providers` command uses `println!()` for output, which bypasses the log framework that `--quiet` and `--verbose` control. This needs to be changed so the flags have effect.

## Goals / Non-Goals

**Goals:**
- Add e2e tests for init flag validation errors (`--features none,commands`, invalid `--features`, invalid `--template`)
- Add e2e tests for config missing-file handling (`local.config.toml` absent, `config.toml` absent)
- Add TUI e2e test for providers interactive select widget
- Fix `providers` command `--quiet`/`--verbose` flag handling, then add e2e tests

**Non-Goals:**
- Testing `--quiet`/`--verbose` across all commands (global concern, out of scope)
- Testing providers with actual network calls or mock servers
- Testing `DOTAGENTS_CI` env var variants for init (already covered by ci-mode.test.ts mechanism)

## Decisions

1. **Init validation tests**: Pure CLI tests. Run with invalid values, assert exit code 2 (Clap error), assert stderr contains "invalid value" and lists valid options. For `--features none,commands`, assert the exclusive-combination error message.

2. **Config missing-file tests**: Use `initWithLocalProvider(d)`, then `unlinkSync` the target config file before running the config command. For missing `local.config.toml`: assert exit 0, stdout contains "No local config" (text mode) or `{}` (JSON mode). For missing `config.toml`: assert exit 1, stderr contains "not found".

3. **Providers TUI test**: Use `shellProgram(BINARY, ["providers", "--offline"])` with a seeded cache (from existing test helpers). Assert the select widget renders with provider names, arrow-down navigates, Enter selects (shows outro), Escape cancels. This requires tui-devtools observation first per CLAUDE.md testing workflow.

4. **Providers `--quiet` fix**: Change the text output path in `src/cli/providers.rs` from `println!()` to conditionally gated output that respects the quiet flag. Options: (a) check `opts.quiet` before printing, or (b) use the log framework (`info!()`) so the global log level applies. Option (a) is simpler and more explicit.

5. **Providers `--verbose` fix**: Add `debug!()` calls in the fetch/cache pipeline for providers that emit cache status, fetch URL, and timing. These become visible at `-v` level.

## Risks / Trade-offs

- **Providers TUI test requires tui-devtools discovery**: Per CLAUDE.md, interactive TUI tests must be observed with tui-devtools before writing assertions. The test assertions should be written from actual terminal output, not from source reading.
- **Providers flag fix scope**: The `--quiet`/`--verbose` implementation change is small (providers.rs only) but could set a precedent for how other commands handle these flags. Keep it local to providers for now.
- **Config missing-file tests modify init scaffold**: These tests delete files from the init scaffold mid-test. The `try/finally` + `cleanup(d)` pattern handles teardown.
