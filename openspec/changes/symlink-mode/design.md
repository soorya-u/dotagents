# Design: Symlink Deploy Mode

## Config Schema

```toml
features = ["command", "instruction", "mcp", "skill", "agent-ignore"]

[feature.command]
mode = "template"
mode_override = { hello = "template" }

[feature.skill]
mode = "link"
mode_override = { my-skill = "template" }
```

- `features` (plural, array) — existing key, unchanged. Lists enabled features.
- `[feature.<name>]` (singular, table) — new top-level key. Per-feature settings.
- `mode` — `"link"` or `"template"`. Hardcoded fallback is `"link"` when no table exists.
- `mode_override` — `map<string, string>`. Per-item overrides for multi-file features (commands, skills). Key is the item name (e.g., `hello`), value is the mode.

## Feature Categories

```
┌──────────────────────────────────────────────────────────┐
│  TYPE 1 (symlinkable)         TYPE 2 (non-symlinkable)   │
│  ──────────────────           ────────────────────────   │
│  skills                       commands                    │
│  agent-ignore                 instructions                │
│  agent config files (#140)    MCP                         │
│                                                           │
│  Same format everywhere       Different format/provider   │
│  No .hbs template needed      .hbs template required      │
│  template field absent        template field required     │
└──────────────────────────────────────────────────────────┘
```

Detection: `FeatureTrait::is_symlinkable(&self) -> bool` — defaults to `false`. Type 1 features override to `true`.

## Mode Behavior Matrix

| Feature Type | Mode       | Phase 1 (target) | Phase 2 (template) | Phase 3 (vars) | File Action                    |
|-------------|------------|-------------------|---------------------|-----------------|--------------------------------|
| Type 1      | `link`     | ✓                 | ✗                   | ✗               | symlink(source, target)        |
| Type 1      | `template` | ✓                 | ✗                   | ✓               | write(render_vars(source), tgt)|
| Type 2      | `link`     | ✓                 | ✓                   | ✗               | write(render(tmpl, feat), tgt) |
| Type 2      | `template` | ✓                 | ✓                   | ✓               | write(render(tmpl, vars(feat)), tgt) |

Note: Type 2 "link" mode does NOT create symlinks — the name is conceptual. It means "skip variable injection." Output is still a regular file.

## FeatureTrait Changes

Two new methods:

```rust
/// Returns true if this feature's content format is identical across providers,
/// making it eligible for symlink deployment.
fn is_symlinkable(&self) -> bool { false }

/// Returns the filesystem path to the source file, for symlink creation.
/// Only meaningful when is_symlinkable() returns true.
fn get_source_path(&self) -> Option<PathBuf> { None }
```

Feature items (CommandFeature, SkillFeature, etc.) will store their source path as a field, set during `from_application()`.

## Deploy Loop

```
deploy_feature<T>(ctx, feature, loader):
  1. Guard: if !app_config.has_feature(feature) → return
  2. Load items via loader
  3. Get enabled providers
  4. build_work_list() (dedup unchanged)
  5. For each work item (parallel via rayon):
     a. If dedup loser → handle_dedup_skip (unchanged)
     b. Resolve mode:
        - Check item-level mode_override
        - Fall back to feature-level mode
        - Fall back to hardcoded "link"
     c. If is_symlinkable() && mode == "link":
        → link_feature_with_settings(provider, feature, settings, vars)
     d. Else:
        → render_feature_with_settings(provider, feature, settings, vars, mode)
  6. Aggregate stats
```

### `link_feature_with_settings()`

```rust
fn link_feature_with_settings<T: FeatureTrait>(
    provider_name: &str,
    feature: &T,
    settings: &FeatureSettings,
    templater: &Templater,
    variables: Option<&Value>,
) -> Result<CacheUpdate>
```

1. Resolve target path from `settings.target` via Handlebars (Phase 1)
2. Get source path from `feature.get_source_path()`
3. Create parent directories for target if needed
4. Create symlink: `std::os::unix::fs::symlink(source, target)`
5. Return `CacheUpdate::Linked { target }` (new variant, no cache entry written)

### `render_feature_with_settings()` Modifications

Receives a new `mode` parameter. Changes:
- `template` field check: only required when mode=template OR Type 2
- Phase 2 (populate_with_values): skipped when mode=link
- Phase 3 (template rendering): skipped when is_symlinkable() (Type 1)
- No behavioral change for Type 2 + mode=template (current behavior)

## Skills Extra Files (#163)

During skill deploy in link mode:
1. Read skill directory entries
2. For each entry that is NOT `SKILL.md`:
   - Resolve relative target path: `{target_dir}/{entry_name}`
   - Create symlink: `symlink(source_dir/entry, target_dir/entry)`
   - These are ALWAYS symlinked (not mode-dependent, not template-rendered)

When mode=template for a skill, extra files are still symlinked — only SKILL.md follows the mode toggle.

## Cache

- Type 1 link mode: NO `CacheEntry` created. `CacheUpdate::Linked` variant with target path only. `process_cache_update()` skips cache.set() for this variant.
- Type 1 template mode: normal cache behavior (CacheEntry with hash + target).
- Type 2: normal cache behavior (unchanged).
- `.gitignore` fence: `finalize_deploy()` extends `cache.all_targets()` to also collect from `DeployStats` for linked items.

## Provider Registry Changes

`provider.toml` for Type 1 features drops `template`:

```toml
# Before (skill):
[providers.claude.skills]
template = "https://.../skill.hbs"
target = "{{dir.workspace}}/.claude/skills/{{skill.name}}/SKILL.md"

# After (skill):
[providers.claude.skills]
target = "{{dir.workspace}}/.claude/skills/{{skill.name}}/SKILL.md"
```

`provider.toml` for Type 2 features unchanged.

## Mode Resolution Order

For each (feature, provider, item):
1. Check `config.feature_maps[feature].mode_override[item_name]` — per-item override
2. Check `config.feature_maps[feature].mode` — feature-level default
3. Fallback: `"link"` (hardcoded)

## Edge Cases

- **Type 1 + mode=link + no source path**: error (source file doesn't exist)
- **Type 2 + mode=link + no template**: error (template required for Phase 2)
- **Type 2 + mode=template + no template**: error (existing behavior)
- **Type 1 + mode=template + no template**: OK (no .hbs template needed)
- **Symlink target already exists**: overwrite (same as today's write behavior)
- **Symlink target is a regular file**: overwrite (convert to symlink)
- **Symlink target is a symlink pointing elsewhere**: overwrite
- **Dedup**: unchanged — if two providers target the same path, alphabetically-first wins
- **Dry-run**: report what would be symlinked, don't create symlink
