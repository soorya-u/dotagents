## Why

Many AI coding agents support ignore files that control which files the agent reads or indexes. Currently, dotagents deploys commands, instructions, MCP configs, and skills to 20+ providers, but does not manage their ignore files. Users must manually create and maintain `.ignore`, `.aiignore`, `.claudeignore`, etc. Adding ignore file support lets dotagents automatically generate and update these files during `deploy`, and scaffold them during `init`, keeping them in sync with deployed content and preventing agents from reading stale or irrelevant files.

## What Changes

- New `ignore` feature added to the feature system (alongside `commands`, `instructions`, `mcp`, `skills`)
- New `Ignore` option in `dotagents init` feature selection (TUI wizard + `--features` flag + `--no-ignore` flag)
- Provider templates updated for all 20 providers that support ignore files:
  - **opencode**: `.ignore`
  - **auggie**: `.augmentignore`
  - **autohand**: `.autohandignore`
  - **junie**: `.aiignore`
  - **pi**: `.piignore`
  - **goose**: `.gooseignore`
  - **cline**: `.clineignore`
  - **gemini**: `.geminiignore`
  - **qwen**: `.qwenignore`
  - **kilocode**: `.kilocodeignore`
  - **cursor**: `.cursorignore`
  - **claude**: `.claudeignore`
  - **copilot**: `.github/copilotignore`
  - **codex**: `.codexignore`
  - **factory-droid**: `.factoryignore`
  - **deepagents**: `.deepagentsignore`
  - **kimi**: `.kimiignore`
  - **mistral-vibe**: `.mistralignore`
  - **qoder-cli**: `.qoderignore`
  - **amp**: `.ampignore`
- New `IgnoreFeature` type implementing `FeatureTrait` to handle ignore pattern lists
- Deploy pipeline renders ignore templates per-provider and writes to target paths
- Init scaffolds a default ignore patterns file when the ignore feature is selected
- Existing gitignore fence logic extended to include deployed ignore files

## Capabilities

### New Capabilities
- `ignore-feature`: New feature type for managing provider-specific ignore files. Handles pattern lists, template rendering, file output, and init scaffolding.
- `ignore-provider-templates`: Template definitions for all 20 providers' ignore file formats.
- `ignore-init-scaffold`: Init-time scaffolding of ignore patterns file when the ignore feature is selected.

### Modified Capabilities
- `deploy-pipeline`: Deploy now processes the `ignore` feature alongside existing features. Cache and gitignore logic extended to track ignore files.
- `init-wizard`: TUI wizard and CLI flags extended to include the ignore feature selection.

## Impact

- **New code**: `src/core/features/ignore.rs` (new `IgnoreFeature` implementation), provider template files under `public/v1/templates/<provider>/ignore.hbs`
- **Modified code**: `src/core/features/common.rs` (add `Ignore` to `Feature` enum), `src/core/config/common.rs` (add `ignore` to `Features`), `src/cli/options.rs` (add `Ignore` to init `Feature` enum), `src/cli/init.rs` (wire ignore scaffolding), `src/cli/ui/init.rs` (add ignore to TUI multiselect), `src/cli/deploy.rs` (wire ignore feature into deploy loop), `src/utils/gitignore.rs` (include ignore files in fence)
- **Config schema**: `Features` struct gains `ignore: Option<FeatureSettings>`
- **Templates**: 20 new `.hbs` template files for provider-specific ignore formats
- **Registry**: `registry.json` updated with new template checksums
