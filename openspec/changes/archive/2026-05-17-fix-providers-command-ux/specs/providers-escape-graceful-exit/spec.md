## ADDED Requirements

### Requirement: TUI Escape exits providers command cleanly
When the user presses Escape or Ctrl-C during the interactive provider select prompt, the `providers` command SHALL exit with code 0 and produce no error output. It SHALL NOT print a fatal error box.

#### Scenario: Escape key exits cleanly
- **WHEN** `dotagents providers` is run in a TTY
- **WHEN** the user presses Escape during the provider select prompt
- **THEN** the process exits with code 0
- **THEN** stderr does NOT contain "Fatal error" or "Failed to"

#### Scenario: Ctrl-C exits cleanly
- **WHEN** `dotagents providers` is run in a TTY
- **WHEN** the user sends SIGINT (Ctrl-C)
- **THEN** the process exits without printing a fatal error box
