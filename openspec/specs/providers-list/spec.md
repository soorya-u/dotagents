## Purpose

Specifies the behaviour of `dotagents providers` — a read-only command that fetches the official registry and displays the available providers, with optional flags for JSON output and offline mode, and an interactive TUI browser in TTY mode.

## Requirements

### Requirement: providers lists all providers from the registry
`dotagents providers` SHALL fetch the official registry, parse it, and display every provider as a list entry. In non-TTY mode, providers are displayed as plain text with slug, name, and URL included where available. Providers SHALL be sorted alphabetically by slug.

#### Scenario: Successful listing of all providers
- **WHEN** `dotagents providers` is run and the registry is accessible
- **THEN** each provider is displayed on its own line and the command exits 0

#### Scenario: Empty registry handled gracefully
- **WHEN** the registry is fetched but contains no provider entries
- **THEN** a message indicating no providers were found is displayed and the command exits 0

#### Scenario: Error message references correct command name
- **WHEN** the registry cannot be read from cache (for cache behavior details, see `template-source-cache` spec)
- **THEN** the error message instructs the user to run `dotagents providers` (NOT `dotagents providers ls`)

### Requirement: providers command emits debug logs for registry fetch
When `--verbose` / `-v` is passed, the `providers` command SHALL emit `debug!()` log entries for: the registry URL being fetched, and the cache file path after successful caching.

#### Scenario: verbose flag shows registry URL
- **WHEN** `dotagents providers -v` is run (non-offline)
- **THEN** stderr contains a debug line with the registry URL being fetched

#### Scenario: verbose flag shows cache path after fetch
- **WHEN** `dotagents providers -v` fetches the registry successfully
- **THEN** stderr contains a debug line with the cache file path

### Requirement: --json flag outputs registry data as JSON
When `--json` is passed to `dotagents providers`, the command SHALL output a JSON array of provider objects with `slug`, `name`, and `url` fields. Fields absent in the registry entry SHALL appear as `null`. The output SHALL be valid JSON on stdout. All log/warning output SHALL go to stderr.

#### Scenario: JSON output matches registry data
- **WHEN** `dotagents providers --json` is run
- **THEN** stdout contains a JSON array where each element has `slug`, `name`, and `url` fields and the command exits 0

#### Scenario: JSON output with empty registry
- **WHEN** `dotagents providers --json` is run and the registry has no providers
- **THEN** stdout contains `[]` and the command exits 0

### Requirement: TUI mode provides interactive scrollable provider list
When stdin is a TTY and `--json` is not passed, `dotagents providers` SHALL display an interactive scrollable selection list using cliclack. Each option SHALL show only the provider slug as the label. No hint text SHALL be displayed on list items. Selecting a provider and pressing Enter SHALL display the provider's name and URL in the closing outro as `Name (url)`. Esc or Ctrl+C SHALL exit cleanly with code 0 and no error output.

#### Scenario: TUI shows all providers in a scrollable list with slug labels
- **WHEN** `dotagents providers` is run in a TTY
- **THEN** a cliclack select prompt titled "Providers" is shown with all providers listed by slug, capped at 10 visible rows

#### Scenario: TUI shows provider details on selection
- **WHEN** the user selects a provider and presses Enter
- **THEN** the closing message displays the provider's name and URL (e.g. `Claude Code (https://docs.anthropic.com/en/docs/claude-code)`); if name/url are absent, a fallback of "Done" is shown

#### Scenario: TUI select label is slug only
- **WHEN** `dotagents providers ls` is run in a TTY and a provider has name "Claude Code" and slug "claude"
- **THEN** the select list item label is "claude", not "Claude Code"

#### Scenario: Escape key exits cleanly
- **WHEN** `dotagents providers` is run in a TTY
- **WHEN** the user presses Escape during the provider select prompt
- **THEN** the process exits with code 0
- **THEN** stderr does NOT contain "Fatal error" or "Failed to"

### Requirement: --offline flag reads registry from cache only
When `--offline` is passed, `dotagents providers` SHALL NOT make any network request. It SHALL read `registry.json` from the template-source cache. If no cached registry exists, the command SHALL error with a clear message instructing the user to run without `--offline` first.

#### Scenario: Offline mode with warm cache succeeds
- **WHEN** `dotagents providers --offline` is run and the template-source cache has `registry.json`
- **THEN** providers are listed from the cache and no network request is made

#### Scenario: Offline mode with cold cache errors
- **WHEN** `dotagents providers --offline` is run and no cached `registry.json` exists
- **THEN** the command exits 1 with an error message containing "cached registry" and directing the user to run without `--offline`

### Requirement: providers is read-only
`dotagents providers` SHALL NOT modify any files on disk — no config files, no workspace files. It SHALL only read from disk and/or network. The registry is cached to disk after a successful online fetch as a side effect of the fetch, not as a write operation driven by this command.

#### Scenario: No workspace files are written
- **WHEN** `dotagents providers` completes successfully
- **THEN** no files in `.dotagents/` or the workspace are modified
