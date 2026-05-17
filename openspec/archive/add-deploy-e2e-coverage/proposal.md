## Why

Manual testing of v0.1.0 (commit `793b8391`) revealed 10 deploy/undeploy behaviors that pass manual verification but have no corresponding e2e tests. These include core safety contracts (user-edit protection, HTTPS-only URLs), flag coverage (`--no-gitignore`), error paths (malformed templates), and TUI prompt paths. One case (old-style gitignore fence cleanup) requires an implementation change alongside the test, as the current behavior does not match the expected contract.

## What Changes

- Add e2e tests to `tests/e2e/deploy.test.ts` covering:
  - **TC-DEPLOY-07**: User-edited deployed file is preserved on redeploy (cache-based edit detection)
  - **TC-DEPLOY-08**: `--force` overrides user-edit protection and overwrites edited files
  - **TC-DEPLOY-10**: `--no-gitignore` flag suppresses gitignore fence creation
  - **TC-DEPLOY-20**: Missing default `.env` file is silently ignored (deploy succeeds)
  - **TC-DEPLOY-ERR-01**: Malformed Handlebars template causes exit 1 with render error
  - **TC-DEPLOY-ERR-02**: Non-HTTPS template URL in provider config causes exit 1 with security error
  - **TC-DEPLOY-16**: TUI offline prompt — navigating to "Yes" selects offline mode
  - **TC-DEPLOY-01**: Full TUI deploy journey (offline prompt → gitignore prompt → summary → done)
- Add e2e tests to `tests/e2e/undeploy.test.ts` covering:
  - **TC-UNDEPLOY-14**: Deployed file manually deleted before undeploy — graceful handling with warning
  - **TC-UNDEPLOY-12**: Old-style `# BEGIN dotagents managed` / `# END dotagents managed` fence markers cleaned up on undeploy (requires implementation change to `src/core/gitignore/` — current code only recognizes `#region`/`#endregion` markers)

## Capabilities

### New Capabilities
- `deploy-user-edit-protection-e2e`: E2e tests for the deploy cache-based user-edit detection and `--force` override
- `deploy-flag-coverage-e2e`: E2e tests for `--no-gitignore`, missing `.env`, and error paths (malformed template, untrusted URL)
- `deploy-tui-prompts-e2e`: E2e tests for full TUI deploy journey and offline prompt Yes path
- `undeploy-edge-cases-e2e`: E2e tests for missing-file handling and old-style fence cleanup

### Modified Capabilities
- `deploy-gitignore-update`: Old-style fence marker (`# BEGIN`/`# END dotagents managed`) cleanup needs to be added to the undeploy gitignore removal logic

## Impact

- `tests/e2e/deploy.test.ts` — new describe blocks and test cases
- `tests/e2e/undeploy.test.ts` — new describe blocks and test cases
- `src/core/gitignore/` — implementation change needed for old-style fence marker recognition in undeploy path
- No API or dependency changes
- Reference: `docs/v0.1.0-testing/results/02-deploy.md`, `docs/v0.1.0-testing/results/03-undeploy.md`
