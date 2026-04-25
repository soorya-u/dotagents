## Context

Currently `Targets` and `Providers` in `src/schema/config/common.rs` each hold three named sub-fields (`ide`, `cli`, `custom`). At deploy time, `AppConfig::get_provider_feature_settings` chains all three into a single flat `HashMap<String, FeatureSettings>` — the grouping is immediately discarded. The same three-field pattern is duplicated in the merge logic, in `cache.rs`'s group-name match, and in the mock config files. The CI registry script (`scripts/ci/generate_registry.sh`) was written expecting templates to live under `public/v1/templates/cli/<provider>/` and `public/v1/templates/ide/<provider>/`, but templates have always lived flat at `public/v1/templates/<provider>/`, so the script produces an empty registry.

## Goals / Non-Goals

**Goals:**
- Replace `Targets { ide, cli, custom }` with a single flat `HashSet<String>` of provider names
- Replace `Providers { ide, cli, custom }` with a single flat `HashMap<String, Features>`
- Simplify all merge, validation, and iteration code that branches on the three groups
- Fix the registry generation script to match the actual flat template layout
- Rewrite mock config files to demonstrate the new shape
- Update all 14 `provider.toml` snippets in `public/v1/templates/`

**Non-Goals:**
- Backward compatibility or migration tooling (hard break, pre-1.0)
- Adding `kinds` metadata (e.g. `kinds = ["cli", "ide"]`) to `provider.toml` — the registry stays a flat list
- Changing the `Features` struct or any per-provider feature settings fields
- Template fetch caching or output caching (separate proposal)

## Decisions

### 1. Flat `targets` as a `Vec<String>` in TOML, `HashSet<String>` in Rust

**Decision**: `targets` in `config.toml` becomes a top-level array (`targets = ["claude", "cursor"]`). In the Rust struct this is `Option<HashSet<String>>` to stay consistent with the optional-everything pattern and to deduplicate entries.

**Alternatives considered**:
- Keep a `[targets]` table with a single `providers` key — adds an unnecessary level of nesting.
- Use `Vec<String>` in Rust — slightly simpler but doesn't prevent duplicate entries. `HashSet` is already used for the current `ide`/`cli`/`custom` fields.

### 2. Flat `providers` as a top-level map

**Decision**: `[providers.<name>.<feature>]` — `Providers` wraps `Option<HashMap<String, Features>>`. The serde shape is a flat TOML table, the same as today except one nesting level is removed.

**Alternatives considered**:
- Rename the wrapper struct to avoid confusion — not worth it; `Providers` is already the right word.

### 3. `Targets::merge` replaces entire set (not union)

**Decision**: Consistent with the current `ide`/`cli`/`custom` merge where each group fully overrides the base group. If `local.config.toml` sets `targets = []`, it disables all shared targets. This is intentional — local override means local override.

**Alternatives considered**:
- Union merge (base ∪ override) — would make it impossible to disable a team target locally, which conflicts with the stated personal-override use case.

### 4. Fix registry script by scanning flat `public/v1/templates/<provider>/`

**Decision**: Update `generate_registry.sh` to iterate `public/v1/templates/*/` directly, reading `provider.toml` from each. The JSON registry output lists providers as a flat array. No `cli/`/`ide/` subdirectory structure is introduced.

**Alternatives considered**:
- Introduce the `cli/`/`ide/` subdirectory layout as the script expected — this would re-introduce the same grouping problem at the registry level.

## Risks / Trade-offs

- **Breaking change for existing users** → Acceptable at v0.1.0; documented clearly in changelog and proposal.
- **All 14 `provider.toml` files need updating** → Mechanical find-and-replace; low risk but easy to miss one. The tasks list enumerates each provider explicitly.
- **Tests in `common.rs` reference `ide`/`cli`/`custom` field names** → Must be rewritten alongside the struct changes; if forgotten, compilation fails so the risk is caught immediately.
- **`cache.rs` match on group names becomes dead code** → Compiler will warn; addressed in the same task.

## Migration Plan

No migration. This is a hard breaking change. Users upgrade by replacing:

```toml
# Before
[targets]
ide = ["cursor"]
cli = ["claude", "gemini"]
custom = ["mycode"]

[providers.ide.cursor.commands]
...
[providers.cli.claude.commands]
...
[providers.custom.mycode.commands]
...

# After
targets = ["cursor", "claude", "gemini", "mycode"]

[providers.cursor.commands]
...
[providers.claude.commands]
...
[providers.mycode.commands]
...
```

## Open Questions

*(none — all decisions made during proposal brainstorm)*
