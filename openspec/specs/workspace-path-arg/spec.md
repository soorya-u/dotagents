## Purpose

Defines the optional positional `[PATH]` argument on `init`, `deploy`, and `undeploy`, and the optional `--cwd <PATH>` flag on `commands`, `skills`, and `config` subcommands, allowing users to target a workspace directory other than the current working directory.

## Requirements

### Requirement: init accepts an optional positional PATH argument
`dotagents init` SHALL accept an optional first positional argument `PATH`. When provided, `.dotagents/` is scaffolded inside `PATH`. When omitted, behaviour is identical to the current CWD-based behaviour.

#### Scenario: PATH omitted — scaffolds in CWD
- **WHEN** `dotagents init` is run with no `PATH` argument
- **THEN** `.dotagents/` is created inside the current working directory, identical to existing behaviour

#### Scenario: PATH provided as absolute path — scaffolds there
- **WHEN** `dotagents init /tmp/myproject` is run
- **THEN** `/tmp/myproject/.dotagents/` is created with the standard scaffold files

#### Scenario: PATH provided as relative path — resolved against CWD
- **WHEN** `dotagents init ./newdir` is run from `/home/user`
- **THEN** `/home/user/newdir/.dotagents/` is created

#### Scenario: PATH does not exist — created automatically
- **WHEN** `dotagents init /tmp/brand-new-dir` is run and `/tmp/brand-new-dir` does not exist
- **THEN** `/tmp/brand-new-dir` (and any missing parents) is created, then `.dotagents/` is scaffolded inside it

#### Scenario: PATH exists and already contains .dotagents — obeys --force / TUI overwrite
- **WHEN** `dotagents init /tmp/existing` is run and `/tmp/existing/.dotagents/` already exists
- **THEN** the existing overwrite protection logic applies unchanged (error without `--force`, or TUI confirmation in interactive mode)

#### Scenario: Interactive wizard still runs when PATH is provided
- **WHEN** `dotagents init /tmp/myproject` is run in a TTY with no `--features` or `--template` flags
- **THEN** the interactive wizard runs exactly as it would without a PATH argument

### Requirement: deploy accepts an optional positional PATH argument
`dotagents deploy` SHALL accept an optional first positional argument `PATH`. When provided, `PATH` is used as the workspace root (the directory that contains `.dotagents/`). When omitted, workspace resolution falls back to walking up from CWD as today.

#### Scenario: PATH omitted — workspace resolved from CWD
- **WHEN** `dotagents deploy` is run with no `PATH`
- **THEN** the workspace is resolved by walking up from CWD to find `.dotagents/`, identical to existing behaviour

#### Scenario: PATH provided — used as workspace root
- **WHEN** `dotagents deploy /home/user/myproject` is run and `/home/user/myproject/.dotagents/` exists
- **THEN** deploy reads config, features, and writes targets relative to `/home/user/myproject`

#### Scenario: PATH provided as relative path — resolved against CWD
- **WHEN** `dotagents deploy ../sibling` is run from `/home/user/current`
- **THEN** `/home/user/sibling` is used as the workspace root

#### Scenario: PATH does not contain .dotagents — error with clear message
- **WHEN** `dotagents deploy /tmp/no-dotagents` is run and no `.dotagents/` exists there
- **THEN** the process exits with a non-zero code and an error message indicating `.dotagents/` was not found at that path

#### Scenario: Template variable dir.workspace reflects PATH
- **WHEN** `dotagents deploy /home/user/myproject` is run
- **THEN** `{{ dir.workspace }}` in all templates resolves to `/home/user/myproject`

#### Scenario: Template variable dir.application reflects PATH
- **WHEN** `dotagents deploy /home/user/myproject` is run
- **THEN** `{{ dir.application }}` in all templates resolves to `/home/user/myproject/.dotagents`

### Requirement: undeploy accepts an optional positional PATH argument
`dotagents undeploy` SHALL accept an optional first positional argument `PATH` with identical semantics to `deploy PATH`.

#### Scenario: PATH omitted — workspace resolved from CWD
- **WHEN** `dotagents undeploy` is run with no `PATH`
- **THEN** the workspace is resolved from CWD, identical to existing behaviour

#### Scenario: PATH provided — used as workspace root
- **WHEN** `dotagents undeploy /home/user/myproject` is run and `.dotagents/` exists there
- **THEN** undeploy loads the cache from that workspace and removes the previously deployed files

#### Scenario: PATH does not contain .dotagents — error with clear message
- **WHEN** `dotagents undeploy /tmp/no-dotagents` is run and no `.dotagents/` exists there
- **THEN** the process exits non-zero with an error message indicating `.dotagents/` was not found

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
