## ADDED Requirements

### Requirement: Integration tests cover expanded MCP server parsing

`tests/integration/features.rs` SHALL cover parsing and round-tripping expanded MCP server configs without spawning the `dotagents` binary.

#### Scenario: McpFeature parses legacy SSE server
- **WHEN** `mcp.jsonc` declares a server with `type: "sse"`, `url`, and `headers`
- **THEN** `McpFeature::from_string` returns a value with the server present and classified as SSE

#### Scenario: McpFeature round-trips expanded optional fields
- **WHEN** `mcp.jsonc` declares servers with `disabled`, `disabledTools`, `enabledTools`, `required`, `startupTimeoutSec`, `toolTimeoutSec`, `bearerTokenEnvVar`, `envVars`, `alwaysAllow`, and `autoConnect`
- **THEN** `McpFeature::to_string` emits those fields using camelCase names and the emitted config parses again

### Requirement: Integration tests cover expanded MCP provider rendering

`tests/integration/render.rs` SHALL cover rendering expanded MCP server configs through representative provider templates without spawning the `dotagents` binary.

#### Scenario: renders expanded MCP fields to TOML provider output
- **WHEN** an expanded MCP config is rendered through a TOML MCP provider template
- **THEN** the output contains provider-compatible field names and values, including snake_case fields where required

#### Scenario: renders expanded MCP fields to JSON provider output
- **WHEN** an expanded MCP config is rendered through a JSON MCP provider template
- **THEN** the output contains provider-compatible field names and values for generic tool filters, disabled state, and transport type

### Requirement: E2E tests cover deploy output for expanded MCP config

`tests/e2e/deploy.test.ts` SHALL cover user-visible deploy output for expanded MCP config fields.

#### Scenario: deploy writes expanded MCP output
- **WHEN** a workspace MCP config includes expanded fields and `dotagents deploy --offline --no-gitignore` runs against representative providers
- **THEN** deploy exits 0 and writes provider MCP files containing the expected mapped fields

#### Scenario: manual validation is performed before E2E assertions
- **WHEN** new E2E assertions are added for expanded MCP deploy output
- **THEN** the expected output is based on manually generated deploy output rather than source-reading alone
