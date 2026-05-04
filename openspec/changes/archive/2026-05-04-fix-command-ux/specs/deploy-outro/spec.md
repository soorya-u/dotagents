## ADDED Requirements

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
