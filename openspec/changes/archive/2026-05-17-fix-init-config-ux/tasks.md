## 1. Fix --template bypassing wizard

- [x] 1.1 In `src/cli/init.rs:52`, change `is_tui_mode()` from `opts.features.is_none() && opts.template.is_none() && is_tui_enabled()` to `opts.features.is_none() && is_tui_enabled()`
- [x] 1.2 In `src/cli/ui/init.rs`, wrap template selection step in `if opts.template.is_none()` so it is skipped when `--template` is already provided

## 2. Fix --json and --edit conflict

- [x] 2.1 In `src/cli/options.rs`, add `conflicts_with = "edit"` to the `#[clap(long)]` attribute on `ConfigOptions.json` field: `#[clap(long, conflicts_with = "edit")]`

## 3. Fix AGENTS.md label

- [x] 3.1 In `src/cli/ui/init.rs:31`, change `"AGENTS.md"` to `"INSTRUCTIONS.md"`
- [x] 3.2 Updated the hint string from `"Sync a global AGENTS.md"` to `"Sync a global INSTRUCTIONS.md"`

## 4. Unit tests

- [x] 4.1 Add unit test in `src/cli/init.rs`: `is_tui_mode()` with only `--template` set matches the baseline (template no longer disables TUI)
- [x] 4.2 Add unit test in `src/cli/init.rs`: `is_tui_mode()` returns `false` when `features.is_some()` (regardless of template) — replaced old `is_tui_mode_false_when_any_headless_flag_set` with `is_tui_mode_false_when_features_flag_set`

## 5. E2e tests

- [x] 5.1 Add e2e TUI test in `tests/e2e/init.test.ts`: T06 runs `init --template starter` in TTY, asserts feature-selection multiselect appears, asserts `"INSTRUCTIONS.md"` label, and confirms template prompt is skipped
- [x] 5.2 Add e2e test in `tests/e2e/config.test.ts`: `config --json --edit` exits 2 with error message containing "cannot be used with"

## 6. Verification

- [x] 6.1 Run `mise check` (fmt + clippy) — exit 0
- [x] 6.2 Run `mise tests` (unit + integration + e2e) — exit 0 (292 unit, 40 integration, 192 e2e passed)
