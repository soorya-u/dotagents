## Context

The `init` command currently uses an all-or-nothing wizard model: if any flag (`--features`, `--template`) is provided, the entire wizard is skipped. Provider targets can only be selected via the interactive wizard — there is no `--targets` flag. The `Feature::None` sentinel was added before `--ci` existed as a way to disable all features headlessly. Provider display formats are inconsistent across `init`, `config --edit`, and `providers ls`.

Key source files:
- `src/cli/options.rs` — `InitOptions`, `Feature` enum, `has_feature()`
- `src/cli/init.rs` — `is_tui_mode()`, `validate_features()`, `build_config_content()`, `initialize_agents_dir()`
- `src/cli/ui/init.rs` — `run_init_wizard()`, `prompt_targets()`
- `src/cli/providers.rs` — `run_tui()`, `print_text()`
- `src/cli/config.rs` — `edit_global_config()`, `edit_local_config()` (both call `prompt_targets`)

## Goals / Non-Goals

**Goals:**
- Add `--targets` flag to `init` for non-interactive provider selection
- Simplify wizard flow: TUI mode = `is_tui_enabled()`, each prompt skips when its flag is provided
- Remove `Feature::None` sentinel (redundant with `--ci`)
- Make provider display consistent: `Name [slug]` in init/config, slug-only select in `providers ls`

**Non-Goals:**
- Changing the `providers ls` plain-text or JSON output format
- Adding `--targets` to `config --edit` (it already uses `prompt_targets` interactively)
- Changing how providers are stored in config files (still slugs)

## Decisions

### 1. Promote `targets` to a proper clap flag
Change `#[clap(skip)] pub targets: Vec<String>` to `#[clap(long, value_delimiter = ',', num_args = 1..)] pub targets: Option<Vec<String>>`. Mirrors the `--features` flag pattern exactly. Downstream code that reads `opts.targets` changes from `Vec<String>` to `Option<Vec<String>>` — use `.as_deref().unwrap_or_default()` where needed.

**Alternative considered:** Adding a separate `targets_flag` field and keeping the internal `targets` field. Rejected — unnecessary indirection when `Option` already distinguishes "not provided" from "provided empty."

### 2. Simplify `is_tui_mode()` to just `is_tui_enabled()`
Remove flag-presence checks from `is_tui_mode()`. The wizard always runs in TUI mode; each prompt internally checks whether its corresponding option is already `Some`. This means `dotagents init --features commands` in a TTY will show the template and targets prompts but skip the features prompt.

**Alternative considered:** Keep the current all-or-nothing model. Rejected — user explicitly wants partial flag + partial wizard behavior.

### 3. Remove `Feature::None` variant
Delete `Feature::None` from the enum. Remove `validate_features()` (the only validation it did was checking None-mixing). Simplify `feature_to_str()` to not handle None. Change `has_feature()` to return `false` when `self.features` is `None` (was: `true`).

This means `--features` absence in non-TUI mode = no features scaffolded. In TUI mode, the wizard always populates `opts.features`, so `None` only persists in headless/CI contexts.

**Alternative considered:** Keep `Feature::None` for explicitness. Rejected — `--ci` without `--features` already communicates "I don't want features."

### 4. Provider display in `prompt_targets()`
`prompt_targets()` already fetches the registry which has `name` and `url` fields. Change the multiselect item display from bare slug to `Name [slug]` format (e.g., "Claude Code [claude]"). For providers without a name, fall back to just the slug.

This function is used by both `init` wizard and `config --edit`, so both get the new format.

### 5. Provider display in `providers ls` TUI
Change `run_tui()` in `providers.rs`: the select label becomes the slug (was: name), and the hint is removed. On selection, the outro shows `Name (url)` (already works this way). This inverts the current behavior where the label was the name and hint was `[slug] url`.

## Risks / Trade-offs

- **CI behavior change**: `dotagents --ci init` without `--features` now scaffolds nothing instead of everything. No current users affected per product owner.
- **Wizard UX**: Providing one flag (e.g., `--targets claude`) now still shows prompts for other options (features, template). Some users might expect all-or-nothing. Mitigated by clear help text.
- **Network call in `prompt_targets()`**: Already exists; the display change is purely formatting. No new network calls introduced.
