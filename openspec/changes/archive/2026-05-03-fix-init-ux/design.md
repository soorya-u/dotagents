## Context

The `dotagents init` command scaffolds the `.dotagents/` workspace. It has an interactive TUI path (via `cliclack`) and a headless CLI path. Three defects exist:

1. `intro("dotagents · init")` in `src/cli/ui/init.rs:11` echoes the command name.
2. `info!("Skipping {}", file.path.display())` in `src/cli/init.rs:190` emits a `●` bullet for every skipped file — noise that surfaces for both feature-gated files (user chose not to enable a feature) and template-variant files (mycode templates skipped when Starter is chosen).
3. `update_config_targets` is called after the wizard to write chosen targets, but no equivalent function exists for features — so `config.toml` always gets `features = ["commands", "instructions", "mcp", "skills"]` from the static mock regardless of wizard selections.

## Goals / Non-Goals

**Goals:**
- Intro text describes the action, not the command path.
- Skip messages disappear from default output; still accessible via `-v`.
- Features selected (or deselected) in the wizard are reflected in both `config.toml` and `local.config.toml`.
- Headless path (`--features none` or `--features commands,mcp`) also persists features correctly.

**Non-Goals:**
- Changing the `variables` key behaviour — it stays hardcoded.
- Any changes to the deploy or other commands.
- Rewriting the mock config structure beyond the features field.

## Decisions

**D1 — `debug!` not silent for skips**

Options: remove the log entirely, demote to `debug!`, or keep `info!`.

Chosen: `debug!`. Removing entirely loses the ability to diagnose init behaviour. `debug!` preserves observability under `-v` while keeping the default TTY output clean. No new logic needed — a single word change at line 190.

**D2 — `update_config_features` mirrors `update_config_targets`**

Both functions: read the TOML file, parse it into `toml::Value`, mutate the target key, serialise with `toml::to_string_pretty`, write back. Reusing this exact pattern keeps the two post-wizard update calls symmetric and avoids introducing a new abstraction.

The features array is serialised from the `Feature` enum: each selected variant maps to its string form (`"commands"`, `"instructions"`, `"mcp"`, `"skills"`). `Feature::None` maps to an empty array.

**D3 — Apply to both TUI and headless paths**

Currently, `update_config_targets` is only called inside the `if tui_mode { … }` block. Features must also be persisted in headless mode (e.g., `dotagents init --features commands,mcp`). The call to `update_config_features` is placed after the `tui_mode` guard — called whenever `opts.features` is `Some`, regardless of mode.

**D4 — Intro text**

Replace `"dotagents · init"` with `"dotagents"` — the app name alone, matching the minimal style used in other CLIs built on cliclack. Avoids encoding the subcommand in the header.

## Risks / Trade-offs

- **Existing unit tests** reference the static mock content in `config.toml`. Tests that assert `features = [...]` in the written file will need updating to expect the persisted selection. Low risk — straightforward test updates.
- **Headless callers** that pass `--features` today get the right config file written for the first time; this is a behaviour fix, not a breaking change. If any script depended on the (incorrect) all-features output, it will need updating — acceptable since the old behaviour was a bug.
