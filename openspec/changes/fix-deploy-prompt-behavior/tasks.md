## 1. Commands changes

- [ ] 1.1 In `src/cli/commands.rs`, rename `deploy: bool` to `no_deploy: bool` in `AddCommandOptions` and `RmCommandOptions`; update Clap attribute to `--no-deploy` with `default_value_t = false`
- [ ] 1.2 Update `maybe_prompt_deploy(deploy_flag: bool)` signature to `maybe_prompt_deploy(no_deploy: bool)` and update logic: if `no_deploy` return early; else if `!is_tui_enabled()` auto-deploy; else prompt in TTY
- [ ] 1.3 Ensure `commands rm` calls `maybe_prompt_deploy` after successful deletion (add if missing)

## 2. Skills changes

- [ ] 2.1 In `src/cli/skills.rs`, rename `deploy: bool` to `no_deploy: bool` in `AddSkillOptions` (skills new) and `RmSkillOptions`; update Clap attribute to `--no-deploy` with `default_value_t = false`
- [ ] 2.2 Update `maybe_prompt_deploy(deploy_flag: bool)` signature to `maybe_prompt_deploy(no_deploy: bool)` and update logic: if `no_deploy` return early; else if `!is_tui_enabled()` auto-deploy; else prompt in TTY
- [ ] 2.3 Ensure `skills rm` calls `maybe_prompt_deploy` after successful deletion (add if missing)

## 3. Unit tests

- [ ] 3.1 Add unit test: `maybe_prompt_deploy(true)` (no_deploy=true) — deploy is skipped, returns without calling deploy
- [ ] 3.2 Add unit test: `maybe_prompt_deploy(false)` in CI context — auto-deploys without prompting
- [ ] 3.3 Add unit test: deploy function called with expected options in CI auto-deploy path
- [ ] 3.4 Add unit test: `maybe_prompt_deploy(false)` in TTY context — stub the prompt function, assert it is invoked; simulate user accepting and verify deploy is called; simulate user declining and verify deploy is not called

## 4. E2e tests

- [ ] 4.1 Add e2e test in `tests/e2e/commands.test.ts`: `commands new NAME --ci --no-deploy --offline` — assert no deployed file is written (TC-CMD-NEW)
- [ ] 4.2 Add e2e test in `tests/e2e/commands.test.ts`: use `initWithLocalProvider(d)` to configure the local provider, run `commands new NAME --ci --offline`, assert deployed output file exists in the provider's target path (TC-CMD-NEW-CI-DEPLOY)
- [ ] 4.3 Add e2e test in `tests/e2e/commands.test.ts`: `commands rm NAME --ci --no-deploy` — assert no re-deploy occurs after deletion
- [ ] 4.4 Add e2e test in `tests/e2e/skills.test.ts`: `skills new NAME --ci --no-deploy --offline` — assert no deployed file is written
- [ ] 4.5 Add e2e test in `tests/e2e/skills.test.ts`: use `initWithLocalProvider(d)` to configure the local provider, run `skills new NAME --ci --offline`, assert deployed output file exists in the provider's target path
- [ ] 4.6 Add e2e test in `tests/e2e/skills.test.ts`: `skills rm NAME --ci --no-deploy` — assert no re-deploy occurs after deletion
- [ ] 4.7 Update any existing tests that used `--deploy` flag to use `--no-deploy` or remove the flag (CI mode now auto-deploys)

## 5. Verification

- [ ] 5.1 Run `mise check` (fmt + clippy) — must exit 0
- [ ] 5.2 Run `mise tests` (unit + integration + e2e) — must exit 0
