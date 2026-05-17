### Requirement: skills add passes --yes in CI mode
When `skills add` is invoked in a non-TTY (CI) environment, the downstream package runner subprocess SHALL receive `--yes` as an additional argument so it does not block waiting for interactive confirmation.

#### Scenario: npm runner in CI mode appends --yes
- **WHEN** `PackageRunner::Npm.args("my-skill", true)` is called
- **THEN** the returned argv list ends with `"--yes"`

#### Scenario: pnpm runner in CI mode appends --yes
- **WHEN** `PackageRunner::Pnpm.args("my-skill", true)` is called
- **THEN** the returned argv list ends with `"--yes"`

#### Scenario: yarn runner in CI mode appends --yes
- **WHEN** `PackageRunner::Yarn.args("my-skill", true)` is called
- **THEN** the returned argv list ends with `"--yes"`

#### Scenario: bun runner in CI mode appends --yes
- **WHEN** `PackageRunner::Bun.args("my-skill", true)` is called
- **THEN** the returned argv list ends with `"--yes"`

#### Scenario: interactive mode does not append --yes
- **WHEN** `PackageRunner::Npm.args("my-skill", false)` is called
- **THEN** the returned argv list does NOT contain `"--yes"`
