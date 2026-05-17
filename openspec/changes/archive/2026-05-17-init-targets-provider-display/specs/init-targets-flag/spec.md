## ADDED Requirements

### Requirement: --targets flag selects provider targets for init
`dotagents init` SHALL accept an optional `--targets` flag that takes a comma-separated list of provider slugs and/or may be repeated. When `--targets` is absent and TUI is available, the wizard SHALL prompt for targets. When `--targets` is absent and TUI is not available (CI/non-TTY), targets SHALL default to an empty list.

#### Scenario: Single target via comma-separated value
- **WHEN** user runs `dotagents init --targets claude,cursor`
- **THEN** config is generated with `targets = ["claude", "cursor"]` and no target prompt is shown

#### Scenario: Multiple targets via repeated flag
- **WHEN** user runs `dotagents init --targets claude --targets cursor`
- **THEN** config is generated with `targets = ["claude", "cursor"]`, same result as comma-separated form

#### Scenario: --targets skips only the target prompt in TUI
- **WHEN** user runs `dotagents init --targets claude` in an interactive terminal without `--features` or `--template`
- **THEN** the features and template wizard prompts are shown, but the target selection prompt is skipped
- **THEN** config contains `targets = ["claude"]`

#### Scenario: --targets in CI mode
- **WHEN** user runs `dotagents --ci init --targets claude`
- **THEN** no prompts are shown, config contains `targets = ["claude"]`, and features default to empty
