## Context

`display_error()` formats the `anyhow` error chain as:
```
Failed to <first_message>
Caused by:
    <second_message>
    <third_message>
```
The first message is supposed to be a verb phrase like `"complete 'skills add' command"`. But today many `.context()` strings already contain `"Failed to"`, which produces `"Failed to Failed to …"`. Inner causes use bare verb phrases like `"resolve workspace directory"`, which appear after `"Caused by:"` without any prefix — confusing to read.

Additionally, `run_skills()` and `run_commands()` hide the subcommand structure inside helper functions rather than in the main dispatch match in `runner.rs`.

## Goals / Non-Goals

**Goals:**
- Ensure `display_error` produces exactly one `"Failed to "` prefix for the outermost message
- Make all `"Caused by:"` entries read as `"unable to X"` — a complete, comprehensible phrase
- Surface the full subcommand tree in `runner.rs` with per-subcommand context strings
- Ensure every action in `runner.rs` has a `.context()` so `bail!()` messages are never the first chain entry

**Non-Goals:**
- Changing `display_error`'s code structure — only the call-site context strings change
- Auditing library errors from third-party crates (only `dotagents`-owned context strings)

## Decisions

1. **Outermost context strings use no prefix**: `runner.rs` wraps each action with `.context("complete 'X' command")`. `display_error` prepends `"Failed to "` → `"Failed to complete 'skills add' command"`.

2. **Inner context strings use `"unable to X"`**: All `.context("Failed to X")` in implementation functions change to `.context("unable to X")`. This produces `"Caused by: unable to resolve workspace directory"` — readable as a sentence fragment.

3. **Inline dispatch in runner.rs**: Remove `run_skills()` and `run_commands()`. The match arms move directly into `runner.rs::run()`. This makes the full CLI surface area visible in one place and allows per-subcommand context strings at the outermost level.

4. **`bail!()` messages stay unchanged**: Root-cause messages (`"No .dotagents directory found"`, `"Configuration already exists"`) are complete sentences and appear in the `"Caused by:"` section — no prefix needed.

## Risks / Trade-offs

- **Large number of context string changes**: Search-and-replace across many files. Mitigation: use a targeted grep for `context("Failed to` to find all sites.
- **Tests may check exact error message text**: Existing unit/e2e tests that assert on exact error strings will need updating. Mitigation: update them as part of this change.
