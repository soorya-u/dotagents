## 1. Registry schema extension

- [ ] 1.1 Add `name: Option<String>` and `url: Option<String>` fields to `ProviderEntry` in `src/schema/registry.rs`
- [ ] 1.2 Update `Registry` and `ProviderEntry` serde derives for new fields (optional = skip if absent)
- [ ] 1.3 Add `name` and `url` to `public/v1/templates/registry.json` for each provider entry
- [ ] 1.4 Add `name` and `url` fields to each provider's `public/v1/templates/<provider>/provider.toml`
- [ ] 1.5 Update `scripts/ci/generate_registry.sh` to extract and include `name`/`url` from provider.toml

## 2. CLI plumbing

- [ ] 2.1 Add `Providers` variant to `Action` enum and `ProvidersAction` enum in `src/cli/options.rs`
- [ ] 2.2 Create `src/cli/providers.rs` module with `handle` function
- [ ] 2.3 Wire `Action::Providers` dispatch in `src/cli/runner.rs`
- [ ] 2.4 Add `--url`, `--json`, `--offline` flags to the `providers ls` subcommand

## 3. Core implementation

- [ ] 3.1 Implement registry fetch/cache-read logic for providers command (reuse existing `Registry::fetch()` pattern)
- [ ] 3.2 Implement CLI text output: default, `--url`, and `--json` modes
- [ ] 3.3 Implement `--offline` mode with template-source cache fallback
- [ ] 3.4 Implement TUI mode with cliclack fuzzy search (filtered Select)
- [ ] 3.5 Implement TUI provider detail view on selection (slug, name, URL)

## 4. Unit tests

- [ ] 4.1 Test `ProviderEntry` deserialization with and without new `name`/`url` fields
- [ ] 4.2 Test registry parsing with mixed entries (some with name/url, some without)
- [ ] 4.3 Test CLI output formatting (default, `--url`, `--json`)
- [ ] 4.4 Test offline mode cache-hit and cache-miss paths

## 5. E2E tests

- [ ] 5.1 tui-devtools discovery pass for `providers ls` TUI flow
- [ ] 5.2 Add e2e test: `providers ls` CLI default output
- [ ] 5.3 Add e2e test: `providers ls --url` output
- [ ] 5.4 Add e2e test: `providers ls --json` valid JSON output
- [ ] 5.5 Add e2e test: `providers ls --offline` with cold cache errors
- [ ] 5.6 Add e2e test: `providers ls` TUI mode (fuzzy search and detail view)

## 6. Verification

- [ ] 6.1 Run `mise check` and fix any format/lint issues
- [ ] 6.2 Run `mise tests` and fix any failures
