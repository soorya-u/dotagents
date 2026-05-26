## 1. Core Feature Implementation

- [ ] 1.1 Add `Ignore` variant to `Feature` enum in `src/core/features/common.rs`
- [ ] 1.2 Create `IgnoreFeature` struct in `src/core/features/ignore.rs` implementing `FeatureTrait`
- [ ] 1.3 Add `ignore` module export in `src/core/features.rs`
- [ ] 1.4 Add `ignore: Option<FeatureSettings>` to `Features` struct in `src/core/config/common.rs`
- [ ] 1.5 Update `Features::get_config()` to handle `Feature::Ignore`
- [ ] 1.6 Update `Features::merge()` to include ignore field
- [ ] 1.7 Add `[ignore]` config parsing with `patterns` field

## 2. Deploy Pipeline Integration

- [ ] 2.1 Wire ignore feature into deploy pipeline in `src/cli/deploy.rs`
- [ ] 2.2 Implement single-phase rendering for ignore feature in `src/templates/renderer.rs`
- [ ] 2.3 Add ignore file paths to gitignore fence tracking in `src/utils/gitignore.rs`
- [ ] 2.4 Update `AppConfig::has_feature()` to recognize "ignore"

## 3. Provider Templates

- [ ] 3.1 Create `public/v1/templates/opencode/ignore.hbs` with target `.ignore`
- [ ] 3.2 Create `public/v1/templates/auggie/ignore.hbs` with target `.augmentignore`
- [ ] 3.3 Create `public/v1/templates/autohand/ignore.hbs` with target `.autohandignore`
- [ ] 3.4 Create `public/v1/templates/junie/ignore.hbs` with target `.aiignore`
- [ ] 3.5 Create `public/v1/templates/pi/ignore.hbs` with target `.piignore`
- [ ] 3.6 Create `public/v1/templates/goose/ignore.hbs` with target `.gooseignore`
- [ ] 3.7 Create `public/v1/templates/cline/ignore.hbs` with target `.clineignore`
- [ ] 3.8 Create `public/v1/templates/gemini/ignore.hbs` with target `.geminiignore`
- [ ] 3.9 Create `public/v1/templates/qwen/ignore.hbs` with target `.qwenignore`
- [ ] 3.10 Create `public/v1/templates/kilocode/ignore.hbs` with target `.kilocodeignore`
- [ ] 3.11 Create `public/v1/templates/cursor/ignore.hbs` with target `.cursorignore`
- [ ] 3.12 Create `public/v1/templates/claude/ignore.hbs` with target `.claudeignore`
- [ ] 3.13 Create `public/v1/templates/copilot/ignore.hbs` with target `.github/copilotignore`
- [ ] 3.14 Create `public/v1/templates/codex/ignore.hbs` with target `.codexignore`
- [ ] 3.15 Create `public/v1/templates/factory-droid/ignore.hbs` with target `.factoryignore`
- [ ] 3.16 Create `public/v1/templates/deepagents/ignore.hbs` with target `.deepagentsignore`
- [ ] 3.17 Create `public/v1/templates/kimi/ignore.hbs` with target `.kimiignore`
- [ ] 3.18 Create `public/v1/templates/mistral-vibe/ignore.hbs` with target `.mistralignore`
- [ ] 3.19 Create `public/v1/templates/qoder-cli/ignore.hbs` with target `.qoderignore`
- [ ] 3.20 Create `public/v1/templates/amp/ignore.hbs` with target `.ampignore`
- [ ] 3.21 Update `provider.toml` for each provider with `[providers.<slug>.ignore]` section

## 4. Init Integration

- [ ] 4.1 Add `Ignore` variant to `Feature` enum in `src/cli/options.rs`
- [ ] 4.2 Add `Feature::Ignore` → `"ignore"` mapping in `Feature::as_str()`
- [ ] 4.3 Add "Ignore Patterns" option to TUI init wizard in `src/cli/ui/init.rs` multiselect
- [ ] 4.4 Add `--no-ignore` flag support in `InitOptions` (parallel to `--no-mcp`, `--no-command`, etc.)
- [ ] 4.5 Create default ignore patterns mock file in `src/constants/mocks.rs`
- [ ] 4.6 Wire ignore file scaffolding into `initialize_agents_dir()` in `src/cli/init.rs` with skip condition
- [ ] 4.7 Add `IGNORE_FILE` constant in `src/constants/file.rs`

## 5. Testing

- [ ] 5.1 Add unit tests for `IgnoreFeature` in `src/core/features/ignore.rs`
- [ ] 5.2 Add unit tests for config parsing and merging in `src/core/config/common.rs`
- [ ] 5.3 Add unit tests for init skip conditions with ignore feature
- [ ] 5.4 Manually test deploy with tui-devtools for all 20 providers
- [ ] 5.5 Manually test init wizard with ignore feature selected/deselected
- [ ] 5.6 Add e2e tests for ignore feature deploy flow
- [ ] 5.7 Add e2e tests for init with `--no-ignore` flag

## 6. Verification

- [ ] 6.1 Run `mise check` (cargo fmt + clippy) — must exit 0
- [ ] 6.2 Run `mise tests` — must exit 0
