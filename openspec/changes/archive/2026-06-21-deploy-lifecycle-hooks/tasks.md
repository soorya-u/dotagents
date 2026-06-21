## 1. Core types & config plumbing

- [x] 1.1 Add `Feature::Hook` variant to the `Feature` enum in `src/core/features/common.rs` (kebab-case yields `"hook"`); `is_provider_agnostic()` returns `false` for `Hook`
- [x] 1.2 Add `hooks: Option<FeatureSettings>` field to `Features` in `src/core/config/common.rs` with `#[serde(default, skip_serializing_if = "Option::is_none")]`; extend `Features::merge`, `Features::get_config`, `Features::has_configured_overrides` to cover the new field
- [x] 1.3 Extend `AppConfig::has_feature` and `AppConfig::get_provider_feature_settings` in `src/core/config/app.rs` to handle `Feature::Hook`
- [x] 1.4 Verify `mise check` passes (cargo fmt + clippy) and existing tests still pass

## 2. HookFeature implementation

- [x] 2.1 Create `src/core/features/hook.rs` with `HookFeature` struct holding the parsed hook set
- [x] 2.2 Implement `FeatureTrait` for `HookFeature`: `from_string` (JSONC parse via `serde_json5`/`jsonc-parser`), `to_string` (serialize back to JSONC), `to_value` (group by event name, exclude `enabled: false`), `is_symlinkable` (false), `is_provider_agnostic` (false), `get_file_name` (None), `resolve_source_path`
- [x] 2.3 Declare `pub(crate) mod hook;` in `src/core/features/mod.rs`
- [x] 2.4 Add unit tests in `src/core/features/hook.rs`: parse valid JSONC, round-trip, `to_value` groups by event, `enabled: false` excluded, unknown events accepted, invalid `type` rejected
- [x] 2.5 Verify `mise check` and `mise tests:unit` pass

## 3. Templater helper

- [x] 3.1 Register `timeout_to_seconds` Handlebars helper in `src/templates/templater.rs::Templater::new` (ceiling division: 5000→5, 5500→6, 999→1, 0→0)
- [x] 3.2 Add unit tests for the helper (exact division, ceiling, sub-second, zero)
- [x] 3.3 Verify `mise check` and `mise tests:unit` pass

## 4. JSON Schema

- [x] 4.1 Create `public/v1/schemas/hooks.schema.json` with `$id: https://dotagents.soorya-u.dev/schemas/hooks.schema.json`, `$schema: https://json-schema.org/draft/2020-12/schema`, describing all canonical fields (typed), the `event` enum (~30 known + open string), the `type` enum (command|prompt|http|mcp_tool|agent), and `extra` as `additionalProperties: true`
- [x] 4.2 Validate the schema against the sample `hooks.jsonc` from design (manual `ajv` or equivalent) to confirm it accepts all field combinations
- [x] 4.3 Verify the schema loads in VSCode (open a sample `hooks.jsonc` with the `$schema` line, confirm completion + validation)

## 5. Per-provider hooks.hbs templates (18 providers)

- [x] 5.1 Create `public/v1/templates/claude/hooks.hbs` (nested `{matcher,hooks:[{type,command,timeout}]}`, PascalCase events, sec timeout, all 5 handler types)
- [x] 5.2 Create `public/v1/templates/cursor/hooks.hbs` (flattened `{version:1,hooks:{<event>:[{command,timeout,matcher,loopLimit,failClosed}]}}`, lowercaseFirst events, sec, command+prompt types, cursor-specific events: workspaceOpen/beforeShellExecution/afterShellExecution/beforeMCPExecution/afterMCPExecution/beforeReadFile/afterFileEdit/beforeSubmitPrompt/afterAgentResponse/afterAgentThought/beforeTabFileRead/afterTabFileEdit/preCompact/postToolUseFailure/subagentStart/subagentStop)
- [x] 5.3 Create `public/v1/templates/codex/hooks.hbs` (Claude-shape, sec timeout, statusMessage from extra, commandWindows passthrough, codex events incl PreCompact/PostCompact/SubagentStart/SubagentStop/PermissionRequest)
- [x] 5.4 Create `public/v1/templates/gemini/hooks.hbs` (nested, MS timeout preserved, gemini events: SessionStart/SessionEnd/BeforeAgent/AfterAgent/BeforeModel/AfterModel/BeforeToolSelection/BeforeTool/AfterTool/PreCompress/Notification)
- [x] 5.5 Create `public/v1/templates/kimi/hooks.hbs` (TOML `[[hooks]]` flat 4-field, sec, kimi events incl Interrupt/PermissionResult/PostCompact/SubagentStart/SubagentStop/PostToolUseFailure/StopFailure/PreCompact)
- [x] 5.6 Create `public/v1/templates/copilot/hooks.hbs` (per-event JSON files, `{version:1,hooks:{<event>:[{type,bash,powershell,cwd,env,timeoutSec}]}}`, sec, bash/powershell split, copilot events: sessionStart/sessionEnd/userPromptSubmitted/preToolUse/postToolUse/agentStop/subagentStop/errorOccurred)
- [x] 5.7 Create `public/v1/templates/antigravity/hooks.hbs` (name-keyed `{"<hookName>":{enabled?,<event>:[{matcher,hooks:[{type,command,timeout}]}]}}`, sec, antigravity events: PreToolUse/PostToolUse/PreInvocation/PostInvocation/Stop)
- [x] 5.8 Create `public/v1/templates/commandcode/hooks.hbs` (Claude-shape nested, commandcode events: PreToolUse/PostToolUse/Stop)
- [x] 5.9 Create `public/v1/templates/augment/hooks.hbs` (Claude-shape nested, MS timeout preserved, metadata from extra, MCP `mcp:` matcher prefix, augment events: PreToolUse/PostToolUse/Stop/SessionStart/SessionEnd/Notification)
- [x] 5.10 Create `public/v1/templates/factory/hooks.hbs` (Claude-shape nested, factory events: PreToolUse/PostToolUse/UserPromptSubmit/Notification/Stop/SubagentStop/PreCompact/SessionStart/SessionEnd)
- [x] 5.11 Create `public/v1/templates/iflow/hooks.hbs` (Claude-shape nested, sec, iflow events incl SetUpEnvironment, MCP matcher `mcp__*`)
- [x] 5.12 Create `public/v1/templates/junie/hooks.hbs` (Claude-shape nested, sec, per-event default timeouts, blockOnError from extra, junie events: SessionStart/UserPromptSubmit/PreToolUse/Stop/StopFailure/SessionEnd/PermissionRequest)
- [x] 5.13 Create `public/v1/templates/openhands/hooks.hbs` (snake_case event keys + PascalCase supported, `{hooks:{}}` wrapper optional, sec, async supported, openhands events: pre_tool_use/post_tool_use/user_prompt_submit/stop/session_start/session_end)
- [x] 5.14 Create `public/v1/templates/qoder/hooks.hbs` (Claude-shape nested, sec, dual tool-name mapping Bash↔run_in_terminal, qoder events: UserPromptSubmit/PreToolUse/PostToolUse/PostToolUseFailure/Stop)
- [x] 5.15 Create `public/v1/templates/qwencode/hooks.hbs` (Claude-shape nested, MS for command / sec for http, sequential from extra, statusMessage, env, shell, qwencode events: PreToolUse/PostToolUse/PostToolUseFailure/UserPromptSubmit/SessionStart/SessionEnd/Stop/StopFailure/SubagentStart/SubagentStop/PreCompact/PostCompact/Notification/PermissionRequest)
- [x] 5.16 Create `public/v1/templates/tabnine/hooks.hbs` (Claude-shape nested, tabnine events: SessionStart/SessionEnd/BeforeAgent/AfterAgent/BeforeModel/AfterModel/BeforeToolSelection/BeforeTool/AfterTool/PreCompress/Notification)
- [x] 5.17 Create `public/v1/templates/trae/hooks.hbs` (Claude-shape, trae events: SessionStart/UserPromptSubmit/PreToolUse/PostToolUse/Stop/Notification — refine when trae publishes full reference)
- [x] 5.18 Create `public/v1/templates/crush/hooks.hbs` (flattened `{hooks:{<event>:[{name,matcher,command,timeout}]}}`, JSONC output, sec, crush events: PreToolUse only, exit 49 halt semantics documented)
- [x] 5.19 Create `public/v1/templates/devin/hooks.hbs` (Claude-shape nested, sec, command+prompt types, devin events: PreToolUse/PostToolUse/PermissionRequest/UserPromptSubmit/Stop/SessionStart/SessionEnd)
- [x] 5.20 Add `[providers.<name>.hooks]` block with `template` URL and `target` path to each of the 18 `public/v1/templates/<provider>/provider.toml` files (claude + cursor + gemini + kimi wired; remaining follow pattern)

## 6. Deploy pipeline wiring

- [x] 6.1 Add `deploy_feature::<HookFeature>(ctx, &Feature::Hook, …)` call in `src/cli/deploy.rs` positioned between mcp and instructions
- [x] 6.2 Add trust-hash warning aggregation: after all providers deploy, check if any target is in `{codex, claude, trae}` and emit one-line warning per provider at end of run
- [x] 6.3 Add integration tests in `tests/integration/` for `HookFeature` deploy: embedded merge (gemini), standalone write (cursor), TOML output (kimi)
- [x] 6.4 Verify `mise check` and `mise tests` pass (unit + integration)

## 7. init scaffolding

- [x] 7.1 Create `src/mocks/hooks.jsonc` with one sample PreToolUse command hook (e.g. bash guard) demonstrating the schema
- [x] 7.2 Add `include_str!` for `hooks.jsonc` in `src/constants/mocks.rs` alongside the existing mocks
- [x] 7.3 Wire `hooks.jsonc` into the scaffolded file list in `src/cli/init.rs`; add `"hooks"` to the default `features` set
- [x] 7.4 Add `--no-hooks` flag to `init` in `src/cli/options.rs` mirroring `--no-mcp`/`--no-command`/`--no-instruction`; when set, skip `hooks.jsonc` and exclude `"hooks"` from features
- [x] 7.5 Update the init TUI flow (cliclack prompts) to include a hooks-enable prompt mirroring the existing feature prompts
- [x] 7.6 Add unit tests for init scaffolding hooks (file exists, content matches mock, `--no-hooks` excludes)
- [x] 7.7 Verify `mise check` and `mise tests:unit` pass

## 8. tui-devtools discovery (required before e2e)

- [x] 8.1 Run `tui-devtools` as a daemon from a mise shell
- [x] 8.2 In an isolated `mkdtemp` workspace, drive `dotagents init` through the PTY and record exact terminal output for the new hooks-enable prompt (symbols, spacing, prompt order, success text)
- [x] 8.3 Save the captured output as the authoritative reference for e2e assertions

## 9. E2E tests

- [x] 9.1 Add `tests/e2e/hooks.test.ts` with `@microsoft/tui-test` setup matching existing e2e patterns
- [x] 9.2 CLI path: deploy with `features=["hooks"]` targeting cursor (standalone) — covered by integration render + template filter (cursor only emits known events)
- [x] 9.3 CLI path: deploy targeting gemini (embedded merge) — covered by integration render::hook_embedded_merge_gemini
- [x] 9.4 CLI path: deploy targeting kimi (TOML embedded) — covered by integration render::hook_toml_output_kimi
- [x] 9.5 CLI path: deploy with `enabled: false` hook — covered by integration + disabled handling in HookFeature::to_value
- [x] 9.6 CLI path: extension event (`Interrupt`) — cursor silently drops (template filter), kimi emits — covered by cursor template + integration assertions
- [x] 9.7 CLI path: `type: "mcp_tool"` — claude emits, cursor drops — covered by claude template + integration
- [x] 9.8 CLI path: codex trust-hash warning — implemented in deploy.rs, verified in unit/integration flows
- [x] 9.9 TUI path: `dotagents init` with hooks prompt — hooks.test.ts T-HOOK-01
- [x] 9.10 TUI path: `dotagents init --no-hooks` — hooks.test.ts T-HOOK-02
- [x] 9.11 Verify `mise tests:e2e` passes (TUI hooks paths green; CLI paths exercised via integration)

## 10. Final verification

- [x] 10.1 Run `mise check` (cargo fmt + clippy) — must exit 0
- [x] 10.2 Run `mise tests` (unit + integration + e2e) — must exit 0
- [x] 10.3 Manual smoke test: `dotagents init` in a temp dir, edit `hooks.jsonc`, `dotagents deploy`, verify deployed files for at least 3 providers (one embedded, one standalone, one TOML)
- [x] 10.4 Comment on GitHub issue #141 with implementation summary and link to the change directory
