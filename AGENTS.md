## Project Overview

**Dotagents** is a Rust CLI that manages and templates configuration files for AI agents (Claude Code, Codex, Cursor, Copilot, Gemini, Windsurf, etc.), inspired by [Dotter](https://github.com/SuperCuber/dotter). Users keep one source-of-truth in `.dotagents/` (commands, instructions, MCP servers, env, variables) and `dotagents deploy` renders provider-specific files (e.g. `.claude/commands/<name>.md`, `.cursor/...`) using Handlebars templates.

## Verification (always run before finishing)

After every change — no exceptions — run both of these and fix anything that fails:

```bash
mise check      # cargo fmt + cargo clippy (format & lint)
mise test-all   # cargo test  (unit + integration + e2e)
```

Both commands must exit 0 before a task is considered done.

## Common Commands

```bash
# Build
mise run build                                    # debug build
mise run build-release                            # optimised release build (LTO + strip)

# Run the CLI
mise run run -- init [--no-mcp|--no-command|--no-instruction] [--force]
mise run run -- deploy
mise run run -- gen-completions --shell bash --to ./completions

# Tests (individual suites)
mise run test              # unit tests only (src/ colocated #[cfg(test)] blocks)
mise run test-integration  # tests/integration/ smoke tests
mise run test-e2e          # tests/e2e/ full end-to-end suite

# Raw cargo (useful for filtering by test name)
cargo test <name>                                 # single test by name (substring match)
cargo test <module>::tests -- --nocapture        # show stdout from a module's tests
```

Notes that bite:
- **Debug builds use a different root directory.** `ROOT_DIR` is `.dotagents-debug` under `cfg(debug_assertions)` and `.dotagents` in release (see `src/constants/dir.rs`). When you `cargo run -- init`, it scaffolds `.dotagents-debug/`; only `cargo run --release` (or the released binary) touches `.dotagents/`. `.dotagents-debug` is in `.gitignore`.
- `init` requires `--force` to overwrite an existing root dir in release; in debug, `force` defaults to true (`#[clap(default_value_t = cfg!(debug_assertions))]`).
- `cargo run -- deploy` must be run from a directory at or below a workspace root containing the root dir — `get_workspace_dir()` walks parents looking for `.dotagents` / `.dotagents-debug` and caches the first hit in a `OnceLock` for the whole process.

## Architecture

### Pipeline: init → edit → deploy

1. `init` (`src/cli/init.rs`) creates the root dir and writes a fixed list of mock files (`config.toml`, `local.config.toml`, `.env`, `.gitignore`, `INSTRUCTIONS.md`, `mcp.jsonc`, sample command, sample `mycode` provider templates). Mock contents are embedded at compile time via `include_str!` in `src/constants/mocks.rs`, sourced from `src/mocks/`.
2. The user edits `config.toml` / `local.config.toml` to declare which `features` are active (`commands` | `instructions` | `mcp`) and which `targets` to deploy to. Targets are grouped under `[targets]` as `ide`, `cli`, or `custom`. Each target maps to `[providers.<group>.<name>.<feature>]` entries giving a `template` path and a `target` path (both Handlebars strings).
3. `deploy` (`src/cli/deploy.rs`) builds an `AppConfig`, then for each enabled feature loads the source data (commands from `.dotagents/commands/*.md`, MCP from `mcp.jsonc`, instructions from `INSTRUCTIONS.md`), and renders the feature once per matching provider. Providers are iterated in **parallel via `rayon::par_iter`**, so any new provider/feature code must be `Sync`.

### Configuration layering (`src/schema/config/`)

Three layered structs all implement `TomlConfig` (de/serialize via TOML):

- `GlobalConfig` (`config.toml`) — base settings, validates `features` against `["commands", "instructions", "mcp"]`.
- `LocalConfig` (`local.config.toml`) — fully optional; everything overrides global. Typically gitignored.
- `AppConfig` — runtime merge of global+local built by `AppConfig::from_application(templater)`. The merge is **field-level with custom merge functions**: `Targets::merge` and `Providers::merge` deep-merge `ide` / `cli` / `custom` maps; per-provider `Features` and `FeatureSettings` further merge inner fields (`template`, `target`, `disabled`, `variables`, `hash`). Helpers live in `src/utils/merge.rs` (`merge_optional`, `merge_optional_or_default`).

`AppConfig::has_feature(name)` controls whether a feature is rendered; `get_provider_feature_settings(name)` returns the `(provider_name -> FeatureSettings)` map after filtering out `disabled = true` entries. `CacheConfig` (`cache.toml`) is a slimmed-down derivative used for tracking deployed-file hashes — wired through `AppConfig::to_cache()` but the cache pipeline is not wired into deploy yet.

### Feature abstraction (`src/schema/features/`)

Adding a new feature means implementing `FeatureTrait` (in `src/schema/features/traits.rs`):

```rust
trait FeatureTrait: Sized {
    fn from_string(value: &str) -> Result<Self>;
    fn to_string(&self) -> Result<String>;
    fn to_value(&self) -> Value;                 // exposes data to templates as JSON
    fn populate_with_values(&self, t: &Templater, v: Option<&Value>) -> Result<Self> { ... }
    fn get_file_name(&self) -> Option<String> { None }  // Some(...) → per-item output (commands, future skills)
}
```

Existing implementations:
- `CommandFeature` (markdown + YAML frontmatter, parsed via `gray_matter`). One file per command, `get_file_name()` returns `metadata.name` so the renderer interpolates `{{ command.name }}` into the target path.
- `InstructionFeature` (single `INSTRUCTIONS.md` content blob).
- `McpFeature` (JSON5 / `jsonc`, parsed via `serde_json5`; `ServerConfig` is a tagged enum `Http` | `Stdio`).

A planned `SkillFeature` is described in `openspec/changes/implement-skills/` — that proposal also adds a `get_name_variable` default method to `FeatureTrait` so the renderer stops being hardcoded to commands. Read it before extending the trait.

### Template engine (`src/templates/`)

- `Templater` is a `LazyLock` global wrapping a `handlebars::Handlebars`. Get it via `get_templater()`.
- Two custom helpers, registered in `Templater::new`:
  - `{{#ifEq a b}}…{{else}}…{{/ifEq}}` (`IfEqHelper`)
  - `{{json value}}` — renders any value as compact JSON (`JsonHelper`)
- Default templates registered at startup: `config.toml` and `local.config.toml` (rendered before parsing, so users can use Handlebars inside their config — e.g., `target = "{{ dir.workspace }}/.claude/commands/{{ command.name }}.md"`).
- **Variable namespaces** available everywhere (built in `src/templates/variables.rs`):
  - `{{ dir.workspace | application | config | home }}` — resolved paths.
  - `{{ env.* }}` — keys from `.dotagents/.env` parsed via `dotenvy`, **lowercased**.
  - `{{ var.* }}` — user-defined vars (`[variables]` table in config, plus per-provider-feature `variables = {…}`); per-provider vars deep-merge over globals.
  - `{{ command.name }}`, `{{ command.content }}`, `{{ instruction.content }}`, `{{ mcp.servers.* }}` — feature-injected via `to_value()`.

Rendering is **two-phase** in `templates/renderer.rs::render_feature_with_settings`:
1. The feature's serialized form is rendered against `var.*` (so users can use `{{ var.foo }}` inside `INSTRUCTIONS.md`, MCP env values, command bodies). Result is parsed back into the same feature type via `from_string`.
2. The provider's template file is rendered with the populated feature's `to_value()` merged in, then written to the target path (parent dirs auto-created in `utils/fs.rs::write_file`).

### CLI shape (`src/cli/`)

`Options` (Clap derive) → `Action::{Init, GenCompletions, Deploy}`. `runner::run` dispatches; if no subcommand is given it prints help and exits 0. `main.rs` maps the result: `Ok(true)` → exit 0, `Ok(false)` → exit 1, `Err` → `display_error` (chains `anyhow` causes via `error.chain()`) then exit 1. Logger is initialized **after** parsing options because verbosity (`-v`/`-vv`/`-vvv`, clamped to 3) and `--quiet` come from the CLI; logs are filtered to only `dotagents` targets (see `utils/logs.rs`).

### Public templates & registry (`public/v1/`)

`public/v1/templates/<provider>/` holds the canonical Handlebars templates and `provider.toml` snippets that users reference by URL. The GitHub Action `.github/workflows/generate-registry.yml` runs on pushes touching `public/v1/templates/**`, executes `scripts/ci/detect_template_changes.sh` and `scripts/ci/generate_registry.sh` (which uses `jq` to enumerate `cli/*` and `ide/*` provider dirs), and opens a PR that updates `public/v1/templates/registry.json`. The registry currently expects the `cli/` and `ide/` subdirectory layout — note that today the templates live flat under `public/v1/templates/<provider>/` (e.g. `claude/`, `codex/`), so the script's `$ROOT/cli/*` glob will find nothing until those dirs exist.

## OpenSpec workflow

This repo uses [OpenSpec](https://github.com/Fission-Codes/openspec) for change proposals. Proposed changes live in `openspec/changes/<change-id>/` with `proposal.md`, `design.md`, `tasks.md`, and `specs/`. The `opsx:propose` / `opsx:apply` / `opsx:archive` / `opsx:explore` skills (also exposed as `.claude/commands/opsx/*.md`) drive the lifecycle. When implementing a new feature, check `openspec/changes/` for an existing proposal first — `implement-skills` is the active one.

## Conventions

- All public modules use `pub(crate)` visibility — keep new modules consistent.
- Errors: return `anyhow::Result<…>` and add `.context("…")` at boundary calls so `display_error` produces a useful chain.
- Tests are colocated in `#[cfg(test)] mod tests` blocks at the bottom of each file (commands, schema, helpers, utils all follow this pattern).
- `#![allow(unused)]` is set in `main.rs` (marked TODO) — feel free to leave new dead code temporarily but prefer wiring things up.
- `WORKSPACE_DIR` is cached in a `OnceLock`, so tests that depend on workspace discovery can interfere with each other; the existing tests document this caveat.
- `serde` rename: configs use `#[serde(rename_all = "kebab-case")]`; MCP uses `camelCase`. Mind the inconsistency when adding fields.

### Adding dependencies

Always add crates via `cargo add`, never by hand-editing `Cargo.toml`. This resolves the latest compatible version automatically and keeps `Cargo.lock` in sync:

```bash
cargo add <crate>                          # runtime dependency
cargo add <crate> --dev                    # dev-only dependency
cargo add <crate> --features feat1,feat2   # with specific features
```

`cargo add` writes the resolved version into `Cargo.toml`, so the entry you commit is already the actual latest rather than a stale guess.

### Comments and docstrings

- Functions, structs, traits, and impls get **one-liner `///` comments only** — just what the item does. No `@param`, `@returns`, `@raises`, or multi-line doc blocks.
- **No top-level file comment** — do not add a module-level `//!` or `///` block at the top of any file.
- Test functions get a **single-line comment** explaining what is being tested (e.g. `// returns error when path does not exist`). No verbose comment blocks or jargon.
