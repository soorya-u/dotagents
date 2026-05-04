## MODIFIED Requirements

### Requirement: skills rm deletes a skill directory
`dotagents skills rm <name>` SHALL delete `.dotagents/skills/<name>/` and all its contents. If the directory does not exist, the command SHALL exit 1 with a clear error. After removing the source directory, the command SHALL also remove all deployed files, cache entries, and `.gitignore` fence entries for that skill across every provider (see `rm-cleanup` spec).

#### Scenario: Existing skill directory is removed
- **WHEN** user runs `dotagents skills rm my-skill` and `.dotagents/skills/my-skill/` exists
- **THEN** the directory and all contents are deleted and a success message is shown

#### Scenario: Non-existent skill errors
- **WHEN** user runs `dotagents skills rm my-skill` and no such directory exists
- **THEN** the command exits 1 with an error indicating the skill was not found

#### Scenario: Deployed output cleaned up after source removal
- **WHEN** user runs `dotagents skills rm my-skill` and the skill has been previously deployed
- **THEN** the deployed file is deleted, the cache entry is removed, and the `.gitignore` entry is removed
