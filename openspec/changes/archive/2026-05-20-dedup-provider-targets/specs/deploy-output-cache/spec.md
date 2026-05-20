## ADDED Requirements

### Requirement: Single cache entry per unique target path for singleton features
After provider deduplication, `cache.toml` SHALL contain exactly one entry per unique target path for singleton features (instructions, MCP), not one per provider.

#### Scenario: Multiple providers deduped to single writer
- **WHEN** 3 providers target `AGENTS.md` for the instructions feature and dedup selects one winner
- **THEN** `cache.toml` contains one cache entry for `AGENTS.md` under the winning provider's key

#### Scenario: all_targets returns unique paths
- **WHEN** `CacheConfig::all_targets()` is called after a deduped deploy
- **THEN** the returned Vec contains each target path exactly once
