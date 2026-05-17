## Why

Manual testing of v0.1.0 (commit `793b8391`) revealed 11 gaps in the commands and skills e2e suites. These cover untested flag behaviors (`--deploy`, `--cwd` error paths, `--runner` validation), CI-mode defaults (empty metadata), duplicate detection for skills, and combined filter flags. All cases pass manual verification but have no automated regression coverage.

## What Changes

- Add e2e tests to `tests/e2e/commands.test.ts` covering:
  - **TC-CMD-NEW-03**: CI mode with no metadata flags produces `description: ''`, no `category`/`tags` keys
  - **TC-CMD-NEW-06**: `--deploy` flag on `commands new` triggers a deploy pass after creation
  - **TC-CMD-RM-06**: `--deploy` flag on `commands rm` re-runs deploy after removal
  - **TC-CMD-NEW-10**: `--cwd` pointing to a directory without `.dotagents/` exits non-zero with error
- Add e2e tests to `tests/e2e/skills.test.ts` covering:
  - **TC-SKILL-NEW-03**: CI mode with no metadata flags produces `description: ''`, no `license`/`compatibility` keys
  - **TC-SKILL-NEW-04**: Creating a duplicate skill without `--force` exits non-zero with "already exists" error
  - **TC-SKILL-NEW-06**: `--deploy` flag on `skills new` triggers a deploy pass after creation
  - **TC-SKILL-RM-06**: `--deploy` flag on `skills rm` re-runs deploy after removal
  - **TC-SKILL-LS-06**: `--json --skill <name>` combined filter returns filtered JSON array
  - **TC-SKILL-ADD-04**: `--runner yarn` when yarn is not on PATH exits non-zero with helpful error
  - **TC-SKILL-ADD-05**: `--runner maven` (invalid value) exits 2 with Clap error listing valid values

## Capabilities

### New Capabilities
- `commands-deploy-flag-e2e`: E2e tests for the `--deploy` flag on `commands new` and `commands rm`
- `commands-error-paths-e2e`: E2e tests for `--cwd` non-workspace and CI empty defaults
- `skills-deploy-flag-e2e`: E2e tests for the `--deploy` flag on `skills new` and `skills rm`
- `skills-validation-e2e`: E2e tests for duplicate detection, `--runner` validation, combined filter flags, and CI empty defaults

### Modified Capabilities

## Impact

- `tests/e2e/commands.test.ts` — new test cases in existing describe blocks and new describe blocks
- `tests/e2e/skills.test.ts` — new test cases in existing describe blocks and new describe blocks
- No implementation changes needed — all behaviors already work correctly per manual testing
- Reference: `docs/v0.1.0-testing/results/04-commands.md`, `docs/v0.1.0-testing/results/05-skills.md`
