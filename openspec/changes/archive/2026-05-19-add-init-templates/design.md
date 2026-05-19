## Design

### Template file matrix

| File | Blank | Starter | Advanced |
|------|-------|---------|----------|
| `config.toml` | ✓ (with --features/--targets) | ✓ | ✓ |
| `local.config.toml` | ✗ | ✓ (identical to global) | ✓ (+ mycode provider block) |
| `.env` | ✗ | ✓ | ✓ |
| `.gitignore` | ✓ | ✓ | ✓ |
| Feature mocks (commands, skills, mcp, instructions) | ✓ | ✓ | ✓ |
| `templates/mycode/*.hbs` | ✗ | ✗ | ✓ |

### `InitTemplate` enum

```rust
pub(crate) enum InitTemplate {
    /// Minimal scaffolding — config.toml, feature files, .gitignore.
    Blank,
    /// Ready to deploy — adds local.config.toml, .env, variables.
    Starter,
    /// Custom provider example — adds mycode templates and provider block.
    Advanced,
}
```

CLI flag values: `blank`, `starter`, `advanced` (kebab-case via `ValueEnum`).

### `build_config_content` behavior

- `Blank` → returns `(config_string, String::new())`; caller skips writing `local.config.toml` when local is empty.
- `Starter` → returns `(config_string, config_string.clone())`; both written identically.
- `Advanced` → returns `(config_string, config_string + MYCODE_PROVIDER_CONFIG)`.

### Default resolution

`opts.template.unwrap_or(InitTemplate::Blank)` — replaces the previous `Starter` default.

### Backward compatibility

- `--template starter` continues to work (same variant, same behavior).
- `--template with-custom-provider` breaks → becomes `--template advanced`. This is acceptable as it's a pre-v1.0 CLI and the new name is clearer.
- TUI users are unaffected (labels change but flow is identical).

### TUI labels

```
Which starting template?
  Blank       Minimal scaffolding
  Starter     Variables, env & rendering
  Advanced    Custom provider & overrides
```
