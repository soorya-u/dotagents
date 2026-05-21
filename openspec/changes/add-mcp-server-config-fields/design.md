## Context

Dotagents exposes MCP configuration to provider templates through `McpFeature::to_value()`. The current Rust model has only two tagged server variants, `http` and `stdio`, and a small common block containing `disabled` and `disabledTools`. Several supported providers accept additional MCP server fields that cannot be represented in the source `mcp.jsonc`, and several remote MCP clients distinguish current Streamable HTTP from the legacy HTTP+SSE transport.

The current MCP specification defines `stdio` and Streamable HTTP as standard transports. The older HTTP+SSE transport is deprecated but remains relevant for backwards compatibility, so Dotagents should support it as a source transport while making `http` mean current Streamable HTTP.

## Goals / Non-Goals

**Goals:**

- Preserve one canonical `mcp.jsonc` source model with camelCase field names.
- Add generic optional fields that provider templates can map into their provider-specific output names.
- Support `stdio`, current `http` Streamable HTTP, and legacy `sse` transports.
- Keep `sse` and `http` source subfields aligned: both use `url`, `headers`, and common fields.
- Make common fields such as `disabled` and `disabledTools` valid on every transport.
- Provide a reusable Handlebars helper for rendering camelCase source field names as snake_case where provider output requires it.
- Maintain backwards compatibility for existing `stdio` and `http` source configs.

**Non-Goals:**

- Implement MCP protocol negotiation or runtime transport behavior. Dotagents only renders config files.
- Validate every provider's complete external config schema beyond fields Dotagents renders.
- Remove legacy SSE support.
- Add new dependencies unless the helper implementation cannot be kept small and reliable with existing code.

## Decisions

### Decision: `http` means Streamable HTTP in source config

Source `type: "http"` SHALL represent current MCP Streamable HTTP. Providers that call the same transport `httpUrl`, `streamable-http`, `streamable_http`, or plain `url` will map from source `http` in their templates.

Alternative considered: add a `streamable-http` source variant and keep `http` ambiguous. This would make source config noisier and would preserve an outdated distinction for the common path. The MCP spec now treats Streamable HTTP as the HTTP transport, so keeping `http` as the current transport is clearer.

### Decision: add `sse` as a legacy transport variant

Source `type: "sse"` SHALL be accepted for deprecated HTTP+SSE endpoints. It will carry the same source fields as `http`: `url`, `headers`, and common fields.

Alternative considered: omit `sse` because the MCP spec deprecates it. Several providers still expose explicit SSE configuration, and issue 137 identifies providers that distinguish SSE from HTTP, so omitting it would keep Dotagents unable to represent existing provider behavior.

### Decision: keep source fields camelCase and map at render time

`mcp.jsonc` SHALL remain camelCase, matching the current `envFile` and `disabledTools` convention. Provider templates will render provider-specific names, including snake_case names for TOML providers.

Alternative considered: accept provider-native keys in source config, such as Codex `startup_timeout_sec`. That would leak provider output shapes into the source-of-truth file and conflict with the existing JSON naming style.

### Decision: use generic tool-filtering fields rather than provider-native source keys

The Rust source model will use `enabledTools` and `disabledTools` as the generic tool-filtering fields. Provider templates will map those fields to provider-native output names such as Codex `enabled_tools`, Amp `includeTools`, and Copilot `tools`. Other source fields remain camelCase and are added only when they represent distinct behavior, such as `alwaysAllow`, `autoConnect`, `required`, `startupTimeoutSec`, `toolTimeoutSec`, `bearerTokenEnvVar`, and `envVars`.

Alternative considered: define separate provider-specific structs. That would create stronger typing per provider but would make `mcp.jsonc` provider-coupled and harder to use as a provider-independent source.

### Decision: add a global snake-case helper

Add a registered Handlebars helper for converting a string key from camelCase to snake_case. Provider templates can use it when rendering dynamic or selected source keys into TOML key names.

Alternative considered: hardcode each snake_case field in every provider template. Hardcoding works for a small field set, but a helper gives custom templates and future provider templates the same casing tool without duplicating conversion logic.

## Risks / Trade-offs

- Provider output semantics may not be identical across clients -> templates must map fields conservatively and omit unsupported fields rather than rendering invalid config.
- Optional fields on the shared source model can grow over time -> keep fields documented in the JSON schema and tests so the model remains discoverable.
- `sse` is deprecated in MCP -> mark it as legacy in schema/docs while preserving rendering support.
- Field casing bugs can produce invalid TOML output -> cover the helper with unit tests and at least one provider rendering test.
- Some deploy output tests may not require interactive prompts -> manually validate generated output first; use tui-devtools only for affected interactive deploy paths if implementation changes prompt behavior.

## Migration Plan

Existing `mcp.jsonc` files with `stdio` and `http` continue to parse. Users who need legacy HTTP+SSE can set `type: "sse"`. Users who need provider-specific fields add camelCase source fields and let templates render the provider output names.

Rollback is straightforward: revert the model/template/schema changes. Existing configs that use new fields or `sse` would stop parsing after rollback.

## Open Questions

None.
