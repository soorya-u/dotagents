## Purpose

Defines the invariant around when the workspace directory is created during `dotagents init`. The directory SHALL NOT be created before the TUI wizard completes, ensuring that cancelling the wizard leaves no filesystem trace.

## Requirements

### Requirement: Cancelling the init wizard leaves no filesystem trace
When the user cancels the `dotagents init` TUI wizard (e.g. by selecting No at any prompt or pressing Ctrl-C), the CLI SHALL exit 0 without creating any directory or file on disk — including the workspace directory itself.

#### Scenario: User cancels at first prompt — no directory created
- **WHEN** `dotagents init` is run in a fresh directory with no pre-existing workspace, and the user cancels the wizard before completing it
- **THEN** the process exits 0 and no new directory has been created in the working directory

#### Scenario: User accepts wizard — workspace directory is created
- **WHEN** `dotagents init` is run and the user completes the wizard without cancelling
- **THEN** the `.dotagents/` directory is created and all scaffold files are written

### Requirement: Workspace directory is not created before TUI prompts run
`fs::create_dir_all` for the workspace root SHALL NOT be called before the TUI wizard block executes. The call SHALL occur after all prompts complete and the user has confirmed they want to proceed.

#### Scenario: Workspace parent already exists
- **WHEN** the workspace root directory already exists and the user completes the wizard
- **THEN** `dotagents init` succeeds and `.dotagents/` is created inside it (unchanged behavior)

#### Scenario: Workspace parent does not exist and user cancels
- **WHEN** the workspace root directory does not yet exist and the user cancels the wizard
- **THEN** neither the workspace root nor any subdirectory is created

#### Scenario: try_exists check works without pre-created parent
- **WHEN** `dotagents init` is run in a directory where the workspace path does not yet exist
- **THEN** the existing-directory detection (checking for `.dotagents/` inside the workspace) correctly reports false without requiring the parent to be created first
