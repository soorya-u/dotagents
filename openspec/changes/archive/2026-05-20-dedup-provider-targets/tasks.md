## 1. Extract target path resolution

- [x] 1.1 Create `resolve_target_path()` function in `src/templates/renderer.rs` that extracts lines 47-55 logic into a reusable `pub(crate)` function
- [x] 1.2 Update `render_feature_with_settings()` to call `resolve_target_path()` instead of inline target resolution
- [x] 1.3 Add unit test for `resolve_target_path()` with valid template, variables, and name_var

## 2. Implement dedup logic in deploy_feature

- [x] 2.1 Add pre-dedup step before `par_iter`: resolve target paths for all (provider, settings) pairs using `resolve_target_path()`
- [x] 2.2 Group providers by resolved target path using `HashMap<PathBuf, Vec<(String, FeatureSettings)>>`
- [x] 2.3 For each path group with >1 provider, sort alphabetically and select first as winner
- [x] 2.4 Build deduplicated work list: `Vec<(provider_name, settings, Option<DedupInfo>)>` where `DedupInfo` tracks winner/losers
- [x] 2.5 Update `par_iter` to skip providers with `DedupInfo::Some`, increment `stats.skipped`, emit `debug!` log
- [x] 2.6 Add unit test: alphabetical winner selection with 3 providers targeting same path
- [x] 2.7 Add unit test: no dedup when providers target different paths

## 3. Update dry-run output for dedup

- [x] 3.1 Add `DedupSkipped { winner: String }` variant to `DeployDryRunStatus` in `src/cli/ui/dry_run.rs`
- [x] 3.2 Add `provider: String` field to `DryRunDeployEntry` to track which provider produced the entry
- [x] 3.3 Update `print_dry_run_deploy_summary()` to show winner provider name and list skipped providers
- [x] 3.4 Update dry-run path in `deploy_feature()` to emit `DedupSkipped` entries for skipped providers
- [x] 3.5 Update dry-run summary count to use unique paths (after dedup)

## 4. Update deploy summary and stats

- [x] 4.1 Verify `DeployStats::skipped` correctly counts dedup-skipped providers (already handled in 2.5)
- [x] 4.2 Update `print_deploy_summary()` and `deploy_outro()` if needed to reflect dedup in output
- [x] 4.3 Add unit test for deploy summary with dedup-skipped providers

## 5. Verify cache and gitignore behavior

- [x] 5.1 Verify `CacheConfig::all_targets()` returns unique paths after dedup (no code change needed — dedup ensures only one entry per path)
- [x] 5.2 Add integration test: deploy with 3 providers targeting same path, verify cache has 1 entry
- [x] 5.3 Add integration test: undeploy after deduped deploy, verify file deleted once

## 6. Update existing tests

- [x] 6.1 Run `cargo test` and fix any test failures caused by dedup behavior changes
- [x] 6.2 Update any e2e tests that expect multiple providers to write to the same path

## 7. Verification

- [x] 7.1 Run `mise check` — cargo fmt + cargo clippy must pass
- [x] 7.2 Run `mise tests` — all unit, integration, and e2e tests must pass
