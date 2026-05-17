## Why

After `commands new`, `commands rm`, `skills new`, and `skills rm`, the CLI has an opt-in `--deploy` flag that users must pass to trigger a deploy. In CI mode the flag is also required, meaning newly created or deleted commands/skills are never auto-deployed in pipelines. The correct default is: always deploy in CI (no prompt needed), prompt in TTY, and provide `--no-deploy` for users who explicitly want to skip.

## What Changes

- Remove `--deploy` flag from `AddCommandOptions`, `RmCommandOptions`, `AddSkillOptions`, `RmSkillOptions`
- Add `--no-deploy` flag (boolean, `default_value_t = false`) to all four option structs
- `maybe_prompt_deploy()` in `src/cli/commands.rs` and `src/cli/skills.rs` is updated: in CI (`!is_tui_enabled()`), deploy automatically; in TTY, prompt; if `--no-deploy` is set, skip entirely
- `rm` subcommands gain the same deploy behavior as `new` (they did not have it before)
- Unit tests cover all three branches of `maybe_prompt_deploy()`
- E2e tests cover: `commands new --no-deploy` skips deploy, `commands new` in CI auto-deploys, `skills rm --no-deploy` skips deploy, `skills rm` in CI auto-deploys

## Capabilities

### New Capabilities
- `commands-deploy-default`: `commands new` and `commands rm` deploy automatically in CI and prompt in TTY unless `--no-deploy` is passed
- `skills-deploy-default`: `skills new` and `skills rm` deploy automatically in CI and prompt in TTY unless `--no-deploy` is passed

### Modified Capabilities
- `commands-subcommand`: `--deploy` flag removed; `--no-deploy` added to `new` and `rm`
- `skills-subcommand-extended`: `--deploy` flag removed; `--no-deploy` added to `new` and `rm`

## Impact

- `src/cli/commands.rs` — `AddCommandOptions.deploy` → `no_deploy`; `RmCommandOptions.deploy` → `no_deploy`; `maybe_prompt_deploy` logic updated
- `src/cli/skills.rs` — `AddSkillOptions.deploy` → `no_deploy`; `RmSkillOptions.deploy` → `no_deploy`; `maybe_prompt_deploy` logic updated
- `tests/e2e/commands.test.ts` — update `--deploy` tests to `--no-deploy`, add CI auto-deploy tests
- `tests/e2e/skills.test.ts` — update `--deploy` tests to `--no-deploy`, add CI auto-deploy tests
