## MODIFIED Requirements

### Requirement: `--no-cache` flag skips hash comparison only
When `--no-cache` is passed to `dotagents deploy`, the cache file SHALL NOT be read for hash comparison — all target files are rendered and written as if no cache existed. However, the cache SHALL still be written at the end of the run with the hashes and paths of every file deployed in that run.

#### Scenario: No-cache deploy skips reading cache for comparison
- **WHEN** `dotagents deploy --no-cache` is run
- **THEN** no hash comparison is performed; all target files are written unconditionally

#### Scenario: No-cache deploy still writes cache.toml
- **WHEN** `dotagents deploy --no-cache` completes
- **THEN** `cache.toml` is written with entries for every file that was deployed in that run

#### Scenario: No-cache and force produce identical observable file output
- **WHEN** both `--no-cache` and `--force` are passed
- **THEN** all target files are written (same as each flag alone); cache is written at the end
