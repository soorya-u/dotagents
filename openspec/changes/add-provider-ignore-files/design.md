## Context

Dotagents currently deploys commands, instructions, MCP configs, and skills to 20+ AI agent providers. Many of these providers support ignore files that control which files the agent reads or indexes, but dotagents does not manage these files. Users must manually create and maintain them. The deploy pipeline already has a feature abstraction (`FeatureTrait`) and a renderer that iterates over providers and features — adding `ignore` as a new feature type fits cleanly into this architecture. Additionally, `init` should scaffold an ignore patterns file when the user selects the ignore feature.

## Goals / Non-Goals

**Goals:**
- Add `ignore` as a first-class feature alongside `commands`, `instructions`, `mcp`, `skills`
- Support all 20 providers with ignore file templates (each follows the `.<name>ignore` convention)
- Each provider gets a template (`ignore.hbs`) that renders a list of ignore patterns
- Ignore patterns are sourced from a configurable list in `config.toml` (global + per-provider)
- Deployed ignore files are tracked in the gitignore fence
- `init` scaffolds a default ignore patterns file when the ignore feature is selected
- TUI wizard includes "Ignore Patterns" as a selectable feature

**Non-Goals:**
- No automatic pattern generation from deployed file paths (users define patterns explicitly)
- No permission-file support (separate concern — tracked in GitHub issue #156)
- No migration of existing manually-created ignore files

## Decisions

### 1. Ignore patterns as a simple list of strings
**Decision:** `IgnoreFeature` holds a `Vec<String>` of patterns, one per line in the rendered output.
**Rationale:** All supported providers use newline-separated glob patterns. No need for complex structures.
**Alternatives considered:**
- Structured patterns with comments/metadata — overkill for current use case
- Single string blob — harder to manipulate in templates and tests

### 2. Global patterns + per-provider overrides via config
**Decision:** Users define `[ignore]` table in config with `patterns = [...]`. Per-provider overrides via `[providers.<name>.ignore.variables]` or a dedicated `patterns` field.
**Rationale:** Matches existing pattern for per-provider feature settings. Keeps config familiar.
**Alternatives considered:**
- Separate `.dotagents/ignore` file — adds complexity, config.toml already handles this
- Only global patterns — too inflexible for provider-specific needs

### 3. Template renders patterns directly (no two-phase rendering)
**Decision:** Unlike commands/instructions, ignore templates skip the two-phase content rendering and render directly against `var.*` and `ignore.patterns`.
**Rationale:** Ignore files don't have frontmatter or complex structure. Two-phase rendering adds no value.
**Alternatives considered:**
- Use two-phase rendering for consistency — unnecessary complexity
- Special renderer path — simpler to skip phase 1 for this feature type

### 4. Ignore feature is singleton (one file per provider, not per-item)
**Decision:** `get_file_name()` returns `None`, so each provider gets exactly one ignore file.
**Rationale:** Providers have a single ignore file (`.ignore`, `.aiignore`, etc.), not one per pattern.
**Alternatives considered:**
- One file per pattern — doesn't match any provider's format

### 5. Provider template paths follow existing convention
**Decision:** Each provider gets `public/v1/templates/<slug>/ignore.hbs` with target path in `provider.toml`.
**Rationale:** Consistent with existing feature templates. Remote template resolution already handles this.
**Alternatives considered:**
- Shared template with provider-specific variables — harder to maintain, less flexible

### 6. Init scaffolds a default ignore patterns file
**Decision:** When the user selects the ignore feature during `init`, a default `ignore` file is created in `.dotagents/` with common patterns (e.g., `node_modules/`, `.git/`, `target/`).
**Rationale:** Matches the existing pattern for commands, instructions, MCP, and skills — each feature gets a mock file during init.
**Alternatives considered:**
- No default file — leaves users with no starting point
- Empty file — less helpful than a sensible default

### 7. All 20 providers get ignore templates
**Decision:** Every provider in the registry gets an `ignore.hbs` template, even if the ignore file format is not well-documented.
**Rationale:** Most providers follow the `.<name>ignore` convention. Users can customize templates later. Better to provide a starting point than leave providers unsupported.
**Alternatives considered:**
- Only add templates for well-documented providers — leaves gaps for users of other providers

## Risks / Trade-offs

- **[Risk]** Provider ignore file formats may change — **Mitigation:** Templates are user-editable; updates only require template changes, not core code
- **[Risk]** Users may have existing ignore files that get overwritten — **Mitigation:** Deploy prompts before overwriting (existing behavior for all templates)
- **[Trade-off]** No automatic pattern inference from deployed files — keeps implementation simple but requires manual pattern definition
- **[Risk]** Some provider ignore formats are not well-documented — **Mitigation:** Use the `.<name>ignore` convention as a sensible default; users can customize templates

## Migration Plan

1. Add `ignore` feature to config schema — backward compatible (new field is optional)
2. Add templates to registry — no impact on existing deployments
3. Users opt-in by adding `ignore` to `features` list and configuring patterns
4. No rollback needed — removing `ignore` from features stops rendering, existing files remain on disk

## Open Questions

- Should ignore patterns support variable interpolation (e.g., `{{ dir.workspace }}/.claude/`)?
- What is the exact ignore file format for providers like codex and amp? (Use `.<name>ignore` as default)
