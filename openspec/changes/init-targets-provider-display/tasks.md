## 1. Remove Feature::None and simplify validation

- [ ] 1.1 Remove `Feature::None` variant from the `Feature` enum in `src/cli/options.rs`
- [ ] 1.2 Change `has_feature()` to return `false` when `self.features` is `None` (was: `true`)
- [ ] 1.3 Remove `validate_features()` function in `src/cli/init.rs`
- [ ] 1.4 Remove `Feature::None` arm from `feature_to_str()` in `src/cli/init.rs`
- [ ] 1.5 Update `build_config_content()` — when `features` is `None`, produce empty features list instead of all four
- [ ] 1.6 Update all tests in `src/cli/init.rs` that reference `Feature::None` or `validate_features`

## 2. Add --targets flag and simplify TUI mode

- [ ] 2.1 Change `targets` field in `InitOptions` from `#[clap(skip)] pub targets: Vec<String>` to `#[clap(long, value_delimiter = ',', num_args = 1..)] pub targets: Option<Vec<String>>`
- [ ] 2.2 Simplify `is_tui_mode()` to just call `is_tui_enabled()` (remove flag-presence checks)
- [ ] 2.3 Update `build_config_content()` to handle `Option<Vec<String>>` for targets
- [ ] 2.4 Update all downstream code that reads `opts.targets` to handle `Option<Vec<String>>`
- [ ] 2.5 Update tests for `is_tui_mode` and `default_opts()` helper

## 3. Conditional wizard prompts

- [ ] 3.1 In `run_init_wizard()`, skip feature multiselect when `opts.features.is_some()`
- [ ] 3.2 In `run_init_wizard()`, skip template select when `opts.template.is_some()`
- [ ] 3.3 In `run_init_wizard()`, skip target prompt when `opts.targets.is_some()`
- [ ] 3.4 Update `opts.targets` assignment in wizard to use `Option<Vec<String>>`

## 4. Provider display in init/config (Name [slug] format)

- [ ] 4.1 In `prompt_targets()` in `src/cli/ui/init.rs`, change multiselect item label from bare slug to `Name [slug]` format using registry name data
- [ ] 4.2 Handle fallback to bare slug when provider has no name in registry

## 5. Provider display in providers ls (slug-only select)

- [ ] 5.1 In `run_tui()` in `src/cli/providers.rs`, change select label from provider name to slug
- [ ] 5.2 Remove hint text from select items (slug bracket and URL were in hint)
- [ ] 5.3 Keep the outro format as `Name (url)` on selection (already works this way)

## 6. Verification

- [ ] 6.1 Run `mise check` and confirm exit 0
- [ ] 6.2 Run `mise tests` and confirm exit 0
- [ ] 6.3 Manually test with tui-devtools: `dotagents init` full wizard flow
- [ ] 6.4 Manually test: `dotagents init --targets claude --features commands`
- [ ] 6.5 Manually test: `dotagents --ci init`
- [ ] 6.6 Manually test: `dotagents providers ls` TUI display
- [ ] 6.7 Manually test: `dotagents config global --edit` provider display
