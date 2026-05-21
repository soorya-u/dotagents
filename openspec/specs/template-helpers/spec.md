## ADDED Requirements

### Requirement: {{toml}} helper renders objects as TOML table-style key-value lines

The `{{toml value}}` Handlebars helper SHALL serialize a JSON object to TOML format using bare key-value lines (one per line), suitable for use inside `[table.section]` blocks. The helper SHALL reject non-object values with a render error.

#### Scenario: renders a simple object
- **WHEN** template contains `{{toml env}}` and `env` is `{"KEY": "val", "FOO": "bar"}`
- **THEN** output is `FOO = "bar"\nKEY = "val"\n` (key order may vary)

#### Scenario: errors on string input
- **WHEN** template contains `{{toml name}}` and `name` is `"hello"`
- **THEN** a render error is produced indicating only objects are supported

#### Scenario: errors on array input
- **WHEN** template contains `{{toml items}}` and `items` is `[1, 2, 3]`
- **THEN** a render error is produced indicating only objects are supported

### Requirement: {{toml-inline}} helper renders objects as TOML inline table syntax

The `{{toml-inline value}}` Handlebars helper SHALL serialize a JSON object to a TOML inline table wrapped in `{ }`, suitable for use inside existing key-value assignments like `env = { ... }`. The helper SHALL reject non-object values with a render error.

#### Scenario: renders a simple object as inline table
- **WHEN** template contains `{{toml-inline env}}` and `env` is `{"KEY": "val", "FOO": "bar"}`
- **THEN** output is `{ FOO = "bar", KEY = "val" }` (key order may vary)

#### Scenario: errors on null input
- **WHEN** template contains `{{toml-inline value}}` and `value` is `null`
- **THEN** a render error is produced indicating only objects are supported

### Requirement: {{yaml}} helper renders values as YAML block syntax

The `{{yaml value}}` Handlebars helper SHALL serialize any JSON value to YAML block syntax. Unlike the TOML helpers, this helper SHALL accept objects, arrays, strings, numbers, and null values.

#### Scenario: renders an object as YAML mapping
- **WHEN** template contains `{{yaml env}}` and `env` is `{"KEY": "val", "FOO": "bar"}`
- **THEN** output is `FOO: bar\nKEY: val\n` (key order may vary)

#### Scenario: renders an array as YAML sequence
- **WHEN** template contains `{{yaml items}}` and `items` is `[1, 2, 3]`
- **THEN** output is `- 1\n- 2\n- 3\n`

#### Scenario: renders a string as YAML scalar
- **WHEN** template contains `{{yaml name}}` and `name` is `"hello"`
- **THEN** output is `hello\n`

### Requirement: all three helpers are registered globally in Templater

The `{{toml}}`, `{{toml-inline}}`, and `{{yaml}}` helpers SHALL be registered in `Templater::new` alongside the existing `{{json}}` and `{{ifEq}}` helpers, making them available in all template rendering contexts (provider templates, user custom templates, config templates).

#### Scenario: helpers are available after Templater initialization
- **WHEN** `get_templater()` is called
- **THEN** the returned Templater has `toml`, `toml-inline`, and `yaml` helpers registered

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
