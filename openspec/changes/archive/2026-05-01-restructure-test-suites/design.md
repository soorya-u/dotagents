## Context

The project currently has two binary-spawning test suites: `tests/integration/` (smoke tests — exit codes, file existence) and `tests/e2e/` (detailed behavioural tests — file contents, variable interpolation). Both compile and spawn the `dotagents` binary for every assertion. Neither tests Rust library logic directly, and neither tests the interactive TTY path (cliclack prompts in `init`, `add`, `rm`, `deploy`) because all tests run with piped stdin (`Stdio::null()`).

The CLI also has a meaningful interactive surface: `init` runs a full wizard (multiselect features, select template, provider multiselect), `add command/skill` prompts for metadata, `rm` asks for confirmation, and `deploy` shows offline/gitignore selects. These paths are exercised by real users on every run but have zero test coverage.

The restructure introduces three layers:
1. **Unit** — already exists as `#[cfg(test)]` blocks in `src/`. No change.
2. **Integration** — new Rust tests in `tests/integration/` that call library functions directly.
3. **E2E** — new TypeScript tests in `tests/e2e/` using `@microsoft/tui-test`, covering both CLI flows and interactive TUI flows via a PTY.

## Goals / Non-Goals

**Goals:**
- Integration tests that exercise Rust logic (config merging, rendering pipeline, feature parsing, cache, gitignore) without spawning a binary, running in milliseconds.
- E2E tests that cover every user flow — CLI (flag-driven) and TUI (interactive cliclack prompts) — using a real PTY so both paths are exercised.
- A discovery phase where an agent observes actual runtime behaviour (terminal output, prompt order, filesystem state) before tests are written, so assertions reflect reality not assumptions.
- All JS tooling (`bun`, `tui-devtools`, `@microsoft/tui-test`) installed and versioned through mise — no manual setup step.
- `mise test-all` continues to be the single command to run everything.

**Non-Goals:**
- Replacing unit tests (`#[cfg(test)]` blocks in `src/`) — those stay as-is.
- Testing network-dependent flows (remote template fetch) in the main suite — those remain `#[ignore]`d.
- Cross-platform PTY testing on Windows.
- Performance/load testing.

## Decisions

### D1 — `@microsoft/tui-test` for all e2e (not split Rust/TypeScript)

**Decision:** Use tui-test for both interactive and non-interactive e2e flows rather than keeping non-interactive flows in Rust.

**Rationale:** Once tui-test is introduced for the interactive path (the only tool that can drive cliclack prompts), using it for everything unifies the e2e layer into one framework and one language. Splitting by "was it interactive?" is splitting on implementation detail, not on user experience. The filesystem assertion gap (tui-test has no file helpers) is bridged trivially with Node.js `fs`.

**Alternative considered:** Keep non-interactive e2e in Rust, add tui-test only for TUI flows. Rejected because it creates two frameworks doing the same job in the same folder, and doubles the conceptual overhead for anyone writing new tests.

### D2 — Discovery phase before writing tui-tests

**Decision:** Before writing the tui-test suite, an agent runs every user flow using `tui-devtools` (for interactive flows) and bare binary invocation (for CLI flows), records exact terminal output and filesystem state per flow, then the tui-tests are written from those observations.

**Rationale:** Writing tests from source-reading alone encodes what you *think* the output looks like. The discovery phase captures: exact visual rendering of cliclack prompts (symbols, spacing, box-drawing), actual option order in multiselects (some are registry-fetched), exact text of success/error messages in TTY mode. This is particularly important for snapshot tests.

**Alternative considered:** Write tests directly from source code. Rejected because cliclack prompt rendering is opaque from source, and any deviation between assumed and actual output causes all snapshot tests to fail on first run.

### D3 — `tui-devtools` as mise npm backend tool, `@microsoft/tui-test` as local package.json dep

**Decision:** `tui-devtools` is installed globally via `"npm:tui-devtools" = "latest"` in `mise.toml`. `@microsoft/tui-test` is a dev dependency in `tests/e2e/package.json` managed via `bun install`.

**Rationale:** `tui-devtools` is a CLI daemon — it needs to be globally available as a command. The mise npm backend handles this without a separate `npm install -g` step. `@microsoft/tui-test` is both a CLI runner and an importable module (test files do `import { test, expect } from "@microsoft/tui-test"`). That import requires a local `node_modules` — a global install via mise does not satisfy module resolution in test files. Hence `package.json` + `bun.lockb` in `tests/e2e/`.

**Alternative considered:** Install both via mise npm backend. Rejected because global install of tui-test does not resolve the `import` in test files.

### D4 — Integration tests as Rust library calls, not binary spawn

**Decision:** `tests/integration/` tests call exported Rust functions directly (e.g., `AppConfig::from`, `render_feature_with_settings`, `CommandFeature::from_string`) rather than spawning the binary.

**Rationale:** Library-level tests are faster (no compile+spawn per test), isolate failures precisely (a rendering bug in `render_feature_with_settings` surfaces in `render.rs`, not in a deploy smoke test), and can test edge cases that are hard to provoke via CLI flags. The existing binary-spawning tests that were in `tests/integration/` move to `tests/e2e/` as CLI flows since they were always e2e smoke tests.

**Alternative considered:** Keep binary-spawning tests in `tests/integration/` alongside new library tests. Rejected because it blurs the layer contract and keeps the slow binary-spawn path in what should be a fast feedback loop.

### D5 — Flow inventory as the contract between discovery and test-writing

**Decision:** A fixed inventory of 57 flows (34 CLI, 15 TUI, 8 journey) is defined upfront and used as the explicit checklist for both the discovery agent and the test-writing phase.

**Rationale:** Without an explicit list, the discovery agent decides what counts as a flow and will miss some. The inventory also becomes the coverage contract — if a flow is in the list, there must be a corresponding tui-test.

## Risks / Trade-offs

**[Risk] Node.js/TypeScript in a Rust project's CI** → Mitigation: all JS tooling installs through a single `mise install` step; `bun` is fast (cold install ~3s). The `e2e-install` task is a `depends` prerequisite of `test-e2e` so the install is never forgotten.

**[Risk] tui-devtools daemon requirement** → tui-devtools runs as a daemon with WebSocket communication. The discovery agent must start the daemon before using it and kill it after. If the daemon crashes mid-discovery, a flow may be skipped. Mitigation: agent retries daemon start on failure; each flow runs in an isolated temp workspace so a crash doesn't corrupt other flows.

**[Risk] cliclack prompt rendering differs across terminal emulators** → tui-test uses xterm.js (the same renderer as VSCode). Actual user terminals may render box-drawing or colour differently. Mitigation: snapshot tests are generated from tui-test's own renderer, so they're consistent in CI even if they differ from a user's iTerm2. Use `getByText` for semantic assertions; reserve snapshots for layout-sensitive output like `ls` tables.

**[Risk] Registry-dependent prompts in `init` wizard (provider multiselect)** → The provider list is fetched from a remote registry; order is non-deterministic and content changes over time. Mitigation: discovery agent runs `init` with `--offline` for provider-list tests, or asserts on structural output (spinner appeared, multiselect appeared) rather than exact provider names.

**[Risk] `tests/e2e` is no longer a Rust test crate** → `cargo test` will no longer find e2e tests. `mise test-all` must be updated to run `bun x tui-test` separately. Mitigation: `test-all` task uses `depends = ["test", "test-integration", "test-e2e"]`; CI must use `mise test-all` not raw `cargo test`.

## Migration Plan

1. Update `mise.toml`: add `bun`, `npm:tui-devtools`, new tasks.
2. Add `tests/e2e/package.json` and run `bun install` to generate `bun.lockb`.
3. Add `tests/e2e/node_modules/` to `.gitignore`.
4. Remove `[[test]]` Cargo target for `e2e` from `Cargo.toml` (or rename so `cargo test` doesn't look for it).
5. Run discovery agent across all 57 flows; store structured observations.
6. Write tui-test files from observations.
7. Rewrite `tests/integration/` as Rust library tests.
8. Verify `mise test-all` exits 0.

**Rollback:** The old binary-spawning tests are deleted as part of this change. Git history preserves them. If tui-test proves unworkable, the Rust binary-invocation approach can be restored from git.

## Open Questions

- Should `tui-devtools` discovery output (flow observations) be committed to the repo as a reference, or used only ephemerally during the write phase?
- Does `@microsoft/tui-test` support Bun as the test runner natively, or does it need Node? (tui-test's README mentions `bun 1.3.5+` as supported — verify during e2e-install step.)
