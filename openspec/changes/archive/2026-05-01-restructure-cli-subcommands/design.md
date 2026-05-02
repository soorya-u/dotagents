## Context

The CLI currently has three top-level generic verbs (`add`, `rm`, `ls`) that each dispatch to domain-specific sub-actions (`command` / `skill`). This flat structure makes the command surface hard to discover and grows awkwardly as more domains are added. Separately, `init` carries four `--no-*` boolean flags — one per feature — that must be extended every time a feature is added. Both issues are solved by moving to domain-scoped subcommand groups and a single whitelist flag.

The project uses Clap (derive API) for all CLI parsing. Feature logic (file creation, deletion, listing) already lives in discrete functions; this change is primarily a routing refactor, not a logic rewrite.

## Goals / Non-Goals

**Goals:**
- Replace top-level `add`, `rm`, `ls` with `commands {new,rm,ls}` and `skills {new,rm,ls}`
- Replace four `--no-*` init flags with a single `--features` whitelist flag
- Delete `src/cli/add.rs`, `src/cli/rm.rs`, `src/cli/ls.rs` — no dead code left behind
- Keep `skills add` (registry install) exactly as-is; it simply gains sibling sub-actions
- Preserve all existing flag behaviour on `new`/`rm`/`ls` sub-actions (`-d`, `-f`, `--deploy`, `--full`, etc.)

**Non-Goals:**
- Changes to deploy, gen-completions, or any non-routing logic
- Changing how features actually work at runtime (file formats, templates, MCP parsing, etc.)
- Backwards-compatible aliases for the removed commands
- Updating README / docs (tracked separately)

## Decisions

### 1. `commands` as a first-class subcommand group

**Decision:** Add a new `CommandsAction` enum in `src/cli/options.rs` with variants `New(AddCommandOptions)`, `Rm(RmCommandOptions)`, `Ls(SubLsOptions)`. Wire it through a new `src/cli/commands.rs` handler that contains the business logic inline (file I/O, TUI prompts).

**Alternatives considered:**
- Keep top-level `add`/`rm`/`ls` and add `commands` as aliases — rejected; leaves dead surface and doubles maintenance burden.
- Merge all logic into `options.rs` directly — rejected; violates the one-file-per-handler pattern already established by `skills.rs`, `deploy.rs`, etc.

### 2. `skills` group expansion

**Decision:** Extend `SkillsAction` in `src/cli/options.rs` with `New(AddSkillOptions)`, `Rm(RmSkillOptions)`, `Ls(LsOptions)` variants alongside the existing `Add(SkillsAddOptions)`. Handle them in `src/cli/skills.rs`.

**Rationale:** `skills add` (registry install) and `skills new` (local scaffold) are semantically distinct — "add" means fetch an existing thing, "new" means author something from scratch. Keeping both under `skills` with different verbs is clear and precedented (e.g., `git remote add` vs `git remote rename`).

### 3. `--features` flag design

**Decision:** Use `Option<Vec<Feature>>` where `Feature` is a Clap-parseable enum (`Commands`, `Instructions`, `Mcp`, `Skills`, `None`). Configure the field with `value_delimiter = ','` and `num_args = 0..` so both `--features commands,mcp` and `--features commands --features mcp` are accepted.

**Validation (post-parse):**
- If `features` is `Some([])` (flag given with no values) → error: ambiguous.
- If `features` contains `None` alongside other variants → error: `none` is exclusive.
- If `features` is `Some([None])` → treat as empty set (no features scaffolded).
- If `features` is `Option::None` (flag absent) → all features enabled; TUI runs normally.

**`is_tui_mode` update:**
```
features.is_none() && template.is_none() && stdin.is_terminal()
```
The four boolean flags are removed entirely; their only job was to set `is_tui_mode` to false and skip certain `InitFile` entries.

**Alternatives considered:**
- Blacklist (keep `--no-*`, add new ones) — rejected; grows without bound and is the problem being solved.
- Separate `--enable-features` / `--disable-features` pair — rejected; over-engineered for the current feature count.

### 4. `SubLsOptions` for `commands ls` and `skills ls`

**Decision:** Introduce a lightweight `SubLsOptions { full: bool }` as the argument type for both `commands ls` and `skills ls`. This avoids the confusing filter fields from the old top-level `LsOptions` struct (`commands ls --commands` is nonsensical). The old `LsOptions` type is deleted along with `src/cli/ls.rs`.

### 5. Remove `-v` → `--full` implicit tie-in

**Decision:** The current `ls` logic enables `--full` when the global `-v` flag is set. This implicit coupling is surprising and undocumented; it is removed. `--full` is now only enabled when the flag is explicitly passed to `commands ls` or `skills ls`.

**Rationale:** Verbosity (`-v`) controls log level, not output format. Conflating the two violates single responsibility and was flagged as incorrect behaviour.

### 6. File deletion strategy

**Decision:** Delete `src/cli/add.rs`, `src/cli/rm.rs`, `src/cli/ls.rs` entirely. The routing logic is not worth keeping; the business logic (file I/O, TUI prompts) is moved inline into `src/cli/commands.rs` and expanded `src/cli/skills.rs`, or extracted into shared utility functions if needed.

## Risks / Trade-offs

- **Breaking change for scripts** → Any user shell scripts calling `dotagents add command`, `dotagents rm skill`, or `dotagents ls` will break silently (wrong subcommand → Clap error). No mitigation planned; this is intentional and documented in the proposal.
- **Shell completion staleness** → Users with cached completions will see outdated suggestions until they regenerate. Mitigation: `gen-completions` is unchanged; users can re-run it.
- **`LsOptions` reuse confusion** → If `LsOptions` is reused with its `commands`/`skills` filter fields, a developer could accidentally wire `commands ls --skills` and show skills instead. Mitigation: use `SubLsOptions` as decided above.

## Migration Plan

This is a CLI-only change with no data migration. Deployment is a single binary swap. There is no rollback complexity beyond reverting to the previous binary version.

## Open Questions

- None. All design decisions are settled based on the explore session.
