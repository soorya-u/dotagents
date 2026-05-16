## 1. Remove generic context from templater

- [ ] 1.1 In `src/templates/templater.rs:113`, remove `.context("failed to render template")` from `render_template()` — the function now returns the raw `handlebars::RenderError` wrapped in `anyhow::Error`

## 2. Add phase-specific context at call sites

- [ ] 2.1 In `src/templates/renderer.rs:47-50`, add `.context("unable to render target path")` after the `render_template` call for the target path
- [ ] 2.2 In `src/templates/renderer.rs` at the `populate_with_values` call (line ~60), add `.context("unable to render feature variables")`
- [ ] 2.3 In `src/templates/renderer.rs:84`, add `.context(format!("unable to render template content for provider '{}'", provider_name))` after the template content `render_template` call
- [ ] 2.4 In `src/core/config/app.rs:77,83`, add specific context strings (e.g. `"unable to render global config"`, `"unable to render local config"`) at the two `render_template` call sites
- [ ] 2.5 In `src/core/features/traits.rs:14`, add `.context("unable to render feature content")` at the `render_template` call site

## 3. Unit tests

- [ ] 3.1 Add unit test: create a `Templater` with a deliberately broken Handlebars template string; call `render_template` and assert the error chain does NOT contain `"failed to render template"` (since the context is now at call sites)
- [ ] 3.2 Add unit test: call `render_feature_with_settings` with a broken target path expression; assert error chain contains `"unable to render target path"`
- [ ] 3.3 Add unit test: call `render_feature_with_settings` with a broken template file content; assert error chain contains `"unable to render template content for provider"`

## 4. Verification

- [ ] 4.1 Run `mise check` (fmt + clippy) — must exit 0
- [ ] 4.2 Run `mise tests` (unit + integration + e2e) — must exit 0
