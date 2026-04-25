## Context

`dotagents deploy` currently writes provider config files (e.g. `CLAUDE.md`, `AGENTS.md`, `.claude/commands/*.md`, `.github/copilot-instructions.md`) into the workspace and returns. It makes no attempt to prevent those files from being tracked by git. The workspace root `.gitignore` is not read or written. `crossterm` is already a dependency (used elsewhere in the project), so interactive prompts require no new dependencies.

The key constraint is that deployed output directories are not exclusively owned by dotagents — `.github/` contains workflows, `.claude/` may contain other user files — so the gitignore entries must be at the **specific file path level**, not directory wildcards.

## Goals / Non-Goals

**Goals:**
- Collect all rendered target paths at the end of deploy and offer to add them to the workspace root `.gitignore`
- Maintain a dotagents-owned fenced section so user entries are never disturbed
- Support three operating modes: always (`--gitignore`), never (`--no-gitignore`), ask (default)
- Accumulate entries — never remove stale ones

**Non-Goals:**
- Writing to `.git/info/exclude` (committed root `.gitignore` is sufficient given deploy is standard setup)
- Wildcard or directory-level patterns (specific paths only)
- Removing stale entries (out of scope for now)
- Modifying `.gitignore` files inside subdirectories

## Decisions

### 1. Collect target paths from deploy return values, not re-reading config

**Decision**: The deploy pipeline already knows which files it wrote (target paths are computed during rendering). Rather than re-reading and re-evaluating config after deploy, each `deploy_feature` call returns the list of paths it wrote. These are aggregated in `deploy()` and passed to the gitignore updater.

**Alternatives considered**:
- Re-derive from config after deploy — requires re-running template evaluation just for path extraction. Rejected.
- Read from filesystem diff — fragile, slow. Rejected.

### 2. Fenced section format

**Decision**: Use a clearly marked fenced section in `.gitignore`:

```
# BEGIN dotagents managed - do not edit manually
.claude/commands/hello.md
CLAUDE.md
AGENTS.md
# END dotagents managed
```

The updater parses the existing `.gitignore`, extracts the set of paths currently inside the fence, appends any new paths not already present, and rewrites the whole file with the updated fence. Lines outside the fence are preserved verbatim.

**Alternatives considered**:
- Append-only without fencing — entries would be scattered throughout `.gitignore`, impossible to identify or update later. Rejected.

### 3. Default mode is interactive prompt

**Decision**: When neither `--gitignore` nor `--no-gitignore` is passed, after deploy completes print:

```
Add 3 deployed path(s) to .gitignore? [y/N]:
```

Default is `N` (no-op). This respects user intent — dotagents doesn't silently modify a tracked file on first use. After the user answers once, they will typically set `--gitignore` in their workflow or alias.

**Alternatives considered**:
- Default to always updating — too invasive for a first run; modifies a committed file without consent. Rejected.
- Default to never updating — defeats the purpose; users would need to discover the flag. Rejected.

### 4. Specific paths, not directory wildcards

**Decision**: Each entry in the fenced section is the exact workspace-relative path of the file that was written (e.g. `.github/copilot-instructions.md`), never a directory pattern (e.g. `.github/`).

**Rationale**: `.github/` also contains workflows, actions configs, and other user-managed files. Adding `.github/` as a pattern would accidentally hide those from git. The same applies to `.claude/`, `.cursor/`, etc. Specific paths are safe regardless of what else lives in the directory.

**For dynamic per-item features (commands)**: each command file is a separate specific entry. If a repo has 10 commands × 3 providers = 30 entries. Acceptable; gitignore files with hundreds of entries are common.

### 5. Accumulate stale entries

**Decision**: When a target is removed from config, its entry stays in the fenced section indefinitely. Gitignoring a file that doesn't exist is harmless. Cleanup can be added later (e.g. `dotagents clean` command) when the caching proposal provides the "last deployed paths" data needed to prune accurately.

## Risks / Trade-offs

- **User edits inside the fenced section are overwritten** → Section is clearly labelled "do not edit manually". Entries outside the fence are never touched.
- **`.gitignore` doesn't exist and creation fails (permissions)** → Wrap in `Result`, surface as a non-fatal warning — deploy already succeeded; gitignore update is best-effort.
- **Interactive prompt breaks non-TTY environments (CI)** → In non-TTY environments (detected via `crossterm` or `std::io::IsTerminal`), default to skipping the update silently (behave as `--no-gitignore`).
- **30+ entries for large command sets** → Harmless; accepted.

## Open Questions

*(none)*
