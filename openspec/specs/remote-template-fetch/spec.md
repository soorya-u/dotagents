### Requirement: Trusted domain remote templates are fetched via HTTP GET
When a provider's `template` field starts with `"https://dotagents.soorya-u.dev/"`, `dotagents deploy` SHALL fetch the template content via HTTP GET and use the response body as the template string. The local filesystem SHALL NOT be consulted for such values.

#### Scenario: Valid remote template is fetched and used
- **WHEN** a provider's `template` is `"https://dotagents.soorya-u.dev/templates/claude/command.hbs"` and the server returns 200 with `.hbs` content
- **THEN** deploy uses that content to render the provider's output, identical to how a local `.hbs` file would be used

#### Scenario: Local template path is unaffected
- **WHEN** a provider's `template` is a local path such as `"{{ dir.application }}/templates/mycode/command.hbs"`
- **THEN** the existing local file read logic is used unchanged; no HTTP request is made

### Requirement: Non-200 HTTP response is a hard error
If the HTTP GET for a remote template returns any status code other than 200, `dotagents deploy` SHALL return an error that stops the deploy, including the URL and the HTTP status code in the error message.

#### Scenario: 404 response stops deploy
- **WHEN** a remote template URL returns HTTP 404
- **THEN** deploy stops with an error message containing the URL and "404"

#### Scenario: 500 response stops deploy
- **WHEN** a remote template URL returns HTTP 500
- **THEN** deploy stops with an error message containing the URL and "500"

### Requirement: Network failure is a hard error
If the HTTP GET cannot be completed due to a network error (DNS failure, connection refused, timeout), `dotagents deploy` SHALL return an error that stops the deploy.

#### Scenario: DNS resolution failure
- **WHEN** the remote host cannot be resolved
- **THEN** deploy stops with an error describing the network failure

#### Scenario: Connection timeout
- **WHEN** the connection times out before a response is received
- **THEN** deploy stops with an error describing the timeout

### Requirement: Non-trusted HTTPS URLs are a hard error
If a provider's `template` field starts with `"https://"` but the host is not `dotagents.soorya-u.dev`, `dotagents deploy` SHALL return a hard error before making any network request.

#### Scenario: Untrusted HTTPS domain rejected
- **WHEN** `template = "https://example.com/mytemplate.hbs"`
- **THEN** deploy stops with an error stating only `dotagents.soorya-u.dev` is supported as a remote template host

### Requirement: Non-HTTPS URLs are a hard error
If a provider's `template` field starts with `"http://"` (plain HTTP, not HTTPS), `dotagents deploy` SHALL return a hard error.

#### Scenario: Plain HTTP URL rejected
- **WHEN** `template = "http://dotagents.soorya-u.dev/templates/claude/command.hbs"`
- **THEN** deploy stops with an error stating only HTTPS URLs are supported

### Requirement: Hard-error behaviour for explicit template URLs is unaffected by registry fallback logic
The existing hard-error requirements for non-200 responses, network failures, untrusted domains, and plain HTTP URLs apply exclusively to cases where `template` is explicitly set to a URL in `config.toml`. Registry and `provider.toml` fetches performed by the auto-resolution path use separate soft-failure logic defined in the `provider-registry-resolution` spec and SHALL NOT change the behaviour for explicit `template` URLs.

#### Scenario: Explicit remote template URL that fails network — still a hard error
- **WHEN** a provider's `template` is explicitly set to `"https://dotagents.soorya-u.dev/templates/claude/command.hbs"` in `config.toml` and the server is unreachable
- **THEN** deploy stops with a hard error, identical to pre-existing behaviour; the soft-fallback logic is not applied
