## MODIFIED Requirements

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
