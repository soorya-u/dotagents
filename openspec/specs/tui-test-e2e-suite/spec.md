# Spec: TUI-Test E2E Suite

## Purpose

Defines the structure and coverage requirements for the end-to-end test suite in `tests/e2e/`, which uses `@microsoft/tui-test` to exercise the binary through CLI flags (non-interactive), interactive TUI prompts, and multi-command user journeys.

## Requirements

### Requirement: E2E suite uses tui-test as the test framework
`tests/e2e/` SHALL be a TypeScript project using `@microsoft/tui-test`. Test files SHALL use the `import { test, expect } from "@microsoft/tui-test"` API. The suite SHALL be runnable via `mise test-e2e` without any manual setup beyond `mise install`.

#### Scenario: Suite runs via mise
- **WHEN** `mise test-e2e` is invoked on a clean checkout (after `mise install`)
- **THEN** bun deps are installed automatically (via `depends`) and all tests run

#### Scenario: Filesystem assertions use Node.js fs
- **WHEN** a test needs to assert on a file created by the binary
- **THEN** it uses `import { readFileSync, existsSync } from "fs"` alongside tui-test terminal assertions

### Requirement: Discovery phase precedes test writing
Before tui-test files are written, a discovery agent SHALL run every flow from the flow inventory and record structured observations (exact terminal output, prompt sequence, filesystem state). The tui-tests SHALL be written from those observations, not from source-code reading alone.

#### Scenario: Discovery captures interactive prompt text
- **WHEN** the discovery agent runs `dotagents init` in a tui-devtools PTY session
- **THEN** it records the exact text and order of each cliclack prompt (intro, multiselect labels, option labels, outro)

#### Scenario: Discovery captures filesystem state
- **WHEN** a CLI flow completes during discovery
- **THEN** the agent records which files were created and their contents alongside the terminal output

### Requirement: CLI flows are covered (non-interactive, flag-driven)
The e2e suite SHALL contain tests for all CLI flows. Each test SHALL run the binary with flags that suppress interactive prompts and assert on both terminal output and filesystem side effects.

#### Scenario: Init with starter template (C01)
- **WHEN** `dotagents init --template starter` runs in an isolated temp workspace
- **THEN** `.dotagents/config.toml`, `.dotagents/.env`, `.dotagents/INSTRUCTIONS.md`, `.dotagents/mcp.jsonc` exist and `templates/mycode/` does not exist

#### Scenario: Init with custom provider template (C02)
- **WHEN** `dotagents init --template with-custom-provider` runs
- **THEN** `templates/mycode/` directory and provider config blocks are created

#### Scenario: Init --no-mcp suppresses mcp file (C06)
- **WHEN** `dotagents init --template starter --no-mcp` runs
- **THEN** `.dotagents/mcp.jsonc` does not exist

#### Scenario: Add command with all flags (C08)
- **WHEN** `dotagents add command hello --description "Say hello" --category "Greetings" --tags "greeting,hello"` runs
- **THEN** `.dotagents/commands/hello.md` exists with correct frontmatter fields

#### Scenario: Deploy creates output files (C14)
- **WHEN** `dotagents deploy` runs after init in non-TTY mode
- **THEN** output files are created at provider target paths with no unrendered Handlebars tokens

#### Scenario: Deploy outside workspace fails helpfully (C31)
- **WHEN** `dotagents deploy` runs with no `.dotagents/` directory present
- **THEN** the process exits non-zero and stderr contains a helpful message

#### Scenario: Ls shows both sections (C20)
- **WHEN** `dotagents ls` runs after init
- **THEN** terminal output contains both "Commands" and "Skills" sections

#### Scenario: Rm --force deletes command (C26)
- **WHEN** `dotagents rm command hello --force` runs after adding a command
- **THEN** `.dotagents/commands/hello.md` no longer exists and exit code is 0

#### Scenario: Gen-completions produces non-empty file (C28)
- **WHEN** `dotagents gen-completions --shell bash --to ./out` runs
- **THEN** `./out/dotagents.bash` exists and has non-zero file size

### Requirement: TUI flows are covered (interactive, cliclack prompts)
The e2e suite SHALL contain tests for all TUI flows. Each test SHALL use tui-test's input API to navigate interactive prompts and assert on rendered terminal output.

#### Scenario: Init full wizard happy path (T01)
- **WHEN** `dotagents init` runs in a PTY, user accepts all feature defaults and selects Starter template
- **THEN** terminal shows intro, multiselect, template select, and outro; all four feature files are created

#### Scenario: Init wizard cancel on overwrite (T05)
- **WHEN** `dotagents init` runs with an existing `.dotagents/` dir and user selects "No, cancel" at overwrite prompt
- **THEN** terminal shows "Init cancelled." and the existing directory is untouched

#### Scenario: Add command interactive prompts (T06)
- **WHEN** `dotagents add command hello` runs in PTY and user types description, category, tags, then answers "No" to deploy
- **THEN** `.dotagents/commands/hello.md` is created with the provided metadata and deploy does not run

#### Scenario: Rm command confirm yes (T10)
- **WHEN** `dotagents rm command hello` runs in PTY and user confirms deletion
- **THEN** `.dotagents/commands/hello.md` is deleted and terminal shows outro

#### Scenario: Rm command confirm no cancels (T11)
- **WHEN** `dotagents rm command hello` runs in PTY and user selects "No" at confirm prompt
- **THEN** `.dotagents/commands/hello.md` still exists and terminal shows "Cancelled."

### Requirement: Journey flows are covered (multi-command user stories)
The e2e suite SHALL contain tests for complete user journeys that chain multiple commands.

#### Scenario: Init → add command → deploy → verify output (J01)
- **WHEN** user runs init, then adds a command, then deploys
- **THEN** the rendered output file at the provider target path exists, contains the command body, and has no YAML frontmatter

#### Scenario: Full CRUD for commands (J03)
- **WHEN** user runs init → add command → ls (sees it) → rm command --force → ls
- **THEN** after removal the command no longer appears in ls output

#### Scenario: Redeploy picks up source changes (J05)
- **WHEN** user deploys, then edits `INSTRUCTIONS.md`, then deploys again
- **THEN** the second deploy's output file reflects the edited content

#### Scenario: Idempotent deploy (J06)
- **WHEN** deploy is run twice without any source changes
- **THEN** the output files are byte-for-byte identical after both runs

### Requirement: Each test uses an isolated temp workspace
Every tui-test test SHALL create a fresh temporary directory for its workspace and clean it up after the test completes. Tests SHALL NOT share workspace state.

#### Scenario: Parallel tests do not interfere
- **WHEN** two tests run concurrently, each in its own temp directory
- **THEN** neither test observes files written by the other
