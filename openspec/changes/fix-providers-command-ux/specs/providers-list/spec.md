## MODIFIED Requirements

### Requirement: providers lists all providers from the registry
`dotagents providers` SHALL fetch the official registry, parse it, and display every provider as a list entry. In non-TTY mode, providers are displayed as plain text with slug, name, and URL included where available. Providers SHALL be sorted alphabetically by slug.

#### Scenario: Successful listing of all providers
- **WHEN** `dotagents providers` is run and the registry is accessible
- **THEN** each provider is displayed on its own line and the command exits 0

#### Scenario: Empty registry handled gracefully
- **WHEN** the registry is fetched but contains no provider entries
- **THEN** a message indicating no providers were found is displayed and the command exits 0

#### Scenario: Error message references correct command name
- **WHEN** the registry cannot be read from cache
- **THEN** the error message instructs the user to run `dotagents providers` (NOT `dotagents providers ls`)

## ADDED Requirements

### Requirement: providers command emits debug logs for registry fetch
When `--verbose` / `-v` is passed, the `providers` command SHALL emit `debug!()` log entries for: the registry URL being fetched, and the cache file path after successful caching.

#### Scenario: verbose flag shows registry URL
- **WHEN** `dotagents providers -v` is run (non-offline)
- **THEN** stderr contains a debug line with the registry URL being fetched

#### Scenario: verbose flag shows cache path after fetch
- **WHEN** `dotagents providers -v` fetches the registry successfully
- **THEN** stderr contains a debug line with the cache file path
