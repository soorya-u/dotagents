# Spec: MCP Server Config Model

## Purpose

Defines the MCP source configuration model including transport types (stdio, http, sse), shared remote server fields, camelCase source fields, common fields across all transports, and provider-rendered optional fields.

## Requirements

### Requirement: MCP server config supports current and legacy transports

The MCP source model SHALL support server `type` values `stdio`, `http`, and `sse`. Source `http` SHALL represent MCP Streamable HTTP. Source `sse` SHALL represent legacy/deprecated HTTP+SSE.

#### Scenario: parses streamable HTTP server
- **WHEN** `mcp.jsonc` declares a server with `type: "http"`, `url`, and optional `headers`
- **THEN** `McpFeature::from_string` returns an HTTP server config preserving the URL and headers

#### Scenario: parses legacy SSE server
- **WHEN** `mcp.jsonc` declares a server with `type: "sse"`, `url`, and optional `headers`
- **THEN** `McpFeature::from_string` returns an SSE server config preserving the URL and headers

#### Scenario: parses stdio server
- **WHEN** `mcp.jsonc` declares a server with `type: "stdio"`, `command`, `args`, `cwd`, `env`, and `envFile`
- **THEN** `McpFeature::from_string` returns a stdio server config preserving those fields

### Requirement: HTTP and SSE share remote server fields

The MCP source model SHALL use the same remote-server subfields for `http` and `sse`: `url`, `headers`, and common fields.

#### Scenario: remote fields are shared
- **WHEN** two servers differ only by `type`, with one set to `http` and one set to `sse`
- **THEN** both servers accept the same `url`, `headers`, `disabled`, `disabledTools`, and `enabledTools` source fields

### Requirement: MCP server config uses camelCase source fields

The MCP source model SHALL expose JSON/JSONC field names in camelCase.

#### Scenario: parses camelCase timeout and auth fields
- **WHEN** a server includes `startupTimeoutSec`, `toolTimeoutSec`, `bearerTokenEnvVar`, and `envVars`
- **THEN** `McpFeature::from_string` preserves those values and `to_string` serializes them using the same camelCase names

#### Scenario: parses camelCase common fields
- **WHEN** a server includes `disabled`, `disabledTools`, and `enabledTools`
- **THEN** `McpFeature::from_string` preserves those values and `to_string` serializes them using the same camelCase names

### Requirement: MCP server config supports common fields on every transport

The MCP source model SHALL allow common fields on `stdio`, `http`, and `sse` servers.

#### Scenario: common fields on stdio server
- **WHEN** a stdio server includes `disabled`, `disabledTools`, and `enabledTools`
- **THEN** parsing succeeds and the server config contains those common field values

#### Scenario: common fields on HTTP server
- **WHEN** an HTTP server includes `disabled`, `disabledTools`, and `enabledTools`
- **THEN** parsing succeeds and the server config contains those common field values

#### Scenario: common fields on SSE server
- **WHEN** an SSE server includes `disabled`, `disabledTools`, and `enabledTools`
- **THEN** parsing succeeds and the server config contains those common field values

### Requirement: MCP server config supports provider-rendered behavior fields

The MCP source model SHALL support optional camelCase fields needed by supported provider templates: `required`, `startupTimeoutSec`, `toolTimeoutSec`, `bearerTokenEnvVar`, `envVars`, `alwaysAllow`, and `autoConnect`.

#### Scenario: parses optional provider-rendered fields
- **WHEN** a server includes `required`, `alwaysAllow`, and `autoConnect`
- **THEN** parsing succeeds and serialization preserves those field values

#### Scenario: omits unset optional fields
- **WHEN** a server does not include provider-rendered optional fields
- **THEN** serialized output does not invent default values for those fields

### Requirement: MCP JSON schema documents expanded server config

The public MCP JSON schema SHALL validate the expanded server config model, including `stdio`, `http`, `sse`, common fields, and provider-rendered optional fields.

#### Scenario: schema accepts expanded valid config
- **WHEN** an MCP config includes one `stdio`, one `http`, and one `sse` server with expanded optional fields
- **THEN** the public MCP schema accepts the config

#### Scenario: schema rejects invalid transport-specific fields
- **WHEN** an MCP config uses a `stdio` server without `command` or a remote server without `url`
- **THEN** the public MCP schema rejects the config
