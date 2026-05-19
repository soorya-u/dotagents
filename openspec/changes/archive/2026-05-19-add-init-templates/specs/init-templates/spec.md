## MODIFIED Requirements

### Requirement: Three init templates are available
`dotagents init` SHALL support three mutually exclusive scaffolding templates: `blank`, `starter`, and `advanced`. The template controls which files are written.

#### Scenario: Blank template file set
- **WHEN** the Blank template is selected
- **THEN** the following files are written and no others: `config.toml`, `.gitignore`, `INSTRUCTIONS.md` (if instructions enabled), `mcp.jsonc` (if mcp enabled), `commands/hello.md` (if commands enabled), `skills/hello-skill/SKILL.md` (if skills enabled)
- **AND** `.env` and `local.config.toml` SHALL NOT be written

#### Scenario: Starter template file set
- **WHEN** the Starter template is selected
- **THEN** all Blank files are written AND additionally: `.env`, `local.config.toml` (identical content to `config.toml`, no providers block)

#### Scenario: Advanced template file set
- **WHEN** the Advanced template is selected
- **THEN** all Starter files are written AND additionally: `templates/mycode/command.hbs`, `templates/mycode/skill.hbs`, `templates/mycode/instructions.hbs`, `templates/mycode/mcp.hbs`, and `local.config.toml` includes the mycode provider block

### Requirement: --template flag selects template non-interactively
When `--template <blank|starter|advanced>` is passed, the template selection prompt SHALL NOT be shown and the specified template SHALL be used.

#### Scenario: --template blank
- **WHEN** `dotagents init --template blank` is run
- **THEN** the Blank file set is written and no template prompt is shown

#### Scenario: --template starter
- **WHEN** `dotagents init --template starter` is run
- **THEN** the Starter file set is written and no template prompt is shown

#### Scenario: --template advanced
- **WHEN** `dotagents init --template advanced` is run
- **THEN** the Advanced file set is written and no template prompt is shown

### Requirement: Template select prompt defaults to Blank
When the wizard shows the template selection prompt, `Blank` SHALL be the default-highlighted option.

#### Scenario: User accepts default template
- **WHEN** the template select prompt is shown and the user presses Enter without moving
- **THEN** the Blank template is used

### Requirement: Blank config.toml reflects --features and --targets
The `config.toml` written by the Blank template SHALL include the features and targets specified via `--features` and `--targets` flags, or empty arrays if none are provided.

#### Scenario: Blank with explicit features
- **WHEN** `dotagents init --template blank --features commands,instructions` is run
- **THEN** `config.toml` contains `features = ["commands", "instructions"]`

#### Scenario: Blank with no flags
- **WHEN** `dotagents init --template blank` is run
- **THEN** `config.toml` contains `features = []` and `targets = []`

### Requirement: Starter local.config.toml has no providers block
The `local.config.toml` written by the Starter template SHALL contain the same content as `config.toml` with no `[providers.*]` sections.

#### Scenario: Starter local.config.toml content
- **WHEN** init completes with Starter template
- **THEN** `.dotagents/local.config.toml` matches `config.toml` exactly

### Requirement: Advanced local.config.toml includes mycode provider
The `local.config.toml` written by the Advanced template SHALL include the full mycode provider block covering all four features (commands, instructions, mcp, skills).

#### Scenario: Advanced local.config.toml content
- **WHEN** init completes with Advanced template
- **THEN** `.dotagents/local.config.toml` contains `targets = ["mycode"]` and `[providers.mycode.*]` sections for each feature
