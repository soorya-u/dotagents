## ADDED Requirements

### Requirement: Hard-error behaviour for explicit template URLs is unaffected by registry fallback logic
The existing hard-error requirements for non-200 responses, network failures, untrusted domains, and plain HTTP URLs apply exclusively to cases where `template` is explicitly set to a URL in `config.toml`. Registry and `provider.toml` fetches performed by the auto-resolution path use separate soft-failure logic defined in the `provider-registry-resolution` spec and SHALL NOT change the behaviour for explicit `template` URLs.

#### Scenario: Explicit remote template URL that fails network — still a hard error
- **WHEN** a provider's `template` is explicitly set to `"https://dotagents.soorya-u.dev/templates/claude/command.hbs"` in `config.toml` and the server is unreachable
- **THEN** deploy stops with a hard error, identical to pre-existing behaviour; the soft-fallback logic is not applied
