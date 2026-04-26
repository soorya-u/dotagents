## Context

`dotagents` is a Rust CLI. The `init` subcommand currently accepts only negative flags (`--no-mcp`, `--no-command`, etc.) and writes a fixed set of mock files silently. The `deploy` gitignore prompt uses raw `crossterm` in `src/utils/gitignore.rs` — raw mode, single keypress, plain `[y/N]` text output.

`crossterm 0.29` is already a direct dependency. No dedicated prompt library exists today.

## Goals / Non-Goals

**Goals:**
- Add a polished first-run experience for `init` using cliclack.
- Introduce two init templates: Starter and With Custom Provider.
- Upgrade the deploy gitignore confirmation to a cliclack select prompt.
- Preserve complete flag-based non-interactive operation for agents and CI.
- Centralise all TUI code in a dedicated `src/cli/ui/` module.

**Non-Goals:**
- Full TUI application (ratatui-style dashboards, real-time views).
- Changing deploy output beyond the gitignore prompt.
- Persisting user selections between runs (no config generated from wizard).

## Decisions

### 1. Library: `cliclack` over `inquire` or `dialoguer`

`cliclack` is chosen for its visual polish (intro/outro framing, `log::step` per-item feedback, boxed `note` callouts) which suits a first-run wizard. `inquire` has better crossterm version alignment and explicit `NotTTY` error, but its aesthetic is plainer. `dialoguer` adds a `console` dependency chain. Since the TUI surface is small (a handful of prompts), `cliclack`'s UX quality outweighs `inquire`'s engineering convenience.

**crossterm version risk**: if `cliclack` requires a different crossterm than `0.29`, Cargo will either unify versions automatically or require a bump. The raw-mode code in `gitignore.rs` moves to `ui/deploy.rs` and is replaced entirely by cliclack, eliminating the conflict surface.

### 2. Dual-mode decision: flag-presence triggers non-interactive path

Any feature flag (`--no-*`) or `--template` present → skip all prompts, derive from flags. This is simpler than per-prompt flag overrides and is predictable for automation.

Non-TTY + no flags → silent defaults (all features enabled, Starter template). This makes piped/CI invocations safe without requiring flags.

Considered: always prompting and using flags as pre-answers. Rejected: harder to script reliably, confusing in CI.

### 3. Template enum as `InitOptions` field, not a separate struct

`InitTemplate { Starter, WithCustomProvider }` is added directly to `InitOptions` as `template: Option<InitTemplate>`. `None` means "use TUI or default". This keeps the existing `InitFile::skip_condition: fn(&InitOptions) -> bool` pattern working — template-gated files just check `opts.template`.

Considered: a separate resolved config type built before iterating files. Deferred — the current skip_condition approach is sufficient for two templates.

### 4. `src/cli/ui/` module — TUI isolated from business logic

All cliclack calls live in `src/cli/ui/init.rs` and `src/cli/ui/deploy.rs`. `init.rs` and `deploy.rs` in `src/cli/` call into `ui::` only after the dual-mode check. This keeps the core logic testable without a terminal.

`prompt_gitignore_update()` moves from `src/utils/gitignore.rs` to `src/cli/ui/deploy.rs`. The remaining gitignore utility functions (parse, update, write) stay in utils — they are not UI.

### 5. Two mock variants for `local.config.toml`

- `src/mocks/local.config.starter.toml` — features list + empty `targets = []`, no `[providers]` block.
- `src/mocks/local.config.toml` (existing) — becomes the "With Custom Provider" variant, unchanged.

Both are embedded via `include_str!` in `src/constants/mocks.rs` as `LOCAL_CONFIG_STARTER` and `LOCAL_CONFIG_WITH_PROVIDER`.

## Risks / Trade-offs

- **cliclack crossterm version conflict** → Cargo usually unifies; if not, bump crossterm project-wide (low risk given `0.29` is recent).
- **cliclack maintenance/maturity** → smaller community than `dialoguer`. Mitigation: TUI surface is small; swapping library later is isolated to `src/cli/ui/`.
- **Silent defaults in non-TTY mode** → operator may get more files than expected without flags. Mitigation: documented in help text; all features + Starter is the safe/additive default.
- **`InitFile::skip_condition` closure captures `InitOptions` by reference** → adding `template` field requires `InitOptions: Copy` or the closures capture specific fields. Current closures only read booleans; the new template field is an `Option<enum>` which is `Copy`. No change needed.

## Migration Plan

No migration needed — this is additive. Existing flag-only invocations continue to work unchanged. The `--no-*` flags remain; `--template` is new and optional.

## Open Questions

- Should the `With Custom Provider` template use `mycode` as the example name permanently, or should the wizard ask for a custom provider name? (Current decision: keep `mycode` as a fixed example — the user renames it after init.)
- Should `cliclack`'s `note` after init hint at `dotagents deploy` or link to docs URL? (Lean toward deploy hint only for now.)
