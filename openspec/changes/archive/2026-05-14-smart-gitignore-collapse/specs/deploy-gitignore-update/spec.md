## MODIFIED Requirements

### Requirement: Entries are specific workspace-relative file paths
Each entry written to the fenced section SHALL be either a workspace-relative file path (e.g. `.claude/commands/hello.md`) or a workspace-relative directory pattern with trailing slash (e.g. `.claude/commands/`) when the directory's entire contents are generated. The collapse algorithm SHALL determine which format to use for each entry.

#### Scenario: Specific path for root-level file
- **WHEN** deploy writes `CLAUDE.md` at the workspace root
- **THEN** the gitignore entry is `CLAUDE.md`

#### Scenario: Directory pattern for fully-generated directory
- **WHEN** deploy writes 8 files into `.claude/commands/` and no other files exist in that directory
- **THEN** the gitignore entry is `.claude/commands/` (single directory pattern)

#### Scenario: Mixed directory gets individual entries
- **WHEN** deploy writes files into `.claude/commands/` but the directory also contains a user-created file
- **THEN** each generated file gets its own gitignore entry

### Requirement: Update workspace root .gitignore with fenced section
When the gitignore update step runs, it SHALL rebuild the fenced section from all cached target paths using the collapse algorithm. The fence is rewritten from scratch each time — not appended to. Lines outside the fenced section SHALL NOT be modified. The fenced section SHALL use `#region dotagents` as the opening marker and `#endregion dotagents` as the closing marker.

#### Scenario: .gitignore does not exist — create it
- **WHEN** no `.gitignore` exists at the workspace root
- **THEN** a new `.gitignore` is created containing only the dotagents fenced section with collapsed patterns

#### Scenario: .gitignore exists without fenced section — append section
- **WHEN** `.gitignore` exists with user content but no dotagents fence
- **THEN** the fenced section is appended at the end with `#region dotagents` / `#endregion dotagents` markers; existing content is preserved verbatim

#### Scenario: .gitignore exists with fenced section — rebuild fence
- **WHEN** the `#region dotagents` / `#endregion dotagents` section already exists
- **THEN** the fenced section is completely rewritten with the current collapsed patterns; existing user content outside the fence is unchanged

#### Scenario: All patterns unchanged — no write
- **WHEN** the rebuilt fence content is identical to the existing fence content
- **THEN** `.gitignore` is not modified

#### Scenario: User content outside fence is preserved
- **WHEN** `.gitignore` contains user entries before and after the dotagents fenced section
- **THEN** after rebuild, those entries remain exactly as they were

## REMOVED Requirements

### Requirement: Stale entries accumulate harmlessly
**Reason**: The fence is now rebuilt from cache each time, so stale entries are automatically cleaned up. There is no append-only accumulation.
**Migration**: No action needed. On the next deploy, the fence is rewritten with only current entries.
