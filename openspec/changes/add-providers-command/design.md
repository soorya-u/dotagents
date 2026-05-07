## Context

The official registry at `public/v1/templates/registry.json` lists 24 providers with `path` and `checksums`. Users only see these through the init wizard multiselect. After init, there is no way to browse available providers — the registry is consumed silently by deploy.

## Goals / Non-Goals

**Goals:**
- Provide a `dotagents providers ls` command to list registered providers
- Extend registry entries with `name` (display label) and `url` (docs link) fields
- Support CLI mode (plain list, `--url`, `--json`) and TUI mode (fuzzy-search browser)
- Offline mode reads from template-source cache

**Non-Goals:**
- Adding/removing registries (this is a future concern)
- Live editing of registry entries
- Provider detail pages beyond name/url/slug

## Decisions

### Registry schema: additive extension with optional fields
`ProviderEntry` gains `name: Option<String>` and `url: Option<String>`. This keeps backward compatibility — existing registries without these fields still parse correctly. The CI registry generation script reads `name` and `url` from each provider's `provider.toml` and writes them into `registry.json`.

### Providers command is read-only
No mutation of config, registry, or deployed files. This mirrors the read-only nature of `--json`/`--full` flags established in the `add-json-full-flags` proposal. The command fetches the registry (or reads from cache) and displays it.

### TUI fuzzy search via cliclack
Cliclack's `Select` prompt with `filter` enabled provides fuzzy search out of the box. No new dependency needed. Each option shows `{name} [{slug}]` with optional URL on a second line when `--url` is active.

### Offline mode reuses template-source cache
In offline mode (`--offline`), the command reads `registry.json` from the template-source cache (the same cache used by deploy for provider resolution). If the cache is cold, it errors with instructions to run without `--offline` to populate it.

### CLI output format consistency
- Default: `slug  (name)` per line
- `--url`: `slug  (name) — https://...`
- `--json`: array of `{ "slug", "name", "url" }` objects

## Risks / Trade-offs

- **Registry changes need script updates**: The CI script must be updated to extract and write `name`/`url` from `provider.toml` files. → Keep fields optional so a missing `name`/`url` in `provider.toml` just omits them from registry output.
- **Cache dependency**: Offline mode requires prior online fetch. → Error message clearly instructs user to run online first.
