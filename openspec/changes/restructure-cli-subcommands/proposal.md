## Why

The current top-level `add`, `rm`, and `ls` commands are generic Unix-ish verbs that lack domain context, making the CLI harder to discover and reason about. The `--no-*` flags on `init` are a fragmented opt-out API that grows with every new feature; a single `--features` opt-in flag is more ergonomic and future-proof.

## What Changes

- **BREAKING** Remove top-level `add`, `rm`, and `ls` subcommands
- **BREAKING** Remove `--no-mcp`, `--no-command`, `--no-instruction`, `--no-skill` flags from `init`; replace with a single `--features <list>` flag
- Add `commands` subcommand group with `new`, `rm`, `ls` sub-actions (equivalent to old `add command`, `rm command`, `ls --commands`)
- Expand `skills` subcommand group with `new`, `rm`, `ls` sub-actions (equivalent to old `add skill`, `rm skill`, `ls --skills`); existing `skills add` (registry install) is unchanged
- `--features` accepts comma-separated values and/or repeated flags; `--features none` disables all features; passing `none` alongside other values or passing `--features` with no values is an error
- When `--features` is explicitly provided, the `init` TUI feature-selection screen is skipped
- `--full` flag is available on both `commands ls` and `skills ls`
- Remove the implicit `-v` → `--full` tie-in from `ls` (was incorrect behaviour)
- Delete `src/cli/add.rs`, `src/cli/rm.rs`, `src/cli/ls.rs`; logic moves into `src/cli/commands.rs` and expanded `src/cli/skills.rs`

## Capabilities

### New Capabilities

- `commands-subcommand`: `commands new / rm / ls` subcommand group replacing top-level `add command`, `rm command`, `ls --commands`
- `skills-subcommand-extended`: `skills new / rm / ls` sub-actions extending the existing `skills` group
- `init-features-flag`: `--features` flag on `init` replacing the four `--no-*` boolean flags

### Modified Capabilities

- `add-command`: requirement changes — `add command` is removed; creation moves to `commands new`
- `rm-command`: requirement changes — `rm command` is removed; deletion moves to `commands rm`
- `ls-command`: requirement changes — top-level `ls` is removed; listing splits into `commands ls` and `skills ls`
- `skills-add`: requirement changes — `skills add` (registry install) is unchanged, but `skills` group gains `new`, `rm`, `ls` peers
- `init-wizard`: requirement changes — `--no-*` flags removed; `--features` flag added; TUI feature screen skips when `--features` is provided

## Impact

- **CLI surface**: three top-level subcommands removed, one new subcommand group added, one existing subcommand group expanded — a breaking change for any scripts using `add`, `rm`, or `ls` directly
- **Source files**: `src/cli/add.rs`, `src/cli/rm.rs`, `src/cli/ls.rs` deleted; `src/cli/commands.rs` added; `src/cli/skills.rs`, `src/cli/options.rs`, `src/cli/init.rs` updated
- **Shell completions**: generated completions must be regenerated after this change
- **Docs / README**: any usage examples referencing the old commands need updating
