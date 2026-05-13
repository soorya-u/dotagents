## Context

`commands ls` and `skills ls` display name + truncated description via cliclack output. Users need two additional output modes: machine-readable JSON for scripting, and full body content for detailed inspection. These flags establish a pattern for future read-only list commands.

## Goals / Non-Goals

**Goals:**
- `--json` flag: output `commands ls` and `skills ls` as structured JSON using `to_value()`
- `--content` flag: include the full markdown body content in output (extends existing behavior)
- Establish a consistent flag pattern for read-only list commands

**Non-Goals:**
- Changing default output format (remains name + description)
- Adding `--json` to mutation commands (`new`, `rm`, `add`)
- Per-item detail subcommands (future work)

## Decisions

### `--json` uses `FeatureTrait::to_value()` natively
Each feature type (Command, Skill) already implements `to_value()` which returns `serde_json::Value`. The `--json` flag collects all features into a `Vec<Value>` and serializes as a single JSON array. This avoids defining a parallel output schema and ensures consistency with template rendering.

### `--content` extends existing flag, includes body content
The `--content` flag already exists in both `commands ls` and `skills ls` specs for showing full descriptions. This change extends it to also include the full markdown body content (`content` / `body`). When `--content` is absent, only name + frontmatter fields (description, category, tags, etc.) are shown.

### `--json` alone outputs frontmatter only; `--content` adds body content when combined
When `--json` is active without `--content`, the output includes frontmatter fields only (name, description, category, tags). Body content is omitted. When both `--json` and `--content` are passed, each JSON object gains a `content` key containing the raw markdown body string. When `--json` is absent, `--content` only affects the human-readable CLI text output by including body content after frontmatter.

### CLI-only; TUI output unchanged
These flags affect CLI text/JSON output. The existing TUI list display (if any) is not modified. In non-TTY mode, text output includes the formatted content; in JSON mode, JSON is emitted.

### Flag names are reusable across commands
`--json` and `--content` use the same flag definitions across `commands ls` and `skills ls`. This establishes a pattern: any future read-only list command (e.g., `providers ls --json`) uses the same flag convention.

## Risks / Trade-offs

- **Large body content in terminal**: `--content` without a pager could flood the terminal for long commands/skills. → Acceptable; users explicitly opt in. A future enhancement could pipe through `less` but that's out of scope.
- **JSON output volume**: Full body content in JSON could be large. → The `--json` output is inherently machine-readable; consumers can filter as needed.
