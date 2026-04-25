## 1. Core Schema Changes

- [x] 1.1 Replace `Targets { ide, cli, custom }` with `Targets { providers: Option<HashSet<String>> }` in `src/schema/config/common.rs`; update serde rename so `providers` serializes as `targets` in TOML
- [x] 1.2 Replace `Providers { ide, cli, custom }` with `Providers(Option<HashMap<String, Features>>)` (or a named-field equivalent) in `src/schema/config/common.rs`
- [x] 1.3 Rewrite `Targets::merge` to override-replace the flat `providers` set (local wins over global)
- [x] 1.4 Rewrite `Providers::merge` to use a single `merge_provider_maps` call on the flat map
- [x] 1.5 Rewrite tests in `common.rs` (`test_targets_merge`, `test_config_agent_settings_merge`, etc.) for the new struct shapes

## 2. Runtime Config Logic

- [x] 2.1 Simplify `AppConfig::get_provider_feature_settings` in `src/schema/config/app.rs` — replace the three-iterator chain with a single flat iterator over `providers`
- [x] 2.2 Remove `custom_providers` validation block in `src/schema/config/global.rs`
- [x] 2.3 Remove `custom_providers` validation block in `src/schema/config/local.rs`
- [x] 2.4 Remove `"ide"` / `"cli"` / `"custom"` match arms in `src/schema/config/cache.rs`; flatten the cache provider map accordingly

## 3. Mock Files

- [x] 3.1 Rewrite `src/mocks/config.toml` — replace `[targets] ide/cli` table with `targets = [...]` flat array; keep `windsurf`, `gemini` as example targets
- [x] 3.2 Rewrite `src/mocks/local.config.toml` — replace `[providers.custom.mycode.*]` keys with `[providers.mycode.*]`; update `targets` to flat list

## 4. Public Provider Templates

- [x] 4.1 Update `public/v1/templates/claude/provider.toml` snippet key from `[providers.cli.claude.*]` to `[providers.claude.*]`
- [x] 4.2 Update `public/v1/templates/codex/provider.toml`
- [x] 4.3 Update `public/v1/templates/cursor/provider.toml`
- [x] 4.4 Update `public/v1/templates/gemini/provider.toml`
- [x] 4.5 Update `public/v1/templates/copilot/provider.toml`
- [x] 4.6 Update `public/v1/templates/windsurf/provider.toml`
- [x] 4.7 Update `public/v1/templates/amp/provider.toml`
- [x] 4.8 Update `public/v1/templates/auggie/provider.toml`
- [x] 4.9 Update `public/v1/templates/codebuddy/provider.toml`
- [x] 4.10 Update `public/v1/templates/kilocode/provider.toml`
- [x] 4.11 Update `public/v1/templates/opencode/provider.toml`
- [x] 4.12 Update `public/v1/templates/qwen/provider.toml`
- [x] 4.13 Update `public/v1/templates/roo/provider.toml`
- [x] 4.14 Update `public/v1/templates/shai/provider.toml`

## 5. Registry Generation Script

- [x] 5.1 Update `scripts/ci/generate_registry.sh` to scan `public/v1/templates/*/provider.toml` (flat) instead of the non-existent `cli/`/`ide/` subdirectories
- [x] 5.2 Update `scripts/ci/detect_template_changes.sh` if it also references `cli/`/`ide/` path patterns
- [x] 5.3 Verify the updated script generates a valid `public/v1/templates/registry.json` locally

## 6. Verification

- [x] 6.1 Run `cargo build` — confirm no compilation errors
- [x] 6.2 Run `cargo test` — confirm all tests pass with the new struct shapes
- [x] 6.3 Run `cargo run -- init` and inspect the scaffolded `.dotagents-debug/config.toml` to confirm flat shape
- [x] 6.4 Run `cargo run -- deploy` from a directory with a valid flat config and confirm output files are written correctly
- [x] 6.5 Run `cargo fmt && cargo clippy` — no warnings
