## Purpose

TBD

## Requirements

### Requirement: Hooks source file in JSONC format
The system SHALL read lifecycle hook definitions from a single `.dotagents/hooks.jsonc` source file. The file SHALL be JSONC (JSON with comments), parsed via the existing `merge_jsonc`/`serde_json5` path used by `mcp.jsonc`. The file SHALL NOT require a `version` field — the `$schema` URL is the version anchor.

#### Scenario: Valid hooks.jsonc with one command hook
- **WHEN** `.dotagents/hooks.jsonc` contains `{"$schema":"...","hooks":[{"name":"block-rm","event":"PreToolUse","type":"command","command":"./x.sh"}]}`
- **THEN** the system SHALL parse it successfully and expose one hook entry for deploy

#### Scenario: JSONC comments preserved during parse
- **WHEN** `.dotagents/hooks.jsonc` contains `// security: blocks rm -rf` on its own line before a hook entry
- **THEN** the system SHALL parse the file without error and exclude the comment from the parsed hook data

#### Scenario: Missing hooks.jsonc when feature enabled
- **WHEN** `config.toml` has `features = ["hooks"]` and `.dotagents/hooks.jsonc` does not exist
- **THEN** deploy SHALL log a warning naming the missing file and skip the hooks feature for all providers (other features proceed)

#### Scenario: Malformed hooks.jsonc skips feature
- **WHEN** `.dotagents/hooks.jsonc` contains invalid JSONC (e.g. trailing comma in strict JSON mode, unclosed brace)
- **THEN** deploy SHALL log a warning with the parse error and skip the hooks feature for all providers (other features proceed)

### Requirement: Canonical hook schema with typed fields and extra passthrough
The system SHALL parse each hook entry against a canonical schema with the following typed fields: `name` (string), `description` (string, optional), `event` (string — enum of ~30 known events plus open string for provider extensions), `matcher` (string, optional), `enabled` (boolean, default true), `type` (enum: `command` | `prompt` | `http` | `mcp_tool` | `agent`), `command` (string, for type:command), `commandWindows` (string, optional, for type:command), `timeout` (integer, milliseconds), `onFailure` (enum: `fail-open` | `fail-closed`, default `fail-open`), `env` (object of string→string, optional), `loopLimit` (integer, optional), `statusMessage` (string, optional), `async` (boolean, default false), `prompt` (string, for type:prompt), `url` (string, for type:http), `headers` (object of string→string, for type:http), `server` (string, for type:mcp_tool), `tool` (string, for type:mcp_tool), `arguments` (object, for type:mcp_tool), `extra` (object, optional — untyped passthrough bag for provider-specific fields). A JSON Schema SHALL be published at `public/v1/schemas/hooks.schema.json` with `$id: https://dotagents.soorya-u.dev/schemas/hooks.schema.json`.

#### Scenario: Command hook with all canonical fields
- **WHEN** a hook entry has `{"name":"x","event":"PreToolUse","type":"command","command":"./x.sh","timeout":5000,"onFailure":"fail-closed","env":{"LOG":"warn"},"matcher":"Bash","loopLimit":5,"statusMessage":"checking","async":false,"enabled":true}`
- **THEN** the system SHALL accept it and expose all fields to templates

#### Scenario: Hook with extra passthrough bag
- **WHEN** a hook entry has `{"name":"x","event":"PreToolUse","type":"command","command":"./x.sh","extra":{"loopLimit":10,"failClosed":true,"metadata":{"includeUserContext":false}}}`
- **THEN** the system SHALL accept it and expose `extra` as an opaque object to templates; templates SHALL read individual keys via `{{lookup extra "loopLimit"}}` and ignore unrecognised keys

#### Scenario: Unknown event name accepted
- **WHEN** a hook entry has `{"name":"x","event":"Interrupt","type":"command","command":"./x.sh"}`
- **THEN** the system SHALL accept it (event is an open string) and group it under the `Interrupt` key for templates to filter

#### Scenario: Invalid type value rejected
- **WHEN** a hook entry has `"type":"webhook"` (not in the command|prompt|http|mcp_tool|agent enum)
- **THEN** the system SHALL reject the file at parse time with a validation error naming the invalid type

### Requirement: HookFeature implements FeatureTrait
The system SHALL implement `HookFeature` in `src/core/features/hook.rs` implementing `FeatureTrait` (`src/core/features/traits.rs`). `from_string` SHALL parse JSONC. `to_string` SHALL serialize back to JSONC. `to_value` SHALL return a JSON object keyed by event name, each value an array of hook entries (with `enabled: false` entries excluded). `is_symlinkable` SHALL return false (hooks are dialect-specific). `is_provider_agnostic` SHALL return false. `get_file_name` SHALL return `None` (hooks deploy as a set to one target, not per-item).

#### Scenario: to_value groups by event name
- **WHEN** `hooks.jsonc` has hooks for `PreToolUse` (2 hooks), `PostToolUse` (1 hook), and `Interrupt` (1 hook)
- **THEN** `HookFeature::to_value()` SHALL return `{"PreToolUse":[hook1,hook2],"PostToolUse":[hook3],"Interrupt":[hook4]}`

#### Scenario: enabled false excluded from to_value
- **WHEN** `hooks.jsonc` has 3 hooks, one with `"enabled": false`
- **THEN** `to_value()` SHALL include only the 2 enabled hooks in their respective event groups; the disabled hook SHALL NOT appear in any event group

#### Scenario: Round-trip from_string to_string
- **WHEN** a valid `hooks.jsonc` is parsed via `from_string` then serialised via `to_string`
- **THEN** the output SHALL parse back to the same hook data (comments may be lost; field values preserved)

### Requirement: Hooks feature added to Feature enum and Features struct
The system SHALL add `Feature::Hook` to the `Feature` enum (`src/core/features/common.rs`) with `strum(serialize_all = "kebab-case")` yielding `"hook"`. The `Features` struct (`src/core/config/common.rs`) SHALL gain a `hooks: Option<FeatureSettings>` field with `#[serde(default, skip_serializing_if = "Option::is_none")]`. The `Features::merge`, `Features::get_config`, and `Features::has_configured_overrides` methods SHALL cover the new field with the same semantics as existing features (deep-merge, match-by-feature-variant, OR across features).

#### Scenario: config.toml with hooks feature enabled
- **WHEN** `config.toml` contains `features = ["hooks"]` and `[providers.cursor.hooks] template = "..." target = "..."`
- **THEN** `AppConfig::has_feature(Feature::Hook)` SHALL return true and `get_provider_feature_settings(Feature::Hook)` SHALL return the cursor settings

#### Scenario: Local config deep-merges hooks settings
- **WHEN** global has `[providers.cursor.hooks] template = "a"` and local has `[providers.cursor.hooks] target = "b"`
- **THEN** the merged config SHALL have `template = "a"` (from global) and `target = "b"` (from local)

#### Scenario: config.toml without hooks unaffected
- **WHEN** `config.toml` does not mention `hooks` in features or providers
- **THEN** the system SHALL parse and deploy as before; no hooks feature is enabled

### Requirement: Hooks deploy in pipeline between mcp and instructions
The system SHALL deploy hooks via `deploy_feature::<HookFeature>(ctx, &Feature::Hook, …)` in `src/cli/deploy.rs`, positioned between mcp and instructions in the deploy sequence. The deploy SHALL reuse the existing `deploy_feature` machinery and `merge_into_existing` for embedded-file providers. Hooks SHALL deploy in parallel with other features via `rayon::par_iter` per provider.

#### Scenario: Embedded-file provider merge-aware deploy
- **WHEN** deploy targets `gemini` with `[providers.gemini.hooks] target = "{{dir.workspace}}/.gemini/settings.json"` and the file already exists with `{"model":"gemini-2.5"}`
- **THEN** the system SHALL merge the rendered hooks under the `hooks` key, preserving `model` and any other existing keys

#### Scenario: Standalone-file provider direct write
- **WHEN** deploy targets `cursor` with `[providers.cursor.hooks] target = "{{dir.workspace}}/.cursor/hooks.json"` and the file does not exist
- **THEN** the system SHALL write the rendered hooks as a new file with no merge

#### Scenario: Hooks deploy between mcp and instructions
- **WHEN** deploy runs with features `[commands, mcp, hooks, instructions]`
- **THEN** the deploy sequence SHALL be commands → skills → mcp → hooks → instructions → ignore (hooks positioned after mcp, before instructions)

### Requirement: Per-provider hooks.hbs template lowers canonical data to dialect
The system SHALL ship a `hooks.hbs` template per in-scope provider under `public/v1/templates/<provider>/hooks.hbs`. Each template SHALL lower canonical hook data to the provider's dialect: event-name casing, timeout unit conversion (ms→sec via a `timeout_to_seconds` Handlebars helper for sec-providers; ms-pass-through for ms-providers), decision-vocab rewriting, schema shape (nested `{matcher,hooks:[…]}` vs flattened), file format (JSON/TOML/JSONC), embed-vs-standalone wrapper, and env-var alias injection. Each template SHALL emit only events it supports — events without a template branch SHALL be silently dropped (no warning).

#### Scenario: Cursor template flattens and converts timeout to seconds
- **WHEN** a canonical hook has `{"event":"PreToolUse","timeout":5500,"command":"./x.sh"}` and deploy targets cursor
- **THEN** the cursor template SHALL emit `{"version":1,"hooks":{"preToolUse":[{"command":"./x.sh","timeout":6}]}}` (ceiling division 5500/1000=6, flattened shape, lowercase-first event name)

#### Scenario: Kimi template emits TOML array-of-tables
- **WHEN** a canonical hook has `{"event":"PreToolUse","matcher":"Bash","command":"./x.sh","timeout":5000}` and deploy targets kimi
- **THEN** the kimi template SHALL emit `[[hooks]]\nevent = "PreToolUse"\nmatcher = "Bash"\ncommand = "./x.sh"\ntimeout = 5` (TOML, seconds, flat 4-field shape)

#### Scenario: Gemini template emits nested shape with ms timeout
- **WHEN** a canonical hook has `{"event":"BeforeTool","timeout":5000,"command":"./x.sh"}` and deploy targets gemini
- **THEN** the gemini template SHALL emit `{"hooks":{"BeforeTool":[{"matcher":"...","hooks":[{"type":"command","command":"./x.sh","timeout":5000}]}}]}` (nested shape, ms preserved, gemini's `BeforeTool` event name)

#### Scenario: Extension event silently dropped by non-supporting template
- **WHEN** a canonical hook has `{"event":"Interrupt","command":"./x.sh"}` and deploy targets cursor (whose template has no `Interrupt` branch)
- **THEN** the cursor template SHALL emit no entry for `Interrupt`; deploy SHALL NOT log a warning

### Requirement: timeout_to_seconds Handlebars helper
The system SHALL register a `timeout_to_seconds` Handlebars helper in `src/templates/templater.rs` that takes a millisecond integer and returns the ceiling division by 1000 (e.g. 5000→5, 5500→6, 999→1, 0→0). The helper SHALL be available to all `hooks.hbs` templates.

#### Scenario: Exact division
- **WHEN** a template calls `{{timeout_to_seconds 5000}}`
- **THEN** the helper SHALL return `5`

#### Scenario: Ceiling on non-exact division
- **WHEN** a template calls `{{timeout_to_seconds 5500}}`
- **THEN** the helper SHALL return `6`

#### Scenario: Sub-second rounds up to 1
- **WHEN** a template calls `{{timeout_to_seconds 999}}`
- **THEN** the helper SHALL return `1`

### Requirement: Template-as-filter for event support
The system SHALL NOT require a `hooks_events` field in `provider.toml`. Each provider's `hooks.hbs` template SHALL be the sole filter for which events that provider supports: the template has `{{#each <EventName>}}` branches only for supported events; events without a branch are silently dropped from that provider's output.

#### Scenario: Canonical event present in all templates
- **WHEN** a canonical hook has `event: "PreToolUse"` and deploy targets any in-scope provider
- **THEN** every provider's `hooks.hbs` SHALL have a `PreToolUse` branch and emit the hook

#### Scenario: Provider-specific event only in supporting template
- **WHEN** a canonical hook has `event: "workspaceOpen"` and deploy targets cursor and kimi
- **THEN** the cursor template SHALL emit the hook (it has a `workspaceOpen` branch); the kimi template SHALL silently drop it (no branch)

### Requirement: enabled false drops hook from deploy
The system SHALL exclude any hook with `"enabled": false` from `HookFeature::to_value()`. The hook SHALL NOT be deployed to any provider. The hook SHALL be preserved in the source file for reference.

#### Scenario: Disabled hook not in any provider output
- **WHEN** `hooks.jsonc` has a hook with `"enabled": false` and deploy targets 3 providers
- **THEN** none of the 3 deployed provider files SHALL contain an entry for that hook

### Requirement: async true dropped by non-supporting templates
The system SHALL NOT reject `async: true` on a hook deployed to a provider whose template doesn't support async. The template SHALL emit the hook without the `async` field; the hook SHALL run synchronously on that provider.

#### Scenario: async true on openhands (supports async)
- **WHEN** a hook has `"async": true` and deploy targets openhands
- **THEN** the openhands template SHALL emit the hook with `"async": true`

#### Scenario: async true on cursor (no async support)
- **WHEN** a hook has `"async": true` and deploy targets cursor
- **THEN** the cursor template SHALL emit the hook without any `async` field; the hook SHALL run synchronously on cursor

### Requirement: Handler type dropped by non-supporting templates
The system SHALL NOT reject a hook whose `type` is unsupported by a target provider. The template SHALL silently skip hooks of unsupported types for that provider. A single `hooks.jsonc` SHALL be deployable to all 18 in-scope providers without manual filtering.

#### Scenario: mcp_tool hook deployed to claude-code and cursor
- **WHEN** a hook has `"type": "mcp_tool"` and deploy targets both claude-code and cursor
- **THEN** the claude-code template SHALL emit the mcp_tool hook; the cursor template SHALL silently skip it (cursor doesn't support mcp_tool)

#### Scenario: http hook deployed to qwencode and kimi
- **WHEN** a hook has `"type": "http"` and deploy targets both qwencode and kimi
- **THEN** the qwencode template SHALL emit the http hook; the kimi template SHALL silently skip it (kimi doesn't support http)

### Requirement: Trust-hash warning at end of deploy
The system SHALL emit a one-line warning at the end of deploy for each target provider that trust-hashes hooks (the set `{codex, claude-code, trae}`). The warning SHALL name the provider and instruct the user to re-trust via the provider's `/hooks` command. The warning SHALL be informational; deploy SHALL still exit 0.

#### Scenario: Deploy to codex triggers re-trust warning
- **WHEN** deploy targets codex and at least one hook was written or changed
- **THEN** deploy SHALL print `codex: re-trust required — run /hooks in codex to review changed hooks` once at end of run

#### Scenario: Deploy to non-trust-hash provider no warning
- **WHEN** deploy targets cursor (not in the trust-hash set) and hooks are written
- **THEN** deploy SHALL NOT print a trust-hash warning for cursor

### Requirement: JSON Schema published for editor intellisense
The system SHALL publish a JSON Schema at `public/v1/schemas/hooks.schema.json` with `$id: https://dotagents.soorya-u.dev/schemas/hooks.schema.json` describing the canonical hook schema. The `extra` field SHALL be `additionalProperties: true` (loose) for v1. Users SHALL reference it via `"$schema": "https://dotagents.soorya-u.dev/schemas/hooks.schema.json"` on line 1 of `hooks.jsonc` for native intellisense in VSCode/JetBrains/Zed/Neovim without extensions.

#### Scenario: VSCode validates hooks.jsonc
- **WHEN** a user opens `.dotagents/hooks.jsonc` in VSCode with the `$schema` line present
- **THEN** VSCode SHALL provide completion for canonical fields and validation against the schema without requiring any extension

#### Scenario: extra field accepts any keys
- **WHEN** a hook entry has `"extra": {"loopLimit": 10, "failClosed": true, "anyCustomKey": "value"}`
- **THEN** the schema SHALL validate it (additionalProperties: true); no warning SHALL be raised by the schema for unknown extra keys

### Requirement: init scaffolds mock hooks.jsonc
The system SHALL include a mock `hooks.jsonc` in the files scaffolded by `dotagents init`. The mock SHALL contain one sample command hook (e.g. a PreToolUse bash guard) to demonstrate the schema. The `hooks` feature SHALL be included in the default `features` set scaffolded by `init`. A `--no-hooks` flag SHALL be added to `init` mirroring `--no-mcp`/`--no-command`/`--no-instruction` to exclude hooks from the scaffold.

#### Scenario: init scaffolds hooks.jsonc
- **WHEN** `dotagents init` runs without `--no-hooks`
- **THEN** `.dotagents/hooks.jsonc` SHALL exist with one sample command hook and `config.toml` SHALL include `"hooks"` in the `features` array

#### Scenario: init --no-hooks skips hooks
- **WHEN** `dotagents init --no-hooks` runs
- **THEN** `.dotagents/hooks.jsonc` SHALL NOT be created and `config.toml` SHALL NOT include `"hooks"` in `features`

### Requirement: TUI-devtools discovery before e2e tests
The implementation SHALL perform a tui-devtools discovery pass for the `init` TUI flow with the new `--no-hooks` prompt before writing e2e tests. The discovery SHALL record exact terminal output (prompt text, ordering, symbols) for the hooks-related prompt. E2E test assertions for the TUI path SHALL be written from these observations, not from source-reading alone.

#### Scenario: tui-devtools captures init hooks prompt
- **WHEN** the implementer runs `dotagents init` through tui-devtools in an isolated temp workspace
- **THEN** the discovery SHALL capture the exact prompt text and order for the hooks-enable question, and the e2e test SHALL assert on that captured text

### Requirement: E2E tests for hooks deploy
The system SHALL add an e2e test suite at `tests/e2e/hooks.test.ts` covering: (a) CLI deploy with `features = ["hooks"]` targeting at least one embedded-file provider (e.g. claude-code or gemini) and one standalone-file provider (e.g. cursor), asserting on deployed file content and merge behavior; (b) CLI deploy with `--no-hooks` equivalent (init path) excluding hooks; (c) the TUI init flow with the hooks prompt. Tests SHALL use `@microsoft/tui-test` and `getByText` for semantic assertions.

#### Scenario: E2E deploys hooks to cursor
- **WHEN** the e2e test runs `dotagents deploy` with a `hooks.jsonc` containing one PreToolUse command hook and targets `["cursor"]`
- **THEN** the test SHALL assert that `.cursor/hooks.json` exists and contains the hook entry with cursor's flattened schema and lowercase-first event name

#### Scenario: E2E deploys hooks to gemini with merge
- **WHEN** the e2e test pre-creates `.gemini/settings.json` with `{"model":"gemini-2.5"}` then runs `dotagents deploy` with a hooks.jsonc and targets `["gemini"]`
- **THEN** the test SHALL assert that `.gemini/settings.json` contains both `model` (preserved) and `hooks` (newly merged)
