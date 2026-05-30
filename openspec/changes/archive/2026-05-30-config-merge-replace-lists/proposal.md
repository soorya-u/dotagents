## Why

Dotagents currently silently drops unknown keys when parsing `config.toml` and `local.config.toml` (no `#[serde(flatten)]` catch-all). Users who add custom sections or metadata to their config files lose them on any round-trip. Additionally, list/array merge semantics should be explicitly "whole-list replace" — no element-wise merging — to keep behavior predictable and simple.

## What Changes

- **Preserve unknown top-level keys**: Config structs capture unrecognized keys via a `#[serde(flatten)] extra: HashMap<String, Value>` field, so dotagents only touches the keys it knows about and leaves everything else intact on read/write.
- **Replace whole lists**: All list-typed fields (`features`, `targets`, and any future `Vec` fields) use whole-list replacement during config layering — local completely overrides global. No union, no append, no element-wise merge. This matches current behavior but must be explicitly documented and enforced.
- **JSON merge preserves non-dotagents keys**: `merge_json_mut` already replaces arrays entirely (correct). Object keys not managed by dotagents are already preserved in recursive merge (correct). Verify and document this contract.

## Capabilities

### New Capabilities
- `config-preserve-unknown-keys`: Capture and round-trip unrecognized TOML keys in config files so dotagents only manages its own top-level keys.

### Modified Capabilities
- `deploy-pipeline`: Document and enforce that list merge semantics are whole-list replace, not element-wise.

## Impact

- **Config structs** (`src/core/config/global.rs`, `local.rs`, `app.rs`): Add `extra` field with `#[serde(flatten)]`.
- **Merge logic** (`src/core/config/app.rs`): Preserve `extra` keys from both global and local during merge.
- **Serialization** (`TomlConfig::to_toml`): Ensure extra keys are written back on round-trip.
- **Tests**: Integration tests for unknown key preservation, list replace semantics.
- **No breaking changes**: Existing configs without unknown keys behave identically.
