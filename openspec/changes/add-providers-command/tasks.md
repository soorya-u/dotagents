## 1. Registry schema extension

- [x] 1.1 Add `name: Option<String>` and `url: Option<String>` fields to `ProviderEntry` in `src/schema/registry.rs`
- [x] 1.2 Update `Registry` and `ProviderEntry` serde derives for new fields (optional = skip if absent)
- [x] 1.3 Add `name` and `url` fields to each provider's `public/v1/templates/<provider>/provider.toml`
- [x] 1.4 Update `scripts/ci/generate_registry.sh` to extract and include `name`/`url` from provider.toml into the generated `registry.json`

## 2. CLI plumbing

- [x] 2.1 Add `Providers` variant to `Action` enum and `ProvidersAction` enum in `src/cli/options.rs`
- [x] 2.2 Create `src/cli/providers.rs` module with `handle` function
- [x] 2.3 Wire `Action::Providers` dispatch in `src/cli/runner.rs`
- [x] 2.4 Add `--url`, `--json`, `--offline` flags to the `providers ls` subcommand

## 3. Core implementation

- [x] 3.1 Implement registry fetch/cache-read logic for providers command (reuse existing `Registry::fetch()` pattern)
- [x] 3.2 Implement CLI text output: default, `--url`, and `--json` modes
- [x] 3.3 Implement `--offline` mode with template-source cache fallback
- [x] 3.4 Implement TUI mode with cliclack fuzzy search (filtered Select)
- [x] 3.5 Implement TUI provider detail view on selection (slug, name, URL)

## 4. Unit tests

- [x] 4.1 Test `ProviderEntry` deserialization with and without new `name`/`url` fields
- [x] 4.2 Test registry parsing with mixed entries (some with name/url, some without)
- [x] 4.3 Test CLI output formatting (default, `--url`, `--json`)
- [x] 4.4 Test offline mode cache-hit and cache-miss paths

## 5. E2E tests

- [x] 5.1 tui-devtools discovery pass for `providers ls` TUI flow
- [x] 5.2 Add e2e test: `providers ls` CLI default output
- [x] 5.3 Add e2e test: `providers ls --url` output (N/A — `--url` flag removed; URL always shown)
- [x] 5.4 Add e2e test: `providers ls --json` valid JSON output
- [x] 5.5 Add e2e test: `providers ls --offline` with cold cache errors
- [x] 5.6 Add e2e test: `providers ls` TUI mode (initial render, navigation, Enter submission)

## 6. Verification

- [x] 6.1 Run `mise check` and fix any format/lint issues
- [x] 6.2 Run `mise tests` and fix any failures
