## Purpose

The `config` command provides visibility into the workspace configuration — the merged `AppConfig`, the raw global `config.toml`, and the optional `local.config.toml`. It supports plain-text CLI output, JSON serialization, interactive TUI viewing, and TUI-based editing of global/local config files.

## Requirements

### Requirement: config command shows the merged AppConfig by default
`dotagents config` or `dotagents config app` SHALL load the workspace configuration, compute the merged AppConfig, and display the active features and targeted providers with their per-feature settings. If no workspace is found, the command SHALL exit 1 with an error referencing `dotagents init`.

#### Scenario: Merged config with features and providers displayed
- **WHEN** `dotagents config` is run in a workspace with `features = ["commands", "mcp"]` and `targets = ["claude", "cursor"]`
- **THEN** the active features and each provider's feature settings (template, target, disabled status, variables) are displayed and the command exits 0

#### Scenario: Empty config displayed
- **WHEN** `dotagents config` is run in a workspace with no features and no targets
- **THEN** a message indicating no features or providers are configured is displayed and the command exits 0

#### Scenario: Missing workspace exits with error
- **WHEN** no `.dotagents/` directory exists in the current or any parent directory
- **THEN** the command exits 1 with an error referencing `dotagents init`

### Requirement: config global shows the global config file
`dotagents config global` SHALL read and display the contents of `config.toml` — features, targets, providers, variables, and package_runner. No merging with local config occurs.

#### Scenario: Global config displayed
- **WHEN** `dotagents config global` is run and `config.toml` contains `features = ["commands"]` and `targets = ["claude"]`
- **THEN** those values are displayed and the command exits 0

#### Scenario: Missing global config errors
- **WHEN** `config.toml` does not exist in the workspace
- **THEN** the command exits 1 with an error indicating the config file is missing

### Requirement: config local shows the local config file
`dotagents config local` SHALL read and display the contents of `local.config.toml`. If the file does not exist, the command SHALL output a message indicating no local config exists and exit 0.

#### Scenario: Local config with overrides displayed
- **WHEN** `dotagents config local` is run and `local.config.toml` contains overrides for features and targets
- **THEN** those overrides are displayed and the command exits 0

#### Scenario: Missing local config shows informational message
- **WHEN** `local.config.toml` does not exist
- **THEN** a message indicating no local config exists is displayed and the command exits 0

### Requirement: --json flag outputs config as structured JSON
When `--json` is passed, `dotagents config [app|global|local]` SHALL output the selected config serialized as JSON. For `app`, the output SHALL be a flat structure with `features` (array of strings) and `providers` (array of objects with provider name and per-feature settings). For `global` and `local`, the output SHALL use the existing TOML-derived JSON serialization.

#### Scenario: App config as JSON
- **WHEN** `dotagents config app --json` is run
- **THEN** stdout contains valid JSON with `features` and `providers` top-level keys

#### Scenario: Global config as JSON
- **WHEN** `dotagents config global --json` is run
- **THEN** stdout contains valid JSON representing the full global config

### Requirement: CLI mode displays plain-text config summary
When stdin is not a TTY and `--json` is not passed, the config command SHALL output a plain-text summary of the config (features list and providers table). No interactive prompts SHALL be shown.

#### Scenario: Non-TTY output is plain text
- **WHEN** stdin is not a TTY and `dotagents config` is run
- **THEN** a plain-text listing of features and providers is output and the command exits 0

### Requirement: TUI mode provides interactive config viewer
When stdin is a TTY and `--json` is not passed, the config command SHALL display an interactive TUI that allows navigating the config details. The TUI SHALL show features with their enabled/disabled status and providers with their per-feature settings.

#### Scenario: TUI shows config in interactive view
- **WHEN** `dotagents config` is run in a TTY
- **THEN** a cliclack-based interactive view displays the config with navigable sections for features and providers

### Requirement: --edit flag enables TUI editing for global and local configs
When `--edit` is passed with `global` or `local`, the command SHALL open an interactive TUI editor that allows the user to add or remove features and providers. Changes SHALL be persisted to the respective config file. When `--edit` is passed with `app`, the command SHALL error.

#### Scenario: Edit global config features
- **WHEN** `dotagents config global --edit` is run in a TTY
- **THEN** an interactive prompt lets the user toggle features and select providers; changes are written to `config.toml`

#### Scenario: Edit local config providers
- **WHEN** `dotagents config local --edit` is run in a TTY
- **THEN** an interactive prompt lets the user modify overrides; changes are written to `local.config.toml`

#### Scenario: --edit on app config errors
- **WHEN** `dotagents config app --edit` or `dotagents config --edit` is run
- **THEN** the command exits 1 with an error explaining that app config is derived and cannot be edited directly

#### Scenario: --edit in non-TTY mode errors
- **WHEN** `dotagents config global --edit` is run and stdin is not a TTY
- **THEN** the command exits 1 with an error directing the user to run in a TTY

### Requirement: config command is read-only by default
Without `--edit`, the config command SHALL NOT modify any files. It SHALL only read config files and display their content.

#### Scenario: Config display does not modify files
- **WHEN** `dotagents config` completes successfully without `--edit`
- **THEN** no files in `.dotagents/`, workspace, or any global config paths read by the command are modified compared to before the command ran
