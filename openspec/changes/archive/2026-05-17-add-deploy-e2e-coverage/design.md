## Context

The deploy/undeploy e2e suite (`tests/e2e/deploy.test.ts`, `tests/e2e/undeploy.test.ts`) covers basic CLI flows, rendered content, dry-run, gitignore, env flags, and PATH arguments. Manual testing of v0.1.0 identified 10 behaviors that pass manually but lack e2e regression coverage. These include user-edit protection (a core safety contract), error paths, the `--no-gitignore` flag, and TUI prompt interactions. One case (old-style gitignore fence cleanup) requires a small implementation change because the current code only recognizes `#region`/`#endregion` markers, not the legacy `# BEGIN`/`# END dotagents managed` format.

All new tests follow the existing patterns: `initWithLocalProvider(d)` for setup, `run(args, d)` for CLI invocations, `shellProgram(BINARY, args)` for TUI tests, filesystem assertions for state verification.

## Goals / Non-Goals

**Goals:**
- Add e2e tests for user-edit protection on deploy (preserve + force-override)
- Add e2e tests for `--no-gitignore`, missing `.env`, malformed template, untrusted URL
- Add TUI tests for offline prompt Yes path and full deploy journey
- Add e2e test for undeploy when a deployed file was manually deleted
- Add e2e test for old-style gitignore fence cleanup on undeploy, with the implementation change to support it

**Non-Goals:**
- Testing registry-dependent code paths (requires network mocking infrastructure)
- Testing `--quiet`/`--verbose` flags (global cross-cutting concern, not deploy-specific)
- Adding CI-output to deploy/undeploy (documented as TTY-only; separate change)

## Decisions

1. **User-edit detection via file modification**: Tests will deploy, modify a deployed file (append text), then redeploy. The cache-based hash comparison will detect the edit. This mirrors the manual test approach.

2. **Untrusted URL test**: Write a `config.toml` with `template = "http://example.com/template.hbs"` (non-HTTPS). Assert exit 1 + error message containing "non-HTTPS". No network call is made — validation happens before fetch.

3. **Malformed template test**: Write a `.hbs` file with `{{ unclosed` syntax. Assert exit 1 + error containing "invalid handlebars syntax" or similar. This tests the render error path, not the missing-file path (which is already covered by dry-run error handling tests).

4. **Old-style fence cleanup**: Extend the gitignore removal logic in `src/core/gitignore/` to also recognize and remove `# BEGIN dotagents managed` / `# END dotagents managed` markers. The e2e test will write a `.gitignore` with old-style markers, run undeploy, and assert the markers are removed.

5. **TUI tests follow existing pattern**: Use `shellProgram(BINARY, args)`, `getByText` for prompt detection, key presses for navigation, text assertions for completion.

## Risks / Trade-offs

- **Old-style fence implementation change**: Small scope (regex match in gitignore removal), but changes production code. Must verify no regression on current `#region`/`#endregion` handling.
- **TUI test fragility**: Full-journey TUI tests (TC-DEPLOY-01) are more brittle than CLI tests because they depend on prompt ordering and cliclack rendering. Mitigation: use `getByText` for semantic matching, not snapshot-based assertions.
- **User-edit test timing**: Cache hash comparison depends on file content, not timestamps. No timing issues expected.
