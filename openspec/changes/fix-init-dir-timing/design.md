## Context

`dotagents init` in `src/cli/init.rs` calls `fs::create_dir_all(&workspace)` on line 113 before the TUI wizard runs. This was added to guarantee the workspace parent exists before `try_exists()` is called on line 115 to check whether `.dotagents/` already exists inside it. However, `try_exists()` returns `Ok(false)` when the parent path does not exist, making the pre-creation unnecessary. The side-effect is that cancelling the wizard still creates the workspace directory on disk.

The actual `.dotagents/` directory is created at line 142, after all TUI prompts complete — that part is correctly deferred. Only the parent workspace creation happens too early.

## Goals / Non-Goals

**Goals:**

- Ensure no filesystem writes occur if the user cancels the init wizard.
- Keep `try_exists()` correct: it does not need a pre-created parent to return `Ok(false)`.
- Add an e2e test that proves cancellation leaves no directory on disk.

**Non-Goals:**

- Changing any wizard prompt behavior, text, or ordering.
- Changing behavior of headless (flag-driven) init.
- Altering how the `.dotagents/` directory itself is created after the wizard.

## Decisions

### Move `create_dir_all` to after the TUI block

`try_exists()` on a path whose parent does not exist returns `Ok(false)`, not an error. The pre-creation was therefore a defensive no-op that introduced a side-effect. Moving `fs::create_dir_all(&workspace)` to just before `fs::create_dir(&main_dir)` on line 142 is the minimal correct fix.

Alternative considered: wrapping `create_dir_all` in a conditional to skip it when in TUI mode. Rejected because it adds complexity for no benefit — deferring is simpler and correct for both modes.

### No change to headless (flag) path behavior

When flags are present, `tui_mode` is false and the wizard block is skipped entirely, so the workspace creation timing change is a no-op for flag-driven invocations. The behavior is identical to before.

## Risks / Trade-offs

- [Risk] `try_exists()` behavior across filesystems when parent is absent — Mitigation: this is guaranteed by the Rust standard library; `try_exists` returns `Ok(false)` rather than `Err` for non-existent paths, which the existing test suite exercises.
- [Risk] e2e test for cancellation requires tui-devtools discovery to capture exact prompt output — Mitigation: the tasks.md calls out the tui-devtools pass as a required step before writing the e2e assertion.
