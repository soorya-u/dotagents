## ADDED Requirements

### Requirement: providers ls lists all providers from the registry
`dotagents providers ls` SHALL fetch the official registry, parse it, and display every provider as a list entry showing the provider slug and display name. Providers SHALL be listed in the order they appear in the registry. If the registry cannot be fetched and `--offline` is not set, the command SHALL fall back to the template-source cache.

#### Scenario: Successful listing of all providers
- **WHEN** `dotagents providers ls` is run and the registry is accessible
- **THEN** each provider is displayed on its own line as `{slug}  ({name})` and the command exits 0

#### Scenario: Empty registry handled gracefully
- **WHEN** the registry is fetched but contains no provider entries
- **THEN** a message indicating no providers were found is displayed and the command exits 0

#### Scenario: Registry fetch failure falls back to cache
- **WHEN** the registry cannot be fetched due to a network error and the template-source cache has a cached `registry.json`
- **THEN** providers are listed from the cached registry, a warning is logged about the fetch failure, and the command exits 0

#### Scenario: Registry unreachable and cache cold — hard error
- **WHEN** the registry cannot be fetched and no cached `registry.json` exists
- **THEN** the command exits 1 with an error indicating the registry is unavailable

### Requirement: --url flag appends documentation URLs to provider listing
When `--url` is passed to `dotagents providers ls`, each provider line SHALL include the documentation URL extracted from the registry entry's `url` field. Providers without a `url` field SHALL show "N/A" or be omitted in place of the URL.

#### Scenario: Provider with URL shows the URL
- **WHEN** `dotagents providers ls --url` is run and a provider has `url = "https://example.com/docs"`
- **THEN** that provider's line reads `{slug}  ({name}) — https://example.com/docs`

#### Scenario: Provider without URL shows placeholder
- **WHEN** `dotagents providers ls --url` is run and a provider has no `url` field
- **THEN** that provider's line reads `{slug}  ({name}) — N/A`

### Requirement: --json flag outputs registry data as JSON
When `--json` is passed to `dotagents providers ls`, the command SHALL output a JSON array of provider objects with `slug`, `name`, and `url` fields. The output SHALL be valid JSON on stdout. All log/warning output SHALL go to stderr.

#### Scenario: JSON output matches registry data
- **WHEN** `dotagents providers ls --json` is run
- **THEN** stdout contains a JSON array like `[{"slug":"claude","name":"Claude Code","url":"https://..."}]` and the command exits 0

#### Scenario: JSON output with empty registry
- **WHEN** `dotagents providers ls --json` is run and the registry has no providers
- **THEN** stdout contains `[]` and the command exits 0

### Requirement: TUI mode provides interactive fuzzy-search browser
When stdin is a TTY and `--json` is not passed, `dotagents providers ls` SHALL display an interactive fuzzy-search selection list using cliclack. Each option SHALL show the provider name with the slug in brackets. When `--url` is active, the URL SHALL be displayed inline. Selecting a provider and pressing Enter SHALL display its full details (slug, name, URL). Esc or Ctrl+C SHALL exit.

#### Scenario: TUI shows all providers with fuzzy filter
- **WHEN** `dotagents providers ls` is run in a TTY
- **THEN** a cliclack select prompt is shown with all providers and a search filter that narrows results as the user types

#### Scenario: TUI shows provider detail on selection
- **WHEN** user selects a provider from the TUI list and presses Enter
- **THEN** a detail view is shown with the provider's slug, name, and URL (if available)

### Requirement: --offline flag reads registry from cache only
When `--offline` is passed, `dotagents providers ls` SHALL NOT make any network request. It SHALL read `registry.json` from the template-source cache. If no cached registry exists, the command SHALL error with a clear message instructing the user to run without `--offline` first.

#### Scenario: Offline mode with warm cache succeeds
- **WHEN** `dotagents providers ls --offline` is run and the template-source cache has `registry.json`
- **THEN** providers are listed from the cache and no network request is made

#### Scenario: Offline mode with cold cache errors
- **WHEN** `dotagents providers ls --offline` is run and no cached `registry.json` exists
- **THEN** the command exits 1 with an error message directing the user to run without `--offline`

### Requirement: providers ls is read-only
`dotagents providers ls` SHALL NOT modify any files on disk — no config files, no cache files, no deployed output. It SHALL only read from disk and/or network.

#### Scenario: No file writes occur
- **WHEN** `dotagents providers ls` completes successfully
- **THEN** no files in `.dotagents/`, workspace, or template-source cache are modified compared to before the command ran (excluding any log files)
