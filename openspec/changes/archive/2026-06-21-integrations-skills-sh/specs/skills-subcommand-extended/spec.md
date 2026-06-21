## MODIFIED Requirements

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
