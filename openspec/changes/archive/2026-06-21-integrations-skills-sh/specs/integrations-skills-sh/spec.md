### Requirement: integrations module wraps the external skills.sh CLI
The system SHALL provide an `src/integrations/skills_sh.rs` module that wraps the external `skills` CLI (from skills.sh) for `add` and `remove` operations. The module SHALL own: the `PackageRunner` enum, the `add()` wrapper, the `remove()` wrapper, and a read-only lockfile reader for provenance checks. Local source-of-truth operations (`new`, local `rm`, `ls`) SHALL remain in `src/cli/skills.rs` and SHALL NOT shell out to the skills CLI.

#### Scenario: add delegates to the integrations module
- **WHEN** user runs `dotagents skills add <name>`
- **THEN** `src/cli/skills.rs` delegates to `integrations::skills_sh::add()`
- **THEN** the integrations module spawns the skills CLI subprocess

#### Scenario: local rm does not shell out
- **WHEN** user runs `dotagents skills rm <name>` and `<name>` is NOT in `.dotagents/skills-lock.json`
- **THEN** the skill is removed via local `fs::remove_dir_all` without spawning the skills CLI

#### Scenario: external rm delegates to the integrations module
- **WHEN** user runs `dotagents skills rm <name>` and `<name>` IS in `.dotagents/skills-lock.json`
- **THEN** `src/cli/skills.rs` delegates to `integrations::skills_sh::remove()`
- **THEN** the integrations module spawns `npx skills remove` as a subprocess

### Requirement: add spawns the skills CLI with cwd set to the application directory
The `integrations::skills_sh::add()` function SHALL spawn the skills CLI with `current_dir` set to the dotagents application directory (`.dotagents/` in release, `.dotagents-debug/` in debug). It SHALL pass `--agent openclaw` (whose flat `skills/` project dir matches dotagents' layout) and `--copy` (real files, not symlinks to a cache). It SHALL NOT set the `CLAUDE_CONFIG_DIR` environment variable.

#### Scenario: subprocess runs with cwd in the application directory
- **WHEN** `integrations::skills_sh::add()` spawns the skills CLI
- **THEN** the subprocess `current_dir` is the dotagents application directory
- **THEN** the skills CLI writes skill files to `<application-dir>/skills/<name>/SKILL.md`

#### Scenario: openclaw agent flag is passed
- **WHEN** `integrations::skills_sh::add()` builds the skills CLI argument list
- **THEN** the argument list includes `--agent openclaw`

#### Scenario: copy flag is passed
- **WHEN** `integrations::skills_sh::add()` builds the skills CLI argument list
- **THEN** the argument list includes `--copy`
- **THEN** the installed skill files are real files, not symlinks

#### Scenario: CLAUDE_CONFIG_DIR is not set
- **WHEN** `integrations::skills_sh::add()` spawns the skills CLI
- **THEN** the `CLAUDE_CONFIG_DIR` environment variable is not set by dotagents

#### Scenario: skills land in the application skills directory
- **WHEN** `dotagents skills add vercel-labs/skills@find-skills` completes successfully
- **THEN** `<application-dir>/skills/find-skills/SKILL.md` exists as a real file
- **THEN** no `.claude/skills/`, `.cursor/skills/`, or other agent-specific directories are created in the workspace

### Requirement: lockfile lands in the application directory
Because the skills CLI writes `skills-lock.json` to its cwd, spawning with `current_dir(application-dir)` SHALL cause the lockfile to land at `<application-dir>/skills-lock.json`. dotagents SHALL treat this file as read-only provenance and SHALL NEVER write to or edit it.

#### Scenario: lockfile created in the application directory
- **WHEN** `dotagents skills add <name>` completes successfully
- **THEN** `<application-dir>/skills-lock.json` exists and contains an entry for the installed skill with `source`, `sourceType`, `skillPath`, and `computedHash` fields

#### Scenario: dotagents does not write the lockfile
- **WHEN** any dotagents command runs (add, rm, new, ls, deploy)
- **THEN** dotagents never opens `<application-dir>/skills-lock.json` for writing

### Requirement: remove wraps the skills CLI for external skills
The `integrations::skills_sh::remove()` function SHALL spawn `npx skills remove <name> --agent openclaw --yes` with `current_dir(application-dir)`. After the subprocess completes, the caller SHALL run the standard undeploy cleanup (deployed files, cache entries, gitignore fence) regardless of whether the subprocess succeeded.

#### Scenario: external remove spawns skills CLI with correct cwd
- **WHEN** `integrations::skills_sh::remove()` is invoked for an external skill
- **THEN** the subprocess `current_dir` is the dotagents application directory
- **THEN** the argument list includes `remove <name> --agent openclaw --yes`

#### Scenario: undeploy cleanup runs after external remove
- **WHEN** `integrations::skills_sh::remove()` completes for an external skill
- **THEN** the caller runs undeploy cleanup for the skill across all providers
- **THEN** cache entries for the skill are removed and the gitignore fence is rebuilt

### Requirement: PackageRunner lives in the integrations module
The `PackageRunner` enum (`Npm` | `Pnpm` | `Yarn` | `Bun`) SHALL be defined in `src/integrations/skills_sh.rs`, not in `src/core/config/common.rs`. Its `binary()` and `args()` methods SHALL produce the skills-CLI invocation appropriate to the runner. The `args()` method SHALL append `--yes` when `ci` is true (non-TTY).

#### Scenario: npm runner produces npx invocation
- **WHEN** `PackageRunner::Npm.args("vercel-labs/skills", false)` is called
- **THEN** the result is `["npx", "skills", "add", "vercel-labs/skills", "--agent", "openclaw", "--copy"]`

#### Scenario: bun runner produces bunx invocation
- **WHEN** `PackageRunner::Bun.args("my-skill", false)` is called
- **THEN** the result is `["bunx", "skills", "add", "my-skill", "--agent", "openclaw", "--copy"]`

#### Scenario: ci mode appends --yes
- **WHEN** `PackageRunner::Npm.args("my-skill", true)` is called
- **THEN** the result ends with `["--agent", "openclaw", "--copy", "--yes"]`

### Requirement: read-only lockfile reader for provenance
The integrations module SHALL provide a function that reads `<application-dir>/skills-lock.json` and returns whether a given skill name is present in the `skills` map. If the lockfile is absent, malformed, or unreadable, the function SHALL return `false` (treat as locally authored) and log a warning.

#### Scenario: skill present in lockfile returns true
- **WHEN** the lockfile contains an entry for `find-skills` and the reader is queried for `find-skills`
- **THEN** the function returns `true`

#### Scenario: skill absent from lockfile returns false
- **WHEN** the lockfile contains no entry for `my-local-skill` and the reader is queried for `my-local-skill`
- **THEN** the function returns `false`

#### Scenario: missing lockfile returns false with warning
- **WHEN** `<application-dir>/skills-lock.json` does not exist and the reader is queried for any name
- **THEN** the function returns `false` and a warning is logged

#### Scenario: malformed lockfile returns false with warning
- **WHEN** `<application-dir>/skills-lock.json` contains invalid JSON and the reader is queried for any name
- **THEN** the function returns `false` and a warning is logged
