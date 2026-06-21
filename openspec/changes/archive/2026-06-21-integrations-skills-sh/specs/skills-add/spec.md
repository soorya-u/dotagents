## MODIFIED Requirements

### Requirement: Skills add command installs into .dotagents/skills

The system SHALL provide a `dotagents skills add <name>` command that installs a skill from the skills.sh registry (or any GitHub owner/repo) into `.dotagents/skills/<skill-name>/`, not into any agent-specific directory directly. The install SHALL be performed by `src/integrations/skills_sh.rs` spawning the external `skills` CLI with `current_dir` set to the dotagents application directory, `--agent openclaw`, and `--copy`. The `CLAUDE_CONFIG_DIR` environment variable SHALL NOT be set.

#### Scenario: Successful install with default runner

- **WHEN** user runs `dotagents skills add vercel-labs/agent-skills` with no runner configured
- **THEN** the skill files are installed into `<application-dir>/skills/agent-skills/` as real files (not symlinks)
- **THEN** no agent-specific directories (`.claude/skills/`, `.cursor/skills/`, etc.) are written to in the workspace
- **THEN** `<application-dir>/skills-lock.json` is created or updated with an entry for the installed skill

#### Scenario: Successful install with explicit runner flag

- **WHEN** user runs `dotagents skills add vercel-labs/agent-skills --runner pnpm`
- **THEN** the command uses `pnpm dlx skills add` as the invocation
- **THEN** the skill files are installed into `<application-dir>/skills/agent-skills/`

#### Scenario: Missing workspace root

- **WHEN** user runs `dotagents skills add <name>` from a directory with no `.dotagents/` ancestor
- **THEN** the command exits with a non-zero status and an error message referencing the missing root directory

### Requirement: PackageRunner resolves with correct priority

The system SHALL resolve the package runner using the following priority order (highest to lowest): `--runner` CLI flag, `package-runner` under `[integrations.skills-sh]` in `local.config.toml`, `package-runner` under `[integrations.skills-sh]` in `config.toml`, silent default of `npm` (npx).

#### Scenario: CLI flag overrides config

- **WHEN** `config.toml` has `[integrations.skills-sh]` with `package-runner = "pnpm"` and user passes `--runner bun`
- **THEN** `bunx skills add` is used for the invocation

#### Scenario: Local config overrides global config

- **WHEN** `config.toml` has `[integrations.skills-sh]` with `package-runner = "npm"` and `local.config.toml` has `[integrations.skills-sh]` with `package-runner = "yarn"`
- **THEN** `yarn dlx skills add` is used for the invocation

#### Scenario: No runner configured uses npm default silently

- **WHEN** no `package-runner` is set under `[integrations.skills-sh]` in any config and no `--runner` flag is passed
- **THEN** `npx skills add` is used without any validation of npm presence

### Requirement: Explicit runner validated against PATH

The system SHALL validate that the configured runner binary is present on PATH when the runner was explicitly set (via config or `--runner` flag), and SHALL emit a clear error if it is absent.

#### Scenario: Explicit runner binary not found

- **WHEN** `[integrations.skills-sh]` has `package-runner = "pnpm"` in config and `pnpm` is not present on PATH
- **THEN** the command exits with a non-zero status
- **THEN** the error message names the missing binary and references `[integrations.skills-sh]` in the config

#### Scenario: Default runner binary not found

- **WHEN** no runner is explicitly configured and `npx` is not on PATH
- **THEN** the command attempts to run `npx` and the OS error is surfaced as-is (no pre-validation)

### Requirement: PackageRunner persisted under integrations config

The system SHALL accept an optional `[integrations.skills-sh]` table in both `config.toml` (GlobalConfig) and `local.config.toml` (LocalConfig) containing a `package-runner` field with values `"npm"`, `"pnpm"`, `"yarn"`, or `"bun"`. The top-level `package-runner` field SHALL NO LONGER be accepted (breaking change).

#### Scenario: Valid runner value in global config

- **WHEN** `config.toml` contains `[integrations.skills-sh]` with `package-runner = "pnpm"`
- **THEN** `AppConfig` deserializes successfully and carries the resolved `PackageRunner::Pnpm`

#### Scenario: Invalid runner value in config

- **WHEN** `config.toml` contains `[integrations.skills-sh]` with `package-runner = "cargo"`
- **THEN** config deserialization fails with an error identifying the invalid value

#### Scenario: Absent runner field

- **WHEN** neither `config.toml` nor `local.config.toml` contains `[integrations.skills-sh]`
- **THEN** the resolved package runner is `None` and the npm default is used at runtime

#### Scenario: Top-level package-runner field is rejected

- **WHEN** `config.toml` contains a top-level `package-runner = "bun"` (the old format)
- **THEN** config deserialization fails or ignores the field (breaking change, no backward compatibility)

### Requirement: skills add passes ci flag to PackageRunner::args
The `skills add` command SHALL invoke `PackageRunner::args(skill_name, ci)` where `ci` is `true` when `is_tui_enabled()` returns `false`.

#### Scenario: skills add in CI environment passes ci=true
- **WHEN** `skills add <name>` is run with `DOTAGENTS_CI=true` or in a non-TTY environment
- **THEN** the package runner subprocess receives `--yes` in its argument list

## REMOVED Requirements

### Requirement: skills new, rm, and ls are peers of skills add
**Reason**: The `skills add` sub-action is no longer implemented in `src/cli/skills.rs`; it delegates to `src/integrations/skills_sh.rs`. The "peers" framing is now inaccurate — `add` is an integration wrapper while `new`/`rm`/`ls` are local source-of-truth operations. The subcommand group structure (four sub-actions) is unchanged and already specified in `skills-subcommand-extended`.
**Migration**: See `skills-subcommand-extended` spec for the `new`/`rm`/`ls` sub-actions and `integrations-skills-sh` spec for the `add` delegation.
