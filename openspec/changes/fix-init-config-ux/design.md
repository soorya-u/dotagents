## Context

**Issue 1 (`--template` bypasses wizard)**: `is_tui_mode()` returns `false` when `opts.template.is_some()`. This was likely intended to skip the template-selection prompt when a template is already specified, but it also skips the feature-selection multiselect. The template selection and feature selection are independent wizard steps. The fix is to separate the `--template` guard from the TUI mode check.

**Issue 2 (`--json` silently wins over `--edit`)**: `config.rs` checks `opts.json` before `opts.edit`. When both are passed, JSON wins without any error message. Clap's `conflicts_with` attribute makes this a parse-time error with a clear message.

**Issue 3 (`"AGENTS.md"` label)**: The actual file written by `init` is `INSTRUCTIONS.md` (see `constants/file.rs: INSTRUCTIONS_FILE`). The `"AGENTS.md"` label is wrong.

## Goals / Non-Goals

**Goals:**
- `--template` stops suppressing the interactive feature-selection wizard
- `--json` and `--edit` together produce a Clap error (exit 2) with usage message
- The `instructions` feature label in the init wizard is corrected to `"INSTRUCTIONS.md"`

**Non-Goals:**
- Adding new wizard steps or changing the wizard flow beyond unblocking the `--template` case
- Adding a `--template` wizard step (template can still be pre-specified; it just no longer skips feature prompts)

## Decisions

1. **`is_tui_mode` change**: Remove `opts.template.is_none()` from the condition. If the user needs the template prompt specifically, they can omit `--template`. The `--template` flag pre-fills the template choice but should not bypass the feature wizard.

2. **`conflicts_with` over runtime guard**: Clap's `conflicts_with` enforces the constraint at parse time before any code runs, which gives the user a proper usage error and `--help` mention. A runtime `if opts.json && opts.edit { bail! }` approach would work but is less idiomatic and doesn't update `--help`.

3. **Label-only change for `"INSTRUCTIONS.md"`**: No behavior change — only the multiselect item label displayed to the user changes.

## Risks / Trade-offs

- **`is_tui_mode` change may surface an extra template prompt in the wizard**: If the wizard has a template-selection step, it will now run when `--template` is also provided. Mitigation: verify in `src/cli/ui/init.rs` that the template step is either skipped when `opts.template` is already set, or doesn't exist as a separate step. Adjust the wizard to skip that step while still running the feature step.
