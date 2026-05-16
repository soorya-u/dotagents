## MODIFIED Requirements

### Requirement: --json and --edit are mutually exclusive
`dotagents config --json --edit` SHALL exit with a Clap usage error (exit code 2) and a message indicating the flags conflict. It SHALL NOT silently ignore `--edit` and run in JSON mode.

#### Scenario: --json and --edit together produce a parse error
- **WHEN** `dotagents config --json --edit` is run
- **THEN** the process exits with code 2
- **THEN** stderr contains a usage error message indicating `--json` and `--edit` conflict
- **THEN** no config output is printed to stdout
