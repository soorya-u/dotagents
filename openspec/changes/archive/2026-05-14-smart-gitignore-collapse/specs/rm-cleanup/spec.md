## MODIFIED Requirements

### Requirement: Removing a skill or command cleans up all deployed output
When `dotagents skills rm` or `dotagents commands rm` removes a source item, the system SHALL also remove all deployed files and cache entries for that item across every provider. After cache entries are removed, the system SHALL rebuild the `.gitignore` fence from the remaining cached targets using the collapse algorithm. If no cache entries remain after removal, the system SHALL clear the entire fence. Cleanup SHALL run unconditionally — it does not depend on the `--deploy` flag.

#### Scenario: Deployed file removed across all providers
- **WHEN** `dotagents skills rm my-skill` is run and `my-skill` has been deployed to two providers
- **THEN** both deployed files are deleted from disk, cache entries are removed, and the `.gitignore` fence is rebuilt from remaining cached targets

#### Scenario: Cache entries pruned after rm
- **WHEN** `dotagents skills rm my-skill` is run
- **THEN** `cache.toml` no longer contains any entry with feature `skills` and item `my-skill`

#### Scenario: Deployed file missing on disk — cleanup continues
- **WHEN** the deployed file has already been manually deleted before `skills rm` is run
- **THEN** the rm command still succeeds, the cache entry is removed, the fence is rebuilt, and no error is reported for the missing file

#### Scenario: Last item removed clears fence entirely
- **WHEN** `dotagents commands rm hello` is run and `hello` was the only remaining deployed item across all providers
- **THEN** the cache is empty after removal and the entire dotagents fence is removed from `.gitignore`

#### Scenario: Fence remains correctly collapsed after single item removal
- **WHEN** `.claude/commands/` is collapsed in the fence, and `dotagents commands rm opsx-apply` removes one of 8 commands
- **THEN** the fence is rebuilt; `.claude/commands/` remains collapsed because the remaining 7 files still fully occupy the directory
