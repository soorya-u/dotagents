## MODIFIED Requirements

### Requirement: TUI mode provides interactive scrollable provider list
When stdin is a TTY and `--json` is not passed, `dotagents providers ls` SHALL display an interactive scrollable selection list using cliclack. Each option SHALL show only the provider slug as the label. No hint text SHALL be displayed on list items. Selecting a provider and pressing Enter SHALL display the provider's name and URL in the closing outro as `Name (url)`. Esc or Ctrl+C SHALL exit.

#### Scenario: TUI shows all providers in a scrollable list with slug labels
- **WHEN** `dotagents providers ls` is run in a TTY
- **THEN** a cliclack select prompt titled "Providers" is shown with all providers listed by slug, capped at 10 visible rows

#### Scenario: TUI shows provider details on selection
- **WHEN** the user selects a provider and presses Enter
- **THEN** the closing message displays the provider's name and URL (e.g. `Claude Code (https://docs.anthropic.com/en/docs/claude-code)`); if name/url are absent, a fallback of "Done" is shown

#### Scenario: TUI select label is slug only
- **WHEN** `dotagents providers ls` is run in a TTY and a provider has name "Claude Code" and slug "claude"
- **THEN** the select list item label is "claude", not "Claude Code"
