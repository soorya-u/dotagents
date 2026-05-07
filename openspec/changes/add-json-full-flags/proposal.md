## Why

`commands ls` and `skills ls` currently only show truncated name + description. Users scripting dotagents need machine-readable output (`--json`), and users inspecting content need the full body (`--full` including markdown body, not just frontmatter). These flags standardize output across read-only commands.

## What Changes

- Add `--json` flag to `dotagents commands ls` and `dotagents skills ls` — outputs using each feature's `to_value()` representation
- Add `--full` flag to `dotagents commands ls` and `dotagents skills ls` — includes the full markdown body content in addition to frontmatter metadata
- Without `--full`, only the command/skill name and frontmatter metadata are shown (current behavior, made explicit)
- The `--full` flag already exists in both `commands ls` and `skills ls` specs but only covers descriptions; this extends it to also include body content
- Both flags are designed to be applicable to any future read-only list commands

## Capabilities

### New Capabilities
- `cli-output-formats`: standardised `--json` and `--full` flags for read-only list commands, providing machine-readable JSON output and verbose body-inclusive display

### Modified Capabilities
- `commands-subcommand`: `commands ls` gains `--json` flag and `--full` is extended to include body content
- `skills-subcommand-extended`: `skills ls` gains `--json` flag and `--full` is extended to include body content

## Impact

- `src/cli/commands.rs` — `ls` subcommand gains `--json` and extended `--full` flags
- `src/cli/skills.rs` — `ls` subcommand gains `--json` and extended `--full` flags
- `src/schema/features/` — may need additional serialization support for JSON output
- `tests/e2e/` — new e2e tests covering CLI and TUI paths with both flags
