## Context

`print_deploy_summary()` opens with `if !is_tui_enabled() { return; }`. This was written to avoid TUI-style output in CI, but the net effect is that CI pipelines get zero visibility into what `deploy` did. The correct fix is to keep output in CI using `println!()` (which bypasses the log system and is always printed).

The no-providers case: `deploy_feature()` calls `app_config.get_provider_feature_settings(feature)` which returns an empty map when no providers are configured. The `par_iter()` produces nothing, `stats` stays at `{ written: 0, skipped: 0 }`, and `print_deploy_summary` prints `"Nothing deployed"` — indistinguishable from "nothing changed" to a new user.

## Goals / Non-Goals

**Goals:**
- CI mode always prints a deploy summary line to stdout
- A missing-provider configuration emits a visible warning, not silent success
- TTY behavior is unchanged

**Non-Goals:**
- Changing the `--quiet` flag behavior (that is handled in the providers-command-ux proposal)
- Surfacing per-provider breakdown in the summary (future enhancement)

## Decisions

1. **`println!()` in non-TTY summary**: The TTY branch keeps the `"✓ "` prefix; the non-TTY branch omits the checkmark and uses a plain `println!()` like `"deployed: 2 written, 1 skipped"`. This separates the TUI styling concern from the visibility concern.

2. **Warn on zero total providers**: After `resolve_provider_defaults()` and before the deploy loop, check `app_config` for any provider entries. If none exist across all features, emit `warn!("No providers configured — nothing to deploy. Add providers to config.toml.")`. This is a single check at the top level, not inside each `deploy_feature()` call.

3. **`warn!()` for no-providers**: Using `warn!()` rather than `println!()` keeps the message at the appropriate severity (it's not an error, but it is noteworthy). It appears on stderr even in quiet mode at default verbosity.

## Risks / Trade-offs

- **Breaking script output format**: Scripts parsing `deploy` stdout will now see lines in CI where they saw nothing before. This is intentional. Since no output was the bug, existing scripts that work silently are not broken — they just gain a new stdout line.
- **`warn!()` for no-providers may fire spuriously on fresh installs**: A user who has just run `init` without configuring providers will see the warning. This is correct — it guides them to the next step.
