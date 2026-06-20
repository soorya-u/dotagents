## ADDED Requirements

### Requirement: CI deploy fails on user-edited files
When `dotagents deploy` runs in CI mode (non-TTY) and detects user-edited target files without `--force`, it SHALL exit with status 1 and display a concise error message with the count of edited files. Individual file paths SHALL NOT be listed in the error output.

#### Scenario: CI deploy exits 1 when edited files detected
- **WHEN** `dotagents deploy` runs in non-TTY mode, at least one target file was manually edited, and `--force` is not passed
- **THEN** the process SHALL exit with status 1

#### Scenario: CI deploy error message shows count not paths
- **WHEN** `dotagents deploy` exits with status 1 due to user-edited files in non-TTY mode
- **THEN** the error output SHALL state the number of edited files and suggest `--force` to override, without listing individual file paths

#### Scenario: CI deploy with --force exits 0 despite edited files
- **WHEN** `dotagents deploy --force` runs in non-TTY mode and target files were manually edited
- **THEN** the process SHALL exit with status 0 after overwriting all files
