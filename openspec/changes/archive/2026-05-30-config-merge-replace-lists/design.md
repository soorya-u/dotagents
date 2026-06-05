## Context

Dotagents config files (`config.toml`, `local.config.toml`) are parsed into strongly-typed structs via serde + TOML. Currently, any key not declared in the struct is silently dropped during deserialization. This means:
- Users cannot add metadata, comments-as-data, or tool-specific sections to their config files.
- Round-tripping a config file through parse → serialize loses unknown keys.
- The `variables` map is the only open-ended container, and it only holds flat `String -> String`.

List-typed fields (`features: HashSet<String>`, `targets: HashSet<String>`) already use whole-list replacement during config layering (local completely overrides global). This is correct but undocumented.

## Goals / Non-Goals

**Goals:**
- Preserve unknown top-level keys in `config.toml` and `local.config.toml` through parse/serialize round-trips.
- Merge unknown keys during config layering (global + local) using shallow union (local wins on collision).
- Document and enforce whole-list replacement semantics for all list/set fields.
- Keep the change backward-compatible: existing configs without unknown keys behave identically.

**Non-Goals:**
- Preserving unknown keys inside nested structs (`FeatureSettings`, `Features`, `Providers` inner maps). Only top-level config structs get the `extra` field.
- Changing the JSON merge behavior in `merge_json_mut` (already correct: arrays replaced, objects merged).
- Adding a schema validation or warning system for unknown keys.

## Decisions

### 1. Use `#[serde(flatten)]` with `HashMap<String, toml::Value>`

Add an `extra` field to `GlobalConfig` and `LocalConfig`:

```rust
#[serde(flatten)]
extra: HashMap<String, toml::Value>,
```

**Rationale:** `serde(flatten)` captures all unrecognized keys during deserialization and re-emits them during serialization. `toml::Value` preserves the full TOML type (string, integer, array, table, etc.) without lossy conversion through `serde_json::Value`.

**Alternative considered:** Using `serde_json::Value` — rejected because TOML has types (datetime, integer width) that don't round-trip cleanly through JSON.

**Alternative considered:** Using `toml::Table` — rejected because `flatten` requires a map type, and `HashMap<String, toml::Value>` is more flexible (allows non-table values at the top level).

### 2. Merge `extra` with shallow union (local wins)

During `AppConfig::from((&GlobalConfig, &LocalConfig))`, merge `extra` maps:

```rust
let mut merged = global.extra.clone();
merged.extend(local.extra.clone());
```

**Rationale:** Consistent with how `variables` is merged. Simple, predictable. Local overrides global on key collision.

### 3. `AppConfig` carries `extra` for round-trip

`AppConfig` gets the same `extra` field so that a loaded-then-saved config preserves unknown keys. The `extra` field is not used for any runtime logic — it's purely pass-through.

### 4. List replacement is the documented default

No code change needed for `features` and `targets` — they already use whole-list replacement. Add unit tests that explicitly assert this behavior and document it in the spec.

## Risks / Trade-offs

- **`serde(flatten)` + `deny_unknown_fields` conflict:** If `deny_unknown_fields` is ever added to a config struct, `flatten` will break. Mitigation: document that these structs must never use `deny_unknown_fields`.
- **`toml::Value` serialization edge cases:** Some TOML values (datetime, inline tables) may serialize differently than the user's original formatting. Mitigation: acceptable trade-off; the semantic content is preserved even if formatting differs.
- **Increased struct size:** `HashMap<String, toml::Value>` adds heap allocation. Mitigation: negligible for config files which are small and parsed once.
