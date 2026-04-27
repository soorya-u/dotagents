## Why

Users author skills and commands in `.dotagents/` but have no CLI surface to inspect, create, or remove them — they must manually create files and folders with the correct structure. A `ls`, `add`, and `rm` command gives dotagents a first-class CRUD interface over the content it manages.

## What Changes

- Add `dotagents ls [--commands] [--skills] [--verbose]` — lists skills and commands from `.dotagents/` with name and description, rendered via cliclack. Descriptions truncate to terminal width by default; `--verbose` shows full text.
- Add `dotagents add command <name>` — creates `.dotagents/commands/<name>.md` with frontmatter collected via flags or interactive cliclack prompts, plus a fixed starter body template.
- Add `dotagents add skill <name>` — creates `.dotagents/skills/<name>/SKILL.md` with frontmatter collected via flags or interactive cliclack prompts, plus a fixed starter body template.
- Add `dotagents rm command <name>` — removes `.dotagents/commands/<name>.md`.
- Add `dotagents rm skill <name>` — removes `.dotagents/skills/<name>/` directory.
- `add` and `rm` both accept `--deploy` to trigger deploy after mutation; in TTY mode without the flag, a cliclack confirm prompt asks the user at the end.

## Capabilities

### New Capabilities

- `ls-command`: List skills and commands from `.dotagents/` with name + description, filtered by `--commands`/`--skills`, truncated to terminal width, with `--verbose` for full output. Uses cliclack for display.
- `add-command`: Create a new command or skill file in `.dotagents/` with frontmatter populated from flags (`--description`, `--category`, `--tags`, `--license`, `--compatibility`) or interactive cliclack prompts in TTY mode. Body is a fixed starter template with the name interpolated.
- `rm-command`: Remove a command or skill from `.dotagents/` by name. Supports `--deploy` flag and TTY deploy prompt.

### Modified Capabilities

## Impact

- `src/cli/options.rs` — new `Ls`, `Add`, `Rm` variants on `Action` enum
- `src/cli/ls.rs` — new module: workspace discovery, frontmatter reads, cliclack display
- `src/cli/add.rs` — new module: dual-mode field collection, file/dir creation, starter template rendering
- `src/cli/rm.rs` — new module: file/dir removal, optional deploy trigger
- `src/cli/ui/ls.rs` — cliclack rendering helpers (truncation, section headers)
- `src/cli/runner.rs` — dispatch arms for new actions
- `gray_matter` already present — reused for frontmatter parsing
- `crossterm::terminal::size()` used for terminal width detection (already a transitive dep via cliclack)
- No new external dependencies required
