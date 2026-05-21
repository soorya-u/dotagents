## 1. MCP source model and schema

- [x] 1.1 Extend `CommonConfig` and `ServerConfig` in `src/core/features/mcp.rs` with common fields, expanded optional fields, and a legacy `sse` remote variant
- [x] 1.2 Keep `mcp.jsonc` serialization/deserialization camelCase and ensure unset optional fields are omitted where appropriate
- [x] 1.3 Update `public/v1/schemas/mcp.schema.json` to validate `stdio`, `http`, `sse`, common fields, and expanded optional fields
- [x] 1.4 Add colocated unit tests for parsing, serializing, and round-tripping expanded MCP configs

## 2. Template helper support

- [x] 2.1 Add a `snake-case` helper implementation in `src/templates/helpers.rs`
- [x] 2.2 Register the helper in `Templater::new` and add the helper constant in `src/constants/helpers.rs`
- [x] 2.3 Add helper unit tests for camelCase, PascalCase, existing snake_case, and non-string error behavior

## 3. Provider MCP template rendering

- [x] 3.1 Update Codex MCP template to render expanded fields with provider-compatible snake_case TOML names
- [x] 3.2 Update providers that distinguish remote transports to map source `http` to current Streamable HTTP output and source `sse` to legacy SSE output
- [x] 3.3 Update JSON provider templates to map `enabledTools`, `disabledTools`, and `disabled` to provider-compatible field names
- [x] 3.4 Update provider templates for supported behavior fields such as `alwaysAllow` and `autoConnect`
- [x] 3.5 Update starter/custom-provider MCP mocks only where needed to keep generated examples valid

## 4. Manual validation before automated tests

- [x] 4.1 Test with tui-devtools applicability: confirm the implementation does not change interactive prompt behavior; if it does, run tui-devtools discovery before adding prompt assertions
- [x] 4.2 Manually generate deploy output for representative TOML and JSON providers using expanded MCP config
- [x] 4.3 Inspect generated output for `stdio`, `http`, `sse`, common fields, tool filters, and provider-rendered optional fields

## 5. Automated tests

- [x] 5.1 Add integration tests in `tests/integration/features.rs` for expanded MCP parsing and round-trip behavior
- [x] 5.2 Add integration tests in `tests/integration/render.rs` for representative TOML and JSON MCP template rendering
- [x] 5.3 Add e2e deploy coverage in `tests/e2e/deploy.test.ts` for expanded MCP output written by representative providers
- [x] 5.4 Ensure schema validation coverage includes valid expanded configs and invalid transport-specific configs

## 6. Verification

- [x] 6.1 Run `mise check` and fix any formatting or lint failures
- [x] 6.2 Run `mise tests` and fix any unit, integration, or e2e failures
