## MODIFIED Requirements

### Requirement: Template render errors identify the failed phase
When a Handlebars template render fails, the error chain SHALL contain exactly one phase-specific context string that identifies which render phase failed (target path, feature variables, or template content). The generic string `"failed to render template"` SHALL NOT appear more than once in the error chain.

#### Scenario: Target path render failure shows phase context
- **WHEN** a provider's `target` path expression contains a Handlebars syntax error
- **WHEN** `deploy` is run
- **THEN** the error chain contains `"unable to render target path"`
- **THEN** the string `"failed to render template"` does NOT appear more than once

#### Scenario: Template content render failure shows phase and provider context
- **WHEN** a provider's template file contains a Handlebars syntax error
- **WHEN** `deploy` is run
- **THEN** the error chain contains a string matching `"unable to render template content for provider"`
- **THEN** the provider name is included in the error context

#### Scenario: Feature variable render failure shows phase context
- **WHEN** a feature's content (e.g. INSTRUCTIONS.md) contains a Handlebars syntax error
- **WHEN** `deploy` is run
- **THEN** the error chain contains `"unable to render feature variables"`
