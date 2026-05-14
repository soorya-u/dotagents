### Requirement: Collect rendered target paths during deploy
`dotagents deploy` SHALL collect the absolute path of every file it successfully writes during a deploy run and make this list available to the gitignore update step.

#### Scenario: Paths collected across all features and providers
- **WHEN** deploy writes `.claude/commands/hello.md`, `CLAUDE.md`, and `.github/copilot-instructions.md`
- **THEN** all three paths are available as the collected target path list at the end of deploy

#### Scenario: No files written — empty list
- **WHEN** deploy runs but writes no files (e.g. all providers disabled)
- **THEN** the collected target path list is empty and the gitignore update step is skipped entirely

### Requirement: Update workspace root .gitignore with fenced section
When the gitignore update step runs, it SHALL rebuild the fenced section from all cached target paths using the collapse algorithm. The fence is rewritten from scratch each time — not appended to. Lines outside the fenced section SHALL NOT be modified. The fenced section SHALL use `#region dotagents` as the opening marker and `#endregion dotagents` as the closing marker.

#### Scenario: .gitignore does not exist — create it
- **WHEN** no `.gitignore` exists at the workspace root
- **THEN** a new `.gitignore` is created containing only the dotagents fenced section with collapsed patterns

#### Scenario: .gitignore exists without fenced section — append section
- **WHEN** `.gitignore` exists with user content but no dotagents fence
- **THEN** the fenced section is appended at the end with `#region dotagents` / `#endregion dotagents` markers; existing content is preserved verbatim

#### Scenario: .gitignore exists with fenced section — rebuild fence
- **WHEN** the `#region dotagents` / `#endregion dotagents` section already exists
- **THEN** the fenced section is completely rewritten with the current collapsed patterns; existing user content outside the fence is unchanged

#### Scenario: All patterns unchanged — no write
- **WHEN** the rebuilt fence content is identical to the existing fence content
- **THEN** `.gitignore` is not modified

#### Scenario: User content outside fence is preserved
- **WHEN** `.gitignore` contains user entries before and after the dotagents fenced section
- **THEN** after rebuild, those entries remain exactly as they were

### Requirement: Entries are specific workspace-relative file paths
Each entry written to the fenced section SHALL be either a workspace-relative file path (e.g. `.claude/commands/hello.md`) or a workspace-relative directory pattern with trailing slash (e.g. `.claude/commands/`) when the directory's entire contents are generated. The collapse algorithm SHALL determine which format to use for each entry.

#### Scenario: Specific path for root-level file
- **WHEN** deploy writes `CLAUDE.md` at the workspace root
- **THEN** the gitignore entry is `CLAUDE.md`

#### Scenario: Directory pattern for fully-generated directory
- **WHEN** deploy writes 8 files into `.claude/commands/` and no other files exist in that directory
- **THEN** the gitignore entry is `.claude/commands/` (single directory pattern)

#### Scenario: Mixed directory gets individual entries
- **WHEN** deploy writes files into `.claude/commands/` but the directory also contains a user-created file
- **THEN** each generated file gets its own gitignore entry

### Requirement: --gitignore flag updates without prompting
When `--gitignore` is passed to `dotagents deploy`, the gitignore update step SHALL run automatically after deploy without any interactive prompt.

#### Scenario: Force update with flag
- **WHEN** `dotagents deploy --gitignore` is run
- **THEN** the workspace root `.gitignore` is updated with new target paths and no prompt is shown

### Requirement: --no-gitignore flag skips update entirely
When `--no-gitignore` is passed to `dotagents deploy`, the gitignore update step SHALL be skipped entirely — `.gitignore` is not read or written.

#### Scenario: Skip with flag
- **WHEN** `dotagents deploy --no-gitignore` is run
- **THEN** `.gitignore` is not modified regardless of what was deployed

### Requirement: Default mode prompts the user interactively
When neither `--gitignore` nor `--no-gitignore` is passed, `dotagents deploy` SHALL prompt the user after deploy completes with the count of new paths using a cliclack `select` prompt with `Yes` and `No` options. Default selection is `No`.

#### Scenario: User selects Yes — update runs
- **WHEN** the cliclack select prompt is shown and the user selects `Yes`
- **THEN** the gitignore update step runs

#### Scenario: User selects No or accepts default — update skipped
- **WHEN** the cliclack select prompt is shown and the user selects `No` or presses Enter on the default
- **THEN** `.gitignore` is not modified

#### Scenario: No new paths to add — prompt is skipped
- **WHEN** all collected target paths are already present in the fenced section
- **THEN** no prompt is shown

### Requirement: Non-TTY environments skip the prompt silently
When running in a non-interactive environment (CI, piped output), `dotagents deploy` SHALL detect the absence of a TTY and behave as if `--no-gitignore` was passed — no prompt, no update.

#### Scenario: Non-TTY skips update
- **WHEN** deploy runs with no TTY attached (e.g. in a CI pipeline)
- **THEN** `.gitignore` is not modified and no prompt output is produced

### Requirement: gitignore write failure is non-fatal
If writing to `.gitignore` fails (e.g. permission error), the error SHALL be reported as a warning and deploy SHALL exit with success. The deploy output itself is not affected.

#### Scenario: Permission error on .gitignore
- **WHEN** `.gitignore` exists but is not writable
- **THEN** a warning is logged, deploy exits 0, and all deployed files are still written correctly
