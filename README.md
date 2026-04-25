# Dotagents

An AI agent configuration manager and templater, written in Rust.
Inspired by [Dotter](https://github.com/SuperCuber/dotter).

---

## The Problem

Every AI coding agent — Claude Code, Cursor, Codex, Gemini, Copilot, Windsurf —
has its own folder layout, config format, and conventions for the same conceptual
things: slash commands, system instructions, MCP servers.

If you use more than one agent, or work on a team, you end up copy-pasting and drifting.

## The Solution

Keep one source of truth in `.dotagents/` and run `dotagents deploy`. It renders
provider-specific config files for every agent you've configured, using Handlebars
templates. Change your instructions once. Deploy to every agent in one command.

---

## Quick Start

```bash
cargo install dotagents

dotagents init    # scaffold .dotagents/ with sample files
dotagents deploy  # render and write all provider config files
```

---

## How It Works

```
.dotagents/
├── config.toml          ← which features + providers to deploy to
├── local.config.toml    ← personal overrides (gitignored)
├── .env                 ← secrets, exposed as {{ env.* }}
├── INSTRUCTIONS.md      ← agent system instructions
├── mcp.jsonc            ← MCP server definitions
└── commands/
    └── review.md        ← one file per slash command
```

`dotagents deploy` reads these files and renders them through Handlebars templates,
once per configured provider:

```
.claude/commands/review.md      (Claude Code)
.cursor/rules/review.md         (Cursor)
...
```

---

## Configuration

`config.toml` declares which **features** to manage and which **providers** to deploy to:

```toml
features = ["commands", "instructions", "mcp"]
targets  = ["claude", "cursor", "gemini"]

[providers.claude.commands]
template = "https://dotagents.soorya-u.dev/templates/claude/command.hbs"
target   = "{{ dir.workspace }}/.claude/commands/{{ command.name }}.md"

[providers.cursor.instructions]
template = "https://dotagents.soorya-u.dev/templates/cursor/instructions.hbs"
target   = "{{ dir.workspace }}/.cursor/rules/instructions.md"
```

**Features** are the things you configure: `commands`, `instructions`, `mcp`.

**Providers** are the agents you deploy to. Community templates for 14+ providers
live in [`public/v1/templates/`](./public/v1/templates/).

Variables are available in templates and config values:
- `{{ var.* }}` — user-defined, from `[variables]` in config
- `{{ env.* }}` — secrets from `.dotagents/.env`
- `{{ dir.workspace }}` — current project root
- `{{ command.name }}` — interpolated per command in target paths

---

## Team Workflow

`config.toml` is committed and shared — it's your team's canonical agent setup.

`local.config.toml` (gitignored by default) lets each developer override providers,
disable features, or add personal targets without touching shared config:

```toml
# local.config.toml — personal, not committed
targets = []  # disable all shared targets

[providers.myagent.commands]
template = "{{ dir.application }}/templates/myagent/command.hbs"
target   = "{{ dir.workspace }}/.myagent/commands/{{ command.name }}.md"
```

---

## Supported Providers

14 providers with community templates:
`amp`, `auggie`, `claude`, `codebuddy`, `codex`, `copilot`, `cursor`, `gemini`,
`kilocode`, `opencode`, `qwen`, `roo`, `shai`, `windsurf`

Full templates at [`public/v1/templates/`](./public/v1/templates/).
To add a new provider, contribute a `.hbs` template and `provider.toml` snippet.

---

## Contributing

This project uses [OpenSpec](https://github.com/Fission-Codes/openspec) for change
proposals. Check `openspec/changes/` for active work before opening a new feature.

```bash
cargo build
cargo test
cargo fmt && cargo clippy
```
