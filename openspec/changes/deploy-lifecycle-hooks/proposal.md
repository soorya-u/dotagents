## Why

AI agent providers fire lifecycle events (PreToolUse, PostToolUse, Stop, SessionStart, etc.) but each provider speaks a different dialect: JSON vs TOML vs JSONC, embedded-in-settings vs standalone `hooks.json`, nested `{matcher,hooks:[{}]}` vs flattened, milliseconds vs seconds, `permissionDecision:"deny"` vs `decision:"block"`, and ~30 different event names with provider-specific casing. Users who target multiple providers must hand-write and hand-sync N different hook configs. dotagents already solves this exact problem for commands, instructions, and MCP — hooks is the missing 4th feature (issue #141).

## What Changes

- **New feature `hooks`**: add `Feature::Hook` to the `Feature` enum (`src/core/features/common.rs`) and a `hooks: Option<FeatureSettings>` field to the `Features` struct (`src/core/config/common.rs`), matching the existing flat `[providers.<name>.<feature>]` config shape from `flat-provider-config`. Users opt in via `features = ["hooks"]` in `config.toml`.
- **New source file `.dotagents/hooks.jsonc`**: one JSONC source-of-truth for all hooks. JSONC chosen over TOML to (a) ship a JSON Schema at `public/v1/schemas/hooks.schema.json` for native intellisense in VSCode/JetBrains/Zed/Neovim without extensions, (b) allow inline comments (valuable for security-sensitive hooks like `rm -rf` guards), and (c) reuse the existing `merge_jsonc` parser already in `src/utils/merge.rs`. No `version` field — the schema URL in `$schema` is the version anchor.
- **Canonical schema (Option B + ext)**: model the full ~30-event superset as a canonical enum plus an open `String` escape hatch for provider innovations between releases. Unknown events are silently filtered at deploy time by template structure (each provider's `hooks.hbs` only has branches for events it supports — no `hooks_events` manifest in `provider.toml`).
- **Canonical fields with `extra` passthrough**: hooks declare provider-agnostic fields (`event`, `matcher`, `type`, `command`, `timeout` in **milliseconds**, `onFailure`, `env`, `enabled`, `name`, `description`, `loopLimit`, `statusMessage`, `async`, `commandWindows`, plus type-specific `prompt`/`url`/`headers`/`server`/`tool`/`arguments`). A single `extra: Record<string, any>` field carries provider-specific passthrough keys (e.g. `loopLimit`, `failClosed`, `allowedEnvVars`) without namespacing — each template reads the keys it recognises via `{{lookup extra "key"}}` and ignores the rest. No per-hook `providers` block.
- **5 handler types**: `command` (all 18 declarative providers), `prompt` (claude-code, cursor, devin), `http` (claude-code, qwencode), `mcp_tool` (claude-code only), `agent` (claude-code only). Templates drop handler types they don't support — a single `hooks.jsonc` can target all providers without manual filtering.
- **`HookFeature` implements `FeatureTrait`** (`src/core/features/traits.rs`): `from_string`/`to_string` parse/serialize JSONC via the existing `serde_json5`/jsonc-parser path; `to_value()` groups hooks by event name into a JSON object keyed by event, each value an array of hook entries — this is the shape templates iterate.
- **Per-provider `hooks.hbs` templates** under `public/v1/templates/<provider>/hooks.hbs` lower canonical data to each dialect: event-name casing, timeout unit conversion (ms→sec via a `timeout_to_seconds` helper), decision-vocab rewriting, schema shape (nested vs flattened), file format (JSON/TOML/JSONC), embed-vs-standalone wrapper, env-var alias injection. Each template is the sole filter for which events a provider supports (no `provider.toml` capability manifest).
- **Deploy pipeline slot**: hooks deploy after mcp, before instructions (`commands → skills → mcp → hooks → instructions → ignore`), grouping structured-config features together. Reuses the existing `deploy_feature` machinery and the done `deploy-merge-write` for embedded-file providers (claude, gemini, kimi, augment, codex-inline, junie, openhands-via-wrapper, qoder, qwencode, tabnine, iflow, crush, commandcode, devin).
- **Trust-hash warning**: deploy emits a one-line end-of-run warning when a target provider trust-hashes hooks (codex, claude-code, trae, grok-skipped-v1): e.g. `codex: re-trust required — run /hooks in codex to review changed hooks`.
- **`enabled: false`** drops the hook from deployment entirely (kept for reference, not deployed to any provider). `async: true` on a provider that doesn't support async is silently dropped by the template (hook runs synchronously on that provider).
- **Schema published** at `public/v1/schemas/hooks.schema.json` with `$id: https://dotagents.soorya-u.dev/schemas/hooks.schema.json`, mirroring `mcp.schema.json`. The `extra` field is `additionalProperties: true` (loose) for v1; can tighten to per-provider subschemas later.

### Scope: 18 providers in v1, 12 skipped

**In scope (declarative config deploy, 18 providers):** claude-code, devin, gemini, codex, antigravity, cursor, kimi, copilot, commandcode, augment, factory, iflow, junie, openhands, qoder, qwencode, tabnine, trae, crush.

**Skipped from v1 (12 providers) — documented on issue #141:**
- **Programmatic TS/JS plugins (4):** cline, opencode, kilocode, omp — hooks are exported functions, not declarative config. dotagents would have to generate `.ts` code + run `bun install`; that's an `integration`/`init` job, not `deploy`.
- **Programmatic SDK callbacks (2):** cortex, deepagents — hooks are in-process callbacks passed to `createSession({hooks})` or `@before_model` decorators. No file to deploy.
- **Plugin-dir deploy (1):** goose — requires a plugin directory (`plugin.json` + `hooks/hooks.json` + scripts), not a settings-file hook entry. The existing deploy model is one `(template, target)` → one file; multi-file plugin scaffolding is a new mechanism. Will come under a future "plugin deploy" change alongside grok's plugin bundles.
- **Fixed-path scripts (2):** mux, grok — no config file exists; the provider looks for an executable at a magic path (`.mux/tool_pre`, `~/.grok/hooks/<name>`). dotagents controls config, not user scripts; users can symlink themselves. grok also bundles hooks inside plugins (covered by the plugin-deploy gap above).
- **Task-runner scripts (1):** zencoder — not event hooks at all; `.zenflow/settings.json` carries `setup_script`/`verification_script` that fire at task-worktree lifecycle points. No PreToolUse/PostToolUse model.
- **Stub/insufficient docs (2):** cline SDK plugins page 404'd during research; trae defers all schema/fields/I/O to "Hook configuration reference" page (not captured). trae is retained in-scope on the strength of its Claude Code import feature + 6 documented events, but its template may need refinement when the reference page is published.

## Capabilities

### New Capabilities
- `deploy-lifecycle-hooks`: Deploy lifecycle hook scripts (PreToolUse, PostToolUse, Stop, SessionStart, SessionEnd, UserPromptSubmit, Notification, plus extension events) from a canonical `.dotagents/hooks.jsonc` source-of-truth to provider-specific hook config files across 18 declarative providers, using the existing deploy pipeline and merge-aware write. Owns: the canonical hook schema, `HookFeature` impl of `FeatureTrait`, the `hooks.hbs` per-provider template family, timeout unit conversion, event filtering by template structure, and the trust-hash deploy warning.

### Modified Capabilities
- `flat-provider-config`: no spec change — hooks reuses the existing flat `[providers.<name>.<feature>]` shape. Listed here only to note the new `hooks` field on `Features` is a backwards-compatible addition (existing configs without `[providers.<name>.hooks]` are unaffected).
- `deploy-merge-write`: no spec change — hooks reuses the existing JSON/JSONC/TOML/YAML merge for embedded-file providers. Listed here only to note the dependency is satisfied.
- `deploy-pipeline`: spec gains a requirement that hooks deploy between mcp and instructions (new feature in the pipeline). Actual delta spec will be written if the `deploy-pipeline` spec needs an explicit requirement for the new feature slot; otherwise this is implementation-only.

## Impact

- `src/core/features/common.rs` — add `Feature::Hook` variant to the enum; `is_provider_agnostic()` returns false for `Hook` (hooks are dialect-specific).
- `src/core/features/hook.rs` — new module with `HookFeature` impl of `FeatureTrait`. Parses `.dotagents/hooks.jsonc`, exposes `to_value()` as `{<EventName>: [hookEntry, ...]}` grouped by event. `from_string`/`to_string` round-trip via JSONC.
- `src/core/features/mod.rs` — declare `hook` module.
- `src/core/config/common.rs` — add `hooks: Option<FeatureSettings>` to `Features`; extend `merge()`, `get_config()`, `has_configured_overrides()` to cover the new field.
- `src/core/config/app.rs` — extend `has_feature`/`get_provider_feature_settings` to handle `Feature::Hook`.
- `src/cli/deploy.rs` — insert `deploy_feature::<HookFeature>(ctx, &Feature::Hook, …)` between mcp and instructions; add trust-hash warning aggregation.
- `src/templates/templater.rs` — register a `timeout_to_seconds` helper (ceiling division: `ms / 1000`, rounded up so 5000ms → 5s, 5500ms → 6s) for templates that emit seconds. Ms-pass-through providers (gemini, augment, qwencode-command) use `this.timeout` directly.
- `public/v1/schemas/hooks.schema.json` — new JSON Schema for the source file, $id matching the existing schema pattern.
- `public/v1/templates/<provider>/hooks.hbs` — new template per in-scope provider (18 files). Each emits only the events the provider supports.
- `public/v1/templates/<provider>/provider.toml` — add `[providers.<name>.hooks]` block with `template` URL and `target` path per provider (18 files updated).
- `src/constants/mocks.rs` and `src/mocks/hooks.jsonc` — add a mock `hooks.jsonc` with one or two sample hooks so `dotagents init` scaffolds the new feature.
- `src/cli/init.rs` — include `hooks.jsonc` in the scaffolded mock file list; add `hooks` to the default `features` set (subject to `--no-hooks` flag — see tasks).
- `src/cli/options.rs` — add `--no-hooks` flag to `init` mirroring `--no-mcp`/`--no-command`/`--no-instruction`.
- `tests/e2e/hooks.test.ts` — new e2e suite covering CLI deploy (flag-driven, exit code, stdout/stderr, deployed file content for at least claude/cursor/kimi/embedded-vs-standalone) and the TUI init flow with hooks enabled.
- `tests/integration/` — smoke test for `HookFeature::from_string`/`to_value` round-trip and merge behavior.
- `Cargo.toml` — likely no new deps; reuse `serde_json5`/`jsonc-parser` already present for MCP. Verify during implementation.
