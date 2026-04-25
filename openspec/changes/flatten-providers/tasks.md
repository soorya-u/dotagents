## 1. Core Schema Changes

- [ ] 1.1 Replace `Targets { ide, cli, custom }` with `Targets { providers: Option<HashSet<String>> }` in `src/schema/config/common.rs`; update serde rename so `providers` serializes as `targets` in TOML
- [ ] 1.2 Replace `Providers { ide, cli, custom }` with `Providers(Option<HashMap<String, Features>>)` (or a named-field equivalent) in `src/schema/config/common.rs`
- [ ] 1.3 Rewrite `Targets::merge` to override-replace the flat `providers` set (local wins over global)
- [ ] 1.4 Rewrite `Providers::merge` to use a single `merge_provider_maps` call on the flat map
- [ ] 1.5 Rewrite tests in `common.rs` (`test_targets_merge`, `test_config_agent_settings_merge`, etc.) for the new struct shapes

## 2. Runtime Config Logic

- [ ] 2.1 Simplify `AppConfig::get_provider_feature_settings` in `src/schema/config/app.rs` — replace the three-iterator chain with a single flat iterator over `providers`
- [ ] 2.2 Remove `custom_providers` validation block in `src/schema/config/global.rs`
- [ ] 2.3 Remove `custom_providers` validation block in `src/schema/config/local.rs`
- [ ] 2.4 Remove `"ide"` / `"cli"` / `"custom"` match arms in `src/schema/config/cache.rs`; flatten the cache provider map accordingly

## 3. Mock Files

- [ ] 3.1 Rewrite `src/mocks/config.toml` — replace `[targets] ide/cli` table with `targets = [...]` flat array; keep `windsurf`, `gemini` as example targets
- [ ] 3.2 Rewrite `src/mocks/local.config.toml` — replace `[providers.custom.mycode.*]` keys with `[providers.mycode.*]`; update `targets` to flat list

## 4. Public Provider Templates

- [ ] 4.1 Update `public/v1/templates/claude/provider.toml` snippet key from `[providers.cli.claude.*]` to `[providers.claude.*]`
- [ ] 4.2 Update `public/v1/templates/codex/provider.toml`
- [ ] 4.3 Update `public/v1/templates/cursor/provider.toml`
- [ ] 4.4 Update `public/v1/templates/gemini/provider.toml`
- [ ] 4.5 Update `public/v1/templates/copilot/provider.toml`
- [ ] 4.6 Update `public/v1/templates/windsurf/provider.toml`
- [ ] 4.7 Update `public/v1/templates/amp/provider.toml`
- [ ] 4.8 Update `public/v1/templates/auggie/provider.toml`
- [ ] 4.9 Update `public/v1/templates/codebuddy/provider.toml`
- [ ] 4.10 Update `public/v1/templates/kilocode/provider.toml`
- [ ] 4.11 Update `public/v1/templates/opencode/provider.toml`
- [ ] 4.12 Update `public/v1/templates/qwen/provider.toml`
- [ ] 4.13 Update `public/v1/templates/roo/provider.toml`
- [ ] 4.14 Update `public/v1/templates/shai/provider.toml`

## 5. Registry Generation Script

- [ ] 5.1 Update `scripts/ci/generate_registry.sh` to scan `public/v1/templates/*/provider.toml` (flat) instead of the non-existent `cli/`/`ide/` subdirectories
- [ ] 5.2 Update `scripts/ci/detect_template_changes.sh` if it also references `cli/`/`ide/` path patterns
- [ ] 5.3 Verify the updated script generates a valid `public/v1/templates/registry.json` locally

## 6. Verification

- [ ] 6.1 Run `cargo build` — confirm no compilation errors
- [ ] 6.2 Run `cargo test` — confirm all tests pass with the new struct shapes
- [ ] 6.3 Run `cargo run -- init` and inspect the scaffolded `.dotagents-debug/config.toml` to confirm flat shape
- [ ] 6.4 Run `cargo run -- deploy` from a directory with a valid flat config and confirm output files are written correctly
- [ ] 6.5 Run `cargo fmt && cargo clippy` — no warnings
