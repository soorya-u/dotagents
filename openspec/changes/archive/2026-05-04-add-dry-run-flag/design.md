## Context

`deploy` renders Handlebars templates in memory and calls `write_file` for each target; it also writes `cache.toml` and updates the `.gitignore` fence. `undeploy` reads `cache.toml`, calls `delete_file` for each entry, prunes empty parent dirs, clears the cache, and removes the `.gitignore` fence. Neither command has a preview mode today. The side effects in both commands are concentrated in a small number of call sites, making a flag-at-call-site approach straightforward.

## Goals / Non-Goals

**Goals:**
- `deploy --dry-run`: render templates, resolve providers, print `[+]`/`[~]` per target path, exit 0/1 — zero side effects.
- `undeploy --dry-run`: read cache, check on-disk hashes, print `[-]`/`[x]` per path, exit 0/1 — zero side effects.
- Template and config validation errors still surface (same as a real run).
- Flag respects all existing peer flags (`--offline`, `--force`, `--no-cache`, etc.).
- Interactive prompts suppressed when `--dry-run` is set.
- CI-friendly plain-text output (`[+]` / `[~]` / `[-]` / `[x]`).

**Non-Goals:**
- Content or diff preview (deferred to v0.2, tracked in issue #58).
- Grouped output by provider or feature (deferred to v0.2, tracked in issue #58).
- `init --dry-run` (scaffolding has no meaningful preview value).
- A TUI option or interactive selector for dry-run mode.

## Decisions

### Branch at call site, not via filesystem abstraction

**Alternatives considered:**
- *Filesystem trait abstraction* (dotter's approach): introduce a `FilesystemOps` trait with `DryRunFilesystem` and `RealFilesystem` implementations. Clean separation but heavyweight for a codebase where the only write sites are `write_file` (deploy) and `delete_file` (undeploy).
- *Flag-at-call-site* (chosen): pass `dry_run: bool` through the call stack and guard each side-effecting call. Template rendering is already fully in-memory up to the point of `write_file`, so no simulation layer is needed — we just skip the final write and record the intent.

**Rationale:** The trait approach adds indirection and boilerplate for two call sites. The flag approach is idiomatic for the codebase's current size and keeps the diff minimal.

---

### Status detection for `deploy --dry-run`

After a template is rendered to a string:
- **`[+]` new** — target path does not exist on disk.
- **`[~]` modified** — target path exists and its on-disk content differs from the rendered output.
- **skip (hidden)** — target path exists and content matches (cache hit or byte-identical render).

This mirrors what a real deploy would do without actually writing anything.

---

### Status detection for `undeploy --dry-run`

For each entry in `cache.toml`:
- **`[-]` would delete** — on-disk hash matches the cached hash (file is unmodified).
- **`[x]` edited** — on-disk hash differs from the cached hash (file was changed; real run would prompt).
- **warn and skip** — file does not exist on disk (same behaviour as real undeploy).

---

### Prompt suppression

Both commands have interactive prompts (gitignore consent, edit-detected confirmation, undeploy bulk confirmation). In dry-run mode all prompts are skipped — the flag implies non-interactive preview intent. The prompts are already guarded by TTY detection; `--dry-run` adds a second guard at the same sites.

---

### Output format

```
Dry run — no files will be written

  [+] .claude/commands/hello.md
  [~] .claude/commands/standup.md

2 files would be affected
```

```
Dry run — no files will be deleted

  [-] .claude/commands/hello.md
  [x] .claude/commands/standup.md  (edited)

2 files would be affected
```

Output goes to stdout. In non-TTY (CI) mode, same format — no color codes, no spinner, same symbols. This is handled by the existing `is_tty()` guard already used in the deploy/undeploy UI helpers.

A new module `src/cli/ui/dry_run.rs` houses `print_dry_run_deploy_summary()` and `print_dry_run_undeploy_summary()` so the formatting logic stays out of the command modules.

---

### Exit codes

- **0** — all operations would succeed (even if the list is empty).
- **1** — any template rendering error, config load failure, or unrecoverable error during the dry-run pass. Same semantics as a real run.

## Risks / Trade-offs

- **`[~]` false positives** — comparing rendered string to on-disk bytes can flag a file as modified if only whitespace or line endings differ. Mitigation: use byte-exact comparison (same as the cache hash), which is consistent with what deploy would actually skip.
- **Rayon parallelism and result collection** — deploy already uses `rayon::par_iter`. Collecting dry-run entries in parallel requires a thread-safe accumulator (`Mutex<Vec<_>>` or collecting into a `Vec` after `par_iter().map().collect()`). The latter is idiomatic and avoids lock contention.
- **Remote registry fetch in dry-run** — the flag does not suppress network calls; `--offline` must be passed explicitly. This is intentional: a dry-run should reflect what a real deploy would do, including fetching missing templates.
