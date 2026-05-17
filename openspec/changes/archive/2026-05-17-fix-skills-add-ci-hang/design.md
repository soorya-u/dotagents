## Context

`PackageRunner::args()` in `src/core/config/common.rs` builds the argv list for invoking `skills add` via npm/pnpm/yarn/bun. None of the four variants append `--yes`, so in non-TTY environments the subprocess immediately blocks on an interactive confirmation menu. The call site in `src/cli/skills.rs` has access to `is_tui_enabled()` but currently passes no CI hint to `args()`.

## Goals / Non-Goals

**Goals:**
- Pass `--yes` to all four package runner variants when running in CI/non-TTY mode
- Keep the API change minimal and backwards-compatible in the codebase (single call site)
- Cover all four variants in unit tests

**Non-Goals:**
- Changing how `is_tui_enabled()` itself determines CI mode — that logic is already correct
- Adding `--yes` in interactive mode — this should only apply in non-TTY

## Decisions

1. **Extend `args` signature to `args(&self, skill_name: &str, ci: bool)`**: A boolean parameter is the simplest approach. Alternative: a separate `ci_args()` method — rejected because it duplicates the argv construction logic for all four variants.

2. **Append `--yes` as the last argument**: Most package runners place `--yes` / `-y` at the end of the command. This is the common convention across npm/pnpm/yarn/bun for non-interactive installs.

3. **Call site passes `!is_tui_enabled()`**: `is_tui_enabled()` already encapsulates the CI/non-TTY detection logic. Re-using it here avoids a second detection path.

## Risks / Trade-offs

- **Older package runner versions may not recognize `--yes`**: Mitigation: `--yes` is a long-established flag in all four runners; risk is negligible.
- **Single call site means a missed update would silently omit `--yes`**: Mitigation: the unit test suite for `args(_, true)` catches any regression.
