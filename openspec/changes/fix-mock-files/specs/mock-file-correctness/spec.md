## ADDED Requirements

### Requirement: Generated .gitignore must not include non-existent paths
The `.gitignore` written by `dotagents init` SHALL only list paths that the tool actually creates. The entry `cache/` SHALL NOT appear in the generated file because no such directory is ever created by the tool.

#### Scenario: Fresh init produces correct gitignore
- **WHEN** user runs `dotagents init` in an empty directory
- **THEN** the generated `.dotagents/.gitignore` does not contain the line `cache/`
- **THEN** the generated `.dotagents/.gitignore` does contain `cache.toml`, `local.config.toml`, and `.env`

### Requirement: Generated mcp.jsonc must be valid against the published MCP schema
The `mcp.jsonc` written by `dotagents init` SHALL pass validation against `public/v1/schemas/mcp.schema.json` with zero errors.

#### Scenario: $schema key is accepted by the schema validator
- **WHEN** the MCP schema file is loaded by an editor or JSON Schema validator
- **THEN** the `$schema` property at the root of `mcp.jsonc` is recognised as valid (not flagged as an unknown property)

#### Scenario: envFile key uses correct camelCase spelling
- **WHEN** the generated `mcp.jsonc` is validated against the MCP schema
- **THEN** the `envFile` property (camelCase) is accepted on a `stdio`-type server entry
- **THEN** no "Property env_file is not allowed" error is reported

#### Scenario: envFile value is a valid string
- **WHEN** the generated `mcp.jsonc` contains an `envFile` entry
- **THEN** its value is a non-null string (e.g. `".env.local"`)
- **THEN** no type mismatch error is reported for that field
