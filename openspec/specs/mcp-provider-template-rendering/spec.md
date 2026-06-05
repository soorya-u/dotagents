# Spec: MCP Provider Template Rendering

## Purpose

Defines how provider MCP templates render source server configurations (transports, tool filters, disabled state, and provider-rendered fields) into provider-specific output formats.

## Requirements

### Requirement: Provider templates render source HTTP as provider Streamable HTTP

Provider MCP templates SHALL map source `type: "http"` to each provider's current Streamable HTTP representation.

#### Scenario: Gemini renders streamable HTTP URL field
- **WHEN** a source server has `type: "http"` and `url`
- **THEN** the Gemini MCP template renders the provider's streamable HTTP URL field for that server

#### Scenario: Mistral Vibe renders streamable HTTP transport
- **WHEN** a source server has `type: "http"` and `url`
- **THEN** the Mistral Vibe MCP template renders `transport = "streamable-http"` for that server

#### Scenario: providers without special streamable naming keep valid HTTP output
- **WHEN** a source server has `type: "http"` and a provider expects plain HTTP config
- **THEN** that provider template renders its existing valid HTTP output shape

### Requirement: Provider templates render source SSE as legacy SSE

Provider MCP templates SHALL map source `type: "sse"` to each provider's legacy SSE representation when the provider distinguishes SSE from HTTP.

#### Scenario: SSE provider renders SSE transport label
- **WHEN** a source server has `type: "sse"` and `url`
- **THEN** providers that distinguish SSE from HTTP render their SSE transport label for that server

#### Scenario: SSE uses remote server fields
- **WHEN** a source SSE server includes `url` and `headers`
- **THEN** provider templates render the URL and headers using the same source values as HTTP remote servers

### Requirement: Provider templates map generic tool filters to provider fields

Provider MCP templates SHALL use `enabledTools` and `disabledTools` as the generic source fields for per-server tool filtering.

#### Scenario: Codex renders enabled tools in snake_case
- **WHEN** a source server includes `enabledTools`
- **THEN** the Codex MCP template renders the provider field as `enabled_tools`

#### Scenario: Amp renders enabled tools as includeTools
- **WHEN** a source server includes `enabledTools`
- **THEN** the Amp MCP template renders the provider field as `includeTools`

#### Scenario: Copilot renders enabled tools as tools
- **WHEN** a source server includes `enabledTools`
- **THEN** the Copilot MCP template renders the provider field as `tools`

#### Scenario: disabled tools render only for supporting providers
- **WHEN** a source server includes `disabledTools`
- **THEN** provider templates that support disabled tool filtering render their provider-specific disabled tools field

### Requirement: Provider templates render common disabled state

Provider MCP templates SHALL map source `disabled` to each provider's supported disabled/enabled representation.

#### Scenario: provider supports disabled flag
- **WHEN** a source server includes `disabled: true`
- **THEN** providers with a native disabled flag render that server as disabled

#### Scenario: provider uses enabled flag
- **WHEN** a source server includes `disabled: true`
- **THEN** providers that use an inverse enabled flag render the server as not enabled

### Requirement: Provider templates render provider-rendered fields where supported

Provider MCP templates SHALL render source provider-rendered fields only for providers that support those fields.

#### Scenario: Codex renders timeout and auth fields
- **WHEN** a source server includes `required`, `startupTimeoutSec`, `toolTimeoutSec`, `bearerTokenEnvVar`, and `envVars`
- **THEN** the Codex MCP template renders the corresponding provider fields with Codex-compatible names

#### Scenario: Cline-style providers render alwaysAllow
- **WHEN** a source server includes `alwaysAllow`
- **THEN** provider templates that support auto-approved tools render the field in their provider-compatible output shape

#### Scenario: Autohand renders autoConnect
- **WHEN** a source server includes `autoConnect`
- **THEN** the Autohand MCP template renders the provider-compatible connection behavior field

### Requirement: Provider MCP rendering is covered by manual and automated tests

Expanded MCP provider rendering SHALL be manually validated before automated test assertions are added, and automated tests SHALL cover representative JSON and TOML provider output.

Qwen, KiloCode, and Mistral Vibe MCP targets SHALL point to the provider's shared config file (not a sidecar file), and deploy SHALL use merge-aware write to preserve existing keys in those files.

#### Scenario: manual validation precedes automated assertions
- **WHEN** provider MCP template behavior is changed
- **THEN** generated deploy output is manually inspected for representative providers before new automated assertions are finalized

#### Scenario: automated tests cover representative provider outputs
- **WHEN** automated tests run
- **THEN** they verify at least one JSON provider output and one TOML provider output containing expanded MCP fields

#### Scenario: Qwen MCP targets settings.json
- **WHEN** Qwen MCP is deployed
- **THEN** the target path SHALL be `.qwen/settings.json` (not `.qwen/mcp.json`)

#### Scenario: KiloCode MCP targets kilo.jsonc
- **WHEN** KiloCode MCP is deployed
- **THEN** the target path SHALL be `.kilo/kilo.jsonc` (not `.kilo/mcp.json`)

#### Scenario: Mistral Vibe MCP targets config.toml
- **WHEN** Mistral Vibe MCP is deployed
- **THEN** the target path SHALL be `.vibe/config.toml` (not `.vibe/mcp.toml`)
