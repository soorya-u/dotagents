## ADDED Requirements

### Requirement: Two init templates are available
`dotagents init` SHALL support two mutually exclusive scaffolding templates: `starter` and `with-custom-provider`. The template controls which files are written and which `local.config.toml` variant is used.

#### Scenario: Starter template file set
- **WHEN** the Starter template is selected
- **THEN** the following files are written and no others: `.env`, `.env.example`, `.gitignore`, `config.toml`, `local.config.toml` (minimal, no providers block), `INSTRUCTIONS.md` (if instructions enabled), `mcp.jsonc` (if mcp enabled), `commands/hello.md` (if commands enabled), `skills/hello-skill/SKILL.md` (if skills enabled)

#### Scenario: With Custom Provider template adds example provider files
- **WHEN** the With Custom Provider template is selected
- **THEN** all Starter files are written AND additionally: `templates/mycode/command.hbs`, `templates/mycode/skill.hbs`, `templates/mycode/instructions.hbs`, `templates/mycode/mcp.hbs`, and `local.config.toml` includes the mycode provider block

### Requirement: --template flag selects template non-interactively
When `--template <starter|with-custom-provider>` is passed, the template selection prompt SHALL NOT be shown and the specified template SHALL be used.

#### Scenario: --template starter
- **WHEN** `dotagents init --template starter` is run
- **THEN** the Starter file set is written and no template prompt is shown

#### Scenario: --template with-custom-provider
- **WHEN** `dotagents init --template with-custom-provider` is run
- **THEN** the With Custom Provider file set is written and no template prompt is shown

### Requirement: Template select prompt defaults to Starter
When the wizard shows the template selection prompt, `Starter` SHALL be the default-highlighted option.

#### Scenario: User accepts default template
- **WHEN** the template select prompt is shown and the user presses Enter without moving
- **THEN** the Starter template is used

### Requirement: Starter local.config.toml has no providers block
The `local.config.toml` written by the Starter template SHALL contain only a `features` list and an empty `targets` array, with no `[providers.*]` sections.

#### Scenario: Starter local.config.toml content
- **WHEN** init completes with Starter template
- **THEN** `.dotagents/local.config.toml` contains `features` and `targets = []` but no `[providers]` table

### Requirement: With Custom Provider local.config.toml includes mycode provider
The `local.config.toml` written by the With Custom Provider template SHALL include the full mycode provider block covering all four features (commands, instructions, mcp, skills).

#### Scenario: With Custom Provider local.config.toml content
- **WHEN** init completes with With Custom Provider template
- **THEN** `.dotagents/local.config.toml` contains `targets = ["mycode"]` and `[providers.mycode.*]` sections for each feature
