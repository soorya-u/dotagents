## ADDED Requirements

### Requirement: Dedup information in dry-run output
When `--dry-run` is set and provider deduplication occurs, the command SHALL show which provider would write to each deduplicated path and which providers were skipped.

#### Scenario: Dedup winner shown in dry-run output
- **WHEN** `--dry-run` is set and multiple providers target the same path
- **THEN** the output shows `[~] <path> (<winner>)` for the winning provider

#### Scenario: Dedup losers listed under winner in dry-run output
- **WHEN** `--dry-run` is set and providers are dedup-skipped
- **THEN** the output lists skipped providers indented under the winner entry

## MODIFIED Requirements

### Requirement: Summary line in dry-run output
After listing affected paths, the command SHALL print `N files would be affected` where N is the count of unique target paths (after dedup), not the count of providers.

#### Scenario: Summary shown after path list with dedup
- **WHEN** `--dry-run` is set, 3 providers target the same file, and 1 provider targets a different file
- **THEN** stdout ends with `2 files would be affected` (not 4)

#### Scenario: Empty dry-run summary
- **WHEN** `--dry-run` is set and all rendered outputs are identical to on-disk files
- **THEN** stdout contains `0 files would be affected` and exits with code 0
