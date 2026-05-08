## 1. CLI flag additions

- [x] 1.1 Add `--json` flag to `CommandsLs` subcommand in `src/cli/commands.rs`
- [x] 1.2 Add `--json` flag to `SkillsLs` subcommand in `src/cli/skills.rs`
- [x] 1.3 Ensure `--full` flag behavior extends to include body content (flag already exists in both commands)

## 2. Core implementation

- [x] 2.1 Implement `--json` output for `commands ls`: collect `to_value()` results, serialize as JSON array, output to stdout
- [x] 2.2 Implement `--json` output for `skills ls`: same pattern as commands
- [x] 2.3 Implement `--full` body content rendering for `commands ls` CLI mode (include body after frontmatter)
- [x] 2.4 Implement `--full` body content rendering for `skills ls` CLI mode (include body after frontmatter)
- [x] 2.5 Ensure `--json` output uses stdout only, all logs/warnings go to stderr
- [x] 2.6 Ensure `--full` without `--json` shows body in text output
- [x] 2.7 Define `--json --full` combined behavior: `--json` alone outputs frontmatter fields only; when `--full` is also present, add a `content` key with the raw markdown body string

## 3. Unit tests

- [x] 3.1 Test `commands ls --json` produces valid JSON array with correct fields
- [x] 3.2 Test `skills ls --json` produces valid JSON array with correct fields
- [x] 3.3 Test `commands ls --full` includes body content in text output
- [x] 3.4 Test `skills ls --full` includes body content in text output
- [x] 3.5 Test `commands ls` without `--full` does NOT include body content
- [x] 3.6 Test `commands ls --json` with empty workspace outputs `[]`
- [x] 3.7 Test `commands ls --json | jq` parses correctly (verify pipeable)

## 4. E2E tests

- [x] 4.1 Add e2e test: `dotagents commands ls --json` valid JSON with body content
- [x] 4.2 Add e2e test: `dotagents skills ls --json` valid JSON with body content
- [x] 4.3 Add e2e test: `dotagents commands ls --full` shows body content in text output
- [x] 4.4 Add e2e test: `dotagents commands ls --json --full` validates both flags together
- [x] 4.5 Add e2e test: `dotagents commands ls` (default) does NOT include body content
- [x] 4.6 Add e2e test: empty workspace `--json` outputs `[]`

## 5. Verification

- [x] 5.1 Run `mise check` and fix any format/lint issues
- [x] 5.2 Run `mise tests` and fix any failures
