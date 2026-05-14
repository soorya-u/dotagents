## ADDED Requirements

### Requirement: commands subcommands accept an optional --cwd flag
`dotagents commands new`, `commands rm`, and `commands ls` SHALL accept an optional `--cwd <PATH>` flag. When provided, `PATH` SHALL be used as the workspace root (the directory containing `.dotagents/`). When omitted, workspace resolution SHALL fall back to walking up from CWD as today.

#### Scenario: --cwd omitted — workspace resolved from CWD
- **WHEN** `dotagents commands ls` is run with no `--cwd`
- **THEN** the workspace is resolved by walking up from CWD to find `.dotagents/`, identical to existing behaviour

#### Scenario: --cwd provided as absolute path — used as workspace root
- **WHEN** `dotagents commands ls --cwd /home/user/myproject` is run and `/home/user/myproject/.dotagents/` exists
- **THEN** commands are read from `/home/user/myproject/.dotagents/commands/`

#### Scenario: --cwd provided as relative path — resolved against CWD
- **WHEN** `dotagents commands ls --cwd ../sibling` is run from `/home/user/current` and `../sibling/.dotagents/` exists
- **THEN** commands are read from `/home/user/sibling/.dotagents/commands/`

#### Scenario: --cwd does not contain .dotagents — error with clear message
- **WHEN** `dotagents commands ls --cwd /tmp/no-dotagents` is run and no `.dotagents/` exists there
- **THEN** the process exits with a non-zero code and an error message indicating `.dotagents/` was not found at that path

#### Scenario: commands new with --cwd creates file in correct workspace
- **WHEN** `dotagents commands new hello --cwd /home/user/myproject` is run and `/home/user/myproject/.dotagents/` exists
- **THEN** the file is created at `/home/user/myproject/.dotagents/commands/hello.md`

#### Scenario: commands rm with --cwd deletes file from correct workspace
- **WHEN** `dotagents commands rm hello --cwd /home/user/myproject` is run and the command file exists there
- **THEN** `/home/user/myproject/.dotagents/commands/hello.md` is deleted and cleanup runs against that workspace

### Requirement: skills subcommands accept an optional --cwd flag
`dotagents skills new`, `skills rm`, `skills ls`, and `skills add` SHALL accept an optional `--cwd <PATH>` flag with identical semantics to the `commands` `--cwd` flag.

#### Scenario: --cwd omitted — workspace resolved from CWD
- **WHEN** `dotagents skills ls` is run with no `--cwd`
- **THEN** the workspace is resolved by walking up from CWD, identical to existing behaviour

#### Scenario: skills ls --cwd reads from correct workspace
- **WHEN** `dotagents skills ls --cwd /home/user/myproject` is run and `/home/user/myproject/.dotagents/` exists
- **THEN** skills are read from `/home/user/myproject/.dotagents/skills/`

#### Scenario: skills new --cwd creates skill in correct workspace
- **WHEN** `dotagents skills new my-skill --cwd /home/user/myproject` is run and the workspace exists
- **THEN** the skill directory is created at `/home/user/myproject/.dotagents/skills/my-skill/`

#### Scenario: skills rm --cwd removes skill from correct workspace
- **WHEN** `dotagents skills rm my-skill --cwd /home/user/myproject` is run and the skill exists there
- **THEN** the skill is removed from that workspace and cleanup (undeploy + cache) runs against that workspace

#### Scenario: skills add --cwd installs into correct workspace
- **WHEN** `dotagents skills add owner/repo --cwd /home/user/myproject` is run and the workspace exists
- **THEN** the skill is installed into `/home/user/myproject/.dotagents/skills/`

#### Scenario: --cwd does not contain .dotagents — error
- **WHEN** `dotagents skills ls --cwd /tmp/no-dotagents` is run and no `.dotagents/` exists there
- **THEN** the process exits non-zero with an error indicating `.dotagents/` was not found

### Requirement: config subcommand accepts an optional --cwd flag
`dotagents config` SHALL accept an optional `--cwd <PATH>` flag. When provided, the config is read from the specified workspace.

#### Scenario: --cwd omitted — workspace resolved from CWD
- **WHEN** `dotagents config` is run with no `--cwd`
- **THEN** the workspace is resolved from CWD, identical to existing behaviour

#### Scenario: config --cwd reads config from correct workspace
- **WHEN** `dotagents config --cwd /home/user/myproject` is run and `/home/user/myproject/.dotagents/` exists
- **THEN** `config.toml` and `local.config.toml` are read from `/home/user/myproject/.dotagents/`

#### Scenario: config --json --cwd outputs JSON for correct workspace
- **WHEN** `dotagents config --json --cwd /home/user/myproject` is run
- **THEN** stdout contains JSON representing the config from that workspace

#### Scenario: config --edit --cwd edits config for correct workspace
- **WHEN** `dotagents config global --edit --cwd /home/user/myproject` is run in a TTY
- **THEN** the TUI editor targets `/home/user/myproject/.dotagents/config.toml`

#### Scenario: --cwd without .dotagents — error
- **WHEN** `dotagents config --cwd /tmp/no-dotagents` is run and no `.dotagents/` exists there
- **THEN** the process exits 1 with an error indicating `.dotagents/` was not found

### Requirement: Template variables reflect --cwd workspace
When `--cwd` is provided, `{{ dir.workspace }}` and `{{ dir.application }}` in template rendering SHALL reflect the overridden workspace path.

#### Scenario: dir.workspace reflects --cwd in commands new with --deploy
- **WHEN** `dotagents commands new hello --cwd /home/user/myproject --deploy` is run
- **THEN** during the triggered deploy, `{{ dir.workspace }}` resolves to `/home/user/myproject`

#### Scenario: dir.application reflects --cwd in skills rm cleanup
- **WHEN** `dotagents skills rm my-skill --cwd /home/user/myproject` is run and the skill has been previously deployed
- **THEN** the undeploy cleanup step resolves `{{ dir.application }}` to `/home/user/myproject/.dotagents`
