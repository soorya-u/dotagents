## Purpose

Defines the optional positional `[PATH]` argument on `init`, `deploy`, and `undeploy`, allowing users to target a workspace directory other than the current working directory.

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
