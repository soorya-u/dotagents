## Purpose

Defines the behaviour of the `--dry-run` flag on the `deploy` command. When set, the command performs all rendering and cache-comparison logic without writing any files, updating `cache.toml`, or modifying `.gitignore`, and prints a preview of what would change.

## Requirements

### Requirement: --dry-run flag on deploy
The `deploy` command SHALL accept a `--dry-run` boolean flag. When set, the command SHALL perform all template rendering, provider resolution, and cache-comparison logic but SHALL NOT write any files to disk, save `cache.toml`, or modify `.gitignore`.

#### Scenario: Flag is recognised
- **WHEN** user runs `dotagents deploy --dry-run`
- **THEN** the command exits without error and without writing any files

#### Scenario: Flag combined with other deploy flags
- **WHEN** user runs `dotagents deploy --dry-run --offline` or `dotagents deploy --dry-run --no-cache`
- **THEN** the peer flags are honoured (e.g. `--offline` skips registry fetch) and dry-run suppression of side effects still applies

---

### Requirement: New-file status in dry-run output
When `--dry-run` is set, for each target path that does not currently exist on disk, the command SHALL print `[+] <path>`.

#### Scenario: Target file does not exist
- **WHEN** `--dry-run` is set and a rendered template targets a path that does not exist
- **THEN** stdout contains `[+] <path>` for that entry

---

### Requirement: Modified-file status in dry-run output
When `--dry-run` is set, for each target path that exists on disk and whose on-disk content differs from the rendered output, the command SHALL print `[~] <path>`.

#### Scenario: Target file exists with different content
- **WHEN** `--dry-run` is set and a rendered template targets a path that exists but whose content differs
- **THEN** stdout contains `[~] <path>` for that entry

---

### Requirement: Unchanged files hidden in dry-run output
When `--dry-run` is set, target paths whose on-disk content is byte-identical to the rendered output SHALL NOT appear in the output.

#### Scenario: Target file is unchanged
- **WHEN** `--dry-run` is set and a rendered template produces content identical to the existing file
- **THEN** that path does not appear in stdout

---

### Requirement: No cache write in dry-run
When `--dry-run` is set, `cache.toml` SHALL NOT be written or modified.

#### Scenario: Cache file unchanged after dry-run
- **WHEN** user runs `dotagents deploy --dry-run`
- **THEN** `cache.toml` on disk is byte-identical to what it was before the command ran

---

### Requirement: No gitignore update in dry-run
When `--dry-run` is set, `.gitignore` SHALL NOT be updated, and no gitignore-related prompts SHALL be shown.

#### Scenario: .gitignore untouched after dry-run
- **WHEN** user runs `dotagents deploy --dry-run`
- **THEN** `.gitignore` is unchanged and no consent prompt appears

---

### Requirement: Template errors surface in dry-run
When `--dry-run` is set, template rendering errors or config load failures SHALL still be reported and the command SHALL exit with code 1.

#### Scenario: Bad template during dry-run
- **WHEN** `--dry-run` is set and a template contains a Handlebars syntax error or references an undefined variable
- **THEN** the error is printed to stderr and the process exits with code 1

---

### Requirement: Summary line in dry-run output
After listing affected paths, the command SHALL print `N files would be affected` where N is the count of `[+]` and `[~]` entries. When no files would be affected, the count SHALL be 0.

#### Scenario: Summary shown after path list
- **WHEN** `--dry-run` is set and 2 files would be written
- **THEN** stdout ends with `2 files would be affected`

#### Scenario: Empty dry-run summary
- **WHEN** `--dry-run` is set and all rendered outputs are identical to on-disk files
- **THEN** stdout contains `0 files would be affected` and exits with code 0
