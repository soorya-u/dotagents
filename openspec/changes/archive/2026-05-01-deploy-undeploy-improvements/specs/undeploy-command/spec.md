## ADDED Requirements

### Requirement: `undeploy` subcommand exists
`dotagents undeploy` SHALL be a valid top-level subcommand. It SHALL accept `--force` and `--no-gitignore` flags.

#### Scenario: Command is recognised
- **WHEN** the user runs `dotagents undeploy`
- **THEN** the undeploy flow starts; the process does not exit with "unknown command"

#### Scenario: --help output is available
- **WHEN** the user runs `dotagents undeploy --help`
- **THEN** a help message is printed describing the command and its flags

### Requirement: Cache is the sole source of truth for undeploy targets
`dotagents undeploy` SHALL load `cache.toml` and use the `target` field of each `CacheEntry` as the list of files to remove. No config loading, registry fetch, or template rendering SHALL be performed.

#### Scenario: Cache has entries — targets collected
- **WHEN** `cache.toml` contains entries for two providers and three features
- **THEN** all `target` paths from those entries are collected for deletion

#### Scenario: Cache is empty or missing — exit early
- **WHEN** `cache.toml` does not exist or contains no entries
- **THEN** undeploy prints "Nothing to undeploy" and exits with code 0

### Requirement: Interactive TTY confirmation before deletion
When running in an interactive TTY without `--force`, `dotagents undeploy` SHALL prompt the user to confirm before deleting any files. The prompt SHALL show the number of files that will be removed.

#### Scenario: User confirms — proceed with deletion
- **WHEN** undeploy runs in a TTY, shows "Remove 5 deployed files?", and the user selects Yes
- **THEN** deletion proceeds

#### Scenario: User declines — abort with no changes
- **WHEN** undeploy runs in a TTY and the user selects No
- **THEN** no files are deleted, `.gitignore` is not modified, cache is not cleared, exit code is 0

#### Scenario: --force skips confirmation prompt
- **WHEN** `dotagents undeploy --force` is run
- **THEN** no confirmation prompt is shown; deletion proceeds immediately

#### Scenario: Non-TTY skips prompt and proceeds
- **WHEN** undeploy runs without an interactive TTY (e.g. in CI)
- **THEN** no prompt is shown; deletion proceeds as if the user confirmed

### Requirement: Deployed files are deleted
For each `target` path in the cache, `dotagents undeploy` SHALL delete the file from the filesystem.

#### Scenario: File exists and hash matches cache — delete
- **WHEN** the target file exists on disk and its hash matches the stored `CacheEntry.hash`
- **THEN** the file is deleted

#### Scenario: File is missing — warn and continue
- **WHEN** the target file does not exist on disk
- **THEN** a warning is emitted (e.g. "already removed: <path>") and undeploy continues to the next file

### Requirement: User-edited files are handled safely
A file whose on-disk content no longer matches its stored `CacheEntry.hash` is considered user-edited.

#### Scenario: User-edited file in TTY without --force — prompt per file
- **WHEN** undeploy runs in a TTY, encounters a user-edited file, and `--force` is not set
- **THEN** the user is asked whether to delete that specific file; they can choose Yes or No

#### Scenario: User-edited file in non-TTY without --force — warn and skip
- **WHEN** undeploy runs without a TTY and encounters a user-edited file
- **THEN** a warning is emitted (e.g. "skipping user-edited file: <path>") and the file is NOT deleted

#### Scenario: User-edited file with --force — delete anyway
- **WHEN** `--force` is passed and the target file has been manually edited
- **THEN** the file is deleted without prompting

### Requirement: Empty parent directories are pruned after deletion
After deleting a file, if its immediate parent directory is now empty, `dotagents undeploy` SHALL remove that directory.

#### Scenario: Parent becomes empty — prune it
- **WHEN** the last file in `.claude/commands/` is deleted during undeploy
- **THEN** the `.claude/commands/` directory is removed

#### Scenario: Parent still has other files — leave it
- **WHEN** `.claude/commands/` contains files not managed by dotagents after undeploy
- **THEN** `.claude/commands/` is left intact

### Requirement: dotagents-managed .gitignore fence is removed
After files are deleted, `dotagents undeploy` SHALL remove the entire `# BEGIN dotagents managed` … `# END dotagents managed` fenced section from the workspace root `.gitignore`. Content outside the fence SHALL NOT be modified.

#### Scenario: Fence exists — remove it
- **WHEN** `.gitignore` contains the dotagents-managed fence
- **THEN** after undeploy, the fence and its contents are gone; user entries outside the fence are preserved

#### Scenario: No fence exists — no change
- **WHEN** `.gitignore` does not contain a dotagents-managed fence
- **THEN** `.gitignore` is not modified

#### Scenario: --no-gitignore flag skips fence removal
- **WHEN** `dotagents undeploy --no-gitignore` is run
- **THEN** `.gitignore` is not read or modified

### Requirement: Cache is cleared after undeploy
After files are deleted and gitignore is updated, `dotagents undeploy` SHALL write an empty `cache.toml`, removing all stored entries.

#### Scenario: Cache is emptied on success
- **WHEN** undeploy completes successfully
- **THEN** `cache.toml` contains no provider entries

### Requirement: TTY undeploy summary after completion
When running in an interactive TTY, `dotagents undeploy` SHALL print a one-line summary of how many files were removed.

#### Scenario: Files removed
- **WHEN** undeploy runs in a TTY and removes 4 files
- **THEN** a summary is printed, e.g. "✓ 4 files removed"

#### Scenario: All files were skipped (user-edited, non-TTY)
- **WHEN** all target files were skipped due to user edits
- **THEN** summary indicates 0 removed, e.g. "✓ 0 files removed (3 skipped)"
