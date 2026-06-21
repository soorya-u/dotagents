## Purpose

Specifies the extended `dotagents skills` subcommand group, adding `new`, `rm`, and `ls` sub-actions alongside the existing `add` (registry install). These replace the removed top-level `add skill`, `rm skill`, and `ls --skills` commands.

## Requirements

### Requirement: skills new creates a new local skill scaffold
`dotagents skills new <name>` SHALL create `.dotagents/skills/<name>/SKILL.md` with YAML frontmatter (`name`, `description`, `license`, `compatibility`, `metadata.version`) and a fixed starter body template with `<name>` interpolated. Behaviour is identical to the removed `dotagents add skill`.

#### Scenario: Skill directory and file created with flags
- **WHEN** user runs `dotagents skills new my-skill --description "Does something" --license MIT`
- **THEN** `.dotagents/skills/my-skill/SKILL.md` is created with provided frontmatter and the skill starter body

#### Scenario: Missing flags prompt in TTY mode
- **WHEN** user runs `dotagents skills new my-skill` with no flags in a TTY
- **THEN** cliclack prompts for description, license, and compatibility before creating the file

#### Scenario: Missing flags use empty defaults in non-TTY
- **WHEN** user runs `dotagents skills new my-skill` with no flags and stdin is not a TTY
- **THEN** missing fields default to empty strings and the file is created without prompting

#### Scenario: Skill already exists errors without --force
- **WHEN** `.dotagents/skills/<name>/SKILL.md` already exists and `--force` is not passed
- **THEN** the command exits 1 with an error indicating the skill already exists

#### Scenario: Skill already exists is overwritten with --force
- **WHEN** `.dotagents/skills/<name>/SKILL.md` already exists and `--force` is passed
- **THEN** the file is overwritten with new frontmatter and the starter body

### Requirement: skills rm deletes a skill directory
`dotagents skills rm <name>` SHALL delete `.dotagents/skills/<name>/` and all its contents. The command SHALL be provenance-aware: it reads `<application-dir>/skills-lock.json` (read-only) to determine whether the skill was installed externally (present in the lockfile) or authored locally (absent from the lockfile).

- **Externally installed** (lockfile entry exists): the command SHALL delegate file removal to `integrations::skills_sh::remove()`, which spawns `npx skills remove <name> --agent openclaw --yes` with `current_dir(application-dir)`. After the subprocess completes, the command SHALL run the standard undeploy cleanup (deployed files, cache entries, gitignore fence) regardless of subprocess success. dotagents SHALL NOT edit the lockfile; if the skills CLI leaves a stale entry, dotagents accepts the staleness.
- **Locally authored** (no lockfile entry): the command SHALL remove the directory via `fs::remove_dir_all` and run the standard undeploy cleanup. This is the pre-existing behavior.

In both cases, if the source directory does not exist, the command SHALL exit 1 with a clear error.

#### Scenario: Existing external skill directory is removed via delegation
- **WHEN** user runs `dotagents skills rm my-skill` and `.dotagents/skills/my-skill/` exists and `my-skill` is present in `.dotagents/skills-lock.json`
- **THEN** `integrations::skills_sh::remove()` spawns `npx skills remove my-skill --agent openclaw --yes` with `current_dir` set to the application directory
- **THEN** the directory and all contents are deleted
- **THEN** deployed files, cache entries, and the gitignore fence are cleaned up across all providers

#### Scenario: Existing local skill directory is removed via fs
- **WHEN** user runs `dotagents skills rm my-skill` and `.dotagents/skills/my-skill/` exists and `my-skill` is NOT present in `.dotagents/skills-lock.json`
- **THEN** the directory and all contents are deleted via `fs::remove_dir_all` (no subprocess spawned)
- **THEN** deployed files, cache entries, and the gitignore fence are cleaned up across all providers
- **THEN** a success message is shown

#### Scenario: Non-existent skill errors
- **WHEN** user runs `dotagents skills rm my-skill` and no such directory exists
- **THEN** the command exits 1 with an error indicating the skill was not found

#### Scenario: Confirm shown in TTY without --force
- **WHEN** user runs `dotagents skills rm my-skill` in a TTY without `--force`
- **THEN** a cliclack confirm prompt is displayed before deletion (applies to both external and local paths)

#### Scenario: Confirm declined aborts deletion
- **WHEN** user declines the confirm prompt
- **THEN** no directory is deleted, no subprocess is spawned, and the command exits 0

#### Scenario: --force skips confirm
- **WHEN** `--force` is passed
- **THEN** deletion proceeds immediately without any confirmation prompt (applies to both external and local paths)

#### Scenario: Non-TTY skips confirm
- **WHEN** stdin is not a TTY
- **THEN** deletion proceeds without prompting regardless of `--force` (applies to both external and local paths)

#### Scenario: Deployed output cleaned up after external source removal
- **WHEN** user runs `dotagents skills rm my-skill` for an external skill that has been previously deployed
- **THEN** the deployed file is deleted, the cache entry is removed, and the `.gitignore` entry is removed

#### Scenario: Deployed output cleaned up after local source removal
- **WHEN** user runs `dotagents skills rm my-skill` for a local skill that has been previously deployed
- **THEN** the deployed file is deleted, the cache entry is removed, and the `.gitignore` entry is removed

#### Scenario: Stale lockfile entry accepted after external remove
- **WHEN** `integrations::skills_sh::remove()` completes and the skills CLI did not remove the lockfile entry
- **THEN** dotagents does not edit the lockfile and logs no error about the stale entry
- **THEN** the command exits 0

### Requirement: skills ls lists local skill source directories
`dotagents skills ls` SHALL read skills from `.dotagents/skills/*/SKILL.md`, parse frontmatter for `name`, `description`, `license`, and `compatibility`, and display them using cliclack output. Descriptions SHALL be truncated to fit terminal width by default.

When `--json` is passed, the command SHALL output a JSON array of skill objects containing frontmatter fields (name, description, license, compatibility). Body content SHALL NOT be included in JSON output unless `--content` is also passed. All log/warning output SHALL go to stderr.

When `--content` is passed, the command SHALL include the full markdown body content of each skill. In TTY mode, each skill is rendered as a `cliclack::note` block with `name — description` as the note header (name styled green+bold) and the body inside the note box; no separate `info!` line is printed for that item. In non-TTY mode, body lines are printed indented below the name-description row. Without `--content`, only name and description are shown. When both `--json` and `--content` are passed, each JSON object SHALL include a `content` key with the raw markdown body string.

When `--skill <name>` is passed, the listing SHALL be filtered to only the skill whose name exactly matches `<name>`. If no skill matches, an empty list is shown.

#### Scenario: Skills listed
- **WHEN** user runs `dotagents skills ls`
- **THEN** all skills found in `.dotagents/skills/` are listed with names and truncated descriptions

#### Scenario: Empty workspace
- **WHEN** `.dotagents/skills/` is empty or absent
- **THEN** a message indicating no skills were found is displayed and the command exits 0

#### Scenario: Missing workspace exits with error
- **WHEN** no `.dotagents/` directory exists in the current or any parent directory
- **THEN** the command exits 1 with an error referencing `dotagents init`

#### Scenario: --content shows complete descriptions and body content
- **WHEN** user runs `dotagents skills ls --content`
- **THEN** each skill's full description is shown, word-wrapped at terminal width, followed by the full markdown body content

#### Scenario: --json outputs skills as JSON array
- **WHEN** user runs `dotagents skills ls --json`
- **THEN** stdout contains a JSON array of skill objects with frontmatter fields (name, description, license, compatibility); body content is absent; stderr contains any log messages

#### Scenario: --json with empty workspace outputs empty array
- **WHEN** user runs `dotagents skills ls --json` with no skills present
- **THEN** stdout contains `[]` and the command exits 0

#### Scenario: Without --content, body content is omitted
- **WHEN** user runs `dotagents skills ls` without `--content`
- **THEN** only the skill name and frontmatter metadata (description, license, compatibility) are shown; body content is not displayed

#### Scenario: --skill filters listing to a single skill
- **WHEN** user runs `dotagents skills ls --skill my-skill`
- **THEN** only the skill named `my-skill` is shown; all other skills are excluded

#### Scenario: Text output shows styled name with separator
- **WHEN** user runs `dotagents skills ls` in a TTY
- **THEN** each row renders as `{cyan+bold name} — {truncated description}` with no intro header and no `Skills (N)` section header; the count appears only in the outro

### Requirement: skills new and rm support --deploy flag
After creating or deleting a local skill, `skills new` and `skills rm` SHALL optionally trigger a deploy.

#### Scenario: --deploy flag triggers deploy immediately
- **WHEN** `--deploy` is passed to `skills new` or `skills rm`
- **THEN** `dotagents deploy` runs automatically after the operation, without prompting

#### Scenario: TTY confirm shown when --deploy is absent
- **WHEN** no `--deploy` flag is passed and stdin is a TTY
- **THEN** a cliclack confirm "Deploy now?" (default: No) is shown after the operation

#### Scenario: No deploy in non-TTY without --deploy
- **WHEN** no `--deploy` flag is passed and stdin is not a TTY
- **THEN** deploy is skipped silently
