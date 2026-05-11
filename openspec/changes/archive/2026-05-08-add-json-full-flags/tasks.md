## 1. CLI flag additions

- [x] 1.1 Add `--json` flag to `SubLsOptions` shared by `commands ls` and `skills ls`
- [x] 1.2 Rename `--full` flag to `--content` in `SubLsOptions`
- [x] 1.3 Add `--command <name>` filter flag to `SubLsOptions` (used by `commands ls`)
- [x] 1.4 Add `--skill <name>` filter flag to `SubLsOptions` (used by `skills ls`)

## 2. Core implementation

- [x] 2.1 Implement `--json` output for `commands ls`: serialize frontmatter as JSON array, output to stdout
- [x] 2.2 Implement `--json` output for `skills ls`: same pattern as commands
- [x] 2.3 Implement `--content` body rendering for `commands ls` in TTY mode via `cliclack::note` (name — description as header, body in note box)
- [x] 2.4 Implement `--content` body rendering for `skills ls` in TTY mode via `cliclack::note`
- [x] 2.5 Implement `--content` body rendering in non-TTY mode (indented lines below name-description row)
- [x] 2.6 Ensure `--json` output uses stdout only; all logs/warnings go to stderr
- [x] 2.7 Define `--json --content` combined behavior: add `content` key with raw markdown body string to each JSON object
- [x] 2.8 Apply `--command <name>` exact-match filter in `ls_commands`
- [x] 2.9 Apply `--skill <name>` exact-match filter in `ls_skills`
- [x] 2.10 Remove `cliclack::intro` and `Commands (N)` / `Skills (N)` section headers from list output
- [x] 2.11 Render item name in cyan+bold via `console::style` in TTY mode; use ` — ` separator between name and description
- [x] 2.12 Set column width to actual longest name length with no artificial minimum (remove `.max(10)`)
- [x] 2.13 In `--content` TTY mode, render note header name in green+bold

## 3. Unit tests

- [x] 3.1 Test `commands ls --json` produces valid JSON array with correct fields
- [x] 3.2 Test `skills ls --json` produces valid JSON array with correct fields
- [x] 3.3 Test `commands ls --content` includes body content in output
- [x] 3.4 Test `skills ls --content` includes body content in output
- [x] 3.5 Test `commands ls` without `--content` does NOT include body content
- [x] 3.6 Test `commands ls --json` with empty workspace outputs `[]`
- [x] 3.7 Test `commands ls --json | jq` parses correctly (verify pipeable)

## 4. E2E tests

- [x] 4.1 Add e2e test: `dotagents commands ls --json` valid JSON with correct frontmatter fields
- [x] 4.2 Add e2e test: `dotagents skills ls --json` valid JSON with correct frontmatter fields
- [x] 4.3 Add e2e test: `dotagents commands ls --content` shows body content
- [x] 4.4 Add e2e test: `dotagents commands ls --json --content` validates both flags together
- [x] 4.5 Add e2e test: `dotagents commands ls` (default) does NOT include body content
- [x] 4.6 Add e2e test: empty workspace `--json` outputs `[]`

## 5. Verification

- [x] 5.1 Run `mise check` and fix any format/lint issues
- [x] 5.2 Run `mise tests` and fix any failures
