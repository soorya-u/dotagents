## Context

`dotagents` is a Rust CLI managing agent configuration files. Users author skills and commands in `.dotagents/` (source of truth) and `dotagents deploy` renders them to provider-specific targets (e.g. `.claude/commands/`). Currently, no CLI surface exists for listing, creating, or removing these source files — users must hand-craft the directory structure and frontmatter.

Existing patterns to build on:
- **Dual-mode TUI**: `init` and `deploy` already use `cliclack` for interactive prompts, falling back to flag-based non-interactive operation. New commands follow the same pattern.
- **Workspace discovery**: `get_workspace_dir()` walks parents looking for `.dotagents[-debug]`. All new commands reuse this cached lookup.
- **Frontmatter parsing**: `gray_matter` is already used for `CommandFeature`. New commands reuse it for both skills and commands.
- **`cliclack`**: Already a direct dep (added for init/deploy wizard).
- **`crossterm`**: Transitive dep via `cliclack`; `terminal::size()` available for width detection.

## Goals / Non-Goals

**Goals:**
- `dotagents ls` — display skills and commands from `.dotagents/` with name + description; `--commands`/`--skills` to filter; `--verbose` for full descriptions; default truncates to terminal width.
- `dotagents add command <name>` / `dotagents add skill <name>` — create source files with frontmatter from flags or interactive prompts; body is a fixed starter template with name interpolated.
- `dotagents rm command <name>` / `dotagents rm skill <name>` — remove source files; `--deploy` flag or TTY confirm triggers deploy after mutation.
- Consistent dual-mode: flag(s) present or non-TTY → silent/flag path; no flags + TTY → cliclack wizard.

**Non-Goals:**
- Editing existing files (use `$EDITOR` directly).
- Managing files in `.claude/` or other deployed locations.
- Body content input at creation time (fixed starter template only).
- Listing or managing features other than skills and commands.
- Interactive filtering or search in `ls` output (no ratatui).

## Decisions

### 1. Source of truth: `.dotagents/` only

`ls`, `add`, and `rm` all operate exclusively on `.dotagents/` (the workspace root). They do not touch `.claude/` or any deployed target. Deployed state reflects `.dotagents/` only after a `deploy` run. This keeps each command single-responsibility.

**Alternative considered**: Reading from `.claude/` (deployed state) to mirror `npx skills ls`. Rejected — dotagents' contract is to manage source files; reading deployed state would conflate authoring and rendering.

### 2. CLI shape: `Add` and `Rm` as subcommand groups

```
Action::Ls(LsOptions)
Action::Add(AddAction)          // subcommand group
  AddAction::Command(AddCommandOptions)
  AddAction::Skill(AddSkillOptions)
Action::Rm(RmAction)            // subcommand group
  RmAction::Command(RmCommandOptions)
  RmAction::Skill(RmSkillOptions)
```

`Action::Skills { action: SkillsAction }` is untouched — it wraps `npx skills add` (installs from npm registry) and is a different operation from `dotagents add skill` (creates a blank local file).

### 3. Dual-mode field collection for `add`

Same pattern as `init`: if any relevant flag is provided OR stdin is non-TTY, use flags with empty-string defaults for missing fields. If no flags and TTY, run the cliclack wizard prompting for each missing field.

Field mapping per feature type:

| Feature   | Flags                                            | Frontmatter fields                               |
|-----------|--------------------------------------------------|--------------------------------------------------|
| command   | `--description/-d`, `--category/-c`, `--tags/-t` | `name` (from arg), `description`, `category`, `tags` |
| skill     | `--description/-d`, `--license/-l`, `--compatibility` | `name` (from arg), `description`, `license`, `compatibility` |

`metadata.author` and `metadata.version` in skill frontmatter are hardcoded defaults (`""` and `"1.0"`) — not exposed as flags, not prompted.

**Alternative considered**: Prompting for every field unconditionally. Rejected — breaks scripting and CI workflows.

### 4. Fixed starter body templates

No `--body` flag. Body is always written from a fixed in-process template string with the name interpolated at file-creation time. This avoids the UX complexity of multiline input (cliclack has no multiline widget) and editor spawning.

Command starter:
```markdown
# {name}

Brief description of what this command does.

## When to use

Describe when this command should be triggered.

## Steps

1. First step
2. Second step
3. Additional steps as needed
```

Skill starter:
```markdown
# {name}

Instructions for the agent to follow when this skill is activated.

## When to use

Describe when this skill should be used.

## Instructions

1. First step
2. Second step
3. Additional steps as needed
```

### 5. `--deploy` flag + TTY confirm on `add` and `rm`

After any mutation:
- `--deploy` present → call `deploy(DeployOptions::default())` immediately, no prompt.
- No `--deploy` + TTY → cliclack `confirm("Deploy now?")` defaulting to `false`.
- No `--deploy` + non-TTY → skip deploy silently.

This reuses the existing `deploy` function directly.

### 6. `ls` display via cliclack + `crossterm::terminal::size()`

`cliclack`'s `intro`, `log::step` (for section headers), and `outro` provide the spine (`│`, `◇`, `└`). Individual items are printed inline with manual ANSI formatting.

Terminal width is read once via `crossterm::terminal::size()` (already available as a transitive dep). If the call fails, fall back to 80 columns.

Description truncation: `available_width = terminal_cols - name_col_width - gap`. Default: truncate with `…`. `--verbose`: wrap full description at `terminal_cols - indent`.

`--commands` / `--skills` flags are additive filters: neither = show both; one = show only that section; both = show both (same as neither).

## Risks / Trade-offs

- **`cliclack` display limitations**: `cliclack` is prompt-oriented, not a layout engine. Alignment and truncation are done manually with string padding. If terminal width detection fails, output degrades gracefully to 80-col truncation. [Risk: misaligned columns on very narrow terminals] → Mitigation: minimum name column of 20 chars, clamp available description width to at least 10.

- **`rm` is destructive without undo**: Removing a source file has no recycle bin. [Risk: accidental deletion] → Mitigation: in TTY mode, cliclack `confirm("Remove <name>? This cannot be undone.")` before deletion; `--force` flag skips confirm.

- **Workspace not found**: If `get_workspace_dir()` returns an error (no `.dotagents/` in any parent), `ls`, `add`, and `rm` all fail with a clear error message referencing `dotagents init`. This is consistent with how `deploy` behaves.

- **Name collisions on `add`**: If `.dotagents/commands/<name>.md` already exists, `add` errors with a message suggesting `--force` to overwrite. `--force` flag added to `AddCommandOptions` and `AddSkillOptions`.

## Open Questions

- Should `ls` show a count summary line at the end (e.g., `4 skills · 3 commands`)? Assumed yes for now; easy to remove.
- Should `rm skill` prompt for confirmation even in non-TTY mode when `--force` is absent? Assumed no — non-TTY implies scripting context.
