## Context

`Templater::render_template()` calls `handlebars.render()` and wraps any error with `.context("failed to render template")`. In `renderer.rs`, `render_feature_with_settings()` calls `render_template()` three times (target path, feature variables, template content). Each call site propagates the same context string, so a single Handlebars syntax error produces `"failed to render template"` three times in the error chain — once per phase.

The render pipeline has three distinct phases:
1. **Target path rendering**: evaluates the Handlebars target path expression
2. **Feature variable population**: `populate_with_values()` internally renders the feature's content with user variables
3. **Template content rendering**: renders the provider's `.md` template file

## Goals / Non-Goals

**Goals:**
- Each phase produces a unique, descriptive error context string
- The Handlebars `"failed to render template"` message no longer repeats

**Non-Goals:**
- Changing Handlebars error message format (that comes from the library)
- Wrapping every Handlebars call — only the three render phases in `renderer.rs`

## Decisions

1. **Remove context from `Templater::render_template()`**: The function is called from multiple places; a generic context at this level adds noise. Callers are responsible for adding phase-specific context.

2. **Add context at each call site in `renderer.rs`**:
   - Target path: `.context("unable to render target path")`
   - `populate_with_values`: `.context("unable to render feature variables")`
   - Template content: `.context(format!("unable to render template content for provider '{}'", provider_name))`

3. **`populate_with_values` context**: This function is defined on `FeatureTrait` and calls `render_template` internally. The context at the call site in `renderer.rs` wraps the entire function call, giving a phase-level error without modifying the trait implementation.

## Risks / Trade-offs

- **Other callers of `render_template()` lose their generic context**: Any code outside `renderer.rs` that calls `render_template()` directly will no longer get a context string. Mitigation: grep for all call sites and add appropriate context strings.
