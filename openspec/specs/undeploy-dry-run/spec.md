## Purpose

Defines the behaviour of the `--dry-run` flag on the `undeploy` command. When set, the command reads `cache.toml` and checks on-disk file hashes but does not delete any files, clear the cache, or remove the `.gitignore` fence, and prints a preview of what would change.

## Requirements

### Requirement: --dry-run flag on undeploy
The `undeploy` command SHALL accept a `--dry-run` boolean flag. When set, the command SHALL read `cache.toml` and check on-disk file hashes but SHALL NOT delete any files, clear the cache, or remove the `.gitignore` fence. The bulk-confirmation prompt SHALL be suppressed.

#### Scenario: Flag is recognised
- **WHEN** user runs `dotagents undeploy --dry-run`
- **THEN** the command exits without error and without deleting any files

#### Scenario: Confirmation prompt suppressed
- **WHEN** user runs `dotagents undeploy --dry-run` in an interactive terminal
- **THEN** no confirmation prompt is shown

---

### Requirement: Would-delete status in dry-run output
When `--dry-run` is set, for each cached path whose on-disk hash matches the cached hash (file unmodified), the command SHALL print `[-] <path>`.

#### Scenario: Unmodified cached file
- **WHEN** `--dry-run` is set and a cached file's on-disk content matches its cached hash
- **THEN** stdout contains `[-] <path>` for that entry

---

### Requirement: Edited-file status in dry-run output
When `--dry-run` is set, for each cached path whose on-disk hash differs from the cached hash (file was edited), the command SHALL print `[x] <path>  (edited)`.

#### Scenario: Modified cached file
- **WHEN** `--dry-run` is set and a cached file's on-disk content has been modified since deploy
- **THEN** stdout contains `[x] <path>  (edited)` for that entry

---

### Requirement: Missing files warned in dry-run output
When `--dry-run` is set, cached paths that do not exist on disk SHALL be reported as a warning and excluded from the affected count.

#### Scenario: Cached file missing from disk
- **WHEN** `--dry-run` is set and a cached path does not exist on disk
- **THEN** a warning is printed for that path and it is not counted as affected

---

### Requirement: No cache clear in dry-run
When `--dry-run` is set, `cache.toml` SHALL NOT be modified.

#### Scenario: Cache unchanged after dry-run
- **WHEN** user runs `dotagents undeploy --dry-run`
- **THEN** `cache.toml` is byte-identical to before the command ran

---

### Requirement: No gitignore fence removal in dry-run
When `--dry-run` is set, the dotagents-managed fence in `.gitignore` SHALL NOT be removed.

#### Scenario: .gitignore untouched after dry-run
- **WHEN** user runs `dotagents undeploy --dry-run`
- **THEN** `.gitignore` is unchanged

---

### Requirement: Summary line in dry-run output
After listing entries, the command SHALL print `N files would be affected` where N is the count of `[-]` and `[x]` entries.

#### Scenario: Summary shown after path list
- **WHEN** `--dry-run` is set and 3 files would be deleted
- **THEN** stdout ends with `3 files would be affected`

#### Scenario: Empty cache dry-run
- **WHEN** `--dry-run` is set and `cache.toml` has no entries
- **THEN** the command exits 0 with `0 files would be affected`
