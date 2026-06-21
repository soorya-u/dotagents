## Context

dotagents deploys three features today — commands, instructions, mcp — from a `.dotagents/` source-of-truth to provider-specific files via Handlebars templates. Each feature implements `FeatureTrait` (`src/core/features/traits.rs`) and is dispatched from `deploy_feature` in `src/cli/deploy.rs`. Merge-aware write (`deploy-merge-write`, done) handles providers that embed config in a shared `settings.json`/`config.toml`/etc.

Issue #141 asks for the same treatment for lifecycle hooks (PreToolUse, PostToolUse, Stop, SessionStart, etc.) across 30 providers. Research across all 30 provider docs (saved at `/tmp/opencode/hooks-research/*.txt` during exploration) shows the providers split into 5 architectural models — only 18 are reachable via declarative config deploy in v1; 12 are skipped (see proposal).

This design covers the canonical schema, the `HookFeature` impl, the per-provider lowering templates, and the deploy-time filtering mechanism. It does not cover plugin-dir deploy, programmatic plugin scaffolding, or fixed-path script generation — those are explicitly out of scope.

## Goals / Non-Goals

**Goals:**
- One JSONC source-of-truth (`.dotagents/hooks.jsonc`) deployable to 18 declarative providers.
- Canonical schema covering the full ~30-event superset with an open escape hatch for provider innovations.
- Per-provider `hooks.hbs` templates handle all dialect translation (event casing, timeout unit, decision vocab, schema shape, file format, embed-vs-standalone wrapper, env-var alias).
- Template-as-filter: no `hooks_events` manifest in `provider.toml`; each template's branch structure is the sole filter for which events a provider supports.
- Reuse existing `FeatureTrait` + `deploy_feature` + `deploy-merge-write` machinery — no new deploy infrastructure.
- Ship a JSON Schema at `public/v1/schemas/hooks.schema.json` for native editor intellisense.

**Non-Goals:**
- Plugin-dir deploy for goose (multi-file output from one template is a new mechanism — future change).
- Programmatic TS/JS plugin generation for cline, opencode, kilocode, omp (that's an `integration` job, not `deploy`).
- SDK callback generation for cortex, deepagents (no file to deploy).
- Fixed-path script generation for mux, grok (dotagents controls config, not user scripts).
- Task-runner script config for zencoder (not event hooks).
- Per-provider subschemas for the `extra` passthrough field (loose `additionalProperties: true` for v1; can tighten later).
- Backward compatibility for any prior hooks format (this is a new feature; no prior art in dotagents).
- A TUI wizard for authoring hooks (out of scope; users edit `hooks.jsonc` directly with schema-assisted intellisense).

## Decisions

### D1: Source format is JSONC, not TOML

**Choice:** `.dotagents/hooks.jsonc` parsed as JSONC.

**Rationale:**
- JSON Schema (`public/v1/schemas/hooks.schema.json`) gives native intellisense in VSCode/JetBrains/Zed/Neovim via a `$schema` line — zero extensions required. TOML has no equivalent schema ecosystem.
- Comments are valuable for security-sensitive hooks (e.g. `// blocks rm -rf across all providers`).
- `merge_jsonc` already exists in `src/utils/merge.rs:68` for the merge-aware deploy path; reuse for parsing.
- The existing MCP source (`mcp.jsonc`) is already JSONC — consistency within `.dotagents/`.

**Alternatives considered:**
- TOML: consistent with `config.toml`/`local.config.toml`. Rejected — no schema ecosystem, no inline comments on arrays-of-tables (TOML comments are line-oriented, awkward inside `[[hooks]]` arrays).
- JSON: no comments. Rejected — security hooks benefit from inline rationale.
- JSON5: superset of JSON with comments, already used by MCP via `serde_json5`. Rejected as the source format — JSONC is a stricter subset with better tooling support (VSCode's built-in JSONC language mode); JSON5's extra features (trailing commas, unquoted keys, single quotes) aren't needed and weaken schema validation.

### D2: One file per set, not one file per hook

**Choice:** Single `.dotagents/hooks.jsonc` with a `hooks: [...]` array.

**Rationale:**
- Hooks deploy *as a set* to one target file (e.g. all hooks land in `.cursor/hooks.json` together). One source file matches one deploy target.
- Commands went one-per-file (`commands/<name>.md`) because each command has its own target path (`{{command.name}}.md`). Hooks almost always share one target per provider.
- One file is easier to schema-validate and review as a policy unit (a security reviewer wants to see all hooks in one place).

**Alternatives considered:**
- `.dotagents/hooks/<name>.jsonc` (one per hook, like commands): rejected — no per-hook target path variation to justify it; would require a directory walker and aggregation step before deploy; harder to review as a cohesive policy.

### D3: Milliseconds canonical, template helper converts

**Choice:** `timeout` field is always milliseconds in the source. A `timeout_to_seconds` Handlebars helper does ceiling division (`ms / 1000` rounded up) for providers that want seconds. Ms-pass-through providers (gemini, augment, qwencode-command) emit `this.timeout` directly.

**Rationale:**
- 3 of the 4 largest-event-set providers (augment, gemini, qwencode-command) already use ms. Standardising on ms aligns with the majority and with the finer-grained unit.
- Ceiling division preserves safety semantics: a user who writes `5500ms` gets `6s` on a sec-provider, not `5s` (which would truncate and shorten the timeout).
- The helper approach (D3-a from exploration) over the core-conversion approach (D3-b) because the user explicitly chose template helper — keeps `to_value()` pure, no dual `timeout_ms`/`timeout_sec` fields in the data model.

**Alternatives considered:**
- Seconds canonical, convert to ms: rejected — fewer providers need ms, but ms is the finer unit and avoids floating-point in the source.
- `to_value()` exposes both `timeout_ms` and `timeout_sec`: rejected by user — two fields is a footgun (template might emit the wrong one).
- Explicit unit object `timeout: {value: 5, unit: "seconds"}`: rejected — verbose, every user pays the unit-verbosity tax for a conversion that only varies by provider.

### D4: Option B + ext for event surface; template-as-filter, no manifest

**Choice:** Canonical enum covers the ~30 known events (PreToolUse, PostToolUse, Stop, SessionStart, SessionEnd, UserPromptSubmit, Notification, SubagentStart, SubagentStop, PreCompact, PostCompact, PermissionRequest, PostToolUseFailure, StopFailure, BeforeModel, AfterModel, BeforeAgent, AfterAgent, BeforeToolSelection, BeforeShellExecution, AfterShellExecution, BeforeMCPExecution, AfterMCPExecution, BeforeReadFile, AfterFileEdit, BeforeSubmitPrompt, AfterAgentResponse, AfterAgentThought, PreInvocation, PostInvocation, workspaceOpen, beforeTabFileRead, afterTabFileEdit, Interrupt, PermissionResult, PermissionDenied, Setup, InstructionsLoaded, UserPromptExpansion, MessageDisplay, PostToolBatch, TaskCreated, TaskCompleted, ConfigChange, CwdChanged, FileChanged, TeammateIdle, SetUpEnvironment) plus an open `String` for provider innovations. No `hooks_events` field in `provider.toml` (hard rule from user).

**Filtering mechanism:** `HookFeature::to_value()` groups hooks by event name into a JSON object: `{ "PreToolUse": [...], "Interrupt": [...], "workspaceOpen": [...] }`. Each provider's `hooks.hbs` template has `{{#each <eventName>}}` branches only for events it supports. Unknown events don't match any branch → silently dropped. No Rust-side filtering, no per-provider capability manifest.

**Rationale:**
- The template IS the capability manifest — it's already per-provider, already maintained, already the source of dialect truth. Adding a separate manifest duplicates knowledge.
- Silently dropping (no warning, per user decision) avoids noise from the common case: a user writes one `hooks.jsonc` targeting 5 providers, 2 of which support `Interrupt` — the other 3 dropping it is expected, not an error.
- Option B (full superset) over Option A (core 7 + passthrough) because the user wants dotagents to be a validator, not just a deployer — built-in event recognition gives "does event X work on provider Y" answers and per-provider event filtering for free.
- The open `String` escape hatch (B + ext) prevents provider innovations from blocking on a dotagents release.

**Alternatives considered:**
- Option A (core 7 canonical + unknown events passthrough): rejected by user — wants full validation.
- `hooks_events` array in `provider.toml`: rejected by user (hard rule) — capability metadata doesn't belong in the deploy-target manifest; the template already encodes capabilities.
- Rust-side filtering with a hardcoded provider→events map: rejected — duplicates template knowledge, churns core code on every provider event addition.

### D5: Provider-specific params via single `extra` field, no per-hook `providers` block

**Choice:** Each hook entry has an optional `extra: Record<string, any>` bag for provider-specific passthrough keys. No `providers: { "cursor": {...} }` block. Templates read keys via `{{lookup extra "loopLimit"}}`.

**Rationale:**
- The `providers` block co-mingled concerns (params bag vs deploy filter), bloated the schema (90+ provider-specific fields), and forced the author to know every target's dialect — that's the template's job.
- `extra` is a single untyped bag — schema stays small, author writes only the keys they care about, each template picks up what it recognises.
- Collision risk (two providers wanting different values for the same key name) is low because provider-specific field names are distinctive (`loopLimit`, `failClosed`, `allowedEnvVars`, `blockOnError`, `metadata`). Accept for v1.

**Alternatives considered:**
- Per-hook `providers: { "<name>": {...} }`: rejected by user during exploration — wrong abstraction.
- File-level `providerConfig: { "<name>": {...} }`: rejected — can't vary per-hook.
- Promote every provider-specific field to canonical: rejected — schema absorbs every provider's fields, grows unboundedly, `metadata` becomes untyped anyway.

### D6: Canonical fields for v1

**Choice:** The following fields are canonical (typed in schema):

| Field | Type | Applies to | Notes |
|---|---|---|---|
| `name` | string | all | display/trust id |
| `description` | string | all | human-readable |
| `event` | string (enum + open) | all | canonical event name |
| `matcher` | string | tool/events with matchers | regex |
| `enabled` | boolean | all | default true; false → drop from deploy |
| `type` | enum: command\|prompt\|http\|mcp_tool\|agent | all | handler type |
| `command` | string | type:command | shell command (Handlebars-rendered) |
| `commandWindows` | string | type:command | Windows override |
| `timeout` | integer (ms) | all | default per-provider |
| `onFailure` | enum: fail-open\|fail-closed | all | default fail-open |
| `env` | Record<string,string> | type:command | env vars for the spawned process |
| `loopLimit` | integer | stop/subagentStop | retry cap (cursor/crush/qwencode) |
| `statusMessage` | string | all | display text (codex/qwencode) |
| `async` | boolean | all | background execution (openhands/qwencode); dropped by template if unsupported |
| `prompt` | string | type:prompt | LLM evaluation prompt |
| `url` | string | type:http | webhook URL |
| `headers` | Record<string,string> | type:http | HTTP headers |
| `server` | string | type:mcp_tool | MCP server name (from mcp.jsonc) |
| `tool` | string | type:mcp_tool | MCP tool name |
| `arguments` | object | type:mcp_tool | tool args |
| `extra` | Record<string, any> | all | provider-specific passthrough |

**Rationale:** These 19 canonical fields cover ~95% of real-world hook usage. `loopLimit`, `statusMessage`, `async`, `commandWindows` are canonicalised because they're broadly useful concepts (not single-provider quirks). The 5 truly provider-specific fields (augment's `metadata` opt-in flags, qwencode's `once`/`allowedEnvVars`/`sequential`, antigravity's `injectSteps`) go in `extra`.

### D7: `enabled: false` drops the hook; `async: true` on unsupported provider drops the field

**Choice:**
- `enabled: false` → `HookFeature::to_value()` excludes the hook from all event groups. Hook is not deployed to any provider. Kept in source for reference.
- `async: true` on a provider whose template doesn't support async → template emits the hook without the `async` field. Hook runs synchronously on that provider. The user can write a separate non-async hook if they care about the distinction.

**Rationale:**
- `enabled: false` as "drop entirely" matches user intent ("I'm keeping this for reference but don't deploy it"). Per-provider disabled (claude/antigravity support `enabled: false` in-config) would require the `providers` block we rejected.
- `async: true` dropping to sync (rather than dropping the hook) preserves the hook's effect — a background linter that runs synchronously is still a linter; the user gets a degraded-but-functional hook rather than no hook.

### D8: Deploy pipeline slot — after mcp, before instructions

**Choice:** `commands → skills → mcp → hooks → instructions → ignore` in `deploy.rs`.

**Rationale:** Groups the structured-config features (mcp, hooks) together. Both use merge-aware deploy for embedded-file providers; both render structured JSON/TOML/JSONC. Instructions and ignore are simpler (single-file overwrite / symlink). No functional dependency on ordering — deploy features are independent — but the grouping aids readability and debug-log coherence.

### D9: Trust-hash warning at end of deploy

**Choice:** After all providers deploy, if any target provider is in the trust-hash set (`{codex, claude-code, trae}`), emit a one-line warning per provider: `codex: re-trust required — run /hooks in codex to review changed hooks`.

**Rationale:**
- codex, claude-code, trae, grok all trust-hash hooks and refuse new/changed ones until the user reviews via `/hooks`. dotagents re-deploying churns the hash.
- Silent re-deploy would leave the user confused why their hooks aren't firing. A one-liner at end-of-deploy is minimal noise.
- grok is skipped from v1, so the set is `{codex, claude-code, trae}`.
- The warning is informational, not an error — deploy still exits 0.

### D10: `HookFeature::to_value()` groups by event name

**Choice:** `to_value()` returns a JSON object keyed by event name, each value an array of hook entries (with `enabled: false` hooks excluded):

```json
{
  "PreToolUse": [
    { "name": "block-rm-rf", "type": "command", "command": "...", "timeout": 5000, ... },
    { "name": "audit", "type": "command", ... }
  ],
  "PostToolUse": [ ... ],
  "Interrupt": [ ... ]
}
```

**Rationale:** Templates iterate `{{#each PreToolUse}}` directly — no client-side grouping logic in Handlebars. Matches the deploy shape (one target file groups hooks by event). Enables template-as-filter (D4): a template without an `Interrupt` branch simply never iterates that key.

**Alternative considered:** Flat array `[{event, ...}, ...]` with templates grouping via `{{#filter}}` helper — rejected, requires a custom helper and pushes grouping logic into every template.

## Risks / Trade-offs

- **[Risk] `extra` field collisions across providers** → Mitigation: provider-specific field names are distinctive in practice (`loopLimit` vs `loop_limit` vs `loop-limit` — only cursor uses `loopLimit`; codex uses `loop_limit`). Document the collision risk in the schema description. If it bites, tighten `extra` to a per-provider subschema in a later change.
- **[Risk] Silent event dropping hides user mistakes** → Mitigation: a user who writes `event: "PreToolUse"` (typo'd as `PretoolUse`) gets silent drop on every provider. The canonical enum in the schema catches casing errors for the ~30 known events; only the open `String` escape hatch is unvalidated. Accept — the enum catches the common case, and the open hatch is for power users who can debug silent drops.
- **[Risk] Template-as-filter means no "does provider X support event Y" query at runtime** → Mitigation: the knowledge lives in the template source, which is public and inspectable. A `dotagents providers info <name>` command could surface supported events in a future change by introspecting the template; out of scope for v1.
- **[Risk] 18 per-provider templates is a large upfront authoring surface** → Mitigation: 14 of 18 are Claude Code-derived (same nested `{matcher,hooks:[{type,command,timeout}]}` shape). Author one "claude-family" base template and specialise via partials; the remaining 4 (cursor flattened, kimi TOML, copilot per-event-files, antigravity name-keyed) are the bespoke work.
- **[Risk] `async: true` dropping to sync changes hook semantics silently** → Mitigation: the user opted into `async` knowing it's provider-specific (it's in the canonical field list with a note). Templates that support async emit it; others don't. Documented in the schema description for `async`.
- **[Trade-off] No per-provider override of canonical fields** → a user who wants `timeout: 5000` for cursor but `timeout: 10000` for codex must write two hooks. Accept for v1 — per-provider override re-introduces the `providers` block we rejected.
- **[Trade-off] 12 providers skipped from v1** → documented on issue #141 with rationale. The 18 in-scope providers cover the Claude Code family (the dominant dialect) plus cursor, kimi, copilot, augment — the most-used providers. Skipped providers are either programmatic (require codegen), plugin-dir (require multi-file deploy), or fixed-path scripts (not config).

## Migration Plan

No migration — hooks is a new feature with no prior art in dotagents. Users who already maintain provider-native hook configs can adopt `.dotagents/hooks.jsonc` incrementally by adding `hooks` to their `features` array and a `[providers.<name>.hooks]` block per target; existing provider config files are preserved by merge-aware deploy (only the `hooks` key is touched).

## Open Questions

None blocking. All design decisions are resolved. The following are implementation-time refinements:
- Exact `timeout` default when omitted (per-provider vs one canonical default) — decide during `HookFeature` implementation; lean toward "no default in source, provider template applies its own default if field absent".
- Whether `dotagents init --no-hooks` is a separate flag or folds into an existing `--no-feature` pattern — check `init.rs` flag conventions during implementation.
- Whether the mock `hooks.jsonc` scaffolded by `init` should include one sample hook per handler type (command + prompt + http) or just one command hook — lean toward one command hook for simplicity.
