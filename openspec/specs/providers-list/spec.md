## Purpose

Specifies the behaviour of `dotagents providers ls` — a read-only command that fetches the official provider registry and displays the available providers, with optional flags for JSON output and offline mode, and an interactive TUI browser in TTY mode.

## Requirements

### Requirement: providers ls lists all providers from the registry
`dotagents providers ls` SHALL fetch the official registry, parse it, and display every provider as a list entry. In non-TTY mode, providers are displayed as plain text with slug, name, and URL included where available. Providers SHALL be sorted alphabetically by slug.

#### Scenario: Successful listing of all providers
- **WHEN** `dotagents providers ls` is run and the registry is accessible
- **THEN** each provider is displayed on its own line and the command exits 0

#### Scenario: Empty registry handled gracefully
- **WHEN** the registry is fetched but contains no provider entries
- **THEN** a message indicating no providers were found is displayed and the command exits 0

### Requirement: --json flag outputs registry data as JSON
When `--json` is passed to `dotagents providers ls`, the command SHALL output a JSON array of provider objects with `slug`, `name`, and `url` fields. Fields absent in the registry entry SHALL appear as `null`. The output SHALL be valid JSON on stdout. All log/warning output SHALL go to stderr.

#### Scenario: JSON output matches registry data
- **WHEN** `dotagents providers ls --json` is run
- **THEN** stdout contains a JSON array where each element has `slug`, `name`, and `url` fields and the command exits 0

#### Scenario: JSON output with empty registry
- **WHEN** `dotagents providers ls --json` is run and the registry has no providers
- **THEN** stdout contains `[]` and the command exits 0

### Requirement: TUI mode provides interactive scrollable provider list
When stdin is a TTY and `--json` is not passed, `dotagents providers ls` SHALL display an interactive scrollable selection list using cliclack. Each option shows the provider name (or slug when name is absent) with the slug in brackets and the URL as a hint on the highlighted item. Selecting a provider and pressing Enter SHALL display the provider's name and URL in the closing outro. Esc or Ctrl+C SHALL exit.

#### Scenario: TUI shows all providers in a scrollable list
- **WHEN** `dotagents providers ls` is run in a TTY
- **THEN** a cliclack select prompt titled "Providers" is shown with all providers, capped at 10 visible rows

#### Scenario: TUI shows provider info on selection
- **WHEN** the user selects a provider and presses Enter
- **THEN** the closing message displays the provider's name and URL (e.g. `Amp Code (https://ampcode.com/manual)`); if name/url are absent, a fallback of "Done" is shown

### Requirement: --offline flag reads registry from cache only
When `--offline` is passed, `dotagents providers ls` SHALL NOT make any network request. It SHALL read `registry.json` from the template-source cache. If no cached registry exists, the command SHALL error with a clear message instructing the user to run without `--offline` first.

#### Scenario: Offline mode with warm cache succeeds
- **WHEN** `dotagents providers ls --offline` is run and the template-source cache has `registry.json`
- **THEN** providers are listed from the cache and no network request is made

#### Scenario: Offline mode with cold cache errors
- **WHEN** `dotagents providers ls --offline` is run and no cached `registry.json` exists
- **THEN** the command exits 1 with an error message containing "cached registry" and directing the user to run without `--offline`

### Requirement: providers ls is read-only
`dotagents providers ls` SHALL NOT modify any files on disk — no config files, no workspace files. It SHALL only read from disk and/or network. The registry is cached to disk after a successful online fetch as a side effect of the fetch, not as a write operation driven by this command.

#### Scenario: No workspace files are written
- **WHEN** `dotagents providers ls` completes successfully
- **THEN** no files in `.dotagents/` or the workspace are modified
