## Why

When deploying singleton features (instructions, MCP), if multiple providers target the same output file (e.g. 13 providers targeting `AGENTS.md`), the deploy pipeline writes the file N times via `rayon::par_iter` with no synchronization. This causes:

1. **Race condition**: Parallel `fs::write` calls to the same path — `O_TRUNC` from one thread can truncate another's in-progress write.
2. **Cache bloat**: N identical cache entries for the same file.
3. **Gitignore duplicates**: `all_targets()` collects N copies of the same path.
4. **Undeploy**: Attempts to delete the same file N times.

## What Changes

- Add deduplication logic in `deploy_feature()` before the `par_iter` loop
- Resolve target paths for all providers, group by resolved path, pick one writer per group
- Skip redundant providers with appropriate logging
- Track dedup decisions in `DeployStats` for dry-run and summary output
- Update dry-run output to show which provider would write and which were skipped

## Capabilities

### New Capabilities
- `provider-dedup`: Deduplication of provider writes when multiple providers target the same file path during deploy

### Modified Capabilities
- `deploy-dry-run`: Dry-run output now includes dedup information showing which provider wins and which are skipped
- `deploy-output-cache`: Cache entries are now per unique target path rather than per provider for singleton features

## Impact

- **src/cli/deploy.rs**: Main dedup logic in `deploy_feature()` 
- **src/templates/renderer.rs**: Extract target path resolution for pre-dedup phase
- **src/core/config/cache.rs**: `all_targets()` naturally returns unique paths after dedup
- **src/cli/ui/dry_run.rs**: Dry-run summary shows dedup decisions
- **src/cli/ui/deploy.rs**: Deploy summary reflects dedup stats
- Existing tests: May need updates to account for dedup behavior
