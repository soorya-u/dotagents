## Context

The deploy pipeline iterates providers in parallel via `rayon::par_iter` in `deploy_feature()`. For singleton features (instructions, MCP) where multiple providers target the same file (e.g. `AGENTS.md`), this causes:

1. **Race condition**: Multiple threads call `fs::write` on the same path simultaneously — `O_TRUNC` from one thread can truncate another's in-progress write.
2. **Cache bloat**: N cache entries for the same file, keyed by `(provider, feature, "_")`.
3. **Gitignore duplicates**: `CacheConfig::all_targets()` returns N copies of the same path.
4. **Undeploy inefficiency**: Attempts to delete the same file N times.

Current flow in `deploy_feature()`:
1. Load items via `loader()`
2. Get provider settings via `app_config.get_provider_feature_settings(feature)`
3. `par_iter()` over providers, each calling `render_feature_with_settings()` which resolves the target path, renders the template, and writes the file

The target path resolution happens inside `render_feature_with_settings()` in `src/templates/renderer.rs` (lines 47-55). It renders the `target` Handlebars template with variables + feature name variable.

## Goals / Non-Goals

**Goals:**
- Eliminate race conditions from parallel writes to the same target path
- Ensure exactly one cache entry per unique target path for singleton features
- Return unique paths from `all_targets()` without additional dedup logic
- Preserve determinism: same config always picks the same writer
- Show dedup decisions in dry-run and normal deploy output

**Non-Goals:**
- No config schema changes — dedup works with existing config
- No cache schema changes — entries remain keyed by `(provider, feature, item)`
- No changes to command/skill features — they already produce unique paths via `get_file_name()`
- No user-facing priority configuration — alphabetical sort is sufficient for determinism

## Decisions

### D1: Dedup before `par_iter` in `deploy_feature()`

**Decision**: Resolve target paths for all providers before entering `par_iter`, group by resolved path, and build a deduplicated work list.

**Rationale**: 
- Avoids rendering templates for providers that will be skipped anyway
- Eliminates the race condition at the source — only one thread ever writes to a given path
- Simpler than adding synchronization (mutex/flock) around `fs::write`

**Alternatives considered**:
- Mutex around `fs::write` — still renders N templates, still creates N cache entries
- Sequential iteration for singleton features — loses parallelism for commands/skills
- Post-write dedup — doesn't solve the race condition

### D2: Alphabetical sort for winner selection

**Decision**: When multiple providers target the same path, sort provider names alphabetically and pick the first.

**Rationale**:
- Deterministic — same config always produces the same result
- No additional config fields needed
- Simple to understand and debug

**Alternatives considered**:
- Priority field in `FeatureSettings` — requires config change, overkill for v0.2
- First-encountered order — HashMap iteration order is non-deterministic
- User prompt — breaks CI/automated deploys

### D3: Extract `resolve_target_path()` from renderer

**Decision**: Create a standalone `pub(crate) fn resolve_target_path()` in `renderer.rs` that takes `target_str`, `templater`, `variables`, and `name_var` and returns `PathBuf`.

**Rationale**:
- `deploy_feature()` needs to resolve target paths during pre-dedup without full template rendering
- Avoids duplicating the target resolution logic (lines 47-55 of renderer.rs)
- The full `render_feature_with_settings()` can call this extracted function internally

### D4: Dedup tracking via `DryRunDeployEntry` extension

**Decision**: Add a `DedupSkipped { winner: String, losers: Vec<String> }` variant to `DeployDryRunStatus` and a corresponding `DryRunDeployEntry` field for the provider name.

**Rationale**:
- Dry-run output should show which provider would write and which were skipped
- Existing `DryRunDeployEntry` already carries `path` and `status` — extend rather than create new type
- Normal deploy uses `debug!` logging for skipped providers, no UI changes needed

**Alternatives considered**:
- Separate dedup entry type — more types, more complexity
- Only log dedup in normal mode — dry-run users can't inspect dedup decisions

### D5: Dedup scope is per-`deploy_feature()` call

**Decision**: Dedup happens within each `deploy_feature()` call (instructions, MCP, commands, skills separately). Cross-feature collisions (e.g. instructions and MCP both targeting `AGENTS.md`) are not deduplicated.

**Rationale**:
- Cross-feature collisions are a user misconfiguration, not a bug
- Each feature has different content — picking a "winner" across features is semantically wrong
- The issue specifically calls out same-feature collisions

## Risks / Trade-offs

**[Risk] Template rendering errors during pre-dedup** → If target path resolution fails for one provider, the entire deploy_feature() call fails. Mitigation: Pre-dedup resolution uses the same error path as the renderer — errors surface with clear context.

**[Risk] Provider order in HashMap is non-deterministic** → Alphabetical sort after collecting providers ensures determinism regardless of HashMap iteration order.

**[Risk] Dry-run output changes** → Existing dry-run tests expect specific output format. Mitigation: Add `DedupSkipped` as a new status variant; existing `New`/`Modified` entries are unchanged.

**[Trade-off] Loser's variables are discarded** → When provider B is deduped in favor of provider A, B's `variables` in `FeatureSettings` are not used. This is correct behavior — only one provider's content should be written.

**[Trade-off] Cache still keyed by provider** → The cache schema doesn't change, so if the "winner" provider changes (e.g. user removes the winning provider), the next deploy will re-render and update cache. This is acceptable — cache entries are cheap for singleton features after dedup.
