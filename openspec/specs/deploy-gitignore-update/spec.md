### Requirement: Collect rendered target paths during deploy
`dotagents deploy` SHALL collect the absolute path of every file it successfully writes during a deploy run and make this list available to the gitignore update step.

#### Scenario: Paths collected across all features and providers
- **WHEN** deploy writes `.claude/commands/hello.md`, `CLAUDE.md`, and `.github/copilot-instructions.md`
- **THEN** all three paths are available as the collected target path list at the end of deploy

#### Scenario: No files written — empty list
- **WHEN** deploy runs but writes no files (e.g. all providers disabled)
- **THEN** the collected target path list is empty and the gitignore update step is skipped entirely

### Requirement: Update workspace root .gitignore with fenced section
When the gitignore update step runs, it SHALL write workspace-relative target paths into a dotagents-managed fenced section in the workspace root `.gitignore`. Lines outside the fenced section SHALL NOT be modified.

#### Scenario: .gitignore does not exist — create it
- **WHEN** no `.gitignore` exists at the workspace root
- **THEN** a new `.gitignore` is created containing only the dotagents fenced section with the collected paths

#### Scenario: .gitignore exists without fenced section — append section
- **WHEN** `.gitignore` exists with user content but no dotagents fence
- **THEN** the fenced section is appended at the end; existing content is preserved verbatim

#### Scenario: .gitignore exists with fenced section — add new paths only
- **WHEN** the fenced section already contains some paths and deploy wrote additional new paths
- **THEN** only the new paths are appended inside the fence; existing entries and user content outside the fence are unchanged

#### Scenario: All paths already present — no write
- **WHEN** every collected target path is already listed inside the fenced section
- **THEN** `.gitignore` is not modified

#### Scenario: User content outside fence is preserved
- **WHEN** `.gitignore` contains user entries before and after the dotagents fenced section
- **THEN** after update, those entries remain exactly as they were

### Requirement: Entries are specific workspace-relative file paths
Each entry written to the fenced section SHALL be the workspace-relative path of the deployed file (e.g. `.claude/commands/hello.md`). Directory patterns and wildcards SHALL NOT be used.

#### Scenario: Specific path for file in subdirectory
- **WHEN** deploy writes `.github/copilot-instructions.md`
- **THEN** the gitignore entry is `.github/copilot-instructions.md`, not `.github/` or `.github/*`

#### Scenario: Specific path for root-level file
- **WHEN** deploy writes `CLAUDE.md` at the workspace root
- **THEN** the gitignore entry is `CLAUDE.md`

### Requirement: Stale entries accumulate harmlessly
The gitignore update step SHALL only add paths — it SHALL NOT remove entries from the fenced section, including entries for targets that are no longer configured.

#### Scenario: Removed target path stays in fence
- **WHEN** a provider is removed from config and deploy no longer writes `AGENTS.md`
- **THEN** the `AGENTS.md` entry remains in the fenced section after the next deploy

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
