## Why

Three small UX issues in `init` and `config` confuse users: (1) the `init` TUI wizard skips all prompts when `--template` is provided (even without `--features`), because `is_tui_mode()` gates on `template.is_none()` — users can't pick features interactively when they've pre-selected a template; (2) `config --json` silently overrides `--edit` with no warning — the two flags are mutually exclusive but Clap doesn't enforce it; (3) the `init` features multiselect labels the `instructions` feature as `"AGENTS.md"` when the actual file is `INSTRUCTIONS.md`.

## What Changes

- `is_tui_mode()` in `src/cli/init.rs` changes from `opts.features.is_none() && opts.template.is_none()` to `opts.features.is_none() && is_tui_enabled()` — the `--template` flag no longer bypasses interactive feature selection
- `ConfigOptions` in `src/cli/options.rs` gets `conflicts_with = "edit"` on the `json` field — Clap now errors if both `--json` and `--edit` are passed
- The `"AGENTS.md"` label in `src/cli/ui/init.rs` changes to `"INSTRUCTIONS.md"`
- Unit tests and e2e tests updated for all three changes

## Capabilities

### New Capabilities

### Modified Capabilities
- `init-wizard`: `--template` no longer suppresses the interactive feature-selection prompt; wizard runs normally and uses the pre-specified template
- `config-inspect`: `--json` and `--edit` are declared mutually exclusive; passing both exits with a Clap usage error

## Impact

- `src/cli/init.rs:52` — one-line change to `is_tui_mode()`
- `src/cli/options.rs` — add `conflicts_with = "edit"` attribute to `ConfigOptions.json` field
- `src/cli/ui/init.rs:31` — change `"AGENTS.md"` to `"INSTRUCTIONS.md"`
- Unit tests in `src/cli/init.rs` and `src/cli/options.rs`
- `tests/e2e/init.test.ts` — test that `--template` still shows feature prompts
- `tests/e2e/config.test.ts` (if exists) — test `--json --edit` conflict
