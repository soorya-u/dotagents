## 1. Mise Toolchain Setup

- [x] 1.1 Add `bun = "latest"` and `"npm:tui-devtools" = "latest"` to `[tools]` in `mise.toml`
- [x] 1.2 Add `e2e-install` task (`bun install --cwd tests/e2e`) to `mise.toml`
- [x] 1.3 Update `test-e2e` task to depend on `build` and `e2e-install` and run `bunx --cwd tests/e2e tui-test`
- [x] 1.4 Update `test-all` task to depend on `test`, `test-integration`, and `test-e2e`

## 2. E2E Project Setup

- [x] 2.1 Create `tests/e2e/package.json` declaring `@microsoft/tui-test` as a dev dependency
- [x] 2.2 Run `bun install --cwd tests/e2e` to generate `tests/e2e/bun.lockb`
- [x] 2.3 Add `tests/e2e/node_modules/` to `.gitignore`
- [x] 2.4 Remove the `[[test]]` Cargo target entry for `e2e` from `Cargo.toml` (so `cargo test` no longer looks for a Rust e2e crate)

## 3. Discovery Phase — CLI Flows

- [x] 3.1 Run and record CLI flow C01: `init --template starter` (terminal output + filesystem state)
- [x] 3.2 Run and record CLI flow C02: `init --template with-custom-provider`
- [x] 3.3 Run and record CLI flow C03: `init --force --template starter` (overwrite)
- [x] 3.4 Run and record CLI flows C04–C07: `--no-command`, `--no-skill`, `--no-mcp`, `--no-instruction`
- [x] 3.5 Run and record CLI flows C08–C09: `add command` and `add skill` with all flags
- [x] 3.6 Run and record CLI flows C10–C13: force overwrite and `--deploy` flag variants
- [x] 3.7 Run and record CLI flows C14–C19: `deploy` variants (force, offline, no-cache, gitignore flags)
- [x] 3.8 Run and record CLI flows C20–C25: `ls` variants
- [x] 3.9 Run and record CLI flows C26–C27: `rm --force` for command and skill
- [x] 3.10 Run and record CLI flows C28–C30: `gen-completions` for bash, zsh, fish
- [x] 3.11 Run and record CLI flows C31–C34: error paths (no workspace, missing target)

## 4. Discovery Phase — TUI Flows (tui-devtools)

- [x] 4.1 Start tui-devtools daemon
- [x] 4.2 Run and record TUI flow T01: `init` full wizard (accept all defaults, Starter template)
- [x] 4.3 Run and record TUI flow T02: `init` wizard deselecting mcp + skills
- [x] 4.4 Run and record TUI flow T03: `init` wizard selecting WithCustomProvider template
- [x] 4.5 Run and record TUI flow T04: `init` with existing dir → overwrite Yes
- [x] 4.6 Run and record TUI flow T05: `init` with existing dir → overwrite No (cancel) — skipped: debug binary force=true suppresses prompt
- [x] 4.7 Run and record TUI flows T06–T07: `add command` interactive (No deploy / Yes deploy)
- [x] 4.8 Run and record TUI flows T08–T09: `add skill` interactive (No deploy / Yes deploy)
- [x] 4.9 Run and record TUI flows T10–T11: `rm command` confirm Yes / No
- [x] 4.10 Run and record TUI flows T12–T13: `rm skill` confirm Yes / No
- [x] 4.11 Run and record TUI flows T14–T15: `deploy` in TTY (online + gitignore / offline)
- [x] 4.12 Stop tui-devtools daemon

## 5. Write E2E Tests — Init

- [x] 5.1 Write `tests/e2e/init.test.ts`: CLI flows C01–C07 (init flag variants, file-tree assertions)
- [x] 5.2 Write TUI init tests in `tests/e2e/init.test.ts`: flows T01–T05 (wizard navigation, overwrite confirm/cancel)

## 6. Write E2E Tests — Add

- [x] 6.1 Write `tests/e2e/add.test.ts`: CLI flows C08–C13 (flag-driven add command + skill)
- [x] 6.2 Write TUI add tests in `tests/e2e/add.test.ts`: flows T06–T09 (interactive prompts, deploy branch)

## 7. Write E2E Tests — Deploy

- [x] 7.1 Write `tests/e2e/deploy.test.ts`: CLI flows C14–C19 (deploy variants, output file content assertions)
- [x] 7.2 Write TUI deploy tests in `tests/e2e/deploy.test.ts`: flows T14–T15 (offline select, gitignore prompt)

## 8. Write E2E Tests — Ls, Rm, Completions, Errors

- [x] 8.1 Write `tests/e2e/ls.test.ts`: CLI flows C20–C25 (output sections, filter flags, verbose)
- [x] 8.2 Write `tests/e2e/rm.test.ts`: CLI flows C26–C27 and TUI flows T10–T13 (force + confirm prompts)
- [x] 8.3 Write `tests/e2e/completions.test.ts`: CLI flows C28–C30 (shell completion file assertions)
- [x] 8.4 Write `tests/e2e/errors.test.ts`: CLI flows C31–C34 (no-workspace and missing-target error paths)

## 9. Write E2E Tests — Journeys

- [x] 9.1 Write `tests/e2e/workflow.test.ts`: journey flows J01–J02 (init → add → deploy → verify output)
- [x] 9.2 Write journey flow J03–J04 in `tests/e2e/workflow.test.ts`: full CRUD for commands and skills
- [x] 9.3 Write journey flows J05–J06 in `tests/e2e/workflow.test.ts`: redeploy picks up changes + idempotency
- [x] 9.4 Write journey flow J07 in `tests/e2e/workflow.test.ts`: full interactive path (TUI init → TUI add → deploy)
- [x] 9.5 Write journey flow J08 in `tests/e2e/workflow.test.ts`: full CRUD both types

## 10. Rewrite Integration Tests

- [x] 10.1 Delete existing binary-spawning tests from `tests/integration/` (or clear the files)
- [x] 10.2 Write `tests/integration/config.rs`: AppConfig merge scenarios (feature override, provider disabled, variable deep-merge)
- [x] 10.3 Write `tests/integration/render.rs`: render pipeline scenarios (var/env interpolation, disabled provider, frontmatter stripping)
- [x] 10.4 Write `tests/integration/features.rs`: feature parsing roundtrips (CommandFeature, McpFeature, SkillFeature, error cases)
- [x] 10.5 Write `tests/integration/cache.rs`: CacheConfig get/set/load/save roundtrip and cache-miss
- [x] 10.6 Write `tests/integration/gitignore.rs`: update_gitignore idempotency and user-content preservation
- [x] 10.7 Update `tests/integration/main.rs` to declare the new test modules

## 11. Verify

- [x] 11.1 Run `mise check` (cargo fmt + clippy) — fix any issues
- [x] 11.2 Run `mise test` (unit tests) — fix any failures
- [x] 11.3 Run `mise test-integration` (new Rust integration tests) — fix any failures
- [x] 11.4 Run `mise test-e2e` (tui-test suite) — fix any failures
- [x] 11.5 Run `mise test-all` — confirm everything exits 0
