## ADDED Requirements

### Requirement: Removing a skill or command cleans up all deployed output
When `dotagents skills rm` or `dotagents commands rm` removes a source item, the system SHALL also remove all deployed files, cache entries, and `.gitignore` fence entries for that item across every provider. Cleanup SHALL run unconditionally — it does not depend on the `--deploy` flag.

#### Scenario: Deployed file removed across all providers
- **WHEN** `dotagents skills rm my-skill` is run and `my-skill` has been deployed to two providers
- **THEN** both deployed files are deleted from disk and the `.gitignore` fence entries for those paths are removed

#### Scenario: Cache entries pruned after rm
- **WHEN** `dotagents skills rm my-skill` is run
- **THEN** `cache.toml` no longer contains any entry with feature `skills` and item `my-skill`

#### Scenario: Deployed file missing on disk — cleanup continues
- **WHEN** the deployed file has already been manually deleted before `skills rm` is run
- **THEN** the rm command still succeeds, the cache entry is removed, and no error is reported for the missing file

### Requirement: Cleanup failures are non-fatal warnings
If deleting a deployed file fails for reasons other than the file not existing (e.g. permission error), the system SHALL log a warning and continue. The overall `rm` command SHALL exit 0.

#### Scenario: Permission error on deployed file
- **WHEN** a deployed file exists but cannot be deleted due to permissions
- **THEN** a warning is logged, the cache entry is still removed, and the command exits 0

### Requirement: Warning shown when item was never deployed
If no cache entries are found for the removed item, the system SHALL log a warning indicating the item may not have been deployed.

#### Scenario: Skill removed that was never deployed
- **WHEN** `dotagents skills rm my-skill` is run and no cache entries exist for `my-skill`
- **THEN** a warning message is logged: `"No deployed files found for 'my-skill' — was it ever deployed?"`
- **THEN** the source directory is still removed and the command exits 0
