## 1. Extract target path resolution

- [ ] 1.1 Create `resolve_target_path()` function in `src/templates/renderer.rs` that extracts lines 47-55 logic into a reusable `pub(crate)` function
- [ ] 1.2 Update `render_feature_with_settings()` to call `resolve_target_path()` instead of inline target resolution
- [ ] 1.3 Add unit test for `resolve_target_path()` with valid template, variables, and name_var

## 2. Implement dedup logic in deploy_feature

- [ ] 2.1 Add pre-dedup step before `par_iter`: resolve target paths for all (provider, settings) pairs using `resolve_target_path()`
- [ ] 2.2 Group providers by resolved target path using `HashMap<PathBuf, Vec<(String, FeatureSettings)>>`
- [ ] 2.3 For each path group with >1 provider, sort alphabetically and select first as winner
- [ ] 2.4 Build deduplicated work list: `Vec<(provider_name, settings, Option<DedupInfo>)>` where `DedupInfo` tracks winner/losers
- [ ] 2.5 Update `par_iter` to skip providers with `DedupInfo::Some`, increment `stats.skipped`, emit `debug!` log
- [ ] 2.6 Add unit test: alphabetical winner selection with 3 providers targeting same path
- [ ] 2.7 Add unit test: no dedup when providers target different paths

## 3. Update dry-run output for dedup

- [ ] 3.1 Add `DedupSkipped { winner: String }` variant to `DeployDryRunStatus` in `src/cli/ui/dry_run.rs`
- [ ] 3.2 Add `provider: String` field to `DryRunDeployEntry` to track which provider produced the entry
- [ ] 3.3 Update `print_dry_run_deploy_summary()` to show winner provider name and list skipped providers
- [ ] 3.4 Update dry-run path in `deploy_feature()` to emit `DedupSkipped` entries for skipped providers
- [ ] 3.5 Update dry-run summary count to use unique paths (after dedup)

## 4. Update deploy summary and stats

- [ ] 4.1 Verify `DeployStats::skipped` correctly counts dedup-skipped providers (already handled in 2.5)
- [ ] 4.2 Update `print_deploy_summary()` and `deploy_outro()` if needed to reflect dedup in output
- [ ] 4.3 Add unit test for deploy summary with dedup-skipped providers

## 5. Verify cache and gitignore behavior

- [ ] 5.1 Verify `CacheConfig::all_targets()` returns unique paths after dedup (no code change needed — dedup ensures only one entry per path)
- [ ] 5.2 Add integration test: deploy with 3 providers targeting same path, verify cache has 1 entry
- [ ] 5.3 Add integration test: undeploy after deduped deploy, verify file deleted once

## 6. Update existing tests

- [ ] 6.1 Run `cargo test` and fix any test failures caused by dedup behavior changes
- [ ] 6.2 Update any e2e tests that expect multiple providers to write to the same path

## 7. Verification

- [ ] 7.1 Run `mise check` — cargo fmt + cargo clippy must pass
- [ ] 7.2 Run `mise tests` — all unit, integration, and e2e tests must pass
