## 1. Core Feature Implementation

- [x] 1.1 Add `Ignore` variant to `Feature` enum in `src/core/features/common.rs`
- [x] 1.2 Create `IgnoreFeature` struct in `src/core/features/ignore.rs` implementing `FeatureTrait`
- [x] 1.3 Add `ignore` module export in `src/core/features.rs`
- [x] 1.4 Add `ignore: Option<FeatureSettings>` to `Features` struct in `src/core/config/common.rs`
- [x] 1.5 Update `Features::get_config()` to handle `Feature::Ignore`
- [x] 1.6 Update `Features::merge()` to include ignore field
- [x] 1.7 Add `.agentignore` file loading (newline-separated patterns from `.dotagents/.agentignore`)
- [x] 1.8 Add per-provider `[providers.<name>.ignore].disabled` config parsing

## 2. Deploy Pipeline Integration

- [x] 2.1 Wire ignore feature into deploy pipeline in `src/cli/deploy.rs`
- [x] 2.2 Implement single-phase rendering for ignore feature in `src/templates/renderer.rs`
- [x] 2.3 Add ignore file paths to gitignore fence tracking in `src/utils/gitignore.rs`
- [x] 2.4 Update `AppConfig::has_feature()` to recognize "ignore"

## 3. Provider Templates

Providers with valid ignore file support (ignore.hbs created):

- [x] 3.1 Create `public/v1/templates/opencode/ignore.hbs` with target `.ignore`
- [x] 3.2 Create `public/v1/templates/auggie/ignore.hbs` with target `.augmentignore`
- [x] 3.3 Create `public/v1/templates/autohand/ignore.hbs` with target `.autohandignore`
- [x] 3.4 Create `public/v1/templates/junie/ignore.hbs` with target `.aiignore`
- [x] 3.5 Create `public/v1/templates/pi/ignore.hbs` with target `.piignore`
- [x] 3.6 Create `public/v1/templates/goose/ignore.hbs` with target `.gooseignore`
- [x] 3.7 Create `public/v1/templates/cline/ignore.hbs` with target `.clineignore`
- [x] 3.8 Create `public/v1/templates/gemini/ignore.hbs` with target `.geminiignore`
- [x] 3.9 Create `public/v1/templates/qwen/ignore.hbs` with target `.qwenignore`
- [x] 3.10 Create `public/v1/templates/kilocode/ignore.hbs` with target `.kilocodeignore`
- [x] 3.11 Create `public/v1/templates/cursor/ignore.hbs` with target `.cursorignore`

Providers without ignore file support (no valid ignore file format — do not add):

- ~~3.12 Create `public/v1/templates/claude/ignore.hbs`~~ — `.claudeignore` not a valid file
- ~~3.13 Create `public/v1/templates/copilot/ignore.hbs`~~ — `.github/copilotignore` not a valid file
- ~~3.14 Create `public/v1/templates/codex/ignore.hbs`~~ — `.codexignore` not a valid file
- ~~3.15 Create `public/v1/templates/factory-droid/ignore.hbs`~~ — `.factoryignore` not a valid file
- ~~3.16 Create `public/v1/templates/deepagents/ignore.hbs`~~ — `.deepagentsignore` not a valid file
- ~~3.17 Create `public/v1/templates/kimi/ignore.hbs`~~ — `.kimiignore` not a valid file
- ~~3.18 Create `public/v1/templates/mistral-vibe/ignore.hbs`~~ — `.mistralignore` not a valid file
- ~~3.19 Create `public/v1/templates/qoder-cli/ignore.hbs`~~ — `.qoderignore` not a valid file
- ~~3.20 Create `public/v1/templates/amp/ignore.hbs`~~ — `.ampignore` not a valid file

- [x] 3.21 Update `provider.toml` for each provider with `[providers.<slug>.ignore]` section (11 providers only)
- [x] 3.22 Update provider registry checksums in `registry.json` for all new `ignore.hbs` files

## 4. Init Integration

- [x] 4.1 Add `AgentIgnore` variant to `Feature` enum in `src/cli/options.rs`
- [x] 4.2 Add `Feature::AgentIgnore` → `"agent-ignore"` mapping (kebab-case via strum)
- [x] 4.3 Add ".agentignore" option to TUI init wizard in `src/cli/ui/init.rs` multiselect
- [x] 4.4 Create default `.agentignore` mock file in `src/constants/mocks.rs`
- [x] 4.5 Wire `.agentignore` scaffolding into `initialize_agents_dir()` in `src/cli/init.rs`
- [x] 4.6 Add `AGENTIGNORE_FILE` constant in `src/constants/file.rs`

## 5. Testing

- [x] 5.1 Add unit tests for `IgnoreFeature` in `src/core/features/ignore.rs`
- [x] 5.2 Add unit tests for config parsing and merging in `src/core/config/common.rs`
- [x] 5.3 Add unit tests for init with ignore feature selected/deselected
- [x] 5.4 Manually test deploy with tui-devtools for all supported providers
- [x] 5.5 Manually test init wizard with ignore feature selected/deselected
- [x] 5.6 Add e2e tests for ignore feature deploy flow
- [x] 5.7 Add e2e tests for init with `--features agent-ignore` flag

## 6. Verification

- [x] 6.1 Run `mise check` (cargo fmt + clippy) — must exit 0
- [x] 6.2 Run `mise tests` — must exit 0
