## Why

Users want dotagents to be the source of truth for skills, but the `skills` CLI (from skills.sh) installs directly into agent-specific directories (`.claude/skills/`, `.cursor/skills/`, etc.), bypassing dotagents entirely. A thin `dotagents skills add` wrapper is needed to redirect installs into `.dotagents/skills/` so that `dotagents deploy` can distribute them from there.

## What Changes

- New `dotagents skills add <name> [--runner <npm|pnpm|yarn|bun>]` subcommand that wraps `npx skills add` and redirects the install destination to `.dotagents/skills/`
- New `PackageRunner` enum (`npm` | `pnpm` | `yarn` | `bun`) added to `src/schema/config/common.rs`
- New optional `package-runner` field on `GlobalConfig` and `LocalConfig` (persists the preferred runner across invocations)
- `AppConfig` carries `Option<PackageRunner>` (unresolved) so call sites can distinguish "explicitly configured" from "never stated"
- `Action::Skills(SkillsAction)` subcommand group added to `options.rs` with a single `Add` variant for now

## Capabilities

### New Capabilities

- `skills-add`: `dotagents skills add` command — resolves runner, sets `CLAUDE_CONFIG_DIR` to redirect the install, spawns the skills CLI child process, validates binary presence when runner is explicit

### Modified Capabilities

- `skill-feature`: No requirement changes — the existing `SkillFeature` deploy pipeline is unchanged; this proposal only adds the *import* side

## Impact

- `src/cli/options.rs` — new `Action::Skills` variant and `SkillsAction` / `SkillsAddOptions` structs
- `src/cli/runner.rs` — dispatch to new `skills::add` handler
- `src/cli/skills.rs` — new file; child process spawn logic
- `src/schema/config/common.rs` — new `PackageRunner` enum
- `src/schema/config/global.rs` — new `package_runner: Option<PackageRunner>` field
- `src/schema/config/local.rs` — new `package_runner: Option<PackageRunner>` field
- `src/schema/config/app.rs` — merge `package_runner` from local → global; carry as `Option<PackageRunner>`
- No new dependencies required (`std::process::Command` for child process)
