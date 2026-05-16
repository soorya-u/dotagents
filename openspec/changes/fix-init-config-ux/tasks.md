## 1. Fix --template bypassing wizard

- [ ] 1.1 In `src/cli/init.rs:52`, change `is_tui_mode()` from `opts.features.is_none() && opts.template.is_none() && is_tui_enabled()` to `opts.features.is_none() && is_tui_enabled()`
- [ ] 1.2 In `src/cli/ui/init.rs`, check if there is a wizard step for template selection; if so, skip that step when `opts.template` is already provided (so the wizard doesn't prompt for template again)

## 2. Fix --json and --edit conflict

- [ ] 2.1 In `src/cli/options.rs`, add `conflicts_with = "edit"` to the `#[clap(long)]` attribute on `ConfigOptions.json` field: `#[clap(long, conflicts_with = "edit")]`

## 3. Fix AGENTS.md label

- [ ] 3.1 In `src/cli/ui/init.rs:31`, change `"AGENTS.md"` to `"INSTRUCTIONS.md"`
- [ ] 3.2 Update the hint string on the same line if it references AGENTS.md

## 4. Unit tests

- [ ] 4.1 Add unit test in `src/cli/init.rs`: `is_tui_mode()` returns `true` when `template.is_some()`, `features.is_none()`, and `is_tui_enabled()` is true
- [ ] 4.2 Add unit test in `src/cli/init.rs`: `is_tui_mode()` returns `false` when `features.is_some()` (regardless of template)

## 5. E2e tests

- [ ] 5.1 Add e2e TUI test in `tests/e2e/init.test.ts`: run `init --template mycode` in TTY, assert feature-selection multiselect appears (use tui-devtools observation for exact text), assert `"INSTRUCTIONS.md"` label appears (not `"AGENTS.md"`)
- [ ] 5.2 Add e2e test in `tests/e2e/config.test.ts` (or create file): `config --json --edit` exits 2 with error message containing "cannot be used with" or similar conflict text

## 6. Verification

- [ ] 6.1 Run `mise check` (fmt + clippy) — must exit 0
- [ ] 6.2 Run `mise tests` (unit + integration + e2e) — must exit 0
