## Context

The commands and skills e2e suites (`tests/e2e/commands.test.ts`, `tests/e2e/skills.test.ts`) cover the main CLI flows for new, rm, and ls subcommands, including TUI prompts, `--force`, `--cwd`, and `--json` flags. Manual testing of v0.1.0 identified 11 behaviors that pass manually but have no automated test. These are all pure test additions — no implementation changes needed. All behaviors already work correctly.

Existing patterns: `initWithLocalProvider(d)` or `run(["init", "--ci"], d)` for setup, `run(args, d)` for assertions, `readFileSync` for content checks, `shellProgram` for TUI tests.

## Goals / Non-Goals

**Goals:**
- Add e2e tests for `--deploy` flag on `commands new`, `commands rm`, `skills new`, `skills rm`
- Add e2e tests for CI-mode empty-defaults behavior (no metadata flags)
- Add e2e tests for skills duplicate detection without `--force`
- Add e2e tests for `--runner` validation (invalid value, not on PATH)
- Add e2e test for `--json --skill` combined filter
- Add e2e test for `commands new --cwd` pointing to non-workspace

**Non-Goals:**
- Testing `commands new` deploy prompt Yes in TUI (already `.skip`'d due to nested offline prompt complexity)
- Testing `skills add` with actual network calls to skills.sh
- Testing missing-NAME Clap errors (framework-handled, low value)

## Decisions

1. **`--deploy` flag tests**: These tests use `initWithLocalProvider(d)` to ensure providers are configured. After `commands new --deploy` or `skills new --deploy`, assert that deployed output files exist. For `rm --deploy`, assert the deployed file is cleaned up and remaining commands/skills are still deployed.

2. **CI empty defaults**: Run `commands new NAME --ci` / `skills new NAME --ci` with no `--description`, `--category`, etc. Read the created file, parse frontmatter, assert `description: ''` and absence of optional keys (`category`, `tags` for commands; `license`, `compatibility` for skills).

3. **Skills duplicate detection**: Create a skill, then try to create again without `--force`. Assert exit 1, stderr contains "already exists" and "Use --force to overwrite".

4. **`--runner` validation**: For invalid value, run `skills add NAME --runner maven` and assert exit 2 (Clap error) with "invalid value" in stderr. For not-on-PATH, run `skills add NAME --runner yarn` (assuming yarn is not installed in the test environment) and assert exit 1 with "not found on PATH" in stderr.

5. **Combined filter**: Run `skills ls --json --skill NAME` and assert valid JSON array with exactly one element matching the filter.

## Risks / Trade-offs

- **`--deploy` tests depend on provider setup**: These tests must use `initWithLocalProvider` (not bare `init --ci`) to have a provider that actually deploys files. The `--offline` flag should be passed to avoid network calls during deploy.
- **`--runner yarn` test assumes yarn is not installed**: If the CI environment has yarn, the test would need adjustment. Mitigation: check for yarn availability and skip if present, or use a deliberately invalid runner binary name.
- **No TUI tests in this proposal**: All missing cases are CLI-only. The `.skip`'d T07 deploy-prompt-Yes test is a known limitation, not addressed here.
