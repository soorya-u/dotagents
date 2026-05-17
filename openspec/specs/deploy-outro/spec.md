## Purpose

Defines the output summary printed at the end of `dotagents deploy` to confirm what was written and what was skipped.

## Requirements

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

#### Scenario: Non-TTY (CI) — plain-text summary printed
- **WHEN** deploy runs without an interactive TTY (e.g. piped in CI)
- **THEN** a summary line is printed to stdout as plain text without the `"✓ "` prefix, e.g. `"3 written, 2 skipped"` or `"Nothing deployed"`

### Requirement: Summary printed before gitignore step
The deploy summary SHALL be printed after cache persistence and before the gitignore update prompt, so the user sees confirmation of the deploy before being asked about gitignore.

#### Scenario: Summary appears before gitignore prompt
- **WHEN** deploy writes files and then prompts for gitignore update
- **THEN** the deploy summary is visible on screen before the gitignore prompt appears

### Requirement: Deploy warns when no providers are configured
If no providers are configured in the workspace config at deploy time, `dotagents deploy` SHALL emit a `warn!()` log message advising the user to add providers to their config.

#### Scenario: No providers configured emits warning
- **WHEN** `dotagents deploy` is run and `app_config` has no provider entries for any feature
- **THEN** a warning is printed: `"No providers configured — nothing to deploy. Add providers to config.toml."`
- **THEN** the command still exits 0

#### Scenario: Providers configured suppresses the warning
- **WHEN** `dotagents deploy` is run and at least one provider is configured
- **THEN** no missing-providers warning is emitted

### Requirement: Deploy is framed with cliclack intro and outro in TTY mode
When `dotagents deploy` runs interactively, it SHALL open with a cliclack `intro` banner and close with a cliclack `outro` message. The intro SHALL appear before the offline prompt. The outro SHALL appear after the gitignore step completes (or is skipped), at all TTY exit paths. In non-TTY mode neither intro nor outro is printed.

#### Scenario: Intro printed before first prompt
- **WHEN** `dotagents deploy` runs in a TTY
- **THEN** a `┌ …` intro line is printed before the offline mode select prompt

#### Scenario: Outro printed after gitignore step
- **WHEN** deploy completes and the gitignore step finishes (accepted, declined, or skipped)
- **THEN** a `└ …` outro line is printed as the final output

#### Scenario: No intro or outro in non-TTY mode
- **WHEN** deploy runs in a non-interactive environment
- **THEN** no intro or outro lines appear in stdout

### Requirement: Intro and outro text for subcommands is descriptive, not the command path
The `intro()` text for `skills new`, `skills rm`, `commands new`, and `commands rm` SHALL be a short descriptive phrase and SHALL NOT mirror the command path the user typed. All `rm` and `new` intro calls SHALL be gated on `is_tty()`.

#### Scenario: skills new intro text
- **WHEN** `dotagents skills new` runs interactively
- **THEN** the intro reads `New skill`, not `dotagents skills new`

#### Scenario: skills rm intro text
- **WHEN** `dotagents skills rm` runs interactively
- **THEN** the intro reads `Remove skill`, not `dotagents skills rm`

#### Scenario: commands new intro text
- **WHEN** `dotagents commands new` runs interactively
- **THEN** the intro reads `New command`, not `dotagents commands new`

#### Scenario: commands rm intro text
- **WHEN** `dotagents commands rm` runs interactively
- **THEN** the intro reads `Remove command`, not `dotagents commands rm`

#### Scenario: rm intro not shown in non-TTY
- **WHEN** `dotagents skills rm my-skill --force` runs in a non-interactive environment
- **THEN** no intro line is printed to stdout
