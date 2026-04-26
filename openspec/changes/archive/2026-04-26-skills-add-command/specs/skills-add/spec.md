## ADDED Requirements

### Requirement: Skills add command installs into .dotagents/skills

The system SHALL provide a `dotagents skills add <name>` command that installs a skill from the skills.sh registry (or any GitHub owner/repo) into `.dotagents/skills/<skill-name>/`, not into any agent-specific directory directly.

#### Scenario: Successful install with default runner

- **WHEN** user runs `dotagents skills add vercel-labs/agent-skills` with no runner configured
- **THEN** the skill files are installed into `.dotagents/skills/agent-skills/`
- **THEN** no agent-specific directories (`.claude/skills/`, `.cursor/skills/`, etc.) are written to

#### Scenario: Successful install with explicit runner flag

- **WHEN** user runs `dotagents skills add vercel-labs/agent-skills --runner pnpm`
- **THEN** the command uses `pnpm dlx skills add` as the invocation
- **THEN** the skill files are installed into `.dotagents/skills/agent-skills/`

#### Scenario: Missing workspace root

- **WHEN** user runs `dotagents skills add <name>` from a directory with no `.dotagents/` ancestor
- **THEN** the command exits with a non-zero status and an error message referencing the missing root directory

### Requirement: PackageRunner resolves with correct priority

The system SHALL resolve the package runner using the following priority order (highest to lowest): `--runner` CLI flag, `package-runner` in `local.config.toml`, `package-runner` in `config.toml`, silent default of `npm` (npx).

#### Scenario: CLI flag overrides config

- **WHEN** `config.toml` has `package-runner = "pnpm"` and user passes `--runner bun`
- **THEN** `bunx skills add` is used for the invocation

#### Scenario: Local config overrides global config

- **WHEN** `config.toml` has `package-runner = "npm"` and `local.config.toml` has `package-runner = "yarn"`
- **THEN** `yarn dlx skills add` is used for the invocation

#### Scenario: No runner configured uses npm default silently

- **WHEN** no `package-runner` is set in any config and no `--runner` flag is passed
- **THEN** `npx skills add` is used without any validation of npm presence

### Requirement: Explicit runner validated against PATH

The system SHALL validate that the configured runner binary is present on PATH when the runner was explicitly set (via config or `--runner` flag), and SHALL emit a clear error if it is absent.

#### Scenario: Explicit runner binary not found

- **WHEN** `package-runner = "pnpm"` is set in config and `pnpm` is not present on PATH
- **THEN** the command exits with a non-zero status
- **THEN** the error message names the missing binary and references `package-runner` in `config.toml`

#### Scenario: Default runner binary not found

- **WHEN** no runner is explicitly configured and `npx` is not on PATH
- **THEN** the command attempts to run `npx` and the OS error is surfaced as-is (no pre-validation)

### Requirement: PackageRunner persisted in config

The system SHALL accept an optional `package-runner` field in both `config.toml` (GlobalConfig) and `local.config.toml` (LocalConfig) with values `"npm"`, `"pnpm"`, `"yarn"`, or `"bun"`.

#### Scenario: Valid runner value in global config

- **WHEN** `config.toml` contains `package-runner = "pnpm"`
- **THEN** `AppConfig` deserializes successfully and carries `package_runner = Some(PackageRunner::Pnpm)`

#### Scenario: Invalid runner value in config

- **WHEN** `config.toml` contains `package-runner = "cargo"`
- **THEN** config deserialization fails with an error identifying the invalid value

#### Scenario: Absent runner field

- **WHEN** neither `config.toml` nor `local.config.toml` contains `package-runner`
- **THEN** `AppConfig.package_runner` is `None`
