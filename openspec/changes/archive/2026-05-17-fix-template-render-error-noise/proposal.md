## Why

When a Handlebars template has a syntax error, the error message `"failed to render template"` appears multiple times in the `"Caused by:"` chain — once per `render_template()` call that touches the broken template. This makes the error output noisy and unhelpful for diagnosing which render phase failed.

## What Changes

- Remove the generic `.context("failed to render template")` from `Templater::render_template()` in `src/templates/templater.rs`
- Add specific context strings at each call site in `src/templates/renderer.rs`:
  - Target path rendering → `.context("unable to render target path")`
  - Feature variable population (`populate_with_values`) → the context is added at the call site in renderer.rs → `.context("unable to render feature variables")`
  - Template content rendering → `.context("unable to render template content for provider {provider_name}")`
- Unit tests added to verify distinct context strings appear in the error chain

## Capabilities

### New Capabilities

### Modified Capabilities
- `error-display-consistency`: Template render errors now emit a single, phase-specific cause string instead of repeating `"failed to render template"` multiple times

## Impact

- `src/templates/templater.rs` — remove `.context("failed to render template")` from `render_template()`
- `src/templates/renderer.rs` — add specific `.context()` wrappers at each `render_template` call site and at the `populate_with_values` call site
- Unit tests in `src/templates/renderer.rs` or `src/templates/templater.rs` — assert each phase produces its specific error string
