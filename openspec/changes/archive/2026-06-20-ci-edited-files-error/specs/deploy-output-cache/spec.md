## MODIFIED Requirements

### Requirement: Detect and preserve user-edited target files
When the stored hash indicates the file should be unchanged but the on-disk content has diverged (user manually edited the target), `dotagents deploy` SHALL skip writing the file and track it as user-edited. In non-TTY (CI) mode, the presence of any user-edited files without `--force` SHALL cause deploy to exit with status 1 and display a summary error with the count of edited files. In TTY mode, deploy SHALL exit with status 0 and display the count in the summary.

#### Scenario: User-edited file — skip and count
- **WHEN** the rendered output hash matches the stored cache hash but the target file content does not match the stored hash
- **THEN** deploy SHALL skip writing the file, emit a debug-level log identifying the file path, increment the user-edited counter, and continue to the next item

#### Scenario: Non-TTY deploy fails on user-edited files
- **WHEN** deploy runs in non-TTY mode (CI), one or more files were manually edited, and `--force` is not passed
- **THEN** deploy SHALL exit with status 1 and display an error message stating the number of files that were manually edited and suggesting `--force` to override

#### Scenario: Non-TTY deploy with --force succeeds despite edited files
- **WHEN** deploy runs in non-TTY mode (CI) with `--force` and target files have been manually edited
- **THEN** the files SHALL be overwritten with the rendered output, cache entries SHALL be updated, and deploy SHALL exit with status 0

#### Scenario: TTY deploy succeeds with edited file count in summary
- **WHEN** deploy runs in TTY mode and one or more files were manually edited without `--force`
- **THEN** deploy SHALL exit with status 0 and the summary SHALL include the count of user-edited files

#### Scenario: Force flag overrides user-edit skip
- **WHEN** `--force` is passed and a target file has been manually edited
- **THEN** the file is overwritten with the rendered output and the cache entry is updated
