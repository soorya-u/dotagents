## Context

The `skills` CLI (`npm: skills`, v1.5.1 by Vercel Labs) installs skill files into agent-specific directories determined by hardcoded paths in its `agents.ts`. There is no `--dir` flag. However, the Claude Code agent path is driven by `process.env.CLAUDE_CONFIG_DIR`, making it the one controllable hook. By spawning the child process with `CLAUDE_CONFIG_DIR=<abs-path-to-.dotagents>` and `--agent claude-code`, the skills CLI deposits files into `.dotagents/skills/<skill-name>/` — exactly the source-of-truth location dotagents manages.

Current CLI shape (`options.rs`) has three top-level actions: `Init`, `Deploy`, `GenCompletions`. The new `Skills` action follows the same `clap` derive pattern, with a nested subcommand `Add` for now.

## Goals / Non-Goals

**Goals:**
- `dotagents skills add <name>` installs a skill into `.dotagents/skills/` (not into any agent dir directly)
- `--runner <npm|pnpm|yarn|bun>` flag overrides the configured runner for one invocation
- `package-runner` optional field in `GlobalConfig` and `LocalConfig`; merged local → global in `AppConfig`
- Binary presence validated when runner is explicit; friendly error pointing to config.toml
- Default runner (npm/npx) applied silently when nothing is configured

**Non-Goals:**
- `dotagents skills list`, `remove`, `update` — deferred; `add` is sufficient for v1
- Auto-detecting the installed package manager from lockfile heuristics
- Any changes to the existing `SkillFeature` deploy pipeline

## Decisions

### D1 — Use `CLAUDE_CONFIG_DIR` env var as the redirect hook

The skills CLI has no `--dir` flag. The only controllable path hook is `CLAUDE_CONFIG_DIR`, which shifts Claude Code's skills dir to `$CLAUDE_CONFIG_DIR/skills`. Setting it to the absolute path of `.dotagents` makes the CLI write to `.dotagents/skills/` — the exact source-of-truth location.

**Alternative considered:** Run in a temp dir, let skills install to `<tmp>/.agents/skills/`, then copy to `.dotagents/skills/`. Rejected — more moving parts, fragile if skills CLI changes its canonical dir name, and requires cleanup logic.

**Risk:** `CLAUDE_CONFIG_DIR` is an internal env var of a third-party tool. If its semantics change in a future version of the skills CLI, the redirect breaks silently. Mitigation: pin behaviour with an integration test that runs the actual `dotagents skills add` and asserts the file lands in `.dotagents/skills/`.

### D2 — `--agent claude-code` is hardcoded internally

Without this flag, the skills CLI attempts to install for all detected agents, writing to multiple directories. We hardcode `--agent claude-code` in the child process args so exactly one agent path (the one we control via `CLAUDE_CONFIG_DIR`) is written to. The user never sees this flag.

### D3 — `Option<PackageRunner>` in `AppConfig`, not resolved to default

`AppConfig` carries `Option<PackageRunner>` rather than resolving to `PackageRunner::Npm`. This preserves the "was this explicitly configured?" signal at the call site, enabling the targeted error: if `Some(runner)` and binary absent → bail with config hint; if `None` → run `npx` silently, OS error handles absence.

### D4 — `PackageRunner` in `common.rs`, not a new file

The type is shared between `GlobalConfig` and `LocalConfig` — the same reason `Targets`, `Providers`, `Features`, and `FeatureSettings` live in `common.rs`. No new config module file needed.

### D5 — New `src/cli/skills.rs` for spawn logic

Keeps `options.rs` clean (pure CLI shape) and `runner.rs` clean (pure dispatch). The skills-specific logic — env var construction, args array, binary resolution, `std::process::Command` spawn — lives in its own file, consistent with `init.rs` and `deploy.rs`.

## Risks / Trade-offs

- **Third-party CLI dependency**: `npx skills add` must be available. If the `skills` npm package changes its CLI interface, the wrapper breaks. → Mitigation: keep the integration test; document the version assumption.
- **`CLAUDE_CONFIG_DIR` side effects**: Setting this env var in the child process only (not the parent), so no leakage. If the skills CLI reads `CLAUDE_CONFIG_DIR` for purposes beyond skills dir resolution (e.g. reading Claude config), it might behave unexpectedly. → Low risk given the env is scoped to the subprocess.
- **`--agent claude-code` hardcode**: If a future skills CLI version renames the agent identifier, this silently fails or errors. → Discoverable via the integration test.
- **No `list`/`remove`**: Users who want to remove a skill must manually delete the `.dotagents/skills/<name>/` directory for now.

## Open Questions

- Should the binary presence check use `which` (requires a crate or shell out) or just attempt the spawn and map `ErrorKind::NotFound`? The latter avoids any new dependency. → Prefer `ErrorKind::NotFound` mapping.
- Should `dotagents skills add` also run `dotagents deploy` automatically after installing, or leave that to the user? → Leave to user; composability over magic.
