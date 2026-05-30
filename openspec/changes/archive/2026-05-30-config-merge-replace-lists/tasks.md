## 1. Add `extra` field to config structs

- [x] 1.1 Add `extra: HashMap<String, toml::Value>` with `#[serde(flatten)]` to `GlobalConfig` in `src/core/config/global.rs`
- [x] 1.2 Add `extra: HashMap<String, toml::Value>` with `#[serde(flatten)]` to `LocalConfig` in `src/core/config/local.rs`
- [x] 1.3 Add `extra: HashMap<String, toml::Value>` to `AppConfig` in `src/core/config/app.rs`
- [x] 1.4 Merge `extra` maps (shallow union, local wins) in `AppConfig::from((&GlobalConfig, &LocalConfig))` in `src/core/config/app.rs`

## 2. Unit tests for unknown key preservation

- [x] 2.1 Add unit test: unknown string key survives parse/serialize round-trip in `GlobalConfig`
- [x] 2.2 Add unit test: unknown table survives parse/serialize round-trip in `GlobalConfig`
- [x] 2.3 Add unit test: unknown array survives parse/serialize round-trip in `GlobalConfig`
- [x] 2.4 Add unit test: unknown key only in global is preserved in merged `AppConfig`
- [x] 2.5 Add unit test: unknown key only in local is preserved in merged `AppConfig`
- [x] 2.6 Add unit test: unknown key in both global and local uses local value

## 3. Unit tests for list replacement semantics

- [x] 3.1 Add unit test: local `features` completely replaces global `features` (no union)
- [x] 3.2 Add unit test: local `targets` completely replaces global `targets` (no union)
- [x] 3.3 Add unit test: omitted local list field falls back to global value

## 4. Verification

- [x] 4.1 Run `mise check` (cargo fmt + cargo clippy) and fix any failures
- [x] 4.2 Run `mise tests` (cargo test) and fix any failures
