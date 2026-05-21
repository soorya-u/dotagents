## Why

`ServerConfig` cannot currently represent several MCP server fields used by supported providers, and it only models `stdio` and `http` transport output narrowly. This prevents `mcp.jsonc` from being a single source of truth for provider-specific MCP settings such as tool filters, timeout controls, disabled tools, and legacy SSE endpoints.

## What Changes

- Extend the source `mcp.jsonc` server model with camelCase optional fields for shared MCP metadata and provider-rendered options, including tool filters, Codex timeout/auth fields, Cline auto-approval, Autohand connection behavior, and common disabled state.
- Treat source `type: "http"` as the current MCP Streamable HTTP transport.
- Add source `type: "sse"` for legacy/deprecated HTTP+SSE transport, with the same source subfields as `http`: `url`, `headers`, and common fields.
- Make common fields such as `disabled` and `disabledTools` available on every transport variant.
- Add a Handlebars helper that converts camelCase keys to snake_case so TOML providers can render source fields such as `startupTimeoutSec` as `startup_timeout_sec`.
- Update provider MCP templates to map the generic source fields and transports to each provider's expected output names and transport labels.
- Update schema and tests so the expanded model is validated, parsed, serialized, and rendered correctly.

## Capabilities

### New Capabilities

- `mcp-server-config-model`: Covers the expanded `mcp.jsonc` source model for MCP server fields, common fields, and `stdio`/`http`/`sse` transport parsing.
- `mcp-provider-template-rendering`: Covers provider template behavior for rendering the expanded MCP model, including transport-specific output and field name mapping.

### Modified Capabilities

- `template-helpers`: Adds a globally registered helper for converting camelCase field names to snake_case during template rendering.
- `rust-integration-tests`: Expands MCP feature parsing and rendering coverage to include the new fields and legacy SSE transport.

## Impact

- `src/core/features/mcp.rs` — add optional fields, common transport fields, and the `sse` variant.
- `public/v1/schemas/mcp.schema.json` — document and validate the expanded source `mcp.jsonc` model.
- `src/templates/helpers.rs`, `src/templates/templater.rs`, `src/constants/helpers.rs` — add and register the camelCase-to-snake_case helper.
- `public/v1/templates/*/mcp.hbs` and `src/constants/mocks.rs` — render provider-specific MCP fields and transport labels.
- `tests/integration/`, `tests/e2e/`, and colocated Rust unit tests — cover parsing, helper behavior, provider rendering, and deploy output.
- No dependency changes expected.
- Testing approach: manually validate affected deploy output first, add focused unit/integration/e2e coverage, then run `mise check` and `mise tests`.
- Reference: GitHub issue https://github.com/soorya-u/dotagents/issues/137.
