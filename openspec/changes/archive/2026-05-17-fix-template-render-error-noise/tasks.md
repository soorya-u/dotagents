## 1. Remove generic context from templater

- [x] 1.1 In `src/templates/templater.rs:113`, remove `.context("failed to render template")` from `render_template()` — the function now returns the raw `handlebars::RenderError` wrapped in `anyhow::Error`

## 2. Add phase-specific context at call sites

- [x] 2.1 In `src/templates/renderer.rs:47-50`, add `.context("unable to render target path")` after the `render_template` call for the target path
- [x] 2.2 In `src/templates/renderer.rs` at the `populate_with_values` call (line ~60), add `.context("unable to render feature variables")`
- [x] 2.3 In `src/templates/renderer.rs:84`, add `.context(format!("unable to render template content for provider '{}'", provider_name))` after the template content `render_template` call
- [x] 2.4 In `src/core/config/app.rs:77,83`, add specific context strings (`"unable to render global config"`, `"unable to render local config"`) at the two `render_template` call sites
- [x] 2.5 In `src/core/features/traits.rs:14`, add `.context("unable to render feature content")` at the `render_template` call site (also added missing `Context` import)

## 3. Unit tests

- [x] 3.1 Add unit test in `src/templates/templater.rs`: create a `Templater` with a deliberately broken Handlebars template string; call `render_template` and assert the error chain does NOT contain `"failed to render template"`
- [x] 3.2 Add unit test in `src/templates/renderer.rs`: call `render_feature_with_settings` with a broken target path expression; assert error chain contains `"unable to render target path"`
- [x] 3.3 Add unit test in `src/templates/renderer.rs`: call `render_feature_with_settings` with a broken template file content; assert error chain contains `"unable to render template content for provider"`

## 4. Verification

- [x] 4.1 Run `mise check` (fmt + clippy) — exits 0
- [x] 4.2 Run `mise tests` (unit + integration + e2e) — exits 0 (294 unit + 40 integration + 190 e2e pass)

## 5. Bonus fixes (discovered during implementation)

- [x] 5.1 Fixed `override_workspace_dir` in `src/utils/path.rs` to return an error when the `OnceLock` is already set (was silently discarding via `let _ =`)
- [x] 5.2 Fixed `maybe_prompt_deploy_ci_calls_deploy_when_no_deploy_false` tests in both `src/cli/commands.rs` and `src/cli/skills.rs` that poisoned the workspace `OnceLock` by setting up proper minimal workspaces with `.dotagents-debug/config.toml`
- [x] 5.3 Updated workspace path tests in `src/utils/path.rs` to accept `"already set"` errors now that `override_workspace_dir` properly reports them
