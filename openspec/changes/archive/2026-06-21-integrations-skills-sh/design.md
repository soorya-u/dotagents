## Context

The `skills add` command (shipped behind the `skills-add` cargo feature in change `2026-04-26-skills-add-command`) wraps the external `skills` CLI from skills.sh to redirect installs into `.dotagents/skills/`. The mechanism chosen was setting `CLAUDE_CONFIG_DIR=<.dotagents path>` and spawning `npx skills add <name> --agent claude-code`.

Empirical investigation during exploration revealed this mechanism is silently broken:

- The skills CLI's `src/agents.ts` defines `claude-code.skillsDir = ".claude/skills"` (hardcoded, cwd-relative) and `claude-code.globalSkillsDir = join(claudeHome, "skills")` where `claudeHome = process.env.CLAUDE_CONFIG_DIR || ~/.claude`.
- `CLAUDE_CONFIG_DIR` only affects the **global** install path (used with `-g`/`--global`). The current `add()` never passes `-g`, so it's a project-level install and `CLAUDE_CONFIG_DIR` is ignored.
- A probe (`npx skills add vercel-labs/skills@find-skills --agent claude-code --yes` with `CLAUDE_CONFIG_DIR` set) confirmed skills land at `<cwd>/.claude/skills/`, not at the `CLAUDE_CONFIG_DIR` path.

The wrapping logic also lives in `src/cli/skills.rs` alongside local-only operations (`new_skill`, `rm_skill`, `ls_skills`), and `package-runner` is an undocumented top-level config field with no namespace for future integrations.

The skills CLI has two separate lockfiles:
- **Global**: `~/.agents/.skill-lock.json` (or `$XDG_STATE_HOME/skills/.skill-lock.json`) — written by every `add`/`remove`, tracks all globally-installed skills with full provenance. Overridable only via `$XDG_STATE_HOME`.
- **Project**: `<cwd>/skills-lock.json` — written by `add`, minimal `{version, skills: {name: {source, sourceType, skillPath, computedHash}}}`, meant to be committed. Path is `join(cwd || process.cwd(), 'skills-lock.json')` — only `cwd` controls it.

A second probe (`npx skills add ... --agent openclaw --copy --yes` with `cwd=.dotagents/`) confirmed:
- openclaw's `skillsDir = "skills"` (flat, no dot-prefix) writes directly to `<cwd>/skills/<name>/SKILL.md`.
- With `cwd=.dotagents/`, the lockfile lands at `.dotagents/skills-lock.json` for free.
- `--copy` produces real files (not symlinks to a cache).
- No `.agents/` canonical directory is created in copy mode.

A third probe (`npx skills remove ... --agent openclaw --yes` with `cwd=.dotagents/`) confirmed:
- `skills remove` deletes the skill files but does **NOT** update the project lockfile (`removeSkillFromLocalLock` is never called in `remove.ts` — only `removeSkillFromLock` for global removes with `--global`).
- The lockfile entry becomes stale after a project-level remove. This is an upstream bug; dotagents accepts the staleness rather than writing to a file it does not own.

## Goals / Non-Goals

**Goals:**
- Fix `skills add` so installs actually land in `.dotagents/skills/` (the original intent that never worked).
- Move external-integration logic (`add`, external `remove`) into a dedicated `src/integrations/skills_sh.rs` module, separated from local source-of-truth operations (`new`, local `rm`, `ls`).
- Namespace the package-runner config under `[integrations.skills-sh]` to leave room for future integrations.
- Make `skills rm` provenance-aware: externally-installed skills (present in `.dotagents/skills-lock.json`) delegate to the skills CLI for removal; locally-authored skills keep the current local-delete + undeploy-cleanup behavior.
- Keep `.dotagents/skills-lock.json` as the single source of install provenance — dotagents reads it but never writes it.

**Non-Goals:**
- Editing or writing `skills-lock.json` (dotagents treats it as read-only provenance; if upstream `skills remove` leaves stale entries, dotagents accepts the staleness).
- A `bin/` directory or any symlink for the lockfile (cwd-based placement is sufficient).
- Provenance tracking in `cache.toml` (the lockfile already records source; duplicating in cache.toml would be redundant and cache.toml is gitignored).
- Making the install target agent configurable beyond openclaw (out of scope; flagged as a follow-on).
- Windows symlink support (not needed — `--copy` avoids symlinks entirely).
- Backward compatibility for the top-level `package-runner` field (the `skills-add` feature is non-default and the field was never scaffolded by `init`).

## Decisions

### Decision 1: Redirect installs via `cwd=.dotagents` + `--agent openclaw` + `--copy`

**Choice**: Spawn the skills CLI with `current_dir(.dotagents/)`, `--agent openclaw`, and `--copy`.

**Rationale**: The skills CLI has no `--target-dir` flag, no env var for the project install path, and no plugin/agent-registration mechanism. The only mechanism is `process.cwd()`. openclaw is the only agent in the skills CLI's agent table with a flat `skillsDir = "skills"` (no dot-prefix, no nesting), so with `cwd=.dotagents/` it writes directly to `.dotagents/skills/<name>/SKILL.md` — exactly dotagents' source-of-truth layout. `--copy` is required because the default mode symlinks from a canonical cache directory; `--copy` produces real files owned by dotagents.

**Alternatives considered**:
- *`--agent claude-code` + post-move*: claude-code writes to `.claude/skills/` (nested), so with `cwd=.dotagents/` it would write to `.dotagents/.claude/skills/<name>/` and require a `fs::rename` to `.dotagents/skills/<name>/` plus cleanup of the empty `.claude/`. More code, more failure modes. Rejected in favor of openclaw's zero-step flat path.
- *Contributing a `--target-dir` flag upstream*: cleaner long-term, but out of scope for this change (we can't block on upstream). Flagged as a follow-on.
- *Setting `CLAUDE_CONFIG_DIR` (current approach)*: broken — only affects global installs, and `add` is a project install. Rejected.

### Decision 2: `[integrations.skills-sh]` config table

**Choice**: Replace the top-level `package-runner` field with:
```toml
[integrations.skills-sh]
package-runner = "bun"
```

**Rationale**: The `[integrations]` table generalizes "external CLIs dotagents shells out to" as a concept distinct from "features dotagents renders" (commands, instructions, mcp, skills). `skills-sh` namespaces everything specific to the skills.sh CLI. This leaves room for future integrations without polluting the top-level config. The `PackageRunner` enum (`Npm` | `Pnpm` | `Yarn` | `Bun`) is unchanged — only its config location moves. The name stays `package-runner` (not `package-manager`) per user decision.

**Alternatives considered**:
- *Keep top-level `package-runner`*: simpler, but no namespace for future integrations and the field is undocumented. Rejected.
- *Rename to `package-manager`*: the word "manager" is more user-natural, but the code already uses `PackageRunner` everywhere. Per user decision, keep `package-runner`.

### Decision 3: `rm` auto-delegates based on lockfile provenance

**Choice**: `skills rm <name>` reads `.dotagents/skills-lock.json` (read-only). If `<name>` is present in the lockfile's `skills` map, the skill is external → delegate to `npx skills remove <name> --agent openclaw --yes` with `cwd=.dotagents/`, then run undeploy cleanup. If `<name>` is absent, the skill is locally authored → run the current `fs::remove_dir_all` + undeploy cleanup.

**Rationale**: One command, branches on source — smoothest UX (per user decision Q8). The lockfile is the single source of provenance (per user decision Q7). dotagents never writes the lockfile; it only reads it to decide the delegation branch (per user decision Q12).

**Alternatives considered**:
- *`rm` refuses and points to a separate command*: more honest about the two operations, but worse UX. Rejected per Q8.
- *Always local `fs::remove_dir_all`, never delegate*: loses the skills CLI's own removal logic (e.g., future lockfile updates if upstream fixes the bug). Rejected per Q12 (we wrap).
- *Dotagents edits the lockfile to clean stale entries*: violates "never write the lockfile." Rejected per Q12/Q14.

### Decision 4: Module shape — single file, split later if needed

**Choice**: Implement `src/integrations/skills_sh.rs` as a single file containing `PackageRunner`, the `add()` wrapper, the `remove()` wrapper, and the read-only lockfile reader. Split into a `skills_sh/` directory only if it grows past ~300–400 lines.

**Rationale**: The empty `src/integrations/skills_sh.rs` file already exists. The expected content (enum + two functions + one reader + config struct) is likely under 300 lines. Premature splitting adds module boilerplate without benefit.

### Decision 5: `PackageRunner` moves to the integrations module

**Choice**: Move `PackageRunner` from `src/core/config/common.rs` to `src/integrations/skills_sh.rs` (or a shared location if another integration ever needs it).

**Rationale**: `PackageRunner` is only used by the skills.sh integration (`add()` resolves it, `args()` builds the skills-CLI invocation). It is conceptually part of the integration, not part of the general config schema. Co-locating it with the wrapping logic keeps the integration self-contained.

## Risks / Trade-offs

- **[openclaw coupling]** We depend on openclaw's `skillsDir: "skills"` staying flat in the skills CLI's agent table. If upstream changes openclaw's path, installs break silently (skills land in the wrong directory, or nowhere). → **Mitigation**: (a) pin the skills CLI version in documentation; (b) add a post-install assertion that `.dotagents/skills/<expected-name>/SKILL.md` exists after the subprocess returns, erroring clearly if not; (c) contribute a `--target-dir` flag upstream as a long-term fix (flagged as follow-on).
- **[stale lockfile after external rm]** `npx skills remove` (project-level) does not call `removeSkillFromLocalLock`, so the lockfile entry persists after the skill files are gone. A subsequent `skills new <same-name>` would see the stale entry and incorrectly classify the new local skill as external. → **Mitigation**: Accept the staleness per user decision Q14. If a user hits this, they can delete the lockfile entry by hand (it's JSON). If upstream fixes the bug, dotagents requires no change (we never wrote the lockfile). Document this in user-facing help.
- **[two lockfiles confusion]** The skills CLI has a global lockfile (`~/.agents/.skill-lock.json`) and a project lockfile (`.dotagents/skills-lock.json`). dotagents only reads the project lockfile. The global lockfile is the skills CLI's own concern. → **Mitigation**: Document clearly that dotagents only consults `.dotagents/skills-lock.json`.
- **[openclaw agent detection side effects]** The skills CLI's `detectAgent()` sees `opencode` in the environment and auto-enables non-interactive mode (observed in the remove probe: "opencode Agent detected — removing non-interactively"). This is benign for our use (we pass `--yes` anyway) but worth noting.
- **[breaking change for existing `package-runner` users]** Anyone with `package-runner = "bun"` at the top level breaks. → **Mitigation**: The `skills-add` feature is non-default (must be enabled at build time) and the field was never scaffolded by `init`. Per user decision Q5, no backward compatibility is required. Note in the changelog.

## Migration Plan

No data migration. The change is code-only:

1. Add `src/integrations/mod.rs` declaring `pub(crate) mod skills_sh;`.
2. Populate `src/integrations/skills_sh.rs` with `PackageRunner` (moved), `add()`, `remove()`, and a read-only `read_lockfile()` helper.
3. Update `src/core/config/{global,local,app,common}.rs` to remove the top-level `package_runner` field and add `integrations: Option<IntegrationsConfig>` with the nested `skills-sh` table.
4. Update `src/cli/skills.rs`: `add()` delegates to `integrations::skills_sh::add()`; `rm_skill()` gains the provenance branch.
5. Update `src/cli/config.rs` display to read from the new config path.
6. Update `src/cli/options.rs` help text referencing the config field location.
7. Update tests (unit + e2e) to reflect the corrected install path and new config shape.

**Rollback**: Revert the commit(s). No persistent state to roll back (the lockfile is owned by the skills CLI, not dotagents).

## Open Questions

None remaining. All decisions settled during exploration (Q1–Q14).
