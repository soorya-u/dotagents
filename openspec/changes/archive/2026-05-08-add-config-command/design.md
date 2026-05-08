## Context

Dotagents has a layered config system: `config.toml` (global), `local.config.toml` (overrides), and `AppConfig` (runtime merge). Users edit these files manually with TOML but have no way to inspect the resolved configuration. TUI tools like `cliclack` enable interactive inspection and editing.

## Goals / Non-Goals

**Goals:**
- Display the resolved AppConfig (active features, targeted providers, per-feature settings)
- Support inspecting individual config layers (`global`, `local`)
- `--json` flag for machine-readable config
- `--edit` flag for interactive TUI editing of `global` and `local` configs
- CLI mode for non-interactive environments

**Non-Goals:**
- Editing `app` config (it's derived, not persisted)
- Schema validation or linting of config values
- Real-time watch/reload of config

## Decisions

### Subcommand argument: `app` as default
`dotagents config` defaults to showing the merged AppConfig. `dotagents config app` is an explicit alias. `dotagents config global` and `dotagents config local` show individual layers. This hierarchy matches user mental model: "show me what's applied" is the primary use case.

### `--edit` only on global/local, not app
AppConfig is computed at runtime from the merge of global+local. Editing it directly makes no sense — changes would have nowhere to persist. The CLI rejects `--edit` on `app` with a clear error.

### TUI editor: cliclack multiselect + form prompts
Editing uses cliclack:
- **Features**: multiselect from `["commands", "instructions", "mcp", "skills"]`
- **Providers**: multiselect from registry names + any custom providers in current config, with an option to type new custom names
- After selection, writes the chosen features as `features = [...]` and providers as `targets = [...]` in the appropriate TOML file
- Provider `FeatureSettings` editing (template, target, variables) is deferred to a future change for scope control

### JSON output uses existing serde serialization
`GlobalConfig`, `LocalConfig`, and `AppConfig` already derive `Serialize`. `--json` serializes the selected config layer directly. For `app`, a display-friendly structure is built at runtime merging features and provider settings.

### CLI mode for non-TTY environments
When stdin is not a TTY, the command outputs a plain-text listing without interactive prompts. `--edit` in non-TTY is rejected with a message directing users to a TTY.

## Risks / Trade-offs

- **Editing scope creep**: Full FeatureSettings editing (template, target paths) in TUI is complex. → Deferred; the TUI editor handles features and targets (provider selection) only, which covers the 80% use case.
- **Config file write conflicts**: If a user manually edits config while the TUI editor is open, changes may be lost. → The editor reads at open time and writes atomically (write to temp file, then rename). A warning is shown if the file was modified since read.
