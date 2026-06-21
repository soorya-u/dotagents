## Why

The `skills add` wrapper is silently broken: it sets `CLAUDE_CONFIG_DIR` expecting skills to land in `.dotagents/skills/`, but that env var only redirects *global* installs and `add` never passes `-g`. Skills actually land at `<workspace>/.claude/skills/`, bypassing dotagents' source-of-truth entirely. The wrapping logic also lives in `src/cli/skills.rs` mixed with local-only operations (`new`, `rm`, `ls`), and `package-runner` is an undocumented top-level config field with no namespace for future integrations. Issue #65 targets this for v0.2.0.

## What Changes

- **BREAKING**: Move `skills add` and `skills rm` external-integration logic from `src/cli/skills.rs` into a new `src/integrations/skills_sh.rs` module (file exists, currently empty). `new`, `rm` (local path), and `ls` stay in `cli/skills.rs` as dotagents source-of-truth operations.
- **BREAKING**: Fix `skills add` to actually redirect installs into `.dotagents/skills/`: spawn the `skills` CLI with `current_dir(.dotagents/)`, `--agent openclaw` (its flat `skills/` project dir matches dotagents' layout), and `--copy` (real files, not symlinks to a cache). Drop the ineffective `CLAUDE_CONFIG_DIR` env var.
- **BREAKING**: Replace the top-level `package-runner` config field with a namespaced `[integrations.skills-sh]` table containing `package-runner`. Remove the old top-level field from `GlobalConfig`, `LocalConfig`, and `AppConfig` (no backward compatibility — the `skills-add` feature is non-default and rarely set).
- `skills rm` becomes provenance-aware: read `.dotagents/skills-lock.json` (written by the skills CLI during `add`); if the skill has a lockfile entry, treat it as external and delegate to `npx skills remove` with `current_dir(.dotagents/)`; if no entry, treat it as locally authored and run the current `fs::remove_dir_all` + undeploy cleanup. Auto-delegate internally — one command, branches on source.
- The skills CLI writes `skills-lock.json` to its cwd, so `.dotagents/skills-lock.json` is produced for free (no symlink, no `bin/` directory, no `cache.toml` provenance duplication). dotagents treats the lockfile as read-only provenance — it never writes or edits it. If the upstream `skills remove` leaves stale lockfile entries (current behavior), dotagents accepts the staleness; if upstream fixes it, dotagents code requires no change.
- Note the openclaw coupling risk in `design.md`: we depend on openclaw's `skillsDir: 'skills'` staying flat in the skills CLI. Suggest contributing a `--target-dir` flag upstream as a long-term fix.

## Capabilities

### New Capabilities
- `integrations-skills-sh`: Module at `src/integrations/skills_sh.rs` wrapping the external `skills` CLI (from skills.sh) for `add` and `remove` operations. Owns: spawning the skills CLI with the correct `cwd`/`--agent`/`--copy`/`--yes` flags, resolving `PackageRunner`, reading `skills-lock.json` as read-only provenance for the `rm` delegation decision.

### Modified Capabilities
- `skills-add`: The `add` command's install mechanism changes completely — `CLAUDE_CONFIG_DIR` env var is dropped, `cwd=.dotagents` + `--agent openclaw` + `--copy` become the redirect mechanism. `PackageRunner` resolution priority is unchanged (CLI flag > local config > global config > npm default). The config field location moves from top-level `package-runner` to `[integrations.skills-sh].package-runner`.
- `skills-subcommand-extended`: The `rm` sub-action gains provenance-aware delegation — externally-installed skills (present in `skills-lock.json`) delegate to `npx skills remove`; locally-authored skills (absent from lockfile) keep the current local-delete + undeploy-cleanup behavior.

## Impact

- `src/integrations/skills_sh.rs` — populate the empty file with `add()` and `remove()` wrappers, `PackageRunner` (moved from `src/core/config/common.rs`), and a read-only lockfile reader for provenance checks.
- `src/integrations/mod.rs` — new module declaration (currently no `integrations` module).
- `src/cli/skills.rs` — `add()` and the external-remove path move out; `new_skill()`, local `rm_skill()`, and `ls_skills()` remain. `rm_skill()` gains a provenance branch that delegates to `integrations::skills_sh::remove` for external skills.
- `src/core/config/common.rs` — `PackageRunner` enum moves to `src/integrations/skills_sh.rs` (or a shared location if reused elsewhere).
- `src/core/config/global.rs`, `local.rs`, `app.rs` — remove top-level `package_runner` field; add `integrations: Option<IntegrationsConfig>` with nested `skills_sh: Option<SkillsShConfig>` carrying `package_runner`.
- `src/cli/config.rs` — update the `config` command display to read from the new `[integrations.skills-sh]` path.
- `src/cli/options.rs` — `--runner` flag unchanged; update any help text referencing `package-runner` config location.
- `tests/e2e/skills.test.ts` — update/add tests for the corrected install path (skills land in `.dotagents/skills/`), the `[integrations.skills-sh]` config, and provenance-aware `rm` delegation.
- Coordinate with the in-progress `symlink-mode` change: this change does not use symlinks for the lockfile (cwd-based placement is sufficient), but may reuse symlink helpers from `symlink-mode` if any cross-cutting fs utilities are shared.
