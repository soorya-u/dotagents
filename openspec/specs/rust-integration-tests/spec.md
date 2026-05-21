# Spec: Rust Integration Tests

## Purpose

Defines what the `tests/integration/` Rust test suite must cover and the constraint that integration tests invoke library functions directly rather than spawning the binary process.

## Requirements

### Requirement: Integration tests call Rust library functions directly
`tests/integration/` SHALL contain Rust tests that invoke exported library functions without spawning the `dotagents` binary. No `std::process::Command` or binary execution SHALL appear in integration test files.

#### Scenario: Config merging test runs without binary
- **WHEN** a test in `tests/integration/config.rs` calls `AppConfig::from((&global, &local))`
- **THEN** the test completes without spawning any subprocess and asserts on the returned `AppConfig` value

#### Scenario: Fast execution
- **WHEN** the full integration suite is run via `mise test-integration`
- **THEN** it completes without needing a pre-built binary (`mise build` is not a prerequisite)

### Requirement: Config merging scenarios are covered
`tests/integration/config.rs` SHALL cover the `AppConfig` merge pipeline with at minimum these scenarios.

#### Scenario: Local config overrides global feature list
- **WHEN** global config enables `["commands", "mcp"]` and local config sets `features = ["commands"]`
- **THEN** `AppConfig::has_feature("mcp")` returns false

#### Scenario: Provider disabled flag is respected
- **WHEN** a provider entry has `disabled = true` in config
- **THEN** `AppConfig::get_provider_feature_settings` omits that provider from the returned map

#### Scenario: Per-provider variables deep-merge over globals
- **WHEN** global `[variables]` sets `agent_name = "global"` and a provider-feature sets `variables = { agent_name = "local" }`
- **THEN** the merged variables for that provider have `agent_name = "local"`

### Requirement: Template rendering pipeline is covered
`tests/integration/render.rs` SHALL cover `render_feature_with_settings` with at minimum these scenarios.

#### Scenario: Variable interpolation in instruction content
- **WHEN** `INSTRUCTIONS.md` contains `{{ var.agent_name }}` and config sets `agent_name = "Mycode"`
- **THEN** the rendered output contains `"Mycode"` and no unrendered Handlebars tokens

#### Scenario: Env variable interpolation
- **WHEN** `.env` sets `APP_NAME=dotagents` and `INSTRUCTIONS.md` contains `{{ env.app_name }}`
- **THEN** the rendered output contains `"dotagents"`

#### Scenario: Disabled provider skips rendering
- **WHEN** a provider feature has `disabled = true`
- **THEN** `render_feature_with_settings` does not write any output file for that provider

#### Scenario: Command frontmatter stripped from output
- **WHEN** a command source file has YAML frontmatter and a provider template that omits it
- **THEN** the rendered output file does not begin with `---`

### Requirement: Feature parsing is covered
`tests/integration/features.rs` SHALL cover `from_string` and `to_string` roundtrips for each feature type.

#### Scenario: CommandFeature roundtrip
- **WHEN** a valid markdown string with YAML frontmatter is parsed via `CommandFeature::from_string`
- **THEN** `to_string` produces output that round-trips without data loss on name and content

#### Scenario: McpFeature parses both server types
- **WHEN** `mcp.jsonc` declares one `stdio` server and one `http` server
- **THEN** `McpFeature::from_string` returns a value with both servers present and types correctly classified

#### Scenario: CommandFeature rejects missing name field
- **WHEN** a markdown file has YAML frontmatter without a `name` key
- **THEN** `CommandFeature::from_string` returns an `Err`

### Requirement: Cache logic is covered
`tests/integration/cache.rs` SHALL cover `CacheConfig` get, set, load, and save.

#### Scenario: Cache round-trip
- **WHEN** a hash is set for `(provider, feature, item)` and `CacheConfig` is saved then reloaded from disk
- **THEN** `get(provider, feature, item)` returns the same hash

#### Scenario: Cache miss returns None
- **WHEN** `get` is called for a key that has never been set
- **THEN** it returns `None`

### Requirement: Gitignore management is covered
`tests/integration/gitignore.rs` SHALL cover `update_gitignore` and `parse_fenced_section`.

#### Scenario: Fenced section is idempotent
- **WHEN** `update_gitignore` is called twice with the same paths
- **THEN** the resulting `.gitignore` content is identical after both calls (no duplicate entries)

#### Scenario: Existing user content is preserved
- **WHEN** a `.gitignore` has user-managed entries outside the fenced section
- **THEN** `update_gitignore` does not remove or modify those entries

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
