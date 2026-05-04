## Why

Three TUI output inconsistencies make the CLI feel rough and unfinished: the `deploy` command has no cliclack framing while every other interactive command does; `rm` and `new` subcommands display the command path as their intro (telling the user what they already typed); and the `.gitignore` fence uses a custom comment format that editors treat as plain text rather than a collapsible region.

## What Changes

- **Deploy intro + outro:** Add `intro("dotagents")` at the start of the interactive deploy flow (gated on `is_tty()`) and `outro("Done.")` at the end, covering all return points after the gitignore step.
- **Descriptive intro text across subcommands:** Replace `intro("dotagents skills rm")` with `intro("Remove skill")`, `intro("dotagents commands rm")` with `intro("Remove command")`, `intro("dotagents skills new")` with `intro("New skill")`, and `intro("dotagents commands new")` with `intro("New command")`. The `rm` intros are also made `is_tty()`-gated (currently unconditional).
- **Gitignore fence format:** Change `FENCE_START` from `"# BEGIN dotagents managed - do not edit manually"` to `"#region dotagents"` and `FENCE_END` from `"# END dotagents managed"` to `"#endregion dotagents"`. No migration for existing `.gitignore` files. All tests referencing the old strings are updated.

## Capabilities

### New Capabilities

*(none)*

### Modified Capabilities

- `deploy-outro`: Deploy now has a cliclack intro and outro in interactive mode.
- `deploy-gitignore-update`: Gitignore fence format changes to `#region`/`#endregion` style.

## Impact

- `src/cli/deploy.rs` — add `intro()` near the top of the interactive path; add `outro()` at all TTY return points.
- `src/cli/skills.rs` — change `intro("dotagents skills rm")` to `intro("Remove skill")` and gate on `is_tty()`; change `intro("dotagents skills new")` to `intro("New skill")`.
- `src/cli/commands.rs` — same changes for `rm` and `new`.
- `src/utils/gitignore.rs` — update `FENCE_START` and `FENCE_END` constants (lines 23–24).
- `tests/integration/gitignore.rs`, `tests/integration/undeploy.rs`, `tests/e2e/undeploy.test.ts` — update all assertions referencing old fence strings.
