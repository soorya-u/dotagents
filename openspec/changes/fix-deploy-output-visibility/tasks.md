## 1. Fix summary output in CI mode

- [ ] 1.1 In `src/cli/ui/deploy.rs`, remove the `if !is_tui_enabled() { return; }` guard from `print_deploy_summary()`
- [ ] 1.2 Split the function body: in `is_tui_enabled()` mode print with `"✓ "` prefix as before; in non-TTY mode use `println!()` with plain text (`"Nothing deployed"` or `"deployed: N written, N skipped"`)

## 2. Warn on no providers

- [ ] 2.1 In `src/cli/deploy.rs`, after `resolve_provider_defaults()` and before the `deploy_feature` calls, check whether any providers are configured: call `app_config.get_provider_feature_settings()` for each feature and check if all return empty maps
- [ ] 2.2 If all providers maps are empty, emit `warn!("No providers configured — nothing to deploy. Add providers to config.toml.")`

## 3. Unit tests

- [ ] 3.1 Add unit test in `src/cli/ui/deploy.rs`: `print_deploy_summary` with `written=2, skipped=1` in a non-TTY environment produces a stdout line containing `"2"` and `"1"` (capture stdout)
- [ ] 3.2 Add unit test in `src/cli/ui/deploy.rs`: `print_deploy_summary` with `written=0, skipped=0` in a non-TTY environment produces a stdout line containing `"Nothing deployed"`

## 4. E2e tests

- [ ] 4.1 Add e2e test in `tests/e2e/deploy.test.ts`: `deploy --ci --offline` with a local provider configured — assert stdout contains the written/skipped count
- [ ] 4.2 Add e2e test in `tests/e2e/deploy.test.ts`: `deploy --ci --offline` with no providers configured — assert stderr (warn) contains `"No providers configured"`

## 5. Verification

- [ ] 5.1 Run `mise check` (fmt + clippy) — must exit 0
- [ ] 5.2 Run `mise tests` (unit + integration + e2e) — must exit 0
