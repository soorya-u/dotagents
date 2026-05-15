## ADDED Requirements

### Requirement: E2e TUI test for offline prompt Yes path
Verify that navigating to "Yes" on the offline prompt enables offline mode.

#### Scenario: Offline prompt Yes selects offline mode (TC-DEPLOY-16)
- **WHEN** the TUI offline prompt appears during deploy and the user navigates down to "Yes" and presses Enter
- **THEN** deploy completes successfully, the summary is displayed, and exit code is 0

### Requirement: E2e TUI test for full deploy journey
Verify the complete interactive deploy flow from start to finish.

#### Scenario: Full TUI deploy journey (TC-DEPLOY-01)
- **WHEN** `deploy` is run interactively in a TTY with a fresh workspace
- **THEN** the offline prompt appears (default No), pressing Enter proceeds, the gitignore prompt appears (default No), pressing Enter proceeds, deploy summary is displayed, and "Done." is shown
