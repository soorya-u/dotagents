## 1. Commands changes

- [x] 1.1 In `src/cli/commands.rs`, rename `deploy: bool` to `no_deploy: bool` in `AddCommandOptions` and `RmCommandOptions`; update Clap attribute to `--no-deploy` with `default_value_t = false`
- [x] 1.2 Update `maybe_prompt_deploy(deploy_flag: bool)` signature to `maybe_prompt_deploy(no_deploy: bool)` and update logic: if `no_deploy` return early; else if `!is_tui_enabled()` auto-deploy; else prompt in TTY
- [x] 1.3 Ensure `commands rm` calls `maybe_prompt_deploy` after successful deletion (add if missing)

## 2. Skills changes

- [x] 2.1 In `src/cli/skills.rs`, rename `deploy: bool` to `no_deploy: bool` in `AddSkillOptions` (skills new) and `RmSkillOptions`; update Clap attribute to `--no-deploy` with `default_value_t = false`
- [x] 2.2 Update `maybe_prompt_deploy(deploy_flag: bool)` signature to `maybe_prompt_deploy(no_deploy: bool)` and update logic: if `no_deploy` return early; else if `!is_tui_enabled()` auto-deploy; else prompt in TTY
- [x] 2.3 Ensure `skills rm` calls `maybe_prompt_deploy` after successful deletion (add if missing)

## 3. Unit tests

- [x] 3.1 Add unit test: `maybe_prompt_deploy(true)` (no_deploy=true) — deploy is skipped, returns without calling deploy
- [x] 3.2 Add unit test: `maybe_prompt_deploy(false)` in CI context — auto-deploys without prompting
- [x] 3.3 Add unit test: deploy function called with expected options in CI auto-deploy path
- [x] 3.4 Add unit test: `maybe_prompt_deploy(false)` in TTY context — stub the prompt function, assert it is invoked; simulate user accepting and verify deploy is called; simulate user declining and verify deploy is not called

## 4. E2e tests

- [x] 4.1 Add e2e test in `tests/e2e/commands.test.ts`: `commands new NAME --ci --no-deploy` — assert no deployed file is written (TC-CMD-NEW)
- [x] 4.2 Add e2e test in `tests/e2e/commands.test.ts`: use `initWithLocalProvider(d)` to configure the local provider, run `commands new NAME --ci`, assert deployed output file exists in the provider's target path (TC-CMD-NEW-CI-DEPLOY)
- [x] 4.3 Add e2e test in `tests/e2e/commands.test.ts`: `commands rm NAME --ci --no-deploy` — assert no re-deploy occurs after deletion
- [x] 4.4 Add e2e test in `tests/e2e/skills.test.ts`: `skills new NAME --ci --no-deploy` — assert no deployed file is written
- [x] 4.5 Add e2e test in `tests/e2e/skills.test.ts`: use `initWithLocalProvider(d)` to configure the local provider, run `skills new NAME --ci`, assert deployed output file exists in the provider's target path
- [x] 4.6 Add e2e test in `tests/e2e/skills.test.ts`: `skills rm NAME --ci --no-deploy` — assert no re-deploy occurs after deletion
- [x] 4.7 Update any existing tests that used `--deploy` flag to use `--no-deploy` or remove the flag (CI mode now auto-deploys)

## 5. Verification

- [x] 5.1 Run `mise check` (fmt + clippy) — must exit 0
- [x] 5.2 Run `mise tests` (unit + integration + e2e) — must exit 0
