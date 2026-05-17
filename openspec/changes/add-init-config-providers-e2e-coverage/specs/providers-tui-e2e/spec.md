## ADDED Requirements

### Requirement: E2e TUI test for interactive provider list
Verify that the providers command renders an interactive select widget with navigation and selection.

#### Scenario: TUI select widget renders and is navigable (TC-PROV-01)
- **WHEN** `providers --offline` is run interactively in a TTY with a seeded registry cache (see `template-source-cache` spec for cache seeding behavior)
- **THEN** the select widget renders with "Providers" title, provider entries are visible with name and URL hints, arrow-down moves the selection indicator, and Enter selects a provider showing an outro with the provider name and URL

#### Scenario: Escape cancels the TUI widget (TC-PROV-01)
- **WHEN** the provider select widget is shown and the user presses Escape
- **THEN** the widget is dismissed and the process exits (exit code 1 with "operation interrupted" — known UX issue documented separately)
