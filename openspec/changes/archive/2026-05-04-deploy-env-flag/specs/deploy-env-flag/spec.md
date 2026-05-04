## ADDED Requirements

### Requirement: Deploy accepts custom env file paths
The `deploy` command SHALL accept a repeatable `--env <path>` flag. Each value SHALL be a path to a `.env`-format file. When one or more `--env` flags are provided, they SHALL replace the default `.dotagents/.env` entirely — the default file is NOT loaded. When no `--env` flag is provided, the command SHALL behave as before (load `.dotagents/.env`, silently ignore if missing).

#### Scenario: Single --env file replaces default
- **WHEN** user runs `dotagents deploy --env ./envs/prod.env`
- **THEN** env variables are loaded exclusively from `./envs/prod.env`
- **THEN** `.dotagents/.env` is not loaded

#### Scenario: Multiple --env files merged left-to-right
- **WHEN** user runs `dotagents deploy --env ./base.env --env ./prod.env`
- **THEN** env variables from `base.env` are loaded first
- **THEN** env variables from `prod.env` are merged on top, overriding any duplicate keys
- **THEN** the resulting merged set is available as `env.*` in templates

#### Scenario: No --env flag preserves existing behaviour
- **WHEN** user runs `dotagents deploy` without any `--env` flag
- **THEN** env variables are loaded from `.dotagents/.env` if it exists
- **THEN** a missing `.dotagents/.env` is silently ignored (no error)

### Requirement: Missing explicitly-specified env file is a hard error
The `deploy` command SHALL exit with a non-zero status and a descriptive error message when a path supplied via `--env` does not exist or is not readable.

#### Scenario: Specified file does not exist
- **WHEN** user runs `dotagents deploy --env ./nonexistent.env`
- **THEN** the command exits with a non-zero exit code
- **THEN** an error message is printed identifying the missing file path

#### Scenario: Specified file exists and is readable
- **WHEN** user runs `dotagents deploy --env ./valid.env` and `./valid.env` exists
- **THEN** the command proceeds normally with env vars loaded from that file

### Requirement: Env file keys are lowercased
The system SHALL lowercase all keys read from `--env` files before making them available in templates, consistent with the behaviour for the default `.dotagents/.env`.

#### Scenario: Uppercase key in custom env file
- **WHEN** a custom env file contains `MY_KEY=value`
- **THEN** the key is accessible in templates as `{{ env.my_key }}`
- **THEN** `{{ env.MY_KEY }}` does NOT resolve

### Requirement: Env vars available to config and feature templates
Env variables loaded from `--env` files SHALL be available in both `config.toml` / `local.config.toml` rendering AND in feature template rendering, identical to the default `.dotagents/.env` behaviour.

#### Scenario: Env var used in config.toml
- **WHEN** `config.toml` contains `{{ env.deploy_target }}` and `--env` provides `DEPLOY_TARGET=prod`
- **THEN** `config.toml` is rendered with `deploy_target` substituted as `prod`

#### Scenario: Env var used in a feature template
- **WHEN** a provider template contains `{{ env.api_key }}` and `--env` provides `API_KEY=abc123`
- **THEN** the rendered output contains `abc123`
