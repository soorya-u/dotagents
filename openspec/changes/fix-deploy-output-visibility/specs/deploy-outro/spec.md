## MODIFIED Requirements

### Requirement: Deploy summary printed in both TTY and CI mode
After all features are deployed and the cache is saved, `dotagents deploy` SHALL print a one-line summary to stdout regardless of whether stdin is a TTY. In TTY mode the summary SHALL include a `"✓ "` prefix. In non-TTY (CI) mode the summary SHALL be printed as plain text without the checkmark prefix.

#### Scenario: Files written and skipped in TTY
- **WHEN** deploy runs in a TTY and writes 3 files while skipping 2
- **THEN** a summary line is printed, e.g. `"✓ 3 written, 2 skipped"`

#### Scenario: Files written and skipped in CI mode
- **WHEN** deploy runs in CI (non-TTY) and writes 3 files while skipping 2
- **THEN** a summary line is printed to stdout, e.g. `"3 written, 1 skipped"`

#### Scenario: No files deployed in CI mode
- **WHEN** deploy runs in CI and no files are written or skipped
- **THEN** a summary line is printed to stdout, e.g. `"Nothing deployed"`

## ADDED Requirements

### Requirement: Deploy warns when no providers are configured
If no providers are configured in the workspace config at deploy time, `dotagents deploy` SHALL emit a `warn!()` log message advising the user to add providers to their config.

#### Scenario: No providers configured emits warning
- **WHEN** `dotagents deploy` is run and `app_config` has no provider entries for any feature
- **THEN** a warning is printed: `"No providers configured — nothing to deploy. Add providers to config.toml."`
- **THEN** the command still exits 0

#### Scenario: Providers configured suppresses the warning
- **WHEN** `dotagents deploy` is run and at least one provider is configured
- **THEN** no missing-providers warning is emitted
