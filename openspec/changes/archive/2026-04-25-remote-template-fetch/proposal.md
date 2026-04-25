## Why

The public registry at `public/v1/templates/` publishes community templates that users reference by URL (e.g. `template = "https://dotagents.soorya-u.dev/templates/claude/command.hbs"`). Today those URLs are treated as local file paths and deploy immediately fails with "Template file not found". Implementing remote fetching makes the public registry actually usable and eliminates the need for every user to copy templates locally.

## What Changes

- When `template_str` in `FeatureSettings` starts with `"https://dotagents.soorya-u.dev/"`, fetch its content via HTTP GET instead of reading from the local filesystem
- Any `https://` URL not from `dotagents.soorya-u.dev` is a hard error — only the trusted domain is allowed
- Any non-`https://` URL (e.g. `http://`) is a hard error
- HTTP fetch failure (network error, non-200 response) is a hard error that stops deploy
- No local caching — templates are re-fetched on every deploy (caching is a separate future proposal at application level, not project level)
- `src/templates/remote.rs` (currently a stub) implements the fetch logic
- `renderer.rs` is updated to detect the URL prefix and route to fetch vs local file read
- Add `ureq` to `[dependencies]` as the sync HTTP client (no async runtime needed; fits the small-binary release profile with `lto = true`, `opt-level = "s"`, `strip = true`)

## Capabilities

### New Capabilities

- `remote-template-fetch`: Defines the behaviour for detecting a remote template URL, restricting to the trusted domain, performing an HTTP GET, and using the response body as template content. Covers hard-error cases for untrusted domains, non-HTTPS URLs, and fetch failures.

### Modified Capabilities

*(none)*

## Impact

- `Cargo.toml` — add `ureq` dependency
- `src/templates/remote.rs` — implement `fetch_template(url: &str) -> Result<String>`
- `src/templates/renderer.rs` — update `render_feature_with_settings` to detect URL prefix and call `fetch_template` instead of `read_file`
- `src/templates/mod.rs` — ensure `remote` module is exported
- No changes to config schema, CLI flags, or feature types
