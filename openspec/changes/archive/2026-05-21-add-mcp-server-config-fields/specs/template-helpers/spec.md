## ADDED Requirements

### Requirement: {{snake-case}} helper renders camelCase strings as snake_case

The `{{snake-case value}}` Handlebars helper SHALL convert an input string from camelCase or PascalCase to snake_case. The helper SHALL reject non-string values with a render error.

#### Scenario: converts camelCase field name
- **WHEN** template contains `{{snake-case field}}` and `field` is `"startupTimeoutSec"`
- **THEN** output is `startup_timeout_sec`

#### Scenario: converts PascalCase field name
- **WHEN** template contains `{{snake-case field}}` and `field` is `"BearerTokenEnvVar"`
- **THEN** output is `bearer_token_env_var`

#### Scenario: preserves already snake_case field name
- **WHEN** template contains `{{snake-case field}}` and `field` is `"tool_timeout_sec"`
- **THEN** output is `tool_timeout_sec`

#### Scenario: errors on non-string input
- **WHEN** template contains `{{snake-case value}}` and `value` is `42`
- **THEN** a render error is produced indicating only strings are supported

### Requirement: {{snake-case}} helper is registered globally in Templater

The `{{snake-case}}` helper SHALL be registered in `Templater::new` alongside the existing `{{json}}`, `{{ifEq}}`, `{{toml}}`, `{{toml-inline}}`, and `{{yaml}}` helpers, making it available in provider templates, custom templates, and config templates.

#### Scenario: helper is available after Templater initialization
- **WHEN** `get_templater()` is called
- **THEN** the returned Templater has the `snake-case` helper registered
