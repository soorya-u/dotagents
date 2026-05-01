## ADDED Requirements

### Requirement: TTY deploy summary after completion
After all features are deployed and the cache is saved, `dotagents deploy` SHALL print a one-line summary to stdout when running in an interactive TTY. The summary SHALL report the total number of files written and the total number skipped across all features and providers.

#### Scenario: Files written and skipped
- **WHEN** deploy runs in a TTY and writes 3 files while skipping 2
- **THEN** a summary line is printed, e.g. "✓ 3 written, 2 skipped"

#### Scenario: All files skipped (nothing changed)
- **WHEN** deploy runs in a TTY and all files are up-to-date (0 written, N skipped)
- **THEN** a summary line is printed indicating nothing was written, e.g. "✓ 0 written, 4 skipped"

#### Scenario: No files at all
- **WHEN** deploy runs in a TTY and no features are enabled or no providers are configured
- **THEN** a summary line is printed indicating nothing was deployed, e.g. "✓ Nothing deployed"

#### Scenario: Non-TTY (CI) — no summary output
- **WHEN** deploy runs without an interactive TTY (e.g. piped in CI)
- **THEN** no summary line is printed; stdout is unchanged

### Requirement: Summary printed before gitignore step
The deploy summary SHALL be printed after cache persistence and before the gitignore update prompt, so the user sees confirmation of the deploy before being asked about gitignore.

#### Scenario: Summary appears before gitignore prompt
- **WHEN** deploy writes files and then prompts for gitignore update
- **THEN** the deploy summary is visible on screen before the gitignore prompt appears
