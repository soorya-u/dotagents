## Context

`cliclack` frames interactive sessions with `intro()` (prints `┌ …`) and `outro()` (prints `└ …`). Every command that uses interactive prompts should open and close with these calls. Currently:

- `deploy` calls neither — output floats with no frame.
- `skills rm` calls `intro("dotagents skills rm")` unconditionally (not gated on `is_tty()`).
- `skills new` and `commands new/rm` have intro calls but use the command path as text.
- `deploy-outro` spec already exists but is unimplemented.

The gitignore fence (`FENCE_START` / `FENCE_END` constants in `src/utils/gitignore.rs`) is a custom comment format. VS Code, JetBrains, and Neovim all natively fold `#region` / `#endregion` blocks — switching to that format makes the managed section collapsible without any editor extension.

## Goals / Non-Goals

**Goals:**
- Deploy has a cliclack frame in TTY mode.
- All `new` and `rm` subcommands use descriptive (not command-path) intro text, gated on `is_tty()`.
- `.gitignore` managed sections use `#region dotagents` / `#endregion dotagents`.
- All tests updated to reflect the new fence strings.

**Non-Goals:**
- Migrating existing `.gitignore` files that already contain the old fence.
- Changing outro text for commands other than deploy (they already use `outro("")`).
- Any functional changes to deploy, skill, or command logic.

## Decisions

**D1 — Deploy intro placement**

`deploy()` is called both directly (from `runner::run`) and indirectly (from `maybe_prompt_deploy` inside `skills new` / `commands new`). Adding `intro()` inside `deploy()` would double-frame the session when called from `new_skill`. The intro is therefore added in `runner::run` (or the top-level `deploy` dispatch), not inside `deploy()` itself — only when the call is top-level interactive. Alternatively, `deploy()` checks if a cliclack session is already open. Simpler approach: add `intro` at the call site in `runner::run`, not inside `deploy()`.

**D2 — outro placement for deploy**

`deploy()` has two early-return paths after `print_deploy_summary`: (a) no paths to gitignore → `return Ok(())`, (b) `new_count == 0` → `return Ok(())`, (c) normal end after gitignore write. An `outro` helper is added and called at all three exit points. Or: restructure to a single exit. Simplest: `outro` called after the gitignore block regardless of branch, via a `defer`-like pattern — but Rust has no defer. Three explicit `outro` calls is acceptable.

**D3 — Gitignore fence: no migration**

Users with existing `.gitignore` files using the old fence will have both the old unmanaged block and new `#region` blocks after their next deploy. This is acceptable — the old block is inert (it's just comments), and the new block is clearly scoped. A migration would require reading and rewriting the gitignore on every deploy, adding complexity for a one-time cosmetic fix.

**D4 — `rm` intro gating**

`skills rm` currently calls `intro()` unconditionally. This means running `dotagents skills rm my-skill --force` in a non-TTY CI environment emits a cliclack intro to stdout. Gate all `rm` (and `new`) intro calls on `is_tty()`, consistent with how `new_skill` gates `use_interactive`.

## Risks / Trade-offs

- **Deploy double-intro** when called from `maybe_prompt_deploy` — mitigated by placing the intro at the top-level dispatch site only (D1).
- **Test churn** — three test files reference the old fence strings; all need a one-line string update each.
