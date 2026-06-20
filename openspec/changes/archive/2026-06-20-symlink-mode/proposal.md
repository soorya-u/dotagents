# Proposal: Symlink Deploy Mode

## Summary

Add a per-feature deploy mode toggle (`link` / `template`) that controls whether source files are symlinked directly or rendered through Handlebars templates. Symlink mode becomes the hardcoded default (#169), template rendering becomes an explicit opt-in (#87). Skills (#163) and agent-ignore files gain support for symlinking extra files alongside the primary content.

## Motivation

Currently every feature passes through the full 3-phase Handlebars render pipeline regardless of whether the output format differs between providers. For "Type 1" features — skills, agent-ignore, agent config files (#140) — the output format is identical across all providers, making template rendering unnecessary overhead. These features benefit most from symlinking: one source file, linked into each provider's directory.

For "Type 2" features — commands, instructions, MCP — the output format varies per provider (e.g., Claude uses `.md`, Codex uses `.json`), so template rendering remains necessary. However, variable injection (Phase 3) can still be skipped in link mode for faster deploys when env/var interpolation in content isn't needed.

## Scope

- New `[feature.<name>]` config table with `mode` and `mode_override` fields
- New `FeatureTrait` methods: `is_symlinkable()` and `get_source_path()`
- New `link_feature_with_settings()` deploy function for Type 1 link mode
- Modified `render_feature_with_settings()` to skip Phase 2 (Type 1 template) or Phase 3 (link mode)
- Skills extra files (#163): all non-SKILL.md files in skill directory always symlinked
- No cache entries for Type 1 link mode
- Provider registry `provider.toml` files drop `template` field for Type 1 features
- Hardcoded fallback mode is `"link"` (#169)

## Non-Goals

- Per-provider mode configuration (unnecessary complexity)
- Intermediate `bin/` directory for rendered content
- Windows symlink support (initial release: Unix-only)

## Testing

- Manual tui-devtools discovery for interactive init prompts (if config wizard changes)
- Unit tests for: mode resolution, default fallback, `mode_override` resolution, trait methods, link vs template branching in renderer
- E2E tests for: deploy with mode=link creates symlinks, deploy with mode=template writes files, skills extra files symlinked, dedup works with symlinks, cache behavior for link mode
