## Context

`dotagents deploy` currently requires every `[providers.<name>.<feature>]` block in `config.toml` to have explicit `template` and `target` fields. The renderer hard-errors if either is absent. The CLI already knows how to fetch remote templates via HTTPS (`src/templates/remote.rs`) and the official provider templates are already hosted at `dotagents.soorya-u.dev/v1/templates/`. The gap is the auto-discovery and caching layer that connects those two facts.

There are two separate caches in this system and it is important they remain distinct:
- **Rendered-output cache** (`<workspace>/.dotagents/cache/cache.toml`) — already exists; tracks SHA-256 of each rendered file to skip unchanged writes.
- **Template-source cache** (`dirs::config_dir()/dotagents/cache/templates/`) — new; caches downloaded `.hbs` files and `provider.toml` across all projects on the machine.

## Goals / Non-Goals

**Goals:**
- Zero-config deploy for any provider listed in the official registry — user adds the provider name to `targets`, nothing more required.
- Template source files are downloaded at most once per content-version across all projects on the machine.
- `--offline` flag gives a fully deterministic, network-free deploy path once the cache is warm.
- Graceful degradation: a flaky network during online deploy produces warnings, not hard failures.

**Non-Goals:**
- Caching rendered output — that is handled by the existing `deploy-output-cache` mechanism.
- Supporting untrusted or self-hosted registries — the trusted domain is baked in at compile time.
- Auto-updating `config.toml` with resolved values — the resolved defaults are in-memory only.
- Windows support for the template cache path — the `dirs` crate handles this correctly; no special casing needed.

## Decisions

### D1 — Resolution hook lives in `deploy.rs`, not in `AppConfig` or `renderer.rs`

**Decision**: Add `resolve_provider_defaults(app_config, opts)` as an explicit step in `deploy.rs` between `AppConfig::from_application()` and `deploy_feature()`.

**Rationale**: Config building (`AppConfig`) should remain a pure TOML merge with no side effects. Lazy resolution in `renderer.rs` would scatter network calls across parallel rayon threads and make it harder to fetch `registry.json` exactly once. Placing the hook in `deploy.rs` keeps it explicit, serial for the registry fetch, and easy to short-circuit with `--offline`.

**Alternative considered**: Extend `AppConfig::from_application()` to accept a registry client and fill in defaults during build. Rejected because it couples config parsing to network I/O, making it harder to test and violates the single-responsibility of config loading.

### D2 — Registry.json is fetched once per deploy, not per provider

**Decision**: Fetch and deserialise `registry.json` once at the start of `resolve_provider_defaults()`, then look up each provider in memory.

**Rationale**: Avoids N redundant HTTP round-trips for N providers with missing config. Registry is small (< 10 KB) and cheap to parse.

### D3 — Template-source cache uses filename + `.sha256` sidecar files

**Decision**: Store each cached file as `<provider>/<file>` with a sibling `<provider>/<file>.sha256` containing the hex digest.

**Rationale**: Simple to implement, survives partial writes (if the `.sha256` file is missing the entry is treated as a cache miss), and requires no additional index file. The alternative of a single `cache-index.toml` was considered but adds a locking requirement when projects run concurrent deploys.

### D4 — Network failure in online mode is a soft failure for registry lookup only

**Decision**: If fetching `registry.json` fails (any error), log a warning and fall back to the template-source cache. If the cached `provider.toml` is also absent, emit a second warning and skip that provider/feature — deploy continues for other providers.

**Rationale**: Users may run deploy in constrained network environments without intending `--offline` mode. A hard failure here would break deploys for all providers even if the templates are already cached. The explicit `--offline` flag is reserved for intentionally network-free environments and errors clearly when the cache is cold.

**Important**: This soft-failure applies only to the registry/provider-discovery path. An explicit `template = "https://..."` URL in `config.toml` that fails to fetch remains a hard error (existing `remote-template-fetch` behaviour is unchanged).

### D5 — `provider.toml` is deserialised as a `GlobalConfig` subset

**Decision**: The downloaded `provider.toml` is deserialised into the existing `GlobalConfig` struct (the same TOML schema used by the user's own `config.toml`). The `providers.<name>.<feature>.template` and `providers.<name>.<feature>.target` values are extracted from the parsed struct.

**Rationale**: No new struct is required; the format is already defined. This also means any future field additions to `FeatureSettings` are automatically available from `provider.toml` without touching the parser.

### D6 — `dirs` crate for cross-platform config path

**Decision**: Use `dirs::config_dir()` to locate the user-level config directory.

**Rationale**: The existing `get_config_dir()` in `path.rs` is already marked `// TODO: Valid only for Unix`. `dirs::config_dir()` returns the correct platform-native path (Linux: `~/.config`, macOS: `~/Library/Application Support`, Windows: `%APPDATA%`) without custom platform detection.

## Risks / Trade-offs

**[Risk] First-deploy latency for users with no warm cache** → On a cold cache, deploy fetches `registry.json` + `provider.toml` + each `.hbs` file for every enabled provider. For a user with 5 providers and 4 features each that is up to 41 small HTTPS requests. These are serialised per-provider during resolution but the rendered-output deploy is still parallelised. Mitigation: keep providers to a minimum in CI; subsequent deploys are fully cached.

**[Risk] Stale cache after registry outage** → If the server is down and all providers are cache-warm, deploy succeeds silently with potentially old templates. Mitigation: the warning log makes the fallback visible; users can force a refresh with `--no-cache` (which should also clear the template-source cache, see Open Questions).

**[Risk] Template-source cache grows unbounded** → Old `.hbs` files are not pruned when a provider is removed from the user's config. Mitigation: the cache is small (template files are < 5 KB each); a `dotagents cache clean` command is out of scope for this change.

**[Trade-off] Per-field resolution adds complexity to the merge** → Filling in only the missing field (e.g., `target` is present but `template` is absent) requires partial merging of the resolved `FeatureSettings` with the user-configured one. This is handled by the existing `FeatureSettings::merge()` method: treat the user config as the override layer and the registry-resolved defaults as the base.

## Migration Plan

1. Ship `generate_registry.sh` changes and the new `registry.schema.json` first; existing clients ignore the new `checksums` field (it is optional in the schema).
2. Ship the Rust changes in the same release. Clients without the feature continue to use explicit `template`/`target` fields; the resolution step is only triggered for absent fields.
3. No rollback complexity — the feature is purely additive. Removing it restores the previous hard-error behaviour.

## Open Questions

- **Should `--no-cache` also clear/bypass the template-source cache?** Current thinking: yes — `--no-cache` should mean "fetch everything fresh", bypassing both caches. This keeps the flag semantics consistent. To be confirmed before implementation of `resolve_provider_defaults()`.
- **Should `dotagents init` pre-warm the template cache for the providers in the scaffolded config?** Out of scope for now but worth a follow-up change.
