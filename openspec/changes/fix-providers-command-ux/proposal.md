## Why

The `providers` command has four small but visible UX issues found during v0.1.0 manual testing: (1) an error message says `"dotagents providers ls"` but the actual command is `"dotagents providers"`; (2) pressing Escape in the TUI select propagates an error instead of exiting cleanly; (3) `--verbose` adds no debug output, making it useless for diagnosing registry fetch problems; (4) the `--quiet` flag (from global options) has no documented effect on this command.

## What Changes

- Fix error string on line 63 of `src/cli/providers.rs`: `"run 'dotagents providers ls'"` → `"run 'dotagents providers'"`
- Handle TUI Escape gracefully: when `select.interact()` returns `Err`, return `Ok(true)` (clean exit) instead of propagating the error
- Add `debug!()` log calls for registry fetch: URL being fetched, HTTP response size, cache path used — visible with `--verbose`/`-v`
- Document (in `--help` long description) that `--quiet` suppresses spinner and progress output; enforce this in `run_providers` by checking `is_tui_enabled()` before starting the spinner
- Update unit tests and e2e tests for the corrected error string and graceful Escape behavior

## Capabilities

### New Capabilities
- `providers-escape-graceful-exit`: Pressing Escape or Ctrl-C in the providers TUI exits cleanly (exit 0) rather than printing a fatal error

### Modified Capabilities
- `providers-list`: Error message references correct command name; debug logs added; quiet-mode spinner suppressed

## Impact

- `src/cli/providers.rs` — fix error string; handle Escape in `run_tui`; add `debug!()` calls; gate spinner on `is_tui_enabled()`
- Unit tests in `src/cli/providers.rs` — assert corrected error string
- `tests/e2e/providers.test.ts` (if exists) or new file — e2e test for graceful Escape exit and verbose output
