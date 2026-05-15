## ADDED Requirements

### Requirement: E2e test for --no-gitignore flag
Verify that `--no-gitignore` suppresses gitignore fence creation during deploy.

#### Scenario: --no-gitignore skips gitignore update (TC-DEPLOY-10)
- **WHEN** `deploy --offline --no-gitignore` is run on a fresh workspace
- **THEN** exit code is 0, deployed files are created, no `.gitignore` file exists in the workspace root

### Requirement: E2e test for missing default .env file
Verify that deploy succeeds when the default `.dotagents/.env` file does not exist.

#### Scenario: Missing .env is silently ignored (TC-DEPLOY-20)
- **WHEN** `.dotagents/.env` is deleted before running `deploy --offline --no-gitignore`
- **THEN** exit code is 0, deployed files are created normally, no error about missing `.env`

### Requirement: E2e test for malformed template render error
Verify that deploy exits with an error when a Handlebars template has invalid syntax.

#### Scenario: Malformed template causes render error (TC-DEPLOY-ERR-01)
- **WHEN** a `.hbs` template file contains `{{ unclosed` syntax and `deploy --offline --no-gitignore` is run
- **THEN** exit code is 1, stderr contains "invalid handlebars syntax" or a template parse error message

### Requirement: E2e test for untrusted URL rejection
Verify that deploy rejects non-HTTPS template URLs as a security boundary.

#### Scenario: Non-HTTPS URL causes error (TC-DEPLOY-ERR-02)
- **WHEN** a provider config references `template = "http://example.com/template.hbs"` and deploy is run
- **THEN** exit code is 1, stderr contains "non-HTTPS" or "not allowed"
